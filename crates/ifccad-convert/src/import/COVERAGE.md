# Drawing-to-CadDocument assessment coverage

This describes the importer for IFCDR 0.9.0/0.10.0 and cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. It is a coverage declaration, not a
preflight scan. Changes to that importer must review this declaration.

The operation accepts one selected drawing with exactly one model layout and
zero or more bound paper layouts. An unbound paper scope is rejected by strict
package validation before conversion; absence of a layout reference is not
permission to silently discard that scope.
The package graph outside that drawing and IFCPR restoration are outside scope.

| Source content | Existing treatment and assessment |
| --- | --- |
| Length unit | All 25 tokens mapped to CAD codes 0–24; no coordinate rescaling |
| Local block definitions | All definitions, including unused ones, allocated before contents; name, base point, description, anonymous flag, insertion unit, explodability and signed-uniform policy retained. Consistent structural markers are created. |
| Drawing `plotStyleMode` | Color-dependent/named maps to cadcodec header `plotstyle_mode`; old 0.11 packages default to color-dependent. |
| Model and paper layout names, scope binding and order | The CAD model layout and named paper layouts are allocated before scope entities. Source layout names and paper owners are retained. The fresh CAD `Layout1` scaffold may remain unused. |
| Layout `limits`, `limitsChecking`, `paperSpaceLinetypeScaling` | Limits and flag bit 2 are mapped. Cadcodec stores PSLTSCALE once in its header; conflicting per-layout values receive `LayoutFieldUnsupported`. |
| Effective `plotSettings.media/area/mapping/output/options` | Millimetre/inch/pixel tokens, media dimensions/margins/rotation, all four supported plot areas, fixed/fit scale, offset/center, shading, active plot-style switch/name and supported flags map to pinned `Layout` fields. Printable-area-relative offsets and plot transparency have no exact target field and receive `LayoutFieldUnsupported`. A page setup name or CTB/STB contents cannot be reconstructed from the native inline value. |
| Paper `Viewport` frame, orthographic view, render and clip state | Mapped to a CAD VIEWPORT owned by the paper block. Perspective is skipped with `ViewportUnsupported` pending CAD fixture calibration; an unresolved active paper clip also skips the viewport. |
| Viewport frozen layers | Each relational frozen override maps to a CAD frozen-layer handle. Pinned cadcodec has no per-viewport appearance-override slots; those report `ViewportUnsupported`. |
| Block instances | Shared references retained without explosion; owner scopes and local child coordinates retained. Non-neutral frames are converted with explicit parameterization-loss evidence and occurrence-space accuracy checks. Setter scale changes are hard `BlockTargetLimitation`, even for empty definitions. |
| Line endpoints | XYZ copied directly; exact geometry assessment |
| Polyline points, plane and closed flag | Closed flag retained. Exact-compatible CAD parameterizations copy local points; other placements are transformed to the actual CAD arbitrary-axis basis and produce `PlaneParameterizationChanged`. Geometric residuals are checked independently. |
| Entity order | Inserted in each scope's logical order |
| Entity identity | New target handles; source-ID mapping retained in outcome |
| Layer reference, name, visibility | Mapped to CAD layer; entity visibility copied |
| ByLayer / ByBlock appearance | Modes mapped for color, opacity, pattern and weight |
| Explicit color | ACI 1–255 preferred when supplied; otherwise RGB; named metadata mapped where supported |
| Line pattern | Continuous/Dashed mapped; other patterns fall back with a grouped loss diagnostic |
| Line weight | Mapped to supported CAD weights; rounding emits a grouped loss diagnostic |
| Opacity | Converted to CAD transparency with upstream's upward byte rounding (0.5 opacity gives transparency byte 128); quantization fidelity is not assessed |
| Color metadata and appearance identity | No comprehensive fidelity assessment; unsupported indexed systems use RGB, and layer 0 updating does not copy all named-color metadata |
| Other drawing/layout/view state | Workspace, named view, page-setup sharing and complete native appearance overrides remain outside this slice and cannot be reconstructed without a diagnostic or a future target-model extension. |
| Bounds, allocation watermark, resource/table identities | No reconstruction guarantee; target storage and handles differ |

The importer therefore always reports `Incomplete` coverage within the selected
drawing scope. Fallback, parameterization and numerical rounding diagnostics establish `LossDetected`;
without them it reports `NotFullyAssessed`. An empty diagnostic list is not a
losslessness guarantee. The separate geometry assessment does not close these non-geometric coverage gaps.

Insertion failures and missing-reference/internal-invariant errors return
`ImportError`, not a completed conversion assessment. Neither a successful
package load nor a converter error claims that a CAD output was produced.

## Accuracy and policy

`drawing_to_cad_document_with_options` accepts `ImportOptions`; the existing
convenience function uses defaults. Both directions share `ConversionLossPolicy`
and `ConversionGeometryTolerance`. A known unit defaults to exactly one
micrometre; unitless defaults to exact geometry. Explicit physical tolerances
on unitless input are errors. `Allow` never bypasses accuracy or range failures.

Fixed exact operands are prepared once per non-direct polyline; direct CAD-compatible
placements do not prepare an unused projection. Preparation is local to that
polyline, with no shared placement cache. Target coordinates are rounded once
from exact affine expressions. Residuals
compare exact source geometry with the geometry of actual emitted CAD scalars,
including the axes obtained from the stored normal. Euclidean comparisons use
squared exact values; reported distances are outward bounds in drawing units.
Within-limit numeric rounding is the only exemption from `Reject`. A native
shifted or rotated frame can preserve geometry exactly and still be rejected
for parameterization loss. Numeric-only acceptance retains loss evidence.

`geometry_assessment()` covers emitted line endpoints and polyline vertices,
including each evaluated block occurrence and nested outer-scale amplification;
for supported straight segments their affine residual also bounds the interior.
It does not certify subsequent DXF/DWG writer behavior. `into_all_parts()` keeps
both assessments; legacy `into_parts()` retains its original tuple shape.

Real codec tests and limitations are recorded in
[block-cad-boundary.md](../../../../docs/geometry/block-cad-boundary.md).
In particular, DWG nonzero-base return conversion is blocked by contradictory
marker metadata (#52), and DXF drops block descriptions (#49). Neither is
silently repaired or treated as lossless by the converter.
