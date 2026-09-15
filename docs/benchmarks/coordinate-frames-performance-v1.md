# Coordinate-frame performance and accuracy

Measured: 2026-09-15. Integrated slice-A reference before per-polyline preparation. The [follow-up measurement and practice-file inventory](placement-preparation-v1.md) describe the subsequent bounded optimization and current decision.

The production release example uses 10,000 straight two-vertex polylines per case, two warmups and five measured runs. All source packages pass the production strict reader. Loading includes file access, decoding and complete validation. CAD conversion and file I/O are measured separately. CAD-to-IFCCAD includes package construction, encoding and production-reader validation. Coordinates use metres.

The cases are identity; an oblique shifted frame; the same frame with repeated segments and independently constructed tight bounds; and an axis-aligned frame shifted by 1e12 with exactly representable integer coordinates. These are controlled stress cases, not a representative project corpus.

## Observed timings

Median milliseconds, with min-max in parentheses.

| Operation | Identity | Oblique | Tight bounds | Large coordinates |
| --- | ---: | ---: | ---: | ---: |
| Load and strict validation | 10.340 (10.115-10.797) | 355.302 (328.718-370.170) | 520.437 (511.475-533.048) | 181.220 (180.824-186.228) |
| 20,000 scope points | 0.733 (0.720-0.742) | 536.904 (514.001-618.020) | 487.214 (477.035-493.596) | 112.267 (107.692-116.087) |
| IFCCAD to CAD | 80.242 (75.908-84.993) | 4520.901 (4252.095-4788.019) | 4470.606 (4426.748-4552.287) | 904.204 (896.252-950.592) |
| CAD to IFCCAD | 61.378 (61.003-66.764) | 2491.605 (2449.179-2495.798) | 2566.087 (2515.509-2597.442) | 588.306 (579.110-592.471) |
| DXF write | 13.499 (7.696-13.718) | 31.420 (29.721-46.739) | 31.910 (30.051-43.602) | 27.402 (26.073-34.318) |
| DXF read | 147.402 (144.109-155.428) | 335.220 (330.017-346.771) | 289.911 (283.653-314.826) | 301.599 (289.583-315.462) |
| DWG write | 7.260 (7.033-8.225) | 14.575 (13.495-15.557) | 15.029 (14.724-15.440) | 13.324 (13.061-13.668) |
| DWG read | 11.472 (11.170-12.701) | 20.850 (18.037-21.619) | 16.285 (15.442-21.211) | 15.529 (14.821-16.166) |

Timings are local observations. The preceding incomplete run measured oblique point evaluation at 400.5 ms and conversion to CAD at 1,873.2 ms; the complete run is slower, including CAD I/O. This variation prevents a precise speed or regression claim. The large separation between identity and general conversion is nevertheless a clear optimization target. No concurrent Cargo test/build was deliberately run during timed operations.

## Accuracy

The following assessment is for IFCCAD-to-CadDocument and covers 20,000 vertices per case. Straight-segment residuals are affine, so the vertex maximum also bounds segment interiors. The acceptance limit is exactly 1/1,000,000 metre; its outward binary64 upper bound is slightly above 1e-6 and is not used to relax acceptance.

| Case | Status | Maximum deviation upper bound, m | Rounded entities |
| --- | --- | ---: | ---: |
| identity | Exact | 0 | 0 |
| oblique | RoundedWithinTolerance | 1.6729123829020846e-12 | 10000 |
| tight | RoundedWithinTolerance | 2.5683860144507791e-16 | 10000 |
| large | Exact | 0 | 0 |

Native parameterization changes are reported separately. CAD I/O timings only establish successful file processing; semantic DXF/DWG evidence is provided by spatial exchange tests and the [controlled experiment](size-baseline-v1.md). This timing run uses the unmodified pinned cadcodec, so it does not establish that the known DWG metadata defects are fixed.

## Exact arithmetic

Instrumentation is excluded from timed production builds. A separate opt-in core test reads the same generated packages and counts exact coordinate evaluations, certified candidate brackets and binary-search fallbacks. Frame-predicate rational arithmetic is not counted as coordinate evaluation. Converter residuals currently use exact rational expressions for non-direct mappings; these core counters are not total converter arithmetic counts.

| Case | Validation: exact coordinates | Point evaluation: exact coordinates | Point evaluation: candidate brackets | Binary-search fallbacks |
| --- | ---: | ---: | ---: | ---: |
| identity | 0 | 0 | 0 | 0 |
| oblique | 0 | 49998 | 49998 | 0 |
| tight | 20000 | 30000 | 30000 | 0 |
| large | 0 | 29999 | 29999 | 0 |

No tested case needed the binary-search fallback. Ordinary supplied bounds were accepted using interval enclosures without exact coordinate evaluation. Tight bounds required 20,000 exact coordinate checks. General correctly rounded scope-point evaluation is a separate cost, as anticipated by the earlier kernel measurement.

## Optimization follow-up

The [per-polyline preparation follow-up](placement-preparation-v1.md) prepares
fixed exact operands once per polyline and skips unused preparation on direct
mappings. No shared placement cache is implemented. Further oblique-placement
optimization is deferred until representative practice files justify it. The
placement contract, hard accuracy limit and loss diagnostics remain unchanged.
Storage-level placement sharing remains a separate encoding choice.

## Reproduction

From the repository root, select a fresh output directory:

```text
cargo run -p ifccad-convert --example spatial_accuracy --release -- target/spatial-accuracy-review
```

Set `IFCCAD_SPATIAL_MEASUREMENT_ROOT` to that generated directory, then run:

```text
cargo test -p ifccad --lib spatial_production_path_counters -- --ignored --nocapture
```

Counters are operation counts, collected separately from release timing. The complete local run is `target/spatial-accuracy-v2/`, with `results.json`, `core-counters.json`, source packages and generated CAD files. The earlier incomplete v1 run is retained: its tight-case checksum updater inserted null attributes on unrelated IFCX nodes. The reader correctly rejected that malformed input. The updater was corrected to touch only existing resource descriptors, without changing validation or bounds rules.

The [measurement example](../../crates/ifccad-convert/examples/spatial_accuracy.rs) contains the recipes and independent rational construction of tight bounds. Machine load and compiler settings affect times; results are not portable performance guarantees.

Measured toolchain: Rust 1.98.0, x86_64-pc-windows-msvc, release opt-level 3, LTO enabled, one codegen unit.
