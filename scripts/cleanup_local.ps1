# Audit local checkouts and Cargo output. Age and size never authorize deletion.
[CmdletBinding()]
param(
    [switch]$Apply,
    [switch]$CleanBuildCache,
    [string[]]$CacheCheckout = @('main'),
    [string[]]$RemoveWorktree = @(),
    [switch]$Sizes,
    [switch]$Json
)

$ErrorActionPreference = 'Stop'
$comparison = [StringComparison]::OrdinalIgnoreCase
$scriptCheckout = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$primary = @(& git -C $scriptCheckout worktree list --porcelain)
if ($LASTEXITCODE -ne 0 -or $primary.Count -eq 0) { throw 'Could not locate the primary checkout' }
$repoRoot = [IO.Path]::GetFullPath($primary[0].Substring(9))
$worktreeRoot = Join-Path $repoRoot '.worktrees'
$dependencyRoot = Join-Path $repoRoot '.superpowers/dependencies'
$knownCacheNames = @('debug', 'release', 'wasm32-unknown-unknown', 'flycheck0', '.rustc_info.json', 'CACHEDIR.TAG')

function Test-ChildPath([string]$parent, [string]$path) {
    $parentFull = [IO.Path]::GetFullPath($parent).TrimEnd([IO.Path]::DirectorySeparatorChar)
    return [IO.Path]::GetFullPath($path).StartsWith($parentFull + [IO.Path]::DirectorySeparatorChar, $comparison)
}

function Assert-ChildPath([string]$parent, [string]$path) {
    $full = [IO.Path]::GetFullPath($path)
    if (-not (Test-ChildPath $parent $full)) { throw "Path escapes its intended parent: $full" }
    $cursor = $full
    $parentFull = [IO.Path]::GetFullPath($parent)
    while ($cursor) {
        if ((Test-Path -LiteralPath $cursor) -and (Get-Item -LiteralPath $cursor -Force).LinkType) { throw "Refusing a linked path: $cursor" }
        if ($cursor.Equals($parentFull, $comparison)) { break }
        $cursor = Split-Path $cursor -Parent
    }
    return $full
}

function Get-TreeInfo([string]$path) {
    [long]$bytes = 0
    $linked = $false
    $locks = [Collections.Generic.List[string]]::new()
    # Enumerate FileSystemInfo objects once rather than invoking Get-Item per file.
    $pending = [Collections.Generic.Stack[IO.FileSystemInfo]]::new()
    $pending.Push((Get-Item -LiteralPath $path -Force))
    while ($pending.Count) {
        $item = $pending.Pop()
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { $linked = $true; continue }
        if ($item.Attributes -band [IO.FileAttributes]::Directory) {
            foreach ($child in $item.EnumerateFileSystemInfos()) { $pending.Push($child) }
        }
        else {
            $bytes += $item.Length
            if ($item.Name -eq '.cargo-lock') { $locks.Add($item.FullName) }
        }
    }
    return [pscustomobject]@{ bytes = $bytes; linked = $linked; locks = $locks.ToArray() }
}

function Get-TargetInfo([string]$checkout, [bool]$Measure = $false) {
    $target = Join-Path $checkout 'target'
    $cache = @()
    $other = @()
    $cacheBytes = $(if ($Measure) { [long]0 } else { $null })
    $otherBytes = $(if ($Measure) { [long]0 } else { $null })
    $linked = $false
    if (Test-Path -LiteralPath $target) {
        try { [void](Assert-ChildPath $repoRoot $target) }
        catch { $linked = $true }
        if ($linked) { $other += 'linked target' }
        else {
            foreach ($entry in Get-ChildItem -LiteralPath $target -Force) {
                # Cargo's empty scratch directory is disposable; populated tmp needs review.
                $emptyTmp = $entry.Name -eq 'tmp' -and $entry.PSIsContainer -and -not $entry.LinkType -and @(Get-ChildItem -LiteralPath $entry.FullName -Force).Count -eq 0
                $isCache = -not $entry.LinkType -and ($entry.Name -in $knownCacheNames -or $emptyTmp)
                if ($isCache) { $cache += $entry.Name } else { $other += $entry.Name }
                if ($Measure) {
                    $tree = Get-TreeInfo $entry.FullName
                    if ($isCache) { $cacheBytes += $tree.bytes } else { $otherBytes += $tree.bytes }
                }
            }
        }
    }
    return [pscustomobject]@{ path = $target; cacheEntries = @($cache); otherEntries = @($other); linked = $linked; cacheBytes = $cacheBytes; otherBytes = $otherBytes }
}

function Get-WorktreeEntries([string]$repository) {
    $lines = @(& git -C $repository worktree list --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Could not list Git worktrees: $repository" }
    $entries = @()
    $entry = @{}
    foreach ($line in @($lines) + @('')) {
        if ($line -eq '') {
            if ($entry.path) { $entries += [pscustomobject]$entry }
            $entry = @{}
        }
        elseif ($line.StartsWith('worktree ')) { $entry.path = [IO.Path]::GetFullPath($line.Substring(9)) }
        elseif ($line.StartsWith('HEAD ')) { $entry.head = $line.Substring(5) }
        elseif ($line.StartsWith('branch ')) { $entry.branch = $line.Substring(7) }
        elseif ($line.StartsWith('locked')) { $entry.locked = $true }
        elseif ($line.StartsWith('prunable')) { $entry.prunable = $true }
    }
    return $entries
}

function Get-ManagedCheckouts {
    # Deliberately shallow. Never recurse into targets, archives, or arbitrary repositories.
    $seeds = @($repoRoot)
    foreach ($root in @($worktreeRoot, $dependencyRoot)) {
        if (-not (Test-Path -LiteralPath $root)) { continue }
        [void](Assert-ChildPath $repoRoot $root)
        foreach ($dir in Get-ChildItem -LiteralPath $root -Directory -Force) {
            if (-not $dir.LinkType -and (Test-Path -LiteralPath (Join-Path $dir.FullName '.git'))) { $seeds += $dir.FullName }
        }
    }
    $seen = @{}
    $results = @()
    foreach ($seed in $seeds) {
        $common = & git -C $seed rev-parse --path-format=absolute --git-common-dir 2>$null
        if ($LASTEXITCODE -ne 0) {
            $results += [pscustomobject]@{
                name = (Split-Path $seed -Leaf); key = [IO.Path]::GetRelativePath($repoRoot, $seed).Replace('\', '/')
                path = $seed; repository = $seed; readable = $false; managedRepository = $false
                head = $null; branch = $null; integrationRef = $null; primary = $true; locked = $false; prunable = $false
            }
            continue
        }
        $identity = [IO.Path]::GetFullPath($common)
        if ($seen.ContainsKey($identity)) { continue }
        $seen[$identity] = $true
        $entries = @(Get-WorktreeEntries $seed)
        $integrationRef = 'refs/heads/main'
        if (-not $seed.Equals($repoRoot, $comparison)) {
            & git -C $seed rev-parse --verify refs/remotes/origin/main *> $null
            if ($LASTEXITCODE -eq 0) { $integrationRef = 'refs/remotes/origin/main' }
        }
        foreach ($entry in $entries) {
            $path = $entry.path
            $isMain = $path.Equals($repoRoot, $comparison)
            $key = $(if ($isMain) { 'main' } elseif (Test-ChildPath $repoRoot $path) { [IO.Path]::GetRelativePath($repoRoot, $path).Replace('\', '/') } else { $path })
            $results += [pscustomobject]@{
                name = $(if ($isMain) { 'main' } else { Split-Path $path -Leaf })
                key = $key; path = $path; head = $entry.head; branch = $entry.branch
                repository = $entries[0].path; integrationRef = $integrationRef; readable = $true
                managedRepository = (Test-ChildPath $repoRoot $identity)
                primary = $path.Equals($entries[0].path, $comparison)
                locked = [bool]$entry.locked; prunable = [bool]$entry.prunable
            }
        }
    }
    return $results
}

function Get-WorktreeReport {
    $results = @()
    foreach ($entry in Get-ManagedCheckouts) {
        if ($entry.path.Equals($repoRoot, $comparison)) { continue }
        $path = $entry.path
        if (-not $entry.readable) {
            $results += [pscustomobject]@{
                name = $entry.name; key = $entry.key; path = $path; branch = $null; repository = $entry.repository
                integrationRef = $null; lastCommit = $null; status = 'review'; reasonCodes = @('unreadable-repository')
                localChangePaths = @(); otherIgnoredPaths = @(); targetOtherEntries = @()
            }
            continue
        }
        $reasons = @()
        $lastCommit = & git -C $entry.repository show -s --format=%cI $entry.head
        $targetInfo = $null
        $localChangePaths = @()
        $otherIgnoredPaths = @()
        if (-not (Test-ChildPath $worktreeRoot $path)) { $reasons += 'outside-worktree-root' }
        if ($entry.primary) { $reasons += 'primary-checkout' }
        if ($entry.branch -eq 'refs/heads/main') { $reasons += 'integration-branch' }
        if (-not $entry.managedRepository) { $reasons += 'external-repository' }
        if ($entry.locked) { $reasons += 'locked' }
        if ($path.Equals($scriptCheckout, $comparison)) { $reasons += 'running-script-checkout' }
        if ($entry.prunable -or -not (Test-Path -LiteralPath $path)) { $reasons += 'missing-or-prunable' }
        else {
            try { [void](Assert-ChildPath $repoRoot $path) }
            catch { $reasons += 'linked-or-external-checkout' }
            & git -C $entry.repository merge-base --is-ancestor $entry.head $entry.integrationRef 2>$null
            if ($LASTEXITCODE -ne 0) { $reasons += 'not-merged-into-main' }
            $changes = @(& git -C $path status --porcelain --untracked-files=all)
            if ($LASTEXITCODE -ne 0) { $reasons += 'unreadable-status' }
            elseif ($changes.Count) { $reasons += 'local-changes'; $localChangePaths = @($changes | ForEach-Object { $_.Substring(3) }) }
            if (Test-ChildPath $repoRoot $path) {
                $targetInfo = Get-TargetInfo $path
                if ($targetInfo.otherEntries.Count) { $reasons += 'target-other-content' }
            }
            $ignored = @(& git -C $path status --porcelain --ignored --untracked-files=normal)
            if ($LASTEXITCODE -ne 0) { $reasons += 'unreadable-ignored-content' }
            else {
                foreach ($line in $ignored) {
                    if (-not $line.StartsWith('!! ')) { continue }
                    $ignoredPath = $line.Substring(3).Trim('"')
                    if ($ignoredPath -notin @('target/', 'Cargo.lock')) { $otherIgnoredPaths += $ignoredPath }
                }
                if ($otherIgnoredPaths.Count) { $reasons += 'other-ignored-content' }
            }
        }
        $results += [pscustomobject]@{
            name = $entry.name; key = $entry.key; path = $path; head = $entry.head; branch = $entry.branch
            repository = $entry.repository; integrationRef = $entry.integrationRef; lastCommit = $lastCommit
            status = $(if ($reasons.Count -eq 0) { 'removable' } else { 'review' })
            reasonCodes = @($reasons); localChangePaths = @($localChangePaths); otherIgnoredPaths = @($otherIgnoredPaths)
            targetOtherEntries = $(if ($targetInfo) { @($targetInfo.otherEntries) } else { @() })
        }
    }
    return $results
}

function Assert-NoBuild {
    $busy = @(Get-Process -ErrorAction Stop | Where-Object { $_.ProcessName -match '^(cargo|rustc|rustdoc|rust-analyzer.*|clippy-driver|wasm-pack|rust-lld)$' })
    if ($busy.Count) { throw "Build tools are running: $(($busy | ForEach-Object { $_.ProcessName + ':' + $_.Id }) -join ', ')" }
}

function Clear-BuildCache([string]$checkout) {
    Assert-NoBuild
    $info = Get-TargetInfo $checkout
    if ($info.linked) { throw 'Linked target is protected' }
    if (-not (Test-Path -LiteralPath $info.path)) { return 0 }
    # Inspect every selected tree before deleting anything. Never follow nested links.
    $locks = @()
    foreach ($name in $info.cacheEntries) {
        $path = Assert-ChildPath $info.path (Join-Path $info.path $name)
        $tracked = @(& git -C $checkout ls-files -- "target/$name")
        if ($LASTEXITCODE -ne 0) { throw 'Could not inspect tracked cache files' }
        if ($tracked.Count) { throw "Tracked cache content is protected: $path" }
        $tree = Get-TreeInfo $path
        if ($tree.linked) { throw "Linked cache content is protected: $path" }
        $locks += $tree.locks
    }
    foreach ($lock in $locks) {
        $stream = [IO.File]::Open($lock, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
        $stream.Dispose()
    }
    Assert-NoBuild
    foreach ($name in $info.cacheEntries) {
        $path = Assert-ChildPath $info.path (Join-Path $info.path $name)
        Remove-Item -LiteralPath $path -Recurse -Force
    }
    if (@(Get-ChildItem -LiteralPath $info.path -Force).Count -eq 0) { Remove-Item -LiteralPath $info.path }
    return $info.cacheEntries.Count
}

function Assert-UnchangedWorktree($worktree) {
    $current = @(Get-WorktreeReport | Where-Object key -eq $worktree.key)
    if ($current.Count -ne 1 -or $current[0].status -ne 'removable' -or
        $current[0].head -ne $worktree.head -or $current[0].branch -ne $worktree.branch) {
        throw 'Worktree changed since audit; review it again before removal'
    }
}

function Remove-SelectedWorktree($worktree) {
    Assert-NoBuild
    $path = Assert-ChildPath $worktreeRoot $worktree.path
    Assert-UnchangedWorktree $worktree
    [void](Clear-BuildCache $path)
    Assert-UnchangedWorktree $worktree
    $lock = Join-Path $path 'Cargo.lock'
    if (Test-Path -LiteralPath $lock) {
        & git -C $path check-ignore -q -- Cargo.lock
        if ($LASTEXITCODE -eq 0) { Remove-Item -LiteralPath (Assert-ChildPath $path $lock) -Force }
    }
    & git -C $worktree.repository worktree remove $path
    if ($LASTEXITCODE -ne 0) { throw "Git refused to remove $($worktree.key)" }
    if ($worktree.branch) {
        # Compare-and-delete preserves a branch updated after the final worktree check.
        & git -C $worktree.repository update-ref -d $worktree.branch $worktree.head
        if ($LASTEXITCODE -ne 0) { throw 'Worktree removed; branch changed or Git refused branch cleanup' }
    }
}

function Select-Checkout($entries, [string]$selector) {
    $exact = @($entries | Where-Object key -eq $selector)
    if ($exact.Count) { return $exact }
    return @($entries | Where-Object name -eq $selector)
}

function Format-Blockers($worktree) {
    $labels = @{
        'outside-worktree-root' = 'staat buiten .worktrees; alleen cache afzonderlijk selecteerbaar'
        'primary-checkout' = 'primaire repository; gedeelde Git-gegevens apart beoordelen'
        'integration-branch' = 'main-branch blijft behouden'
        'external-repository' = 'Git-gegevens staan buiten de beheerde repositorymap'
        'locked' = 'Git-werkboom is vergrendeld'
        'running-script-checkout' = 'bevat het momenteel uitgevoerde opruimscript'
        'missing-or-prunable' = 'map ontbreekt of Git-registratie moet worden beoordeeld'
        'linked-or-external-checkout' = 'gekoppeld pad of pad buiten de repositorymap'
        'not-merged-into-main' = 'commit staat niet in main van de eigen repository'
        'local-changes' = 'lokale wijzigingen'
        'unreadable-status' = 'Git-status kon niet worden gelezen'
        'target-other-content' = 'overige target-inhoud moet eerst worden beoordeeld'
        'other-ignored-content' = 'genegeerde bestanden moeten eerst worden beoordeeld'
        'unreadable-ignored-content' = 'genegeerde bestanden konden niet worden gelezen'
        'unreadable-repository' = 'Git-repository niet leesbaar; controleer toegang, eigenaarschap en registratie'
    }
    return ($worktree.reasonCodes | ForEach-Object { if ($labels.ContainsKey($_)) { $labels[$_] } else { $_ } }) -join '; '
}

$actions = @()
if ($Apply) {
    if ($CleanBuildCache) {
        foreach ($selector in $CacheCheckout) {
            $selected = @(Select-Checkout @(Get-ManagedCheckouts) $selector)
            try {
                if ($selected.Count -ne 1) { throw 'Checkout not found or name is ambiguous; use its relative key' }
                $checkout = $selected[0]
                if (-not $checkout.readable -or -not $checkout.managedRepository) { throw 'Unreadable or external repository is protected' }
                if (-not $checkout.path.Equals($repoRoot, $comparison) -and -not (Test-ChildPath $worktreeRoot $checkout.path) -and -not (Test-ChildPath $dependencyRoot $checkout.path)) { throw 'Checkout is outside the managed roots' }
                [void](Assert-ChildPath $repoRoot (Join-Path $checkout.path 'target'))
                $count = Clear-BuildCache $checkout.path
                $actions += [pscustomobject]@{ name = $checkout.name; key = $checkout.key; result = 'cache-cleaned'; detail = "$count cache entries" }
            }
            catch { $actions += [pscustomobject]@{ name = $selector; result = 'blocked'; detail = $_.Exception.Message } }
        }
    }
    foreach ($selector in $RemoveWorktree) {
        try {
            $selected = @(Select-Checkout @(Get-WorktreeReport) $selector)
            if ($selected.Count -ne 1) { throw 'Worktree not found or name is ambiguous; use its relative key' }
            $worktree = $selected[0]
            if ($worktree.status -ne 'removable') { throw ($worktree.reasonCodes -join ', ') }
            Remove-SelectedWorktree $worktree
            $actions += [pscustomobject]@{ name = $worktree.name; key = $worktree.key; result = 'removed'; detail = '' }
        }
        catch { $actions += [pscustomobject]@{ name = $selector; result = 'blocked'; detail = $_.Exception.Message } }
    }
}

$report = [pscustomobject]@{
    root = $repoRoot; mode = $(if ($Apply) { 'apply' } else { 'preview' })
    worktrees = @(Get-WorktreeReport)
    target = @(
        foreach ($entry in Get-ManagedCheckouts) {
            if ($entry.path.Equals($repoRoot, $comparison) -or (Test-ChildPath $worktreeRoot $entry.path) -or (Test-ChildPath $dependencyRoot $entry.path)) {
                [pscustomobject]@{ name = $entry.name; key = $entry.key; info = (Get-TargetInfo $entry.path ([bool]$Sizes)) }
            }
        }
    )
    actions = @($actions)
}
if ($Json) { $report | ConvertTo-Json -Depth 8 -Compress; return }

Write-Output "IFCCAD lokaal opruimen ($($report.mode)); omvang of ouderdom geeft geen verwijdertoestemming"
Write-Output 'Technisch verwijderbaar na expliciete selectie als afgerond en ongebruikt:'
$ready = @($report.worktrees | Where-Object status -eq 'removable')
if ($ready.Count -eq 0) { Write-Output '  geen' }
foreach ($item in $ready) { Write-Output "  $($item.key) (laatste commit: $($item.lastCommit); $($item.integrationRef))" }
Write-Output 'Werkmappen met blokkades om te beoordelen:'
foreach ($item in $report.worktrees | Where-Object status -ne 'removable') {
    Write-Output "  $($item.key) (laatste commit: $($item.lastCommit)): $(Format-Blockers $item)"
    if ($item.localChangePaths.Count) { Write-Output "    lokale wijzigingen: $($item.localChangePaths -join ', ')" }
    if ($item.otherIgnoredPaths.Count) { Write-Output "    genegeerd, te beoordelen: $($item.otherIgnoredPaths -join ', ')" }
}
Write-Output 'Buildcache en overige target-inhoud (overige inhoud blijft behouden):'
foreach ($item in $report.target) {
    $sizeText = $(if ($Sizes) { 'cache {0:N2} GB; overige {1:N2} GB' -f ($item.info.cacheBytes / 1e9), ($item.info.otherBytes / 1e9) } else { 'gebruik -Sizes voor omvang' })
    Write-Output "  $($item.key): $sizeText; overige: $($item.info.otherEntries -join ', ')"
}
foreach ($action in $report.actions) { Write-Output "Actie $($action.name): $($action.result) $($action.detail)" }
if (-not $Apply) { Write-Output 'Na taakafronding: selecteer alleen ongebruikte werkbomen met -Apply -RemoveWorktree naam. Cache apart: -Apply -CleanBuildCache -CacheCheckout naam (standaard alleen main).' }
