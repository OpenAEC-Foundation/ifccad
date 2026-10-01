$ErrorActionPreference = 'Stop'

function Assert-True($condition, $message) {
    if (-not $condition) { throw $message }
}

$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$testParent = Join-Path $projectRoot 'target'
$repo = Join-Path $testParent ('cleanup-local-test-' + [guid]::NewGuid().ToString('N'))
$tool = Join-Path $repo 'scripts/cleanup_local.ps1'

try {
    New-Item -ItemType Directory -Path (Join-Path $repo 'scripts') -Force | Out-Null
    git init -b main $repo | Out-Null
    Assert-True ($LASTEXITCODE -eq 0) 'Could not initialize test repository'
    git -C $repo config user.name 'Cleanup Test'
    git -C $repo config user.email 'cleanup@example.invalid'
    Set-Content -LiteralPath (Join-Path $repo '.gitignore') -Value "/.worktrees/`n/target/`nCargo.lock`n"
    Set-Content -LiteralPath (Join-Path $repo 'drawing.txt') -Value 'original'
    git -C $repo add .gitignore drawing.txt
    git -C $repo commit -m 'Initial state' | Out-Null
    Assert-True ($LASTEXITCODE -eq 0) 'Could not commit test repository'

    foreach ($name in @('done', 'dirty', 'ignored', 'unmerged')) {
        $path = Join-Path $repo ('.worktrees/' + $name)
        git -C $repo worktree add -b $name $path main | Out-Null
        Assert-True ($LASTEXITCODE -eq 0) "Could not create $name worktree"
    }
    Set-Content -LiteralPath (Join-Path $repo '.worktrees/dirty/drawing.txt') -Value 'unfinished'
    Set-Content -LiteralPath (Join-Path $repo '.worktrees/done/Cargo.lock') -Value 'generated lockfile'
    $ignoredTarget = Join-Path $repo '.worktrees/ignored/target'
    New-Item -ItemType Directory -Path (Join-Path $ignoredTarget 'debug') -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $ignoredTarget 'debug/build.bin') -Value 'cache'
    Set-Content -LiteralPath (Join-Path $ignoredTarget 'important.txt') -Value 'keep me'
    $unmergedPath = Join-Path $repo '.worktrees/unmerged'
    Set-Content -LiteralPath (Join-Path $unmergedPath 'branch.txt') -Value 'not in main'
    git -C $unmergedPath add branch.txt
    git -C $unmergedPath commit -m 'Unmerged work' | Out-Null
    Assert-True ($LASTEXITCODE -eq 0) 'Could not commit unmerged test branch'

    Copy-Item -LiteralPath (Join-Path $projectRoot 'scripts/cleanup_local.ps1') -Destination $tool
    $report = (& $tool -Json | ConvertFrom-Json -AsHashtable)
    $done = @($report.worktrees | Where-Object name -eq 'done')[0]
    $dirty = @($report.worktrees | Where-Object name -eq 'dirty')[0]
    $ignored = @($report.worktrees | Where-Object name -eq 'ignored')[0]
    $unmerged = @($report.worktrees | Where-Object name -eq 'unmerged')[0]
    Assert-True ($done.status -eq 'removable') 'Merged clean worktree was not identified'
    Assert-True ($dirty.status -eq 'review' -and $dirty.reasonCodes -contains 'local-changes') 'Local edits were not reported as a blocker'
    Assert-True ($ignored.status -eq 'review' -and $ignored.reasonCodes -contains 'target-other-content') 'Unknown target content was not reported as a blocker'
    Assert-True ($unmerged.status -eq 'review' -and $unmerged.reasonCodes -contains 'not-merged-into-main') 'Unmerged commit was not reported as a blocker'
    Assert-True ($dirty.lastCommit) 'The report omitted the last commit date'
    Assert-True (Test-Path -LiteralPath (Join-Path $repo '.worktrees/done')) 'Preview removed a worktree'
    $human = (& $tool) -join "`n"
    Assert-True ($human -match 'drawing.txt') 'Human report did not identify the locally changed file'
    Assert-True ($human -match 'important.txt') 'Human report did not identify the retained target file'
    Assert-True ($human -match 'niet in main') 'Human report did not explain the unmerged branch'

    $applied = (& $tool -Apply -CleanBuildCache -RemoveWorktree @('done', 'dirty', 'ignored', 'unmerged') -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $repo '.worktrees/done'))) 'Selected safe worktree was not removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $repo '.worktrees/dirty/drawing.txt')) 'Worktree with local edits was removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $ignoredTarget 'important.txt')) 'Unknown target content was removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $unmergedPath 'branch.txt')) 'Unmerged branch was removed'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $ignoredTarget 'debug'))) 'Known build cache was not removed'
    Assert-True (@($applied.actions | Where-Object { $_.name -eq 'dirty' -and $_.result -eq 'blocked' }).Count -eq 1) 'Apply did not report the blocked dirty worktree'
    Assert-True (@($applied.actions | Where-Object { $_.name -eq 'ignored' -and $_.result -eq 'blocked' }).Count -eq 1) 'Apply did not report the blocked ignored content'
    'cleanup_local.ps1: all scenarios passed'
}
finally {
    $parent = [IO.Path]::GetFullPath($testParent)
    $resolved = [IO.Path]::GetFullPath($repo)
    if (-not $resolved.StartsWith($parent + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Unsafe test cleanup path'
    }
    if (Test-Path -LiteralPath $repo) {
        foreach ($name in @('done', 'dirty', 'ignored', 'unmerged')) {
            $path = Join-Path $repo ('.worktrees/' + $name)
            if (Test-Path -LiteralPath $path) { git -C $repo worktree remove --force $path 2>$null | Out-Null }
        }
        Remove-Item -LiteralPath $repo -Recurse -Force
    }
}
