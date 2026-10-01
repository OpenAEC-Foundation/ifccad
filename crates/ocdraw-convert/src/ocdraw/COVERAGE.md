# Standalone OCDraw conversion coverage

Dependency: cadcodec/acadrust revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`, as pinned in Cargo.lock.

The source public-model classification in ../source/COVERAGE.md defines the pinned source classification. OCDraw is read through typed validated records;
encoding metadata and raw JSON are outside the CAD conversion boundary.

| Area | Standalone behavior |
| --- | --- |
| Geometry | LINE, POINT, CIRCLE, ARC, ELLIPSE, planar LWPOLYLINE/POLYLINE2D, spatial POLYLINE/POLYLINE3D; placement and active bulges retained |
| Blocks | Shared local definitions, base points, nested instances, signed scale, rotation and oblique placement; ownership is separate from the definition reference |
| Order | Mixed entity families follow each scope's stored drawing order |
| Layers/appearance | Explicit layer defaults; per-entity ByLayer/ByBlock/explicit values; local layer IDs and drawing-wide entity IDs |
| Layout/plot | Model and named paper layouts, plot units/settings/styles and flags; no extra bootstrap layout, including model-only sources |
| Saved state | Named/current UCS, active tiled model view, model windows, paper canvas views |
| Paper viewports | Frame, camera, clip, render mode, enabled/locked status, frozen local layers |
| Numeric accuracy | Hard tolerance, whole-curve checks and nested occurrence-space checks; structured assessment on both outcomes |
| Source coverage | Unsupported fields diagnosed, bootstrap/runtime scaffolding classified, inconsistent references rejected |

Perspective CAD viewport export awaits a calibrated pinned-codec fixture.
Viewport appearance overrides, authored shading quality, and viewport workspace
state have explicit target/source loss diagnostics where not supported.
Xrefs, external content, attributes/arrays/dynamic blocks, widths, unsupported
entity families, opaque vertex identities, and extended source metadata are
not silently approximated. Losses may be allowed with evidence or rejected.
No preservation transport exists in the current standalone contract.

Tiny block scales below the CAD setter threshold may fail conversion. The
known pinned DWG nonzero block-base marker inconsistency is a structural error,
not an accepted roundtrip. Guarded repairs for stale DXF modelspace handles and
anonymous DWG block names remain narrow and reject ambiguous structure.

Tests: ocdraw_conversion (typed production readback, mixed order, saved state,
oblique geometry and tolerances) and ocdraw_block_exchange (actual DXF/DWG IO).

The ordered-scopes fixture preserves model/paper/nested-block ownership and
order through direct CadDocument, actual DXF and actual DWG. Its single named
paper layout reuses the reserved primary paper block. Additional authored
paper layouts still expose conflicting DWG BLOCK marker identity/ownership
for `*Paper_Space0`: the pinned codec returns the conventional `*Paper_Space`
name and owner on that additional paper block. DXF preserves these layouts;
DWG return is rejected as inconsistent source structure. The layout regression
in `ocdraw_conversion` records both behavior and this remaining boundary.

Fractional and negative XY/XYZ polyline coordinates pass strict native
production readback and direct conversion. Fractional planar polylines also
have actual DXF/DWG readback. Source 3D polyline vertex handles/properties
remain subject to the existing source classification; ordinary codec-created
non-null vertex identities can trigger that explicit skip diagnostic.
