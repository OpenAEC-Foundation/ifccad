# Migrate only unpublished 0.9.0 package fixtures to the active 0.10.0
# drawing contract. Released collections and deliberate old-version probes stay
# unchanged. Refresh only checksums that matched the original resource bytes.
$ErrorActionPreference = 'Stop'
$fixtureRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'conformance/next/packages'
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Write-Json($path, $value) {
    $text = ($value | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
    [System.IO.File]::WriteAllText($path, $text, $utf8)
}

foreach ($ifcxFile in Get-ChildItem -LiteralPath $fixtureRoot -Recurse -File -Filter 'package.ifcx.json') {
    $graph = Get-Content -LiteralPath $ifcxFile.FullName -Raw | ConvertFrom-Json -AsHashtable
    $resources = @($graph.data | ForEach-Object { $_.attributes.resource } |
        Where-Object { $_ -and $_.format -eq 'openaec.ifcdr' -and $_.version -eq '0.9.0' })
    if ($resources.Count -eq 0) { continue }

    $packageDirectory = $ifcxFile.DirectoryName
    $originalHashes = @{}
    foreach ($resource in $resources) {
        if (-not $resource.uri) { continue }
        $path = [System.IO.Path]::GetFullPath((Join-Path $packageDirectory $resource.uri))
        $packagePrefix = $packageDirectory.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
        if (-not $path.StartsWith($packagePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Resource path escapes its package: $($resource.uri)"
        }
        if ((Test-Path -LiteralPath $path -PathType Leaf) -and -not $originalHashes.ContainsKey($path)) {
            $originalHashes[$path] = 'sha256:' + (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }
    foreach ($resource in $resources) {
        if ($resource.content -and $resource.content.header.version -eq '0.9.0') {
            $resource.content.header.version = '0.10.0'
        }
        if ($resource.uri) {
            $path = [System.IO.Path]::GetFullPath((Join-Path $packageDirectory $resource.uri))
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                $drawing = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable
                if ($drawing.header.version -eq '0.9.0') {
                    $drawing.header.version = '0.10.0'
                    Write-Json $path $drawing
                }
                if ($resource.checksum -ceq $originalHashes[$path]) {
                    $resource.checksum = 'sha256:' + (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
                }
            }
        }
        $resource.version = '0.10.0'
    }

    # Some inline cases retain a loose IFCDR copy for direct fixture inspection.
    foreach ($file in Get-ChildItem -LiteralPath $packageDirectory -Recurse -File -Filter '*.ifcdr.json') {
        $drawing = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json -AsHashtable
        if ($drawing.header.format -eq 'openaec.ifcdr' -and $drawing.header.version -eq '0.9.0') {
            $drawing.header.version = '0.10.0'
            Write-Json $file.FullName $drawing
        }
    }

    $layerPaths = @($graph.data | Where-Object { $_.type -eq 'openaec:Layer' } | ForEach-Object { $_.path })
    $appearancePaths = @($graph.data | Where-Object { $_.type -eq 'openaec:Appearance' } | ForEach-Object { $_.path })
    foreach ($node in $graph.data) {
        switch ($node.type) {
            'openaec:Drawing' {
                $node.children.Layers = $layerPaths
                $node.children.Appearances = $appearancePaths
            }
            'openaec:DrawingLayout' {
                if ($node.attributes.Contains('limitsCheckEnabled')) {
                    $node.attributes.limitsChecking = $node.attributes.limitsCheckEnabled
                    $node.attributes.Remove('limitsCheckEnabled')
                }
                $node.attributes.Remove('tabOrder')
            }
            'openaec:Layer' {
                $node.attributes.frozen = $false
                $node.attributes.locked = $false
                $node.attributes.plottable = $true
                $node.attributes.frozenInNewViewports = $false
            }
        }
    }
    Write-Json $ifcxFile.FullName $graph
}
