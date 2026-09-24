# Layout, plot and viewport candidate evidence

This is the per-condition evidence index for section 14 of the approved local
layout design. Each named package below is an independent `validatePackage`
manifest case. `tests/ifccad_package_conformance.rs` loads every case with
`load_directory_package`, compares its complete ordered diagnostic code and
severity list, and requires a strict typed package exactly for the `valid`
category. It also reads the four plot areas and viewport child/optional values
through the production typed API. The companion writer test constructs Limits
and Window values, writes a directory package, reloads it strictly, and compares
typed values; existing writer tests cover Layout and Extents.

| Approved condition group | Positive evidence | Negative evidence |
| --- | --- | --- |
| 1. Model-only, unused and shared definitions | `valid.empty-model-candidate`, `valid.unused-drawing-definitions`, `valid.shared-drawing-definitions` | `invalid.drawing-list-closure`, `invalid.drawing-appearance-list-closure` |
| 2. Model and ordered independent papers | `valid.two-paper-layouts`, `valid.layout-viewport-plot` | `invalid.missing-model-layout-selection`, `invalid.unselected-paper-scope`, `invalid.duplicate-paper-scope-selection`, `invalid.layout-scope-missing` |
| 3. Viewport records, states, clips and overrides | `valid.layout-viewport-plot`, `valid.viewport-active-rectangular-clip`, `valid.viewport-disabled-boundary-reference`, `valid.viewport-perspective-at-camera`, `valid.viewport-visible-default`, `valid.viewport-omitted-lens`, `valid.viewport-zero-orthographic-lens` | `invalid.viewport-zero-height`, `invalid.viewport-zero-view-height`, `invalid.viewport-zero-direction`, `invalid.viewport-perspective-missing-lens`, `invalid.viewport-perspective-zero-lens`, `invalid.viewport-null-lens` |
| 4. Layout/reference/list identity | `valid.shared-drawing-definitions`, `valid.two-paper-layouts` | Active-profile `invalid.layout-representation-cross-resource`, `invalid.layout-paper-kind-mismatch`, `invalid.duplicate-layout-name`, `invalid.duplicate-layer-name`, `invalid.drawing-list-closure`, `invalid.drawing-appearance-list-closure`, and the selection cases in group 2. The earlier `invalid.layout-representation-mismatch` and `invalid.block-layout-kind-mismatch` have also been migrated to 0.10.0 and now exercise both the wrong binding and the resulting uncovered model scope. |
| 5. Media, four plot areas and conditional values | `valid.two-paper-layouts` (Layout and Extents), `valid.plot-extents`, `valid.plot-model-limits`, `valid.plot-window-fit`; writer readback covers all four modes | `invalid.layout-paper-limits`, `invalid.layout-fit-scale`, `invalid.plot-layout-centered`, `invalid.plot-model-limits-missing`, `invalid.plot-media-margin-invalid`, `invalid.plot-window-degenerate`, `invalid.plot-fixed-zero`, `invalid.plot-custom-quality-missing-dpi`, `invalid.plot-custom-quality-extra-dpi` |
| 6. Viewport ownership, view and relational constraints | `valid.layout-viewport-plot`, `valid.viewport-active-rectangular-clip` | `invalid.viewport-model-owner`, `invalid.viewport-wrong-view-scope`, `invalid.viewport-reversed-clip-planes`, `invalid.viewport-unresolved-boundary`, `invalid.viewport-shared-boundary`, `invalid.viewport-boundary-outside-frame`, `invalid.viewport-noop-override`, `invalid.viewport-duplicate-layer-override`, `invalid.viewport-child-range`, `invalid.viewport-child-overlap`, `invalid.viewport-child-orphan`, `invalid.viewport-missing-child-range` |
| 7. Straight clip topology | `valid.layout-viewport-plot` has an active self-intersecting closed boundary; `valid.viewport-active-rectangular-clip` has a separate closed rectangle | `invalid.open-viewport-boundary`; there is deliberately no even-odd interior assertion |
| 8. Physical/logical equivalence | `valid.viewport-visible-default` (absent defaultable column), `valid.viewport-omitted-lens` (absent optional member), `valid.viewport-zero-orthographic-lens` (present dormant zero), `valid.viewport-disabled-boundary-reference` (dormant resolved reference), `valid.layout-viewport-plot` (two relational child rows), plus four plot modes and writer readback | `invalid.viewport-null-visible`, `invalid.viewport-null-lens`, `invalid.viewport-missing-shading-column`, `invalid.viewport-missing-child-range`, `invalid.viewport-child-overlap`, `invalid.viewport-child-orphan` |

The table is a finite condition matrix, not an exhaustive Cartesian product of
all values. It proves native IFCCAD parsing, semantic validation and selected
writer readback. It does **not** prove CAD-file or visual fidelity.

## CAD evidence boundary

| CAD case | Evidence now | Still unproved or unsupported |
| --- | --- | --- |
| Ordinary model/paper layout, media, plot values, orthographic viewport, frozen layer and closed straight clip through `CadDocument` | Converter tests in `crates/ifccad-convert/tests/export_layouts.rs` exercise both conversion directions and strict IFCCAD readback for the supported in-memory profile, including a zero dormant orthographic lens on export. | Authored multi-layout, plot, viewport and clip state has no semantic DXF **and** DWG codec write/read fixture. The existing DXF/DWG layout codec checks cover only the untouched empty scaffold. Do not infer CAD-file roundtrip fidelity from the in-memory tests. |
| Perspective viewport | IFCDR/IFCX accepts valid native perspective data (`valid.viewport-perspective-at-camera`). The converter currently skips perspective with a loss diagnostic. | Real DXF and DWG fixtures must calibrate lens length, view height, `AtCamera` versus `Disabled` front clipping, and stored direction magnitude against the pinned CAD model before conversion support can be claimed. |
| Active self-intersecting straight paper clip | Package validity and reference/shape checks pass for `valid.layout-viewport-plot`; the open-boundary case fails. | Display and plot equivalence in AutoCAD and Open CAD Studio is not established. No even-odd fill or rendered aperture behavior is claimed. |
| Viewport-dependent appearance through nested blocks | Native symbolic override values and block geometry remain separate. The pinned CAD model roundtrips frozen-layer handles. | A nested BlockInstance with two viewport-effective appearance contexts lacks a visual CAD/Open CAD Studio comparison. The pinned cadcodec viewport has no appearance-override slots, so native override patches cannot be reconstructed there. |
| CTB/STB style tables | A table name and plot-style switch can be retained; a missing active table is diagnosed. | Table **contents**, external-file acquisition and effective plot-style evaluation are not represented or roundtripped. A retained name is not proof of equal plotted output. |
| Display/NamedView plot area and reusable page setups | Source cases receive explicit loss diagnostics; native effective plot settings have only Layout, Extents, Limits and Window. | No native Display/NamedView state, WorkspaceState or shared PageSetup object exists in this candidate. These are known unsupported cases, not merely untested reader cases. |
| Other active clip geometry | A resolved same-paper closed straight polyline is covered; an unresolved active boundary skips the viewport with a diagnostic. | Curved, forward-referenced or otherwise unsupported CAD boundaries have no native conversion claim. |

The separate [converter export](../../crates/ifccad-convert/src/export/COVERAGE.md)
and [import](../../crates/ifccad-convert/src/import/COVERAGE.md) inventories define
the pinned `CadDocument` boundary. Known cross-cutting codec gaps for DWG
nonzero block base points and DXF block descriptions are documented in
[the block CAD boundary](../../docs/geometry/block-cad-boundary.md); the native
layout conformance cases do not resolve them. IFCPR validation remains
incomplete as listed in [COMPATIBILITY.md](COMPATIBILITY.md#ifcpr-limitations).
