# IFCCAD, DXF and DWG: controlled size and exchange experiment

**Result, 2026-09-15 (after per-polyline preparation):** all direct readbacks and complete conversion chains pass for twelve controlled drawings, including three spatial cases. Two independent generations produce identical complete artifact inventories. [results-v1.json](../../benchmarks/size/results-v1.json) contains the matching detailed measurements and provenance.

## Purpose and dependency boundary

This experiment compares complete IFCCAD JSON packages, text DXF and native compressed DWG, and verifies the declared recipe semantics. It does not select a physical encoding or establish fidelity for arbitrary CAD drawings. Performance and numerical stress measurements are [reported separately](coordinate-frames-performance-v1.md), with the current [preparation follow-up](placement-preparation-v1.md). The preparation optimization leaves the complete artifact inventory byte-identical to the preceding coordinate-frame run.

The implementation uses IFCDR 0.8.0, IFCX overlay 0.10.0 and cadcodec/acadrust 0.5.4 at `2f2cd25832db298524fb5eb36ced5a438a877e95`, with the two documented [DWG fixes and regression tests](../../patches/cadcodec-upstream/README.md). The normal dependency remains unmodified. Issues [#41](https://github.com/HakanSeven12/cadcodec/issues/41) and [#42](https://github.com/HakanSeven12/cadcodec/issues/42) track current-lineweight and MEASUREMENT behavior. The local override is recorded; `baseline_accepted` remains false because the automatic gate does not accept patched results as an unmodified upstream baseline.

A fresh unmodified-pin probe (`target/size-baseline/coordinate-frames-unpatched-probe/`, empty drawing, two generations) passes DXF and fails the DWG return conversion under Reject with `header.other_semantics`. This is retained evidence of the pinned metadata limitation, not a spatial geometry failure. The unpatched spatial exchange tests independently pass their explicitly checked geometry.

## Corpus and method

The [inventory](../../benchmarks/size/corpus-v1.json) retains the original nine recipes in meaning and adds spatial lines, mixed identity/elevated polylines, and tilted/shifted polylines. Three existing package fixtures are migrated to the active 0.8.0 contract. All generated drawings use millimetres and one model layout. Short polylines have four vertices; long ones have 128. The mixed case exercises order, layers, visibility, closure and appearance. Fractional coordinates are exact binary fractions.

| Drawing | Lines | Polylines | Polyline vertices |
| --- | ---: | ---: | ---: |
| empty | 0 | 0 | 0 |
| lines-100 | 100 | 0 | 0 |
| lines-10000 | 10,000 | 0 | 0 |
| short-polylines-100 | 0 | 100 | 400 |
| short-polylines-10000 | 0 | 10,000 | 40,000 |
| long-polylines-10 | 0 | 10 | 1,280 |
| long-polylines-1000 | 0 | 1,000 | 128,000 |
| mixed-1000 | 500 | 500 | 2,000 |
| fractional-lines-1000 | 1,000 | 0 | 0 |
| spatial-line-1000 | 1,000 | 0 | 0 |
| elevated-1000 | 0 | 1,000 | 4,000 |
| tilted-shifted-1000 | 0 | 1,000 | 4,000 |

Each recipe independently constructs external IFCCAD, inline IFCCAD, text DXF and DWG (AC1032). CAD preparation fixes dates and allocates missing layer handles. Production readers load all outputs. Separate IFCCAD-to-DXF-to-IFCCAD and IFCCAD-to-DWG-to-IFCCAD chains use Reject on return and strict validation of the final package.

IFCCAD-to-CAD uses Allow and asserts the exact expected diagnostics: none for the original nine and the spatial-line/elevated recipes, exactly 1,000 `PlaneParameterizationChanged` diagnostics for the tilted/shifted recipe. Return export has no diagnostics. Both directions retain the hard default accuracy limit, exactly 0.001 mm. Every nonempty recipe has an exact zero geometric residual in both conversions; empty input reports Empty. The tilted recipe uses an exact 90-degree frame, so this does not replace the separate oblique-rounding stress tests.

Comparisons check exact recipe XYZ geometry, closure, entity order, layer, visibility, all four appearance modes/values, units and every layer, including unused layers. IFCCAD variants also preserve ordered IDs. The projection independently maps the restricted axis-aligned recipe planes into XYZ. This intentionally treats native and CAD-compatible plane parameterizations as geometrically equivalent; the separate diagnostic assertions keep parameterization loss visible. It is not a general approximate-geometry comparator. CAD handles and technical default tables are excluded. Import retains incomplete general fidelity coverage.

Every physical file or blob counts once. Inline bytes count inside IFCX, not again as IFCDR. Conversion-chain files are separate from the four direct size measurements.

## Size results

| Drawing | External IFCCAD, bytes | Inline IFCCAD, bytes | DXF, bytes | DWG, bytes |
| --- | ---: | ---: | ---: | ---: |
| empty | 4,923 | 6,300 | 47,078 | 20,438 |
| lines-100 | 16,466 | 27,003 | 60,438 | 21,846 |
| lines-10000 | 1,224,825 | 2,126,362 | 1,444,724 | 167,417 |
| short-polylines-100 | 26,004 | 43,551 | 79,498 | 22,070 |
| short-polylines-10000 | 2,243,953 | 3,845,500 | 3,396,624 | 187,415 |
| long-polylines-10 | 41,549 | 69,496 | 114,414 | 23,414 |
| long-polylines-1000 | 3,898,791 | 6,540,338 | 7,026,611 | 256,225 |
| mixed-1000 | 187,928 | 324,825 | 312,593 | 41,670 |
| fractional-lines-1000 | 155,034 | 246,571 | 215,971 | 46,750 |
| spatial-line-1000 | 152,887 | 264,484 | 187,763 | 41,669 |
| elevated-1000 | 383,896 | 635,473 | 377,071 | 38,504 |
| tilted-shifted-1000 | 533,923 | 865,500 | 408,211 | 38,533 |

| Drawing | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: |
| empty | 0.105 | 0.241 | 0.134 | 0.308 |
| lines-100 | 0.272 | 0.754 | 0.447 | 1.236 |
| lines-10000 | 0.848 | 7.316 | 1.472 | 12.701 |
| short-polylines-100 | 0.327 | 1.178 | 0.548 | 1.973 |
| short-polylines-10000 | 0.661 | 11.973 | 1.132 | 20.519 |
| long-polylines-10 | 0.363 | 1.775 | 0.607 | 2.968 |
| long-polylines-1000 | 0.555 | 15.216 | 0.931 | 25.526 |
| mixed-1000 | 0.601 | 4.510 | 1.039 | 7.795 |
| fractional-lines-1000 | 0.718 | 3.316 | 1.142 | 5.274 |
| spatial-line-1000 | 0.814 | 3.669 | 1.409 | 6.347 |
| elevated-1000 | 1.018 | 9.970 | 1.685 | 16.504 |
| tilted-shifted-1000 | 1.308 | 13.856 | 2.120 | 22.461 |

For the original nine drawings, DXF and DWG output files remain byte-identical to the preceding measured reference. New IFCCAD metadata adds 24 external / 34 inline bytes for the empty case, and 86 external / 116 inline bytes for each nonempty original case. Identity placement and zero-Z columns remain omitted, so the original planar recipes incur no per-entity placement or Z-column cost.

Pretty JSON is larger than DWG for the large entity/vertex cases. External IFCCAD is smaller than text DXF for the original nine recipes and spatial lines, but is larger for the elevated and tilted/shifted recipes. Explicit nine-scalar plane records and pretty-printing contribute to that difference. These observations motivate later encoding measurements, not an unmeasured placement-sharing or compression claim.

### Fixture accounting

| Fixture | IFCX | IFCDR | IFCPR | Blob | Total bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| minimal-no-preservation | 3,467 | 4,136 | 0 | 0 | 7,603 |
| inline-drawing | 9,929 | 0 | 0 | 0 | 9,929 |
| source-archive | 4,119 | 4,136 | 4,269 | 94 | 12,618 |

The first two fixtures have complete valid assessments. Source-archive retains an incomplete preservation assessment and a shared 94-byte blob; its total is preservation-inclusive, not a native-only size.

## Exchange and repeatability

| Check | Result in each generation |
| --- | --- |
| External IFCCAD strict readback | 12/12 pass |
| Inline IFCCAD strict readback | 12/12 pass |
| DXF production readback | 12/12 pass |
| DWG production readback | 12/12 pass |
| IFCCAD to DXF to IFCCAD, Reject on return | 12/12 pass |
| IFCCAD to DWG to IFCCAD, Reject on return | 12/12 pass |
| Expected fixture assessments | 3/3 pass |
| Direct output hashes and sizes | All 48 identical across generations |
| Complete generated artifact inventory | Identical across generations |

IFCDR scope bounds are derived from emitted exact geometry; cached CAD extents are not independent drawing settings. Numerical tolerance never licenses silent loss of drawing limits, source plane values, vertex IDs or other metadata. The shifted-frame diagnostic is expected and explicitly assessed.

## Explanatory whitespace and compression probes

These are explanatory probes, not supported encodings or formal output variants. Compact JSON reparses to the same values. Gzip uses Python level 9, mtime 0, separately per external JSON file and per complete DWG. Decompression is checked byte-for-byte; archive-container overhead is excluded.

| Mixed drawing | Pretty JSON bytes | Compact JSON bytes |
| --- | ---: | ---: |
| ifccad-external | 187,928 | 63,523 |
| ifccad-inline | 324,825 | 63,412 |

| Drawing | Pretty JSON | JSON + gzip | DWG | DWG + gzip |
| --- | ---: | ---: | ---: | ---: |
| lines-10000 | 1,224,825 | 56,415 | 167,417 | 96,772 |
| short-polylines-10000 | 2,243,953 | 85,290 | 187,415 | 104,867 |
| long-polylines-1000 | 3,898,791 | 70,609 | 256,225 | 141,246 |
| tilted-shifted-1000 | 533,923 | 12,882 | 38,533 | 27,302 |

The repetitive generated corpus compresses strongly, including DWG. These probes do not predict real-project compression or the best output of either format. Production compressed readback and full package accounting are prerequisites for adding a formal compressed variant.

## Reproduction and provenance

Follow the [patch instructions](../../patches/cadcodec-upstream/README.md), using a fresh checkout and run name:

```text
cargo run --config patches/cadcodec-upstream.toml -p ifccad-convert --example size_baseline -- --run coordinate-frames-review --cargo-config patches/cadcodec-upstream.toml
```

Omit `--case` for the full corpus. Existing run directories are never overwritten; failed checks retain an incomplete report and return failure. Different dependency configurations must run sequentially because they share Cargo.lock.

The retained run is `target/size-baseline/polyline-preparation-v1/`. It reuses
the documented candidate checkout at `target/cadcodec-coordinate-frames` and
local config `target/coordinate-frames-cadcodec.toml`, with the same base and
three patches as the preceding coordinate-frame run. The dependency source hash
is unchanged. Source manifests, the lockfile, compiler/host, dependency versions,
corpus hashes, per-file hashes and both generations remain in the run directory.
The complete 180-file artifact inventory and all measurements are identical to
the preceding run; provenance records the updated IFCCAD source.

Only this report, its matching result JSON and corpus inventory form the current curated size experiment. Generated files remain under target. Passing these controlled checks does not certify arbitrary CAD source semantics, external CAD applications, raw-file preservation or IFCPR restoration.

## Verification

After the preparation change, the workspace with the unmodified dependency
passes formatting, Clippy with warnings denied and all 379 tests, including
documentation tests. Three opt-in tests are excluded from the default run.
The controlled experiment with the documented dependency patches passes both
independent generations, all direct readbacks and strict conversion chains.
