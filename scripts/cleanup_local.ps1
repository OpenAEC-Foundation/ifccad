# Audit local worktrees and generated Cargo output. Removal is always explicit.
[CmdletBinding()]
param(
    [switch]$Apply,
    [switch]$CleanBuildCache,
    [string[]]$RemoveWorktree = @(),
    [switch]$Json
)

$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$worktreeRoot = Join-Path $repoRoot '.worktrees'
$knownCacheNames = @('debug', 'release', 'wasm32-unknown-unknown', 'flycheck0', '.rustc_info.json', 'CACHEDIR.TAG')
$comparison = [StringComparison]::OrdinalIgnoreCase

function Assert-ChildPath([string]$parent, [string]$path) {
    $parentFull = [IO.Path]::GetFullPath($parent).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $full = [IO.Path]::GetFullPath($path)
    if (-not $full.StartsWith($parentFull + [IO.Path]::DirectorySeparatorChar, $comparison)) {
        throw "Path escapes its intended parent: $full"
    }
    if ((Test-Path -LiteralPath $full) -and (Get-Item -LiteralPath $full -Force).LinkType) {
        throw "Refusing a linked path: $full"
    }
    return $full
}

function Get-TargetInfo([string]$checkout) {
    $target = Join-Path $checkout 'target'
    if (-not (Test-Path -LiteralPath $target)) {
        return [pscustomobject]@{ path = $target; cacheEntries = @(); otherEntries = @(); linked = $false }
    }
    if ((Get-Item -LiteralPath $target -Force).LinkType) {
        return [pscustomobject]@{ path = $target; cacheEntries = @(); otherEntries = @('linked target'); linked = $true }
    }
    $target = Assert-ChildPath $checkout $target
    $cache = @()
    $other = @()
    foreach ($entry in Get-ChildItem -LiteralPath $target -Force) {
        if ($entry.LinkType -or $entry.Name -notin $knownCacheNames) { $other += $entry.Name }
        else { $cache += $entry.Name }
    }
    return [pscustomobject]@{ path = $target; cacheEntries = @($cache); otherEntries = @($other); linked = $false }
}

function Get-WorktreeEntries {
    $lines = @(& git -C $repoRoot worktree list --porcelain)
    if ($LASTEXITCODE -ne 0) { throw 'Could not list Git worktrees' }
    $entries = @()
    $entry = @{}
    foreach ($line in @($lines) + @('')) {
        if ($line -eq '') {
            if ($entry.path) { $entries += [pscustomobject]$entry }
            $entry = @{}
            continue
        }
        if ($line.StartsWith('worktree ')) { $entry.path = $line.Substring(9) }
        elseif ($line.StartsWith('HEAD ')) { $entry.head = $line.Substring(5) }
        elseif ($line.StartsWith('branch ')) { $entry.branch = $line.Substring(7) }
        elseif ($line.StartsWith('locked')) { $entry.locked = $true }
        elseif ($line.StartsWith('prunable')) { $entry.prunable = $true }
    }
    return $entries
}

function Get-WorktreeReport {
    $results = @()
    $rootPath = [IO.Path]::GetFullPath($repoRoot)
    foreach ($entry in Get-WorktreeEntries) {
        $path = [IO.Path]::GetFullPath($entry.path)
        if ($path.Equals($rootPath, $comparison)) { continue }
        $reasons = @()
        $name = Split-Path $path -Leaf
        $lastCommit = (& git -C $repoRoot show -s --format=%cI $entry.head)
        if ($LASTEXITCODE -ne 0) { $lastCommit = $null }
        $targetInfo = $null
        $localChangePaths = @()
        $otherIgnoredPaths = @()
        if (-not $path.StartsWith(([IO.Path]::GetFullPath($worktreeRoot).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar), $comparison)) {
            $reasons += 'outside-worktree-root'
        }
        if ($entry.locked) { $reasons += 'locked' }
        if ($entry.prunable -or -not (Test-Path -LiteralPath $path)) {
            $reasons += 'missing-or-prunable'
        }
        else {
            if ((Get-Item -LiteralPath $path -Force).LinkType) { $reasons += 'linked-worktree' }
            & git -C $repoRoot merge-base --is-ancestor $entry.head refs/heads/main
            if ($LASTEXITCODE -ne 0) { $reasons += 'not-merged-into-main' }
            $changes = @(& git -C $path status --porcelain --untracked-files=all)
            if ($LASTEXITCODE -ne 0) { $reasons += 'unreadable-status' }
            elseif ($changes.Count -gt 0) {
                $reasons += 'local-changes'
                $localChangePaths = @($changes | ForEach-Object { $_.Substring(3) })
            }
            $targetInfo = Get-TargetInfo $path
            if ($targetInfo.otherEntries.Count -gt 0) { $reasons += 'target-other-content' }
            $ignored = @(& git -C $path status --porcelain --ignored --untracked-files=normal)
            if ($LASTEXITCODE -ne 0) { $reasons += 'unreadable-ignored-content' }
            else {
                foreach ($line in $ignored) {
                    if (-not $line.StartsWith('!! ')) { continue }
                    $ignoredPath = $line.Substring(3).Trim('"')
                    if ($ignoredPath -notin @('target/', 'Cargo.lock')) {
                        $otherIgnoredPaths += $ignoredPath
                    }
                }
                if ($otherIgnoredPaths.Count -gt 0) { $reasons += 'other-ignored-content' }
            }
        }
        $results += [pscustomobject]@{
            name = $name
            path = $path
            branch = $entry.branch
            lastCommit = $lastCommit
            status = $(if ($reasons.Count -eq 0) { 'removable' } else { 'review' })
            reasonCodes = @($reasons)
            localChangePaths = @($localChangePaths)
            otherIgnoredPaths = @($otherIgnoredPaths)
            targetOtherEntries = $(if ($targetInfo) { @($targetInfo.otherEntries) } else { @() })
        }
    }
    return $results
}

function Clear-BuildCache([string]$checkout) {
    $info = Get-TargetInfo $checkout
    if ($info.linked -or -not (Test-Path -LiteralPath $info.path)) { return 0 }
    $removed = 0
    foreach ($name in $info.cacheEntries) {
        $path = Assert-ChildPath $info.path (Join-Path $info.path $name)
        Remove-Item -LiteralPath $path -Recurse -Force
        $removed++
    }
    if (@(Get-ChildItem -LiteralPath $info.path -Force).Count -eq 0) {
        Remove-Item -LiteralPath $info.path
    }
    return $removed
}

function Remove-SelectedWorktree($worktree) {
    if ((Test-Path -LiteralPath $worktreeRoot) -and (Get-Item -LiteralPath $worktreeRoot -Force).LinkType) {
        throw 'The .worktrees directory is a link'
    }
    $path = Assert-ChildPath $worktreeRoot $worktree.path
    [void](Clear-BuildCache $path)
    $lock = Join-Path $path 'Cargo.lock'
    if (Test-Path -LiteralPath $lock) {
        & git -C $path check-ignore -q -- Cargo.lock
        if ($LASTEXITCODE -eq 0) {
            $lock = Assert-ChildPath $path $lock
            Remove-Item -LiteralPath $lock -Force
        }
    }
    & git -C $repoRoot worktree remove $path
    if ($LASTEXITCODE -ne 0) { throw "Git refused to remove $($worktree.name)" }
}

function Format-Blockers($worktree) {
    $details = @()
    foreach ($code in $worktree.reasonCodes) {
        switch ($code) {
            'outside-worktree-root' { $details += 'staat buiten .worktrees' }
            'locked' { $details += 'Git-werkboom is vergrendeld' }
            'missing-or-prunable' { $details += 'map ontbreekt of Git markeert hem als opruimbaar' }
            'linked-worktree' { $details += 'map is een koppeling' }
            'not-merged-into-main' { $details += 'commit staat niet in main' }
            'unreadable-status' { $details += 'Git-status kon niet worden gelezen' }
            'local-changes' { $details += "lokale wijzigingen: $($worktree.localChangePaths -join ', ')" }
            'target-other-content' { $details += "overige target-inhoud: $($worktree.targetOtherEntries -join ', ')" }
            'unreadable-ignored-content' { $details += 'genegeerde bestanden konden niet worden gelezen' }
            'other-ignored-content' { $details += "andere genegeerde bestanden: $($worktree.otherIgnoredPaths -join ', ')" }
        }
    }
    return $details -join '; '
}

& git -C $repoRoot rev-parse --verify refs/heads/main *> $null
if ($LASTEXITCODE -ne 0) { throw 'The repository has no local main branch' }

$actions = @()
if ($Apply) {
    if ($CleanBuildCache) {
        $managed = @((Get-WorktreeEntries | Where-Object {
            $path = [IO.Path]::GetFullPath($_.path)
            $path.StartsWith(([IO.Path]::GetFullPath($worktreeRoot).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar), $comparison) -and (Test-Path -LiteralPath $path)
        }) | ForEach-Object { $_.path })
        foreach ($checkout in @($repoRoot) + $managed) {
            $count = Clear-BuildCache $checkout
            if ($count -gt 0) {
                $actions += [pscustomobject]@{ name = $(if ($checkout -eq $repoRoot) { 'main' } else { Split-Path $checkout -Leaf }); result = 'cache-cleaned'; detail = "$count cache entries" }
            }
        }
    }
    foreach ($name in $RemoveWorktree) {
        $worktree = @(Get-WorktreeReport | Where-Object name -eq $name)
        if ($worktree.Count -ne 1) {
            $actions += [pscustomobject]@{ name = $name; result = 'blocked'; detail = 'Worktree not found or name is ambiguous' }
        }
        elseif ($worktree[0].status -ne 'removable') {
            $actions += [pscustomobject]@{ name = $name; result = 'blocked'; detail = ($worktree[0].reasonCodes -join ', ') }
        }
        else {
            try {
                Remove-SelectedWorktree $worktree[0]
                $actions += [pscustomobject]@{ name = $name; result = 'removed'; detail = '' }
            }
            catch {
                $actions += [pscustomobject]@{ name = $name; result = 'blocked'; detail = $_.Exception.Message }
            }
        }
    }
}

$report = [pscustomobject]@{
    root = $repoRoot
    mode = $(if ($Apply) { 'apply' } else { 'preview' })
    worktrees = @(Get-WorktreeReport)
    target = @(
        [pscustomobject]@{ name = 'main'; info = (Get-TargetInfo $repoRoot) }
        foreach ($entry in Get-WorktreeEntries) {
            if ([IO.Path]::GetFullPath($entry.path).Equals($repoRoot, $comparison) -or -not (Test-Path -LiteralPath $entry.path)) { continue }
            [pscustomobject]@{ name = (Split-Path $entry.path -Leaf); info = (Get-TargetInfo $entry.path) }
        }
    )
    actions = @($actions)
}

if ($Json) {
    $report | ConvertTo-Json -Depth 8 -Compress
    return
}

Write-Output "IFCCAD lokaal opruimen ($($report.mode))"
Write-Output 'Technisch verwijderbaar na expliciete selectie:'
$ready = @($report.worktrees | Where-Object status -eq 'removable')
if ($ready.Count -eq 0) { Write-Output '  geen' }
foreach ($item in $ready) { Write-Output "  $($item.name) (laatste commit: $($item.lastCommit))" }
Write-Output 'Werkbomen met blokkades om te beoordelen:'
$blocked = @($report.worktrees | Where-Object status -eq 'review')
if ($blocked.Count -eq 0) { Write-Output '  geen' }
foreach ($item in $blocked) {
    Write-Output "  $($item.name) (laatste commit: $($item.lastCommit)): $(Format-Blockers $item)"
}
Write-Output 'Target-inhoud buiten de herkenbare buildcache:'
foreach ($item in $report.target) {
    if ($item.info.otherEntries.Count -gt 0) { Write-Output "  $($item.name): $($item.info.otherEntries -join ', ')" }
}
foreach ($action in $report.actions) { Write-Output "Actie $($action.name): $($action.result) $($action.detail)" }
if (-not $Apply) { Write-Output 'Voorbeeld: scripts/cleanup_local.ps1 -Apply -CleanBuildCache -RemoveWorktree naam' }
