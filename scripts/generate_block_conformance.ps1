# Reproducible development-only scope/block fixtures. Never touches releases.
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$candidateRoot = Join-Path $repositoryRoot 'conformance/next'
$baseRoot = Join-Path $candidateRoot 'packages/valid/minimal-no-preservation'
$utf8 = [System.Text.UTF8Encoding]::new($false)
function Write-Json($path, $value) {
    [System.IO.File]::WriteAllText($path, ($value | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n", $utf8)
}
$cases = @(
    @('valid','block-empty-defaults',$null),
    @('valid','block-negative-uniform',$null),
    @('valid','block-subnormal-scale',$null),
    @('valid','block-nonneutral-frame',$null),
    @('valid','block-conservative-bounds',$null),
    @('invalid','block-zero-scale','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-null-scale','IFCCAD_IFCDR_STRUCTURE_INVALID'),
    @('invalid','block-missing-transform','IFCCAD_IFCDR_STRUCTURE_INVALID'),
    @('invalid','block-unknown-kind','IFCCAD_IFCDR_STRUCTURE_INVALID'),
    @('invalid','block-duplicate-definition','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-layout-kind-mismatch','IFCCAD_PACKAGE_BINDING_INVALID'),
    @('invalid','block-casefold-collision','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-too-small-bounds','IFCCAD_IFCDR_BOUNDS_INVALID'),
    @('invalid','block-mixed-sign-uniform','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-unused-cycle','IFCCAD_IFCDR_BLOCK_CYCLE'),
    @('invalid','block-missing-target','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-missing-definition','IFCCAD_IFCDR_BLOCK_INVALID'),
    @('invalid','block-null-transform','IFCCAD_IFCDR_STRUCTURE_INVALID'),
    @('invalid','block-proof-gap-external','IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE'),
    @('invalid','block-proof-gap-inline','IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE')
)
$manifestPath = Join-Path $candidateRoot 'manifest.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -AsHashtable
foreach ($case in $cases) {
    $category, $name, $code = $case
    $resource = @'
{
  "header":{"format":"openaec.ifcdr","version":"0.9.0","resourceId":"drawing-main","unit":"m","nextEntityId":4},
  "scopeTable":[{"id":7,"kind":0,"bounds":null},{"id":21,"kind":2,"bounds":null}],
  "blockDefinitionTable":[{"scopeId":21,"name":"Door"}],
  "layerBindings":[{"id":0,"ifcxLayer":"layer-0"}],
  "appearanceBindings":[{"id":0,"ifcxAppearance":null,"colorMode":0,"opacityMode":0,"linePatternMode":0,"lineWeightMode":0,"overrideId":null}],
  "streamDirectory":{"version":"ifccad.ifcdr.streamDirectory.v1","streams":[
    {"name":"blockInstance","schema":"ifccad.ifcdr.blockInstance.v1","role":"object","count":1,"columns":["entityId","scopeId","definitionScopeId","transform","layerId","appearanceId"]},
    {"name":"entityOrder","schema":"ifccad.ifcdr.entityOrder.v1","role":"order","count":2,"columns":["scopeId","entryOffset","entryCount"],"children":["entityOrderEntry"]},
    {"name":"entityOrderEntry","schema":"ifccad.ifcdr.entityOrderEntry.v1","role":"child","count":1,"columns":["entityId"],"parent":"entityOrder"}
  ]},
  "streams":{
    "blockInstanceStream":{"count":1,"entityId":[3],"scopeId":[7],"definitionScopeId":[21],"transform":[{}],"layerId":[0],"appearanceId":[0]},
    "entityOrderStream":{"count":2,"scopeId":[7,21],"entryOffset":[0,1],"entryCount":[1,0]},
    "entityOrderEntryStream":{"count":1,"entityId":[3]}
  }
}
'@ | ConvertFrom-Json -AsHashtable
    switch ($name) {
        'block-negative-uniform' {
            $resource.blockDefinitionTable[0].scaling = 1
            $resource.streams.blockInstanceStream.transform[0].scale = @{x=-2.0;y=-2.0;z=-2.0}
        }
        'block-subnormal-scale' {
            $resource.streams.blockInstanceStream.transform[0].scale = @{x=[double]::Epsilon;y=1.0;z=1.0}
        }
        'block-mixed-sign-uniform' {
            $resource.blockDefinitionTable[0].scaling = 1
            $resource.streams.blockInstanceStream.transform[0].scale = @{x=-1.0;y=1.0;z=1.0}
        }
        'block-unused-cycle' {
            $resource.streams.blockInstanceStream.scopeId = @(21)
            $resource.streams.entityOrderStream.entryOffset = @(0,0)
            $resource.streams.entityOrderStream.entryCount = @(0,1)
        }
        'block-missing-target' { $resource.streams.blockInstanceStream.definitionScopeId = @(99) }
        'block-missing-definition' { $resource.blockDefinitionTable = @() }
        'block-null-transform' { $resource.streams.blockInstanceStream.transform = @($null) }
        'block-zero-scale' { $resource.streams.blockInstanceStream.transform[0].scale = @{x=0;y=1;z=1} }
        'block-null-scale' { $resource.streams.blockInstanceStream.transform[0].scale = $null }
        'block-missing-transform' {
            $resource.streams.blockInstanceStream.Remove('transform')
            $resource.streamDirectory.streams[0].columns = @($resource.streamDirectory.streams[0].columns | Where-Object { $_ -ne 'transform' })
        }
        'block-unknown-kind' { $resource.scopeTable[1].kind = 99 }
        'block-duplicate-definition' { $resource.blockDefinitionTable += @{scopeId=21;name='Duplicate'} }
        'block-nonneutral-frame' {
            $resource.streams.blockInstanceStream.transform[0].placement = @{origin=@{x=0;y=0;z=0};X=@{x=0;y=1;z=0};Y=@{x=-1;y=0;z=0}}
        }
        'block-casefold-collision' {
            $resource.blockDefinitionTable[0].name = 'Straße'
            $resource.blockDefinitionTable += @{scopeId=22;name='STRASSE'}
            $resource.scopeTable += @{id=22;kind=2;bounds=$null}
            $resource.streams.entityOrderStream.count=3
            $resource.streams.entityOrderStream.scopeId=@(7,21,22)
            $resource.streams.entityOrderStream.entryOffset=@(0,1,1)
            $resource.streams.entityOrderStream.entryCount=@(1,0,0)
            $resource.streamDirectory.streams[1].count=3
        }
    }
    $proofGap = $name.StartsWith('block-proof-gap-')
    if ($proofGap -or $name -in @('block-conservative-bounds','block-too-small-bounds')) {
        $resource.streams.blockInstanceStream.transform[0].rotation = 0.7
        $resource.scopeTable[0].bounds = @{minX=-0.644217687237691;maxX=-0.644217687237691;minY=0.7648421872844885;maxY=0.7648421872844885;minZ=0;maxZ=0}
        $resource.scopeTable[1].bounds = @{minX=0;maxX=0;minY=1;maxY=1;minZ=0;maxZ=0}
        $resource.streamDirectory.streams += @{name='line';schema='ifccad.ifcdr.line.v3';role='object';count=1;columns=@('entityId','scopeId','x1','y1','x2','y2','layerId','appearanceId')}
        $resource.streams.lineStream = @{count=1;entityId=@(1);scopeId=@(21);x1=@(0);y1=@(1);x2=@(0);y2=@(1);layerId=@(0);appearanceId=@(0)}
        $resource.streams.entityOrderStream.entryCount = @(1,1)
        $resource.streams.entityOrderEntryStream.count = 2
        $resource.streams.entityOrderEntryStream.entityId = @(3,1)
        $resource.streamDirectory.streams[2].count = 2
        if (!$proofGap) {
            $resource.streams.blockInstanceStream.transform[0].rotation = 0
            $resource.scopeTable[0].bounds = @{minX=-10;maxX=10;minY=-10;maxY=10;minZ=-10;maxZ=10}
            if ($name -eq 'block-too-small-bounds') { $resource.scopeTable[0].bounds.maxY = 0 }
        }
    }
    $root = Join-Path $candidateRoot "packages/$category/$name"
    New-Item -ItemType Directory -Path $root -Force | Out-Null
    $resourcePath = Join-Path $root 'drawing.ifcdr.json'
    Write-Json $resourcePath $resource
    $package = Get-Content -LiteralPath (Join-Path $baseRoot 'package.ifcx.json') -Raw | ConvertFrom-Json -AsHashtable
    foreach ($node in $package.data) {
        if ($node.type -eq 'openaec:DrawingLayout') {
            $node.attributes.scopeId = 7
            if ($name -eq 'block-layout-kind-mismatch') { $node.attributes.scopeId = 21 }
        }
        if ($node.type -eq 'openaec:DrawingRepresentation') {
            $node.attributes.resource.checksum = 'sha256:' + (Get-FileHash -LiteralPath $resourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($name -eq 'block-proof-gap-inline') {
                $node.attributes.resource.Remove('checksum')
                $node.attributes.resource.Remove('uri')
                $node.attributes.resource.content = $resource
            }
        }
    }
    Write-Json (Join-Path $root 'package.ifcx.json') $package
    Copy-Item -LiteralPath (Join-Path $baseRoot 'package.json') -Destination (Join-Path $root 'package.json')
    $id = "$category.$name"
    $manifest.cases = @($manifest.cases | Where-Object { $_.caseId -ne $id })
    $diagnostics = @()
    if ($code) { $diagnostics = @(@{code=$code;severity='error';category='contractViolation'}) }
    if ($name -eq 'block-missing-definition') { $diagnostics += @{code=$code;severity='error';category='contractViolation'} }
    if ($proofGap) { $diagnostics[0].category = 'executionBlocked' }
    $expected = @{diagnostics=$diagnostics}
    if ($proofGap) {
        $uri = 'drawing.ifcdr.json'
        $location = '/scopeTable/0/bounds'
        $resourceLocation = ''
        if ($name -eq 'block-proof-gap-inline') { $uri = 'package.ifcx.json'; $resourceLocation = '/data/3/attributes/resource/content'; $location = $resourceLocation + $location }
        # No strict resource proof means dependent package checks cannot finish.
        $expected.packageAssessment = @{validity='notFullyAssessed';completeness='incomplete';gaps=@(
            @{resourceId='drawing-main';resourceUri=$uri;location=$resourceLocation;reason='contentNotAssessable'},
            @{resourceId='drawing-main';resourceUri=$uri;location=$location;reason='numericalProofIncomplete'}
        )}
    }
    $manifest.cases += @{caseId=$id;category=$category;description="Scope/block contract: $name";entrypoint="packages/$category/$name/package.json";operations=@(@{name='validatePackage';expected=$expected})}
}
Write-Json $manifestPath $manifest
