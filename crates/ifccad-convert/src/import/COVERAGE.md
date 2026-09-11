# Drawing-to-CadDocument assessment coverage

This describes the existing importer for IFCDR 0.7.0 and cadcodec revision
`a0f7d444f1607bc4b2c881060cbe7ea1014253cb`. It is a coverage declaration, not a
preflight scan. Changes to that importer must review this declaration.

The operation accepts one selected drawing with exactly one model layout.
Other layout structures return `UnsupportedDrawingStructure` without an output.
The package graph outside that drawing and IFCPR restoration are outside scope.

| Source content | Existing treatment and assessment |
| --- | --- |
| Length unit | Mapped to document units by units.rs |
| Line endpoints | XY copied directly; target Z is zero |
| Polyline points and closed flag | Copied directly to LwPolyline |
| Entity order | Inserted in logical drawing order |
| Entity identity | New target handles; source-ID mapping retained in outcome |
| Layer reference, name, visibility | Mapped to CAD layer; entity visibility copied |
| ByLayer / ByBlock appearance | Modes mapped for color, opacity, pattern and weight |
| Explicit color | ACI 1–255 preferred when supplied; otherwise RGB; named metadata mapped where supported |
| Line pattern | Continuous/Dashed mapped; other patterns fall back with a grouped loss diagnostic |
| Line weight | Mapped to supported CAD weights; rounding emits a grouped loss diagnostic |
| Opacity | Converted to CAD transparency; quantization fidelity is not assessed |
| Color metadata and appearance identity | No comprehensive fidelity assessment; unsupported indexed systems use RGB, and layer 0 updating does not copy all named-color metadata |
| Scope name, base and flags; drawing/layout metadata | Not comprehensively transferred or assessed |
| Bounds, allocation watermark, resource/table identities | No reconstruction guarantee; target storage and handles differ |

The importer therefore always reports `Incomplete` coverage within the selected
drawing scope. Existing fallback/rounding diagnostics establish `LossDetected`;
without them it reports `NotFullyAssessed`. An empty diagnostic list is not a
losslessness guarantee. This reporting change adds no new fidelity checks or
conversion behavior.

Insertion failures and missing-reference/internal-invariant errors return
`ImportError`, not a completed conversion assessment. Neither a successful
package load nor a converter error claims that a CAD output was produced.
