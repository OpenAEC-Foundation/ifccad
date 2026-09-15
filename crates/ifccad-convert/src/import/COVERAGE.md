# Drawing-to-CadDocument assessment coverage

This describes the existing importer for IFCDR 0.8.0 and cadcodec revision
`2f2cd25832db298524fb5eb36ced5a438a877e95`. It is a coverage declaration, not a
preflight scan. Changes to that importer must review this declaration.

The operation accepts one selected drawing with exactly one model layout.
Other layout structures return `UnsupportedDrawingStructure` without an output.
The package graph outside that drawing and IFCPR restoration are outside scope.

| Source content | Existing treatment and assessment |
| --- | --- |
| Length unit | Mapped to document units by units.rs |
| Line endpoints | XYZ copied directly; exact geometry assessment |
| Polyline points, plane and closed flag | Closed flag retained. Exact-compatible CAD parameterizations copy local points; other placements are transformed to the actual CAD arbitrary-axis basis and produce `PlaneParameterizationChanged`. Geometric residuals are checked independently. |
| Entity order | Inserted in logical drawing order |
| Entity identity | New target handles; source-ID mapping retained in outcome |
| Layer reference, name, visibility | Mapped to CAD layer; entity visibility copied |
| ByLayer / ByBlock appearance | Modes mapped for color, opacity, pattern and weight |
| Explicit color | ACI 1–255 preferred when supplied; otherwise RGB; named metadata mapped where supported |
| Line pattern | Continuous/Dashed mapped; other patterns fall back with a grouped loss diagnostic |
| Line weight | Mapped to supported CAD weights; rounding emits a grouped loss diagnostic |
| Opacity | Converted to CAD transparency with upstream's upward byte rounding (0.5 opacity gives transparency byte 128); quantization fidelity is not assessed |
| Color metadata and appearance identity | No comprehensive fidelity assessment; unsupported indexed systems use RGB, and layer 0 updating does not copy all named-color metadata |
| Scope name, base and flags; drawing/layout metadata | Not comprehensively transferred or assessed |
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

`geometry_assessment()` covers emitted line endpoints and polyline vertices;
for supported straight segments their affine residual also bounds the interior.
It does not certify subsequent DXF/DWG writer behavior. `into_all_parts()` keeps
both assessments; legacy `into_parts()` retains its original tuple shape.
