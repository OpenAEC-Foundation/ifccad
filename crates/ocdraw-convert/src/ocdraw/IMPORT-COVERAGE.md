# OCDraw to CadDocument coverage

The complete typed `OcdrawDocument` is the logical conversion boundary.
Logical export returns it with the existing loss/mapping/accuracy evidence;
encoded export delegates to the core encoder. Raw document import validates
before construction; validated reader import uses the same implementation.
These are fresh conversions: no identity-preserving CAD editing session,
watermark reconstruction or new source coverage is implied.

The validated standalone drawing is the conversion boundary. The pinned codec
revision is `5b682ed66ea2c89be8142c8dd83d83774fc3de08`. There is no package
graph or preservation transfer. Geometry accuracy and semantic losses are
reported separately; unknown target semantics are diagnosed.

| Source content | Existing treatment and assessment |
| --- | --- |
| Length unit | All 25 tokens mapped to CAD codes 0â€“24; no coordinate rescaling |
| Local block definitions | All definitions, including unused ones, allocated before contents; name, base point, description, anonymous flag, insertion unit, explodability and signed-uniform policy retained. Consistent structural markers are created. |
| Drawing `plotStyleMode` | Color-dependent/named maps to cadcodec header `plotstyle_mode`; omission defaults to color-dependent. |
| Drawing `pointDisplay` | Glyph/enclosure and tagged size map to cadcodec header PDMODE/PDSIZE. Absent display uses dot and CAD default five-percent size. |
| Model and paper layout names, scope binding and order | The CAD model layout and named paper layouts are allocated before scope entities. Source layout names and paper owners are retained. The first paper layout reuses and renames the fresh CAD scaffold; subsequent layouts are allocated in tab order. Without a source paper layout the scaffold layout and its dictionary entry are removed; the codec's reserved paper block/header remain, with no layout tab. An authored `Layout1` is retained like any other source name. |
| Layout `limits`, `limitsChecking`, `paperSpaceLinetypeScaling` | Limits and flag bit 2 are mapped. Cadcodec stores PSLTSCALE once in its header; conflicting per-layout values receive `LAYOUT_FIELD_UNSUPPORTED`. |
| Effective `plotSettings.media/area/mapping/output/options` | Millimetre/inch/pixel tokens, media dimensions/margins/rotation, all four supported plot areas, fixed/fit scale, offset/center, shading, active plot-style switch/name and supported flags map to pinned `Layout` fields. Printable-area-relative offsets and plot transparency have no exact target field and receive `LAYOUT_FIELD_UNSUPPORTED`. A page setup name or CTB/STB contents cannot be reconstructed from the native inline value. |
| Paper `Viewport` frame, orthographic view, render and clip state | Mapped to a CAD VIEWPORT owned by the paper block. Perspective is skipped with `VIEWPORT_UNSUPPORTED` pending CAD fixture calibration; an unresolved active paper clip also skips the viewport. |
| Viewport frozen layers | Each relational frozen override maps to a CAD frozen-layer handle. Pinned cadcodec has no per-viewport appearance-override slots; those report `VIEWPORT_UNSUPPORTED`. |
| Drawing workspace current Layer | A local layer ID selects the CAD header current layer. |
| Named UCS and model workspace | Named UCS definitions become CAD UCS table entries, including unused ones. Current World/named/unnamed model UCS and ordered model windows map to the header and active VPORT records, including dormant grid/snap values. CAD handles are newly allocated. Grid dot style or an out-of-range major frequency receives `WORKSPACE_UNSUPPORTED`. |
| Paper workspace | A present paper canvas view, grid, snap and stored UCS map to the layout's conventional overall VIEWPORT ID 1. The importer creates that viewport for a reused `Layout1` scaffold before authored viewports. Without a paper canvas, the importer removes cadcodec's newly allocated overall viewport scaffold so the layout remains viewport-free. A canvas's screen-sized frame and active viewport context are not reconstructed. Per-viewport workspace state is diagnosed as `WORKSPACE_UNSUPPORTED`. A saved active paper layout selects the CAD paper-space header block and clears `show_model_space`. |
| Block instances | Shared references retained without explosion; owner scopes and local child coordinates retained. Non-neutral frames are converted with explicit parameterization-loss evidence and occurrence-space accuracy checks. Setter scale changes are hard `CAD construction error`, even for empty definitions. |
| Line endpoints | XYZ copied directly; exact geometry assessment |
| Point placement | Origin becomes CAD WCS location; stored normal and X/Y orientation determine CAD normal and X-axis marker angle. The geometric position is assessed independently of presentation. |
| Circle and Arc | Centre is projected into CAD OCS using the selected arbitrary-axis frame. Positive Arc sweep maps directly; negative sweep flips the CAD normal and reparameterizes start angle so directed traversal is retained. Sample residuals locate proven errors; a conservative centre-and-axis bound also covers the entire parameter-matched curve and stored sweep rounding. |
| Ellipse and EllipseArc | Centre and major-axis vector map to CAD WCS, minor/major radii to CAD ratio. A negative EllipseArc sweep flips the CAD normal and negates the start parameter to preserve traversal. Full/partial kind maps to CAD Ellipse parameters; sample residuals and the entire parameter-matched curve are assessed separately. |
| PlanarPolyline vertices, bulges, plane and closed flag | Closed flag and every bulge, including the final dormant bulge of an open polyline, map to CAD LwPolyline. Exact-compatible CAD parameterizations copy local points; other placements are transformed to the actual CAD arbitrary-axis basis and produce `PARAMETERIZATION_CHANGED`. Vertex and active curved-segment midpoint residuals are checked independently. |
| SpatialPolyline XYZ vertices and closure | Directly maps to CAD Polyline3D with unchanged coordinates and closure. Source variant identity and unsupported CAD-only fit/mesh properties are not synthesized. |
| Entity order | Inserted in each scope's logical order |
| Entity identity | New target handles; source-ID mapping retained in outcome |
| Layer reference, name, visibility | Mapped to CAD layer; entity visibility copied |
| ByLayer / ByBlock appearance | Modes mapped for color, opacity, pattern and weight |
| Explicit color | ACI 1â€“255 preferred when supplied; otherwise RGB; named metadata mapped where supported |
| Line pattern | Every local named simple definition, including unused and named empty records, allocated before layers/entities; reference modes and global/entity scales retained. Polyline generation maps to CAD flags. |
| Line weight | Mapped to supported CAD weights; rounding emits a grouped loss diagnostic |
| Opacity | Converted to CAD transparency with upstream's upward byte rounding (0.5 opacity gives transparency byte 128); quantization fidelity is not assessed |
| Color metadata and appearance identity | No comprehensive fidelity assessment; unsupported indexed systems use RGB, and unsupported target color metadata is diagnosed |
| Other drawing/layout/view state | Named views, page-setup sharing and complete native appearance overrides remain outside this slice and cannot be reconstructed without a diagnostic or a future target-model extension. |
| Bounds, allocation watermark, drawing/table identities | No reconstruction guarantee; target storage and handles differ |

Coverage remains incomplete for the native semantics listed above. The outcome
contains semantic diagnostics, a separate numerical geometry assessment, and
source entity to target handle mappings. It has no aggregate fidelity grade.
An empty diagnostic list is not a losslessness guarantee: opacity quantization
and some presentation metadata are not comprehensively assessed.

Insertion failures, numerical failures and missing-reference/internal-invariant
errors return `DirectImportError`, not a completed conversion outcome. Successful
strict OCDraw readback does not itself establish successful CAD output.
