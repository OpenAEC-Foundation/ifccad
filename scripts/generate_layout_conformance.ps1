# Reproducible candidate-only fixtures for the approved layout/viewport/plot matrix.
# Existing cases and numbered conformance releases are never modified.
$ErrorActionPreference = 'Stop'
$candidateRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'conformance/next'
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Write-Json($path, $value) {
    [System.IO.File]::WriteAllText(
        $path,
        ($value | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n",
        $utf8
    )
}

function Clone-Value($value) {
    return ($value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -AsHashtable)
}

$cases = @(
    @('valid', 'unused-drawing-definitions', 'empty-model-candidate'),
    @('valid', 'plot-extents', 'two-paper-layouts'),
    @('valid', 'plot-model-limits', 'two-paper-layouts'),
    @('valid', 'plot-window-fit', 'two-paper-layouts'),
    @('valid', 'viewport-active-rectangular-clip', 'layout-viewport-plot'),
    @('valid', 'viewport-disabled-boundary-reference', 'layout-viewport-plot'),
    @('valid', 'viewport-perspective-at-camera', 'layout-viewport-plot'),
    @('valid', 'viewport-visible-default', 'layout-viewport-plot'),
    @('valid', 'viewport-omitted-lens', 'layout-viewport-plot'),
    @('valid', 'viewport-zero-orthographic-lens', 'layout-viewport-plot'),
    @('invalid', 'missing-model-layout-selection', 'two-paper-layouts'),
    @('invalid', 'unselected-paper-scope', 'two-paper-layouts'),
    @('invalid', 'duplicate-paper-scope-selection', 'two-paper-layouts'),
    @('invalid', 'drawing-appearance-list-closure', 'shared-drawing-definitions'),
    @('invalid', 'layout-scope-missing', 'two-paper-layouts'),
    @('invalid', 'viewport-model-owner', 'layout-viewport-plot'),
    @('invalid', 'viewport-wrong-view-scope', 'layout-viewport-plot'),
    @('invalid', 'viewport-zero-view-height', 'layout-viewport-plot'),
    @('invalid', 'viewport-reversed-clip-planes', 'layout-viewport-plot'),
    @('invalid', 'viewport-unresolved-boundary', 'layout-viewport-plot'),
    @('invalid', 'viewport-shared-boundary', 'layout-viewport-plot'),
    @('invalid', 'viewport-boundary-outside-frame', 'layout-viewport-plot'),
    @('invalid', 'viewport-noop-override', 'layout-viewport-plot'),
    @('invalid', 'viewport-child-overlap', 'layout-viewport-plot'),
    @('invalid', 'viewport-child-orphan', 'layout-viewport-plot'),
    @('invalid', 'plot-media-margin-invalid', 'two-paper-layouts'),
    @('invalid', 'plot-window-degenerate', 'two-paper-layouts'),
    @('invalid', 'plot-model-limits-missing', 'two-paper-layouts'),
    @('invalid', 'plot-layout-centered', 'two-paper-layouts'),
    @('invalid', 'plot-fixed-zero', 'two-paper-layouts'),
    @('invalid', 'plot-custom-quality-extra-dpi', 'two-paper-layouts'),
    @('invalid', 'viewport-null-visible', 'layout-viewport-plot'),
    @('invalid', 'viewport-null-lens', 'layout-viewport-plot'),
    @('invalid', 'viewport-missing-shading-column', 'layout-viewport-plot'),
    @('invalid', 'viewport-missing-child-range', 'layout-viewport-plot'),
    @('invalid', 'viewport-zero-direction', 'layout-viewport-plot'),
    @('invalid', 'viewport-perspective-missing-lens', 'layout-viewport-plot'),
    @('invalid', 'viewport-perspective-zero-lens', 'layout-viewport-plot'),
    @('invalid', 'layout-representation-cross-resource', 'empty-model-candidate'),
    @('invalid', 'layout-paper-kind-mismatch', 'two-paper-layouts')
)
$manifest = Get-Content -Raw -LiteralPath (Join-Path $candidateRoot 'manifest.json') | ConvertFrom-Json -AsHashtable
foreach ($case in $cases) {
    $category, $name, $base = $case
    $id = "$category.$name"
    $entrypoint = "packages/$category/$name/package.json"
    $matches = @($manifest.cases | Where-Object { $_.caseId -ceq $id })
    if ($matches.Count -ne 1 -or $matches[0].entrypoint -cne $entrypoint) {
        throw "Expected exactly one manifest entry at $entrypoint for $id"
    }
}

foreach ($case in $cases) {
    $category, $name, $base = $case
    $sourceRoot = Join-Path $candidateRoot "packages/valid/$base"
    $document = Get-Content -Raw -LiteralPath (Join-Path $sourceRoot 'package.ifcx.json') | ConvertFrom-Json -AsHashtable
    $resource = Get-Content -Raw -LiteralPath (Join-Path $sourceRoot 'resources/drawing.ifcdr.json') | ConvertFrom-Json -AsHashtable
    $document.header.id = "matrix-$name"
    $drawing = @($document.data | Where-Object { $_.type -eq 'openaec:Drawing' })[0]
    $layouts = @($document.data | Where-Object { $_.type -eq 'openaec:DrawingLayout' })
    $paper = if ($layouts.Count -gt 1) { $layouts[1] } else { $null }
    $viewports = $resource.streams.viewportStream
    $overrides = $resource.streams.viewportLayerOverrideStream
    $clip = $resource.streams.polylineStream
    $otherResource = $null

    switch ($name) {
        'unused-drawing-definitions' {
            $appearance = @{path='appearance-unused';type='openaec:Appearance';attributes=@{
                name='Unused style';color=@{mode='explicit';value=@{rgb=@(1,2,3)}};
                opacity=@{mode='explicit';value=1.0};linePattern=@{mode='explicit';value='continuous'};
                lineWeight=@{mode='explicit';value=0.25}
            }}
            $layer = @{path='layer-unused';type='openaec:Layer';attributes=@{
                name='Unused layer';appearance='appearance-unused';visible=$true;frozen=$false;
                locked=$false;plottable=$true;frozenInNewViewports=$false
            }}
            $document.data += $appearance
            $document.data += $layer
            $drawing.children.Appearances = @('appearance-unused')
            $drawing.children.Layers = @('layer-unused')
        }
        'plot-extents' { $paper.attributes.plotSettings.area = @{mode='Extents'} }
        'plot-model-limits' {
            $layouts[0].attributes.limits = @{minX=0;minY=0;maxX=10;maxY=10}
            $layouts[0].attributes.plotSettings = Clone-Value $paper.attributes.plotSettings
            $layouts[0].attributes.plotSettings.area = @{mode='Limits'}
        }
        'plot-window-fit' {
            $layouts[2].attributes.plotSettings.area = @{
                mode='Window';window=@{minX=1;minY=2;maxX=5;maxY=8}
            }
        }
        'viewport-active-rectangular-clip' {
            $clip.x = @(29,31,31,29)
            $clip.y = @(19.5,19.5,20.5,20.5)
        }
        'viewport-disabled-boundary-reference' {
            $viewports.paperClip[1] = @{enabled=$false;boundaryEntityId=2}
            $clip.closed[0] = $false
        }
        'viewport-perspective-at-camera' {
            $viewports.view[0].projection = 1
            $viewports.view[0].frontClip = @{mode=1;distance=1.0}
            $viewports.view[0].backClip = @{mode=0;distance=50.0}
        }
        'viewport-visible-default' {
            $viewports.Remove('visible')
            $directory = @($resource.streamDirectory.streams | Where-Object { $_.name -eq 'viewport' })[0]
            $directory.columns = @($directory.columns | Where-Object { $_ -ne 'visible' })
        }
        'viewport-omitted-lens' { $viewports.view[0].Remove('lensLength') }
        'viewport-zero-orthographic-lens' { $viewports.view[0].lensLength = 0.0 }
        'missing-model-layout-selection' {
            $drawing.children.Layouts = @('layout-1','layout-2')
        }
        'unselected-paper-scope' {
            $drawing.children.Layouts = @('layout-0','layout-1')
        }
        'duplicate-paper-scope-selection' { $layouts[2].attributes.scopeId = 1 }
        'drawing-appearance-list-closure' { $drawing.children.Appearances = @() }
        'layout-scope-missing' { $layouts[2].attributes.scopeId = 99 }
        'viewport-model-owner' {
            $viewports.scopeId[0] = 0
            $resource.streams.entityOrderStream.entryCount = @(1,2)
            $resource.streams.entityOrderStream.entryOffset = @(0,1)
            $resource.scopeTable[0].bounds = @{minX=8;minY=19;minZ=0;maxX=12;maxY=21;maxZ=0}
        }
        'viewport-wrong-view-scope' { $viewports.viewScopeId[0] = 1 }
        'viewport-zero-view-height' { $viewports.view[0].height = 0 }
        'viewport-reversed-clip-planes' {
            $viewports.view[0].frontClip = @{mode=2;distance=5.0}
            $viewports.view[0].backClip = @{mode=1;distance=6.0}
        }
        'viewport-unresolved-boundary' { $viewports.paperClip[1].boundaryEntityId = 999 }
        'viewport-shared-boundary' {
            $viewports.paperClip[0] = @{enabled=$true;boundaryEntityId=2}
            $viewports.frame[0].center.x = 30
        }
        'viewport-boundary-outside-frame' {
            $clip.x[1] = 33
            $resource.scopeTable[1].bounds.maxX = 33
        }
        'viewport-noop-override' {
            $overrides.frozen[0] = $false
            $overrides.appearanceOverrideId[0] = $null
        }
        'viewport-child-overlap' { $viewports.layerOverrideOffset[1] = 1 }
        'viewport-child-orphan' {
            $viewports.layerOverrideCount[0] = 1
            $viewports.layerOverrideOffset[1] = 1
        }
        'plot-media-margin-invalid' { $paper.attributes.plotSettings.media.printableArea.maxX = 211 }
        'plot-window-degenerate' {
            $paper.attributes.plotSettings.area = @{
                mode='Window';window=@{minX=1;minY=1;maxX=1;maxY=2}
            }
        }
        'plot-model-limits-missing' {
            $layouts[0].attributes.plotSettings = Clone-Value $paper.attributes.plotSettings
            $layouts[0].attributes.plotSettings.area = @{mode='Limits'}
        }
        'plot-layout-centered' { $paper.attributes.plotSettings.mapping.placement = @{mode='Centered'} }
        'plot-fixed-zero' { $paper.attributes.plotSettings.mapping.scale.outputLength = 0 }
        'plot-custom-quality-extra-dpi' {
            $paper.attributes.plotSettings.output.shadedPlot.quality.dpi = 300
        }
        'viewport-null-visible' { $viewports.visible[0] = $null }
        'viewport-null-lens' { $viewports.view[0].lensLength = $null }
        'viewport-missing-shading-column' {
            $viewports.Remove('plotShadingOverride')
            $directory = @($resource.streamDirectory.streams | Where-Object { $_.name -eq 'viewport' })[0]
            $directory.columns = @($directory.columns | Where-Object { $_ -ne 'plotShadingOverride' })
        }
        'viewport-missing-child-range' {
            $viewports.Remove('layerOverrideOffset')
            $directory = @($resource.streamDirectory.streams | Where-Object { $_.name -eq 'viewport' })[0]
            $directory.columns = @($directory.columns | Where-Object { $_ -ne 'layerOverrideOffset' })
        }
        'viewport-zero-direction' { $viewports.view[0].direction = @{x=0;y=0;z=0} }
        'viewport-perspective-missing-lens' {
            $viewports.view[0].projection = 1
            $viewports.view[0].Remove('lensLength')
        }
        'viewport-perspective-zero-lens' {
            $viewports.view[0].projection = 1
            $viewports.view[0].lensLength = 0.0
        }
        'layout-representation-cross-resource' {
            $otherResource = Clone-Value $resource
            $otherResource.header.resourceId = 'other-geometry'
            $otherRepresentation = Clone-Value @($document.data | Where-Object { $_.type -eq 'openaec:DrawingRepresentation' })[0]
            $otherRepresentation.path = 'representation-other'
            $otherRepresentation.attributes.resource.resourceId = 'other-geometry'
            $otherRepresentation.attributes.resource.uri = 'resources/other.ifcdr.json'
            $document.data += $otherRepresentation
            $layouts[0].children.Representation = 'representation-other'
        }
        'layout-paper-kind-mismatch' {
            $resource.scopeTable += @{id=3;kind=2;bounds=$null}
            $resource.blockDefinitionTable += @{scopeId=3;name='Unused block'}
            $resource.streams.entityOrderStream.count = 4
            $resource.streams.entityOrderStream.scopeId = @(0,1,2,3)
            $resource.streams.entityOrderStream.entryOffset = @(0,0,0,0)
            $resource.streams.entityOrderStream.entryCount = @(0,0,0,0)
            $orderDirectory = @($resource.streamDirectory.streams | Where-Object { $_.name -eq 'entityOrder' })[0]
            $orderDirectory.count = 4
            $layouts[1].attributes.scopeId = 3
        }
        default { throw "Unknown matrix fixture: $name" }
    }

    $destination = Join-Path $candidateRoot "packages/$category/$name"
    New-Item -ItemType Directory -Path (Join-Path $destination 'resources') -Force | Out-Null
    $resourcePath = Join-Path $destination 'resources/drawing.ifcdr.json'
    Write-Json $resourcePath $resource
    $representation = @($document.data | Where-Object { $_.type -eq 'openaec:DrawingRepresentation' })[0]
    $representation.attributes.resource.checksum = 'sha256:' + (Get-FileHash -LiteralPath $resourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($otherResource) {
        $otherPath = Join-Path $destination 'resources/other.ifcdr.json'
        Write-Json $otherPath $otherResource
        $otherRepresentation.attributes.resource.checksum = 'sha256:' + (Get-FileHash -LiteralPath $otherPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    Write-Json (Join-Path $destination 'package.ifcx.json') $document
    Copy-Item -LiteralPath (Join-Path $sourceRoot 'package.json') -Destination (Join-Path $destination 'package.json') -Force
    if ($otherResource) {
        $index = Get-Content -Raw -LiteralPath (Join-Path $destination 'package.json') | ConvertFrom-Json -AsHashtable
        $index.drawingResources['resources/other.ifcdr.json'] = 'resources/other.ifcdr.json'
        Write-Json (Join-Path $destination 'package.json') $index
    }
}
