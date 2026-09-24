# IFCCAD JSON / DXF / DWG size baseline v1

**Outcome: passed.** Repeatability: passed. Corpus: full-v1.

Retained run: `geometry-families-proof-20260924` (2026-09-24), IFCDR
0.11.0, IFCX overlay 0.13.0, unmodified cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. No local dependency
override was used; `baseline_accepted` is true. The matching detailed
measurements and provenance are in [results-v1.json](../../benchmarks/size/results-v1.json).
The full direct-readback, IFCCAD/DXF/DWG exchange and two-run repeatability
checks passed. Relative to the previous accepted 0.10.0/0.12.0 run, eleven
of twelve IFCCAD cases grew by 3,150 external or 4,610 inline bytes from the
larger schema and package metadata. The elevated-polyline case instead shrank
by 91,850 external or 140,390 inline bytes because its axis-aligned placements
now use the origin-only encoding. All DXF and DWG output sizes were unchanged.
The corpus has straight lines and polylines but no Point, Circle,
Arc, Ellipse, EllipseArc, bulged polyline or SpatialPolyline recipes.

This report is a snapshot of the recorded primitive corpus. Rerun the full
experiment when an exercised writer, codec, converter, cadcodec dependency or
corpus recipe changes. For families absent from the corpus, use focused strict
readback and CAD conversion tests, then add representative recipes before
drawing size or exchange conclusions about those families. Keep the last
accepted report and matching result JSON until an applicable run replaces them.

Controlled fixtures and generated XYZ lines/placed straight polylines; millimetres, AC1032. IFCCAD is uncompressed pretty JSON, DXF is text, DWG uses its normal native compression. These ratios compare complete writer outputs, not compression algorithms or representative CAD practice.

## Observed complete-file bytes

| Case | Lines | Polylines | Vertices | IFCCAD external | IFCCAD inline | DXF | DWG | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| empty | 0 | 0 | 0 | 10301 | 13928 | 46978 | 20854 | 0.219 | 0.494 | 0.296 | 0.668 |
| lines-100 | 100 | 0 | 0 | 21844 | 34631 | 60338 | 22262 | 0.362 | 0.981 | 0.574 | 1.556 |
| lines-10000 | 10000 | 0 | 0 | 1230203 | 2133990 | 1444626 | 168218 | 0.852 | 7.313 | 1.477 | 12.686 |
| short-polylines-100 | 0 | 100 | 400 | 31382 | 51179 | 79398 | 22486 | 0.395 | 1.396 | 0.645 | 2.276 |
| short-polylines-10000 | 0 | 10000 | 40000 | 2249331 | 3853128 | 3396526 | 187933 | 0.662 | 11.969 | 1.134 | 20.503 |
| long-polylines-10 | 0 | 10 | 1280 | 46927 | 77124 | 114314 | 23830 | 0.411 | 1.969 | 0.675 | 3.236 |
| long-polylines-1000 | 0 | 1000 | 128000 | 3904169 | 6547966 | 7026512 | 257964 | 0.556 | 15.135 | 0.932 | 25.383 |
| mixed-1000 | 500 | 500 | 2000 | 193743 | 332890 | 312494 | 42117 | 0.620 | 4.600 | 1.065 | 7.904 |
| fractional-lines-1000 | 1000 | 0 | 0 | 160412 | 254199 | 215872 | 47198 | 0.743 | 3.399 | 1.178 | 5.386 |
| spatial-line-1000 | 1000 | 0 | 0 | 158265 | 272112 | 187664 | 42213 | 0.843 | 3.749 | 1.450 | 6.446 |
| elevated-1000 | 0 | 1000 | 4000 | 294274 | 498101 | 376972 | 38917 | 0.781 | 7.562 | 1.321 | 12.799 |
| tilted-shifted-1000 | 0 | 1000 | 4000 | 539301 | 873128 | 408112 | 38982 | 1.321 | 13.835 | 2.139 | 22.398 |

## Component accounting (bytes)

Inline bodies count only inside IFCX. Every physical blob is counted once. Preservation-inclusive fixture totals are not a native-package estimate.

| Case / mode | IFCX | External IFCDR | External IFCPR | Blobs | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| empty / ifccad-external | 2435 | 7866 | 0 | 0 | 10301 |
| empty / ifccad-inline | 13928 | 0 | 0 | 0 | 13928 |
| lines-100 / ifccad-external | 2435 | 19409 | 0 | 0 | 21844 |
| lines-100 / ifccad-inline | 34631 | 0 | 0 | 0 | 34631 |
| lines-10000 / ifccad-external | 2435 | 1227768 | 0 | 0 | 1230203 |
| lines-10000 / ifccad-inline | 2133990 | 0 | 0 | 0 | 2133990 |
| short-polylines-100 / ifccad-external | 2435 | 28947 | 0 | 0 | 31382 |
| short-polylines-100 / ifccad-inline | 51179 | 0 | 0 | 0 | 51179 |
| short-polylines-10000 / ifccad-external | 2435 | 2246896 | 0 | 0 | 2249331 |
| short-polylines-10000 / ifccad-inline | 3853128 | 0 | 0 | 0 | 3853128 |
| long-polylines-10 / ifccad-external | 2435 | 44492 | 0 | 0 | 46927 |
| long-polylines-10 / ifccad-inline | 77124 | 0 | 0 | 0 | 77124 |
| long-polylines-1000 / ifccad-external | 2435 | 3901734 | 0 | 0 | 3904169 |
| long-polylines-1000 / ifccad-inline | 6547966 | 0 | 0 | 0 | 6547966 |
| mixed-1000 / ifccad-external | 4025 | 189718 | 0 | 0 | 193743 |
| mixed-1000 / ifccad-inline | 332890 | 0 | 0 | 0 | 332890 |
| fractional-lines-1000 / ifccad-external | 2435 | 157977 | 0 | 0 | 160412 |
| fractional-lines-1000 / ifccad-inline | 254199 | 0 | 0 | 0 | 254199 |
| spatial-line-1000 / ifccad-external | 2435 | 155830 | 0 | 0 | 158265 |
| spatial-line-1000 / ifccad-inline | 272112 | 0 | 0 | 0 | 272112 |
| elevated-1000 / ifccad-external | 2435 | 291839 | 0 | 0 | 294274 |
| elevated-1000 / ifccad-inline | 498101 | 0 | 0 | 0 | 498101 |
| tilted-shifted-1000 / ifccad-external | 2435 | 536866 | 0 | 0 | 539301 |
| tilted-shifted-1000 / ifccad-inline | 873128 | 0 | 0 | 0 | 873128 |
| fixture minimal-no-preservation (fixture-native) | 3858 | 4051 | 0 | 0 | 7909 |
| fixture inline-drawing (fixture-native) | 10185 | 0 | 0 | 0 | 10185 |
| fixture source-archive (fixture-preservation-inclusive) | 4510 | 4051 | 4269 | 94 | 12924 |

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

Repository revision: b7b2c7b581f9cd9a25d5419d78fb5640cc2ee190 (dirty: true). Two fresh runs compare measurements and hashes of every produced artifact, including chain outputs.

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
