# IFCCAD JSON / DXF / DWG size baseline v1

**Outcome: passed.** Repeatability: passed. Corpus: full-v1.

Retained run: `scopes-blocks-verified-20260922` (2026-09-22), IFCDR 0.9.0,
IFCX overlay 0.11.0, unmodified cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. No local dependency override was
used; `baseline_accepted` is true. The matching detailed measurements and
provenance are in [results-v1.json](../../benchmarks/size/results-v1.json).

The [twelve-recipe corpus](../../benchmarks/size/corpus-v1.json) is unchanged in
meaning. Its three package fixtures were migrated to the active format. This
replaces the previous patched-pin experiment; differences from that report
cannot be attributed solely to the block implementation because both the
format and dependency changed. The corpus intentionally remains primitive-only:
it measures activation overhead and existing exchange paths, not block storage
savings. Block reuse and codec limitations are verified separately by the
[block boundary tests](../geometry/block-cad-boundary.md).

Both generations use production readers, strict final package validation and
Reject on the CAD-to-IFCCAD return. The tilted native-plane recipe permits and
checks exactly its expected parameterization diagnostics on import. The hard
default accuracy limit remains exactly 0.001 mm. These axis-aligned recipes
have zero geometric residual; oblique and amplified-error tests remain separate.
Previous explanatory gzip/compact-JSON probes are not current formal variants
and are not carried forward as new measurements. Performance and practice-file
evidence remain in [placement-preparation-v1.md](placement-preparation-v1.md).

Controlled fixtures and generated XYZ lines/placed straight polylines; millimetres, AC1032. IFCCAD is uncompressed pretty JSON, DXF is text, DWG uses its normal native compression. These ratios compare complete writer outputs, not compression algorithms or representative CAD practice.

## Observed complete-file bytes

| Case | Lines | Polylines | Vertices | IFCCAD external | IFCCAD inline | DXF | DWG | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| empty | 0 | 0 | 0 | 5365 | 6932 | 46978 | 20854 | 0.114 | 0.257 | 0.148 | 0.332 |
| lines-100 | 100 | 0 | 0 | 16908 | 27635 | 60338 | 22262 | 0.280 | 0.760 | 0.458 | 1.241 |
| lines-10000 | 10000 | 0 | 0 | 1225267 | 2126994 | 1444626 | 168218 | 0.848 | 7.284 | 1.472 | 12.644 |
| short-polylines-100 | 0 | 100 | 400 | 26446 | 44183 | 79398 | 22486 | 0.333 | 1.176 | 0.556 | 1.965 |
| short-polylines-10000 | 0 | 10000 | 40000 | 2244395 | 3846132 | 3396526 | 187933 | 0.661 | 11.943 | 1.132 | 20.465 |
| long-polylines-10 | 0 | 10 | 1280 | 41991 | 70128 | 114314 | 23830 | 0.367 | 1.762 | 0.613 | 2.943 |
| long-polylines-1000 | 0 | 1000 | 128000 | 3899233 | 6540970 | 7026512 | 257964 | 0.555 | 15.115 | 0.931 | 25.356 |
| mixed-1000 | 500 | 500 | 2000 | 188370 | 325457 | 312494 | 42117 | 0.603 | 4.473 | 1.041 | 7.727 |
| fractional-lines-1000 | 1000 | 0 | 0 | 155476 | 247203 | 215872 | 47198 | 0.720 | 3.294 | 1.145 | 5.238 |
| spatial-line-1000 | 1000 | 0 | 0 | 153329 | 265116 | 187664 | 42213 | 0.817 | 3.632 | 1.413 | 6.280 |
| elevated-1000 | 0 | 1000 | 4000 | 384338 | 636105 | 376972 | 38917 | 1.020 | 9.876 | 1.687 | 16.345 |
| tilted-shifted-1000 | 0 | 1000 | 4000 | 534365 | 866132 | 408112 | 38982 | 1.309 | 13.708 | 2.122 | 22.219 |

## Component accounting (bytes)

Inline bodies count only inside IFCX. Every physical blob is counted once. Preservation-inclusive fixture totals are not a native-package estimate.

| Case / mode | IFCX | External IFCDR | External IFCPR | Blobs | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| empty / ifccad-external | 2057 | 3308 | 0 | 0 | 5365 |
| empty / ifccad-inline | 6932 | 0 | 0 | 0 | 6932 |
| lines-100 / ifccad-external | 2057 | 14851 | 0 | 0 | 16908 |
| lines-100 / ifccad-inline | 27635 | 0 | 0 | 0 | 27635 |
| lines-10000 / ifccad-external | 2057 | 1223210 | 0 | 0 | 1225267 |
| lines-10000 / ifccad-inline | 2126994 | 0 | 0 | 0 | 2126994 |
| short-polylines-100 / ifccad-external | 2057 | 24389 | 0 | 0 | 26446 |
| short-polylines-100 / ifccad-inline | 44183 | 0 | 0 | 0 | 44183 |
| short-polylines-10000 / ifccad-external | 2057 | 2242338 | 0 | 0 | 2244395 |
| short-polylines-10000 / ifccad-inline | 3846132 | 0 | 0 | 0 | 3846132 |
| long-polylines-10 / ifccad-external | 2057 | 39934 | 0 | 0 | 41991 |
| long-polylines-10 / ifccad-inline | 70128 | 0 | 0 | 0 | 70128 |
| long-polylines-1000 / ifccad-external | 2057 | 3897176 | 0 | 0 | 3899233 |
| long-polylines-1000 / ifccad-inline | 6540970 | 0 | 0 | 0 | 6540970 |
| mixed-1000 / ifccad-external | 3210 | 185160 | 0 | 0 | 188370 |
| mixed-1000 / ifccad-inline | 325457 | 0 | 0 | 0 | 325457 |
| fractional-lines-1000 / ifccad-external | 2057 | 153419 | 0 | 0 | 155476 |
| fractional-lines-1000 / ifccad-inline | 247203 | 0 | 0 | 0 | 247203 |
| spatial-line-1000 / ifccad-external | 2057 | 151272 | 0 | 0 | 153329 |
| spatial-line-1000 / ifccad-inline | 265116 | 0 | 0 | 0 | 265116 |
| elevated-1000 / ifccad-external | 2057 | 382281 | 0 | 0 | 384338 |
| elevated-1000 / ifccad-inline | 636105 | 0 | 0 | 0 | 636105 |
| tilted-shifted-1000 / ifccad-external | 2057 | 532308 | 0 | 0 | 534365 |
| tilted-shifted-1000 / ifccad-inline | 866132 | 0 | 0 | 0 | 866132 |
| fixture minimal-no-preservation (fixture-native) | 3467 | 4032 | 0 | 0 | 7499 |
| fixture inline-drawing (fixture-native) | 9775 | 0 | 0 | 0 | 9775 |
| fixture source-archive (fixture-preservation-inclusive) | 4119 | 4032 | 4269 | 94 | 12514 |

## Executed checks

Every passing comparison checks each entity in order, exact recipe XYZ geometry, closure, layer, visibility and all four appearance modes/values, plus unit and layers including unused ones. IFCCAD storage variants additionally retain the same ordered entity IDs. CAD handles and technical default tables are outside this projection.

| Case | Route | Result | First failure |
| --- | --- | --- | --- |
| empty | Direct ifccad-external readback | passed | — |
| empty | Direct ifccad-inline readback | passed | — |
| empty | Direct dxf readback | passed | — |
| empty | Direct dwg readback | passed | — |
| empty | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| empty | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| lines-100 | Direct ifccad-external readback | passed | — |
| lines-100 | Direct ifccad-inline readback | passed | — |
| lines-100 | Direct dxf readback | passed | — |
| lines-100 | Direct dwg readback | passed | — |
| lines-100 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| lines-100 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| lines-10000 | Direct ifccad-external readback | passed | — |
| lines-10000 | Direct ifccad-inline readback | passed | — |
| lines-10000 | Direct dxf readback | passed | — |
| lines-10000 | Direct dwg readback | passed | — |
| lines-10000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| lines-10000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| short-polylines-100 | Direct ifccad-external readback | passed | — |
| short-polylines-100 | Direct ifccad-inline readback | passed | — |
| short-polylines-100 | Direct dxf readback | passed | — |
| short-polylines-100 | Direct dwg readback | passed | — |
| short-polylines-100 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| short-polylines-100 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| short-polylines-10000 | Direct ifccad-external readback | passed | — |
| short-polylines-10000 | Direct ifccad-inline readback | passed | — |
| short-polylines-10000 | Direct dxf readback | passed | — |
| short-polylines-10000 | Direct dwg readback | passed | — |
| short-polylines-10000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| short-polylines-10000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| long-polylines-10 | Direct ifccad-external readback | passed | — |
| long-polylines-10 | Direct ifccad-inline readback | passed | — |
| long-polylines-10 | Direct dxf readback | passed | — |
| long-polylines-10 | Direct dwg readback | passed | — |
| long-polylines-10 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| long-polylines-10 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| long-polylines-1000 | Direct ifccad-external readback | passed | — |
| long-polylines-1000 | Direct ifccad-inline readback | passed | — |
| long-polylines-1000 | Direct dxf readback | passed | — |
| long-polylines-1000 | Direct dwg readback | passed | — |
| long-polylines-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| long-polylines-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| mixed-1000 | Direct ifccad-external readback | passed | — |
| mixed-1000 | Direct ifccad-inline readback | passed | — |
| mixed-1000 | Direct dxf readback | passed | — |
| mixed-1000 | Direct dwg readback | passed | — |
| mixed-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| mixed-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| fractional-lines-1000 | Direct ifccad-external readback | passed | — |
| fractional-lines-1000 | Direct ifccad-inline readback | passed | — |
| fractional-lines-1000 | Direct dxf readback | passed | — |
| fractional-lines-1000 | Direct dwg readback | passed | — |
| fractional-lines-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| fractional-lines-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| spatial-line-1000 | Direct ifccad-external readback | passed | — |
| spatial-line-1000 | Direct ifccad-inline readback | passed | — |
| spatial-line-1000 | Direct dxf readback | passed | — |
| spatial-line-1000 | Direct dwg readback | passed | — |
| spatial-line-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| spatial-line-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| elevated-1000 | Direct ifccad-external readback | passed | — |
| elevated-1000 | Direct ifccad-inline readback | passed | — |
| elevated-1000 | Direct dxf readback | passed | — |
| elevated-1000 | Direct dwg readback | passed | — |
| elevated-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| elevated-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |
| tilted-shifted-1000 | Direct ifccad-external readback | passed | — |
| tilted-shifted-1000 | Direct ifccad-inline readback | passed | — |
| tilted-shifted-1000 | Direct dxf readback | passed | — |
| tilted-shifted-1000 | Direct dwg readback | passed | — |
| tilted-shifted-1000 | IFCCAD → dxf → IFCCAD (Reject) | passed | — |
| tilted-shifted-1000 | IFCCAD → dwg → IFCCAD (Reject) | passed | — |

The JSON retains executed stages, import/export diagnostics and scoped assessments. Import retains incomplete general coverage even when this recipe's explicit properties match. Shifted native frames intentionally use Allow for the diagnosed parameterization change; numeric accuracy remains a hard gate. The source-archive fixture retains preservationSemanticsNotAssessed; its accounting does not prove IFCPR fidelity. Failed stages stop dependent stages, which are not counted as executed.

## Repeatability and reproduction

Repository revision: 6bbb2146d95e94ee789940c9fab8937fdc9c1fec (dirty: true). Two fresh runs compare measurements and hashes of every produced artifact, including chain outputs.

| Case | Output | Same byte count | Identical files |
| --- | --- | --- | --- |
| empty | ifccad-external | true | true |
| empty | ifccad-inline | true | true |
| empty | dxf | true | true |
| empty | dwg | true | true |
| lines-100 | ifccad-external | true | true |
| lines-100 | ifccad-inline | true | true |
| lines-100 | dxf | true | true |
| lines-100 | dwg | true | true |
| lines-10000 | ifccad-external | true | true |
| lines-10000 | ifccad-inline | true | true |
| lines-10000 | dxf | true | true |
| lines-10000 | dwg | true | true |
| short-polylines-100 | ifccad-external | true | true |
| short-polylines-100 | ifccad-inline | true | true |
| short-polylines-100 | dxf | true | true |
| short-polylines-100 | dwg | true | true |
| short-polylines-10000 | ifccad-external | true | true |
| short-polylines-10000 | ifccad-inline | true | true |
| short-polylines-10000 | dxf | true | true |
| short-polylines-10000 | dwg | true | true |
| long-polylines-10 | ifccad-external | true | true |
| long-polylines-10 | ifccad-inline | true | true |
| long-polylines-10 | dxf | true | true |
| long-polylines-10 | dwg | true | true |
| long-polylines-1000 | ifccad-external | true | true |
| long-polylines-1000 | ifccad-inline | true | true |
| long-polylines-1000 | dxf | true | true |
| long-polylines-1000 | dwg | true | true |
| mixed-1000 | ifccad-external | true | true |
| mixed-1000 | ifccad-inline | true | true |
| mixed-1000 | dxf | true | true |
| mixed-1000 | dwg | true | true |
| fractional-lines-1000 | ifccad-external | true | true |
| fractional-lines-1000 | ifccad-inline | true | true |
| fractional-lines-1000 | dxf | true | true |
| fractional-lines-1000 | dwg | true | true |
| spatial-line-1000 | ifccad-external | true | true |
| spatial-line-1000 | ifccad-inline | true | true |
| spatial-line-1000 | dxf | true | true |
| spatial-line-1000 | dwg | true | true |
| elevated-1000 | ifccad-external | true | true |
| elevated-1000 | ifccad-inline | true | true |
| elevated-1000 | dxf | true | true |
| elevated-1000 | dwg | true | true |
| tilted-shifted-1000 | ifccad-external | true | true |
| tilted-shifted-1000 | ifccad-inline | true | true |
| tilted-shifted-1000 | dxf | true | true |
| tilted-shifted-1000 | dwg | true | true |

First measurement difference: —. First artifact difference: —.

Run from the repository root with a **new** run name:

```text
cargo run -p ifccad-convert --example size_baseline -- --run baseline-v1-review
```

Add `--case mixed-1000` for a labelled partial diagnostic run. The command returns failure for failed checks or nondeterminism, retaining its report. It never replaces an accepted baseline or overwrites an existing run.

The local run directory contains results.json, report.md, provenance.json, source-manifest.json, Cargo.lock, first/ and second/. Each case contains ifccad-external/, ifccad-inline/, drawing.dxf, drawing.dwg and separately labelled chain artifacts. Header snapshots support investigation of rejected CAD metadata. Exact dependency versions and source/lock/corpus hashes are in the JSON. Generated outputs and environment details remain below target/ and are not committed.

## Interpretation limits

The empty case measures fixed overhead; the two line/polyline scales expose growth, and long polylines isolate vertex-heavy storage. Inline placement changes JSON nesting and package metadata as well as the number of files. Fractional coordinates test numeric text at exact binary fractions. No result generalizes to arbitrary decimal precision, unsupported entities, external CAD applications, real-world files, runtime or memory. No size threshold or new physical encoding follows from this experiment.
