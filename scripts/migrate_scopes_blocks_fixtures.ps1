# Explicit development-fixture migration. Numbered conformance releases and
# historical schema assets are never modified. Existing wrong checksums remain
# wrong; only checksums that matched their original resource are refreshed.
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$fixtureRoot = Join-Path $repositoryRoot 'conformance/next/packages'
$fixtureFiles = @(Get-ChildItem -LiteralPath $fixtureRoot -Recurse -File -Filter '*.json')
$originalHashes = @{}
foreach ($file in $fixtureFiles) { $originalHashes[$file.FullName] = 'sha256:' + (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
$utf8 = [System.Text.UTF8Encoding]::new($false)
foreach ($file in $fixtureFiles) {
    $originalText = [System.IO.File]::ReadAllText($file.FullName)
    try { $document = $originalText | ConvertFrom-Json -AsHashtable } catch { continue }
    if ($document.header.format -eq 'openaec.ifcdr' -and $document.header.version -eq '0.8.0') {
        $document.header.version = '0.9.0'
        foreach ($scope in $document.scopeTable) {
            foreach ($key in @('name','baseX','baseY','baseZ','flags')) { $scope.Remove($key) }
        }
        [System.IO.File]::WriteAllText($file.FullName, ($document | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n", $utf8)
    } elseif ($originalText.Contains("`r`n")) {
        # Normalize before refreshing references, including already migrated files.
        [System.IO.File]::WriteAllText($file.FullName, $originalText.Replace("`r`n", "`n"), $utf8)
    }
}
foreach ($file in $fixtureFiles) {
    try { $document = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json -AsHashtable } catch { continue }
    $changed = $false
    foreach ($node in $document.data) {
        $resource = $node.attributes.resource
        if ($resource -and $resource.format -eq 'openaec.ifcdr' -and $resource.version -eq '0.8.0') {
            $resource.version = '0.9.0'; $changed = $true
            if ($resource.content -and $resource.content.header.version -eq '0.8.0') {
                $resource.content.header.version = '0.9.0'
                foreach ($scope in $resource.content.scopeTable) {
                    foreach ($key in @('name','baseX','baseY','baseZ','flags')) { $scope.Remove($key) }
                }
            }
        }
        if ($resource.uri) {
            $resolved = [System.IO.Path]::GetFullPath((Join-Path $file.DirectoryName $resource.uri))
            if ($originalHashes.ContainsKey($resolved) -and $resource.checksum -ceq $originalHashes[$resolved]) {
                $newHash = 'sha256:' + (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
                if ($newHash -cne $resource.checksum) { $resource.checksum = $newHash; $changed = $true }
            }
        }
    }
    if ($changed) { [System.IO.File]::WriteAllText($file.FullName, ($document | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n", $utf8) }
}
