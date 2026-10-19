$ErrorActionPreference = 'Stop'

function Assert-True($condition, $message) {
    if (-not $condition) { throw $message }
}

# Isolate the process-inventory boundary: unrelated real builds must neither
# stop disposable-fixture tests nor get stopped by them. Filesystem/Git stay real.
$global:cleanupTestBusy = $false
function Get-Process {
    [CmdletBinding()]
    param()
    if ($global:cleanupTestBusy) { [pscustomobject]@{ ProcessName = 'cargo'; Id = 12345 } }
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
    Set-Content -LiteralPath (Join-Path $repo '.gitignore') -Value "/.worktrees/`n/.superpowers/`n/target/`nCargo.lock`n"
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

    # A different repository must use its own integration history, not IFCCAD main.
    $codecRoot = Join-Path $repo '.superpowers/dependencies/codec'
    $codecTask = Join-Path $repo '.worktrees/codec-task'
    git init -b main $codecRoot | Out-Null
    git -C $codecRoot config user.name 'Cleanup Test'
    git -C $codecRoot config user.email 'cleanup@example.invalid'
    Set-Content -LiteralPath (Join-Path $codecRoot '.gitignore') -Value "/target/`nCargo.lock`n"
    Set-Content -LiteralPath (Join-Path $codecRoot 'codec.txt') -Value 'independent source'
    git -C $codecRoot add .
    git -C $codecRoot commit -m 'Independent codec' | Out-Null
    git -C $codecRoot worktree add -b codec-task $codecTask main | Out-Null
    Assert-True ($LASTEXITCODE -eq 0) 'Could not create codec worktree'
    New-Item -ItemType Directory -Path (Join-Path $codecTask 'target/debug') -Force | Out-Null
    [IO.File]::WriteAllBytes((Join-Path $codecTask 'target/debug/cache.bin'), [byte[]]@(1, 2, 3))
    [IO.File]::WriteAllBytes((Join-Path $codecTask 'target/evidence.json'), [byte[]]@(4, 5, 6, 7))
    Set-Content -LiteralPath (Join-Path $codecTask 'Cargo.lock') -Value 'exact evidence dependency identity'

    Copy-Item -LiteralPath (Join-Path $projectRoot 'scripts/cleanup_local.ps1') -Destination $tool
    $report = (& $tool -Json | ConvertFrom-Json -AsHashtable)
    $done = @($report.worktrees | Where-Object name -eq 'done')[0]
    $dirty = @($report.worktrees | Where-Object name -eq 'dirty')[0]
    $ignored = @($report.worktrees | Where-Object name -eq 'ignored')[0]
    $unmerged = @($report.worktrees | Where-Object name -eq 'unmerged')[0]
    Assert-True ($done.status -eq 'removable') 'Merged clean worktree was not identified'
    $codec = @($report.worktrees | Where-Object name -eq 'codec-task')
    Assert-True ($codec.Count -eq 1) 'Independent codec worktree was omitted or duplicated'
    Assert-True ($codec[0].reasonCodes -notcontains 'not-merged-into-main') 'Codec ancestry was checked against IFCCAD history'
    Assert-True ($codec[0].reasonCodes -contains 'target-other-content') 'Codec measurement evidence was not protected'
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
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $repo '.worktrees/done'))) "Selected safe worktree was not removed: $($applied.actions | ConvertTo-Json -Compress)"
    Assert-True (Test-Path -LiteralPath (Join-Path $repo '.worktrees/dirty/drawing.txt')) 'Worktree with local edits was removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $ignoredTarget 'important.txt')) 'Unknown target content was removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $unmergedPath 'branch.txt')) 'Unmerged branch was removed'
    Assert-True (Test-Path -LiteralPath (Join-Path $ignoredTarget 'debug')) 'Blanket cache cleanup touched an unselected worktree'
    Assert-True (Test-Path -LiteralPath (Join-Path $codecTask 'target/debug/cache.bin')) 'Blanket cache cleanup touched an active codec cache'
    Assert-True (@($applied.actions | Where-Object { $_.name -eq 'dirty' -and $_.result -eq 'blocked' }).Count -eq 1) 'Apply did not report the blocked dirty worktree'
    Assert-True (@($applied.actions | Where-Object { $_.name -eq 'ignored' -and $_.result -eq 'blocked' }).Count -eq 1) 'Apply did not report the blocked ignored content'

    $sized = (& $tool -Sizes -Json | ConvertFrom-Json -AsHashtable)
    $codecSize = @($sized.target | Where-Object name -eq 'codec-task')[0].info
    Assert-True ($codecSize.cacheBytes -eq 3 -and $codecSize.otherBytes -eq 4) 'Cache size was not separated from measurement evidence'
    $selected = (& $tool -Apply -CleanBuildCache -CacheCheckout codec-task -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $codecTask 'target/debug'))) 'Explicit codec cache selection was not cleaned'
    Assert-True (Test-Path -LiteralPath (Join-Path $codecTask 'target/evidence.json')) 'Selected cache cleanup deleted measurement evidence'
    Assert-True (Test-Path -LiteralPath (Join-Path $codecTask 'Cargo.lock')) 'Cache cleanup deleted dependency provenance'
    Assert-True (Test-Path -LiteralPath (Join-Path $ignoredTarget 'debug')) 'Selected codec cleanup touched another worktree'

    # Empty Cargo tmp is harmless, but a nonempty tmp remains protected.
    Remove-Item -LiteralPath (Join-Path $codecTask 'target/evidence.json')
    New-Item -ItemType Directory -Path (Join-Path $codecTask 'target/tmp') -Force | Out-Null
    $removedCodec = (& $tool -Apply -RemoveWorktree codec-task -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (-not (Test-Path -LiteralPath $codecTask)) 'Merged codec worktree with empty tmp was not removed'
    git -C $codecRoot show-ref --verify --quiet refs/heads/codec-task
    Assert-True ($LASTEXITCODE -ne 0) 'Removed worktree left its merged task branch behind'

    # Holding Cargo's profile lock must make even an explicitly selected cleanup fail closed.
    New-Item -ItemType Directory -Path (Join-Path $repo 'target/debug') -Force | Out-Null
    $lockPath = Join-Path $repo 'target/debug/.cargo-lock'
    [IO.File]::WriteAllBytes($lockPath, [byte[]]@(0))
    $heldLock = [IO.File]::Open($lockPath, 'Open', 'ReadWrite', 'None')
    try {
        $locked = (& $tool -Apply -CleanBuildCache -Json | ConvertFrom-Json -AsHashtable)
        Assert-True (Test-Path -LiteralPath $lockPath) 'Locked build cache was removed'
        Assert-True (@($locked.actions | Where-Object result -eq 'blocked').Count -gt 0) 'Busy cache was not reported as blocked'
    }
    finally { $heldLock.Dispose() }

    $global:cleanupTestBusy = $true
    try {
        $busy = (& $tool -Apply -CleanBuildCache -Json | ConvertFrom-Json -AsHashtable)
        Assert-True (Test-Path -LiteralPath $lockPath) 'Cache cleanup ignored a running build process'
        Assert-True (@($busy.actions | Where-Object { $_.result -eq 'blocked' -and $_.detail -match 'Build tools are running' }).Count -gt 0) 'Running build was not reported as a blocker'
    }
    finally { $global:cleanupTestBusy = $false }

    # A cache-looking path must not override tracked source protection.
    $trackedCache = Join-Path $repo 'target/debug/keep.txt'
    Set-Content -LiteralPath $trackedCache -Value 'tracked source'
    git -C $repo add -f -- target/debug/keep.txt
    $tracked = (& $tool -Apply -CleanBuildCache -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (Test-Path -LiteralPath $trackedCache) 'Cache cleanup deleted a tracked source file'
    Assert-True (@($tracked.actions | Where-Object result -eq 'blocked').Count -gt 0) 'Tracked cache content was not reported as blocked'
    git -C $repo reset -- target/debug/keep.txt | Out-Null
    Remove-Item -LiteralPath $trackedCache

    # Unknown repositories cannot make the rest of the audit disappear.
    $broken = Join-Path $repo '.superpowers/dependencies/unreadable'
    New-Item -ItemType Directory -Path $broken -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $broken '.git') -Value 'gitdir: absent'
    $withBroken = (& $tool -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (@($withBroken.worktrees | Where-Object { $_.name -eq 'unreadable' -and $_.reasonCodes -contains 'unreadable-repository' }).Count -eq 1) 'Unreadable repository was not retained for review'
    Assert-True (@($withBroken.worktrees | Where-Object name -eq 'dirty').Count -eq 1) 'Unreadable repository suppressed other worktrees'

    # No ancestry in this independent repository: the source and its branch must survive.
    git -C $codecRoot worktree add -b codec-task $codecTask main | Out-Null
    Set-Content -LiteralPath (Join-Path $codecTask 'new.txt') -Value 'unmerged codec repair'
    git -C $codecTask add new.txt
    git -C $codecTask commit -m 'Unmerged codec repair' | Out-Null
    $unmergedCodec = (& $tool -Apply -RemoveWorktree codec-task -Json | ConvertFrom-Json -AsHashtable)
    Assert-True (Test-Path -LiteralPath (Join-Path $codecTask 'new.txt')) 'Unmerged independent codec worktree was removed'
    Assert-True (@($unmergedCodec.worktrees | Where-Object { $_.name -eq 'codec-task' -and $_.reasonCodes -contains 'not-merged-into-main' }).Count -eq 1) 'Independent codec ancestry failure was not reported'

    # Junctions inside recognized caches must not expose source to recursive deletion.
    $junction = Join-Path $repo 'target/debug/source-link'
    New-Item -ItemType Junction -Path $junction -Target $codecTask | Out-Null
    try {
        $linked = (& $tool -Apply -CleanBuildCache -Json | ConvertFrom-Json -AsHashtable)
        Assert-True (Test-Path -LiteralPath $junction) 'Linked cache content was deleted'
        Assert-True (Test-Path -LiteralPath (Join-Path $codecTask 'new.txt')) 'Cache cleanup followed a junction into source'
        Assert-True (@($linked.actions | Where-Object result -eq 'blocked').Count -gt 0) 'Nested linked cache was not reported as blocked'
    }
    finally { Remove-Item -LiteralPath $junction }

    # A previously safe audit must not authorize deleting work committed afterwards.
    $stalePath = Join-Path $repo '.worktrees/codec-stale'
    git -C $codecRoot worktree add -b codec-stale $stalePath main | Out-Null
    . $tool -Json | Out-Null
    $candidate = @(Get-WorktreeReport | Where-Object name -eq 'codec-stale')[0]
    Assert-True ($candidate.status -eq 'removable') 'Stale-candidate test did not start with an integrated worktree'
    Set-Content -LiteralPath (Join-Path $stalePath 'late.txt') -Value 'new work after audit'
    git -C $stalePath add late.txt
    git -C $stalePath commit -m 'New work after cleanup audit' | Out-Null
    $refusal = ''
    try { Remove-SelectedWorktree $candidate }
    catch { $refusal = $_.Exception.Message }
    Assert-True ($refusal -match 'Worktree changed since audit' -and (Test-Path -LiteralPath (Join-Path $stalePath 'late.txt'))) "Stale audit did not protect the new commit: $refusal"
    'cleanup_local.ps1: all scenarios passed'
}
finally {
    $parent = [IO.Path]::GetFullPath($testParent)
    $resolved = [IO.Path]::GetFullPath($repo)
    if (-not $resolved.StartsWith($parent + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Unsafe test cleanup path'
    }
    if (Test-Path -LiteralPath $repo) {
        foreach ($codecName in @('codec-task', 'codec-stale')) {
            $codecPath = Join-Path $repo ".worktrees/$codecName"
            if (Test-Path -LiteralPath $codecPath) {
                git -C (Join-Path $repo '.superpowers/dependencies/codec') worktree remove --force $codecPath 2>$null | Out-Null
            }
        }
        foreach ($name in @('done', 'dirty', 'ignored', 'unmerged')) {
            $path = Join-Path $repo ('.worktrees/' + $name)
            if (Test-Path -LiteralPath $path) { git -C $repo worktree remove --force $path 2>$null | Out-Null }
        }
        Remove-Item -LiteralPath $repo -Recurse -Force
    }
}
