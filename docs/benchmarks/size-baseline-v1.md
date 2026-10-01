# IFCCAD JSON / DXF / DWG size baseline v1

> Historical IFCCAD package experiment. Its reader/schema configuration is retained in Git history. The active standalone experiment is [OCDraw size and exchange](ocdraw-size-exchange-v1.md); package ratios do not measure the new format.

**Outcome: passed.** Repeatability: passed. Corpus: full-v1.

Retained run: `workspace-state-merged-20260928` (2026-09-28), unmodified cadcodec revision `5b682ed66ea2c89be8142c8dd83d83774fc3de08`. No local dependency override was used. The matching detailed measurements and provenance are in [results-v1.json](../../benchmarks/size/results-v1.json). Direct IFCCAD writer recipes use IFCDR 0.12.0 / IFCX 0.14.0, while the corpus has no authored workspace state. These sizes therefore measure the new package baseline, but not the additional cost of workspace records.

Controlled fixtures and generated XYZ lines/placed straight polylines; millimetres, AC1032. IFCCAD is uncompressed pretty JSON, DXF is text, DWG uses its normal native compression. These ratios compare complete writer outputs, not compression algorithms or representative CAD practice.

## Observed complete-file bytes

| Case | Lines | Polylines | Vertices | IFCCAD external | IFCCAD inline | DXF | DWG | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| empty | 0 | 0 | 0 | 10338 | 13965 | 46978 | 20854 | 0.220 | 0.496 | 0.297 | 0.670 |
| lines-100 | 100 | 0 | 0 | 21881 | 34668 | 60338 | 22262 | 0.363 | 0.983 | 0.575 | 1.557 |
| lines-10000 | 10000 | 0 | 0 | 1230240 | 2134027 | 1444626 | 168218 | 0.852 | 7.313 | 1.477 | 12.686 |
| short-polylines-100 | 0 | 100 | 400 | 31419 | 51216 | 79398 | 22486 | 0.396 | 1.397 | 0.645 | 2.278 |
| short-polylines-10000 | 0 | 10000 | 40000 | 2249368 | 3853165 | 3396526 | 187933 | 0.662 | 11.969 | 1.134 | 20.503 |
| long-polylines-10 | 0 | 10 | 1280 | 46964 | 77161 | 114314 | 23830 | 0.411 | 1.971 | 0.675 | 3.238 |
| long-polylines-1000 | 0 | 1000 | 128000 | 3904206 | 6548003 | 7026512 | 257964 | 0.556 | 15.135 | 0.932 | 25.383 |
| mixed-1000 | 500 | 500 | 2000 | 193780 | 332927 | 312494 | 42117 | 0.620 | 4.601 | 1.065 | 7.905 |
| fractional-lines-1000 | 1000 | 0 | 0 | 160449 | 254236 | 215872 | 47198 | 0.743 | 3.399 | 1.178 | 5.387 |
| spatial-line-1000 | 1000 | 0 | 0 | 158302 | 272149 | 187664 | 42213 | 0.844 | 3.750 | 1.450 | 6.447 |
| elevated-1000 | 0 | 1000 | 4000 | 294311 | 498138 | 376972 | 38917 | 0.781 | 7.563 | 1.321 | 12.800 |
| tilted-shifted-1000 | 0 | 1000 | 4000 | 539338 | 873165 | 408112 | 38982 | 1.322 | 13.836 | 2.140 | 22.399 |

## Component accounting (bytes)

Inline bodies count only inside IFCX. Every physical blob is counted once. Preservation-inclusive fixture totals are not a native-package estimate.

| Case / mode | IFCX | External IFCDR | External IFCPR | Blobs | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| empty / ifccad-external | 2472 | 7866 | 0 | 0 | 10338 |
| empty / ifccad-inline | 13965 | 0 | 0 | 0 | 13965 |
| lines-100 / ifccad-external | 2472 | 19409 | 0 | 0 | 21881 |
| lines-100 / ifccad-inline | 34668 | 0 | 0 | 0 | 34668 |
| lines-10000 / ifccad-external | 2472 | 1227768 | 0 | 0 | 1230240 |
| lines-10000 / ifccad-inline | 2134027 | 0 | 0 | 0 | 2134027 |
| short-polylines-100 / ifccad-external | 2472 | 28947 | 0 | 0 | 31419 |
| short-polylines-100 / ifccad-inline | 51216 | 0 | 0 | 0 | 51216 |
| short-polylines-10000 / ifccad-external | 2472 | 2246896 | 0 | 0 | 2249368 |
| short-polylines-10000 / ifccad-inline | 3853165 | 0 | 0 | 0 | 3853165 |
| long-polylines-10 / ifccad-external | 2472 | 44492 | 0 | 0 | 46964 |
| long-polylines-10 / ifccad-inline | 77161 | 0 | 0 | 0 | 77161 |
| long-polylines-1000 / ifccad-external | 2472 | 3901734 | 0 | 0 | 3904206 |
| long-polylines-1000 / ifccad-inline | 6548003 | 0 | 0 | 0 | 6548003 |
| mixed-1000 / ifccad-external | 4062 | 189718 | 0 | 0 | 193780 |
| mixed-1000 / ifccad-inline | 332927 | 0 | 0 | 0 | 332927 |
| fractional-lines-1000 / ifccad-external | 2472 | 157977 | 0 | 0 | 160449 |
| fractional-lines-1000 / ifccad-inline | 254236 | 0 | 0 | 0 | 254236 |
| spatial-line-1000 / ifccad-external | 2472 | 155830 | 0 | 0 | 158302 |
| spatial-line-1000 / ifccad-inline | 272149 | 0 | 0 | 0 | 272149 |
| elevated-1000 / ifccad-external | 2472 | 291839 | 0 | 0 | 294311 |
| elevated-1000 / ifccad-inline | 498138 | 0 | 0 | 0 | 498138 |
| tilted-shifted-1000 / ifccad-external | 2472 | 536866 | 0 | 0 | 539338 |
| tilted-shifted-1000 / ifccad-inline | 873165 | 0 | 0 | 0 | 873165 |
| fixture minimal-no-preservation (fixture-native) | 3895 | 4051 | 0 | 0 | 7946 |
| fixture inline-drawing (fixture-native) | 10222 | 0 | 0 | 0 | 10222 |
| fixture source-archive (fixture-preservation-inclusive) | 4547 | 4051 | 4269 | 94 | 12961 |

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

Repository revision: f81f8ac1f6abab6055c40874c6a466a97d5b8778 (dirty: true). Two fresh runs compare measurements and hashes of every produced artifact, including chain outputs.

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
cargo run -p ocdraw-convert --example size_baseline -- --run baseline-v1-review
```

Add `--case mixed-1000` for a labelled partial diagnostic run. The command returns failure for failed checks or nondeterminism, retaining its report. It never replaces an accepted baseline or overwrites an existing run.

The local run directory contains results.json, report.md, provenance.json, source-manifest.json, Cargo.lock, first/ and second/. Each case contains ifccad-external/, ifccad-inline/, drawing.dxf, drawing.dwg and separately labelled chain artifacts. Header snapshots support investigation of rejected CAD metadata. Exact dependency versions and source/lock/corpus hashes are in the JSON. Generated outputs and environment details remain below target/ and are not committed.

## Interpretation limits

The empty case measures fixed overhead; the two line/polyline scales expose growth, and long polylines isolate vertex-heavy storage. Inline placement changes JSON nesting and package metadata as well as the number of files. Fractional coordinates test numeric text at exact binary fractions. No result generalizes to arbitrary decimal precision, unsupported entities, external CAD applications, real-world files, runtime or memory. No size threshold or new physical encoding follows from this experiment.
