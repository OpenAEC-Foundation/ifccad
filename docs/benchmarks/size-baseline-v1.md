# IFCCAD JSON / DXF / DWG size baseline v1

**Outcome: passed.** Repeatability: passed. Corpus: full-v1.

Retained run: `issue-9-inventory-final-20260924` (2026-09-24), IFCDR
0.10.0, IFCX overlay 0.12.0, unmodified cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. No local dependency
override was used; `baseline_accepted` is true. The matching detailed
measurements and provenance are in [results-v1.json](../../benchmarks/size/results-v1.json).
The inventory-led converter coverage passed the full exchange and repeatability
checks. Every artifact hash and measurement matches the previous accepted
`layouts-viewports-final-fixed-20260923` run; only source provenance changed.
The unchanged primitive corpus measures candidate activation overhead and
existing exchange paths, not the compression benefit of repeated blocks or
viewports. An earlier run exposed inert DXF `Layout1` scaffold handling; a
subsequent run caught a redundant DWG model-layout flag. Both have regression
tests. This fresh full run passes after both corrections.

This report is a snapshot of the recorded primitive corpus, not a general
completion gate for later features. Rerun the full experiment only when the
corpus or one of its exercised writer, codec, conversion or cadcodec dependency
paths changes, or to evaluate an explicit size/exchange question. For blocks,
viewports, plot settings and other absent families, use focused strict-readback
and CAD roundtrip tests; first add representative recipes before drawing
size/exchange conclusions about them. Keep the last accepted report and its
matching result JSON until an applicable run replaces them.

Controlled fixtures and generated XYZ lines/placed straight polylines; millimetres, AC1032. IFCCAD is uncompressed pretty JSON, DXF is text, DWG uses its normal native compression. These ratios compare complete writer outputs, not compression algorithms or representative CAD practice.

## Observed complete-file bytes

| Case | Lines | Polylines | Vertices | IFCCAD external | IFCCAD inline | DXF | DWG | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| empty | 0 | 0 | 0 | 7151 | 9318 | 46978 | 20854 | 0.152 | 0.343 | 0.198 | 0.447 |
| lines-100 | 100 | 0 | 0 | 18694 | 30021 | 60338 | 22262 | 0.310 | 0.840 | 0.498 | 1.349 |
| lines-10000 | 10000 | 0 | 0 | 1227053 | 2129380 | 1444626 | 168218 | 0.849 | 7.294 | 1.474 | 12.658 |
| short-polylines-100 | 0 | 100 | 400 | 28232 | 46569 | 79398 | 22486 | 0.356 | 1.256 | 0.587 | 2.071 |
| short-polylines-10000 | 0 | 10000 | 40000 | 2246181 | 3848518 | 3396526 | 187933 | 0.661 | 11.952 | 1.133 | 20.478 |
| long-polylines-10 | 0 | 10 | 1280 | 43777 | 72514 | 114314 | 23830 | 0.383 | 1.837 | 0.634 | 3.043 |
| long-polylines-1000 | 0 | 1000 | 128000 | 3901019 | 6543356 | 7026512 | 257964 | 0.555 | 15.122 | 0.931 | 25.365 |
| mixed-1000 | 500 | 500 | 2000 | 190593 | 328280 | 312494 | 42117 | 0.610 | 4.525 | 1.051 | 7.794 |
| fractional-lines-1000 | 1000 | 0 | 0 | 157262 | 249589 | 215872 | 47198 | 0.728 | 3.332 | 1.156 | 5.288 |
| spatial-line-1000 | 1000 | 0 | 0 | 155115 | 267502 | 187664 | 42213 | 0.827 | 3.675 | 1.425 | 6.337 |
| elevated-1000 | 0 | 1000 | 4000 | 386124 | 638491 | 376972 | 38917 | 1.024 | 9.922 | 1.694 | 16.406 |
| tilted-shifted-1000 | 0 | 1000 | 4000 | 536151 | 868518 | 408112 | 38982 | 1.314 | 13.754 | 2.128 | 22.280 |

## Component accounting (bytes)

Inline bodies count only inside IFCX. Every physical blob is counted once. Preservation-inclusive fixture totals are not a native-package estimate.

| Case / mode | IFCX | External IFCDR | External IFCPR | Blobs | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| empty / ifccad-external | 2435 | 4716 | 0 | 0 | 7151 |
| empty / ifccad-inline | 9318 | 0 | 0 | 0 | 9318 |
| lines-100 / ifccad-external | 2435 | 16259 | 0 | 0 | 18694 |
| lines-100 / ifccad-inline | 30021 | 0 | 0 | 0 | 30021 |
| lines-10000 / ifccad-external | 2435 | 1224618 | 0 | 0 | 1227053 |
| lines-10000 / ifccad-inline | 2129380 | 0 | 0 | 0 | 2129380 |
| short-polylines-100 / ifccad-external | 2435 | 25797 | 0 | 0 | 28232 |
| short-polylines-100 / ifccad-inline | 46569 | 0 | 0 | 0 | 46569 |
| short-polylines-10000 / ifccad-external | 2435 | 2243746 | 0 | 0 | 2246181 |
| short-polylines-10000 / ifccad-inline | 3848518 | 0 | 0 | 0 | 3848518 |
| long-polylines-10 / ifccad-external | 2435 | 41342 | 0 | 0 | 43777 |
| long-polylines-10 / ifccad-inline | 72514 | 0 | 0 | 0 | 72514 |
| long-polylines-1000 / ifccad-external | 2435 | 3898584 | 0 | 0 | 3901019 |
| long-polylines-1000 / ifccad-inline | 6543356 | 0 | 0 | 0 | 6543356 |
| mixed-1000 / ifccad-external | 4025 | 186568 | 0 | 0 | 190593 |
| mixed-1000 / ifccad-inline | 328280 | 0 | 0 | 0 | 328280 |
| fractional-lines-1000 / ifccad-external | 2435 | 154827 | 0 | 0 | 157262 |
| fractional-lines-1000 / ifccad-inline | 249589 | 0 | 0 | 0 | 249589 |
| spatial-line-1000 / ifccad-external | 2435 | 152680 | 0 | 0 | 155115 |
| spatial-line-1000 / ifccad-inline | 267502 | 0 | 0 | 0 | 267502 |
| elevated-1000 / ifccad-external | 2435 | 383689 | 0 | 0 | 386124 |
| elevated-1000 / ifccad-inline | 638491 | 0 | 0 | 0 | 638491 |
| tilted-shifted-1000 / ifccad-external | 2435 | 533716 | 0 | 0 | 536151 |
| tilted-shifted-1000 / ifccad-inline | 868518 | 0 | 0 | 0 | 868518 |
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

Repository revision: eca18ddead8766889a7ee76cf2554caa5b5cd8d9 (dirty: true). Two fresh runs compare measurements and hashes of every produced artifact, including chain outputs.

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
