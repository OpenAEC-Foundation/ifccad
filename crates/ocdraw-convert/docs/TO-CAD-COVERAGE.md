# OCDraw to CadDocument coverage

The complete typed `OcdrawDocument` is the logical conversion boundary.
Logical export returns it with the existing loss/mapping/accuracy evidence;
encoded export delegates to the core encoder. Raw document import validates
before construction; validated reader import uses the same implementation.
These are fresh conversions: no identity-preserving CAD editing session,
watermark reconstruction or new source coverage is implied.

The validated standalone drawing is the conversion boundary. The pinned codec
revision is `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`. There is no package
graph. The bounded typed spline preservation route is separate from native
geometry support. Geometry accuracy and semantic losses are
reported separately; unknown target semantics are diagnosed.

| Source content | Existing treatment and assessment |
| --- | --- |
| Length unit | All 25 tokens mapped to CAD codes 0–24; no coordinate rescaling |
| Text styles | Qualified symbolic font selectors and creation metadata map before contents, including unused definitions. Unsupported face flags/context are diagnosed; dependent text is skipped as a whole. Missing last-height history becomes CAD's 2.5 default with a diagnostic. IDs/handles are conversion-local. |
| Text / MText | Separate qualified OCS/WCS preparation, literal-safe emission, supported layouts/decorations, paragraphs, stacks and explicit column/background state. Dynamic manual columns require a final native Auto entry, emitted as the qualified zero sentinel; a native fixed final cap is a whole-entity restriction. Mixed order remains authoritative. Unsupported active text features skip the whole entity under Allow; Reject refuses. Dependency/formatting normalization and inactive-state losses are diagnosed. |
| Local block definitions | All definitions, including unused ones, allocated before contents; name, base point, description, anonymous flag, insertion unit, explodability and signed-uniform policy retained. Consistent structural markers are created. |
| Drawing `plotStyleMode` | Color-dependent/named maps to opencadcodec header `plotstyle_mode`; omission defaults to color-dependent. |
| Drawing `pointDisplay` | Glyph/enclosure and tagged size map to opencadcodec header PDMODE/PDSIZE. Absent display uses dot and CAD default five-percent size. |
| Model and paper layout names, scope binding and order | The CAD model layout and named paper layouts are allocated before scope entities. Source layout names and paper owners are retained. The first paper layout reuses and renames the fresh CAD scaffold; subsequent layouts are allocated in tab order. Without a source paper layout the scaffold layout and its dictionary entry are removed; the codec's reserved paper block/header remain, with no layout tab. An authored `Layout1` is retained like any other source name. |
| Layout `limits`, `limitsChecking`, `paperSpaceLinetypeScaling` | Limits and flag bits 1/2 preserve linetype scaling/checking per layout. The current header follows the selected layout; differences between layouts are preserved. |
| Layout `media` and effective `plotSettings.plotUnit/page/area/mapping/output/options` | Independent medium dimensions and plot units, page margins/rotation, all four supported plot areas, fixed/fit scale, offset/center, shading, active plot-style switch/name and supported flags map to pinned `Layout` fields. Printable-area-relative offsets and plot transparency have no exact target field and receive `LAYOUT_FIELD_UNSUPPORTED`. A page setup name or CTB/STB contents cannot be reconstructed from the native inline value. |
| Paper `Viewport` frame, orthographic view, render and active clip state | Mapped to a CAD VIEWPORT owned by the paper block. Circle, full Ellipse and closed straight/bulged PlanarPolyline boundaries map through normal geometry conversion. Clip handles bind after ordered entity construction, allowing forward references without changing draw order. Perspective is skipped pending CAD fixture calibration. A required unconstructed boundary returns a typed construction error; no unresolved clipped viewport escapes as a rectangle. Locally patched DXF/DWG exchange preserves activation and references; the DWG profile includes an overall paper canvas. |
| Dormant paper clip reference | Stored reference binds after ordered geometry construction while activation remains false. Convertible dormant boundary families need no active-clip eligibility; missing construction mappings return a typed error. The locally patched codec retains this state through DXF/DWG. |
| Viewport frozen layers | Each relational frozen override maps to a CAD frozen-layer handle. Pinned opencadcodec has no per-viewport appearance-override slots; those report `VIEWPORT_UNSUPPORTED`. |
| Drawing workspace current Layer | A local layer ID selects the CAD header current layer. |
| Named UCS and model workspace | Named UCS definitions become CAD UCS table entries, including unused ones. Current World/named/unnamed model UCS and ordered model windows map to the header and active VPORT records, including dormant grid/snap values. CAD handles are newly allocated. Grid dot style or an out-of-range major frequency receives a field-specific `WORKSPACE` diagnostic. |
| Paper workspace | A present paper canvas view, grid, snap and stored UCS map to the layout's conventional overall VIEWPORT ID 1. The importer creates that viewport for a reused `Layout1` scaffold before authored viewports. Without a paper canvas, the importer removes opencadcodec's newly allocated overall viewport scaffold so the layout remains viewport-free. A canvas frame is copied independently of drawable geometry, and per-viewport grid/snap/UCS bind to the existing entity. Current Paper context/UCS association is diagnosed when authored. A saved Paper selection clears `show_model_space` and binds the chosen layout to the reserved `*Paper_Space` role, synchronizing block-record and BLOCK-begin names with the header cache while retaining handles, ownership and tab order. |
| Block instances | Shared references retained without explosion; owner scopes and local child coordinates retained. Non-neutral frames are converted with explicit parameterization-loss evidence and occurrence-space accuracy checks. Setter scale changes are hard `CAD construction error`, even for empty definitions. |
| Line endpoints | XYZ copied directly; exact geometry assessment |
| Point placement | Origin becomes CAD WCS location; stored normal and X/Y orientation determine CAD normal and X-axis marker angle. The geometric position is assessed independently of presentation. |
| Circle and Arc | Centre is projected into CAD OCS using the selected arbitrary-axis frame. Positive Arc sweep maps directly; negative sweep flips the CAD normal and reparameterizes start angle so directed traversal is retained. Sample residuals locate proven errors; a conservative centre-and-axis bound also covers the entire parameter-matched curve and stored sweep rounding. |
| Ellipse and EllipseArc | Centre and major-axis vector map to CAD WCS, minor/major radii to CAD ratio. A negative EllipseArc sweep flips the CAD normal and negates the start parameter to preserve traversal. Full/partial kind maps to CAD Ellipse parameters; sample residuals and the entire parameter-matched curve are assessed separately. |
| PlanarPolyline vertices, bulges, plane and closed flag | Closed flag and every bulge, including the final dormant bulge of an open polyline, map to CAD LwPolyline. Exact-compatible CAD parameterizations copy local points; other placements are transformed to the actual CAD arbitrary-axis basis and produce `PARAMETERIZATION_CHANGED`. Vertex and active curved-segment midpoint residuals are checked independently. |
| SpatialPolyline XYZ vertices and closure | Directly maps to CAD Polyline3D with unchanged coordinates and closure. Source variant identity and unsupported CAD-only fit/mesh properties are not synthesized. |
| Entity order | Inserted in each scope's logical order |
| Opaque spline content | RestoreSupported defaults to evaluating the persisted source snapshot and mandatory owner/unit/reference predicates. Eligible splines restore into fresh identities with authoritative current native layer/appearance/visibility and combined mixed scope order. Missing/unqualified context gets located evidence; Allow may skip, Reject refuses dropped live content. No curve evaluation, approximation, certified bounds or raw-record replay. |
| Entity identity | New target handles; source-ID mapping retained in outcome |
| Layer reference, name, visibility | Mapped to CAD layer; entity visibility copied |
| ByLayer / ByBlock appearance | Modes mapped for color, opacity, pattern and weight |
| Explicit color | ACI 1–255 preferred when supplied; otherwise RGB; named metadata mapped where supported |
| Line pattern | Every local named simple definition, including unused and named empty records, allocated before layers/entities; reference modes and global/entity scales retained. Polyline generation maps to CAD flags. |
| Line weight | Mapped to supported CAD weights; rounding emits a grouped loss diagnostic |
| Opacity | Converted to CAD transparency with upstream's upward byte rounding (0.5 opacity gives transparency byte 128); quantization fidelity is not assessed |
| Color metadata and appearance identity | No comprehensive fidelity assessment; unsupported indexed systems use RGB, and unsupported target color metadata is diagnosed |
| Other drawing/layout/view state | Named views, page-setup sharing and complete native appearance overrides remain outside this slice and cannot be reconstructed without a diagnostic or a future target-model extension. |
| Bounds, allocation watermark, drawing/table identities | No reconstruction guarantee; target storage and handles differ |

Coverage remains incomplete for the native semantics listed above.

The full native text model exceeds this qualified CAD subset. Independent MText
mirrors, TEXT strike-through, unqualified intra-paragraph line-break exchange,
advanced stack variants, decimal/local-spacing profiles, unsupported font/color
metadata, annotative contexts and nonopaque background transparency receive
whole-entity/profile restrictions. No plain-text fallback or text preservation
provider conceals those restrictions. Authored formatting presence/scope can
normalize in the CAD state machine; `TEXT_DEPENDENCY_CHANGED` exposes that change
and Reject refuses it. `text_assessment()` separates anchors from unassessed glyph
geometry. See [OCDraw text](../../../docs/text.md); empty diagnostics do not prove
font layout or every authored editing dependency.

Physical clipping and VIEWPORT angle exchange now use upstream repairs.
The [remaining local repair](../../../patches/opencadcodec-viewports/README.md)
preserves viewport-off state. Selected tests cover circles, rotated full
ellipses and supported straight/bulged planar paths in both draw-order positions,
negative/major bulges, nonzero twist and dormant stored references. Raw DXF
50/51 values are independently checked as degrees. Prior clipping-loss evidence
applies to the previous pin; application rendering remains a separate qualification.
Without a conventional overall paper canvas, pinned DWG readback classifies
the first authored viewport as the overall canvas. A characterization test
records the lost authored-viewport identity. The passing clip exchange profile
therefore requires that canvas; native files without one remain valid OCDraw.

The outcome contains semantic diagnostics, a separate numerical geometry assessment, and
source entity to target handle mappings. It has no aggregate fidelity grade.
An empty diagnostic list is not a losslessness guarantee: opacity quantization
and some presentation metadata are not comprehensively assessed.

Insertion failures, numerical failures and missing-reference/internal-invariant
errors return `OcdrawToCadError`, not a completed conversion outcome. Successful
strict OCDraw readback does not itself establish successful CAD output.

## Layout-output and Paper accuracy boundary

Model/Paper media-only values map independently of complete plots. Physical CAD
fields are millimetres and exact binary64 unit conversion is required; inexact
values return typed PlotNumeric errors under both policies. Native raster data
remain valid but CAD pixel calibration is unqualified and diagnosed. See
[layout output](../../../docs/layout-output.md) for canonical default-plot absence,
medium restrictions and public API migration.

Paper assessments use individual fixed output factors. Unknown/Fit/pixel mapping
uses 1e-9 coordinates by default; explicit physical requests fail without known
source/target mappings, including empty layouts, unless an independent coordinate fallback is explicitly requested. Signed/nonuniform nested-root
occurrences obey their root limit; aggregate status/counts remain, global maximum
does not. Preserved spline evidence stays incomplete and unassessed.

## Upstream pin update — 2026-10-07

Historical note: this records the previous base. The 2026-10-08 update below supersedes its pin and remaining spline/paperspace limitations.


Both converters use opencadcodec `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
Clipping activation/group 340 and VIEWPORT angle units now come from merged
upstream PRs #88/#89; only the independent viewport-off repair remains selected.
References above to clipping/angle defects of the previous unmodified pin are
historical evidence, not limitations of this new base. The [current dependency
audit](../../../docs/geometry/opencadcodec-update-2026-10-07.md) records new public
fields and compatibility decisions. Native spline semantics remain absent;
new OCDraw snapshots use payload v2 and retain v1 read/restore support. Fit-only
spline DXF parameterization remains a target-codec limitation pending PR #99.
Unresolved source layer handles must not silently become a native layer 0.
Known resolved handles are relationship identity and are rebuilt from native
layer references. Canonical one-byte-per-INSERT count framing is derived;
unfamiliar/mismatched count storage retains loss evidence. Additional table,
associative/count and solid-history data remain at the existing unsupported
family boundaries. No benchmark evidence is extended by this update.

## Workspace and upstream update — 2026-10-08

Both converters select opencadcodec 0.6.0 at `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` plus the explicit viewport-off repair [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). Merged spline DXF parameterization and DWG Paper owner/overall-role repairs now come from upstream. See the [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-08.md) and [workspace contract](../../../docs/workspace-state.md) for the current field/transport boundary.

UCS definitions, Model windows, canvas frame/grid/snap/UCS and authored Paper viewport aids map through shared ID-free scalar helpers. Native identities, ownership, source classification and located losses stay format-specific. Disabled zero snap spacing and stored-UCS activation survive independently of current choices. Model-viewport aids retain Model coordinates inside Paper; canvas aids use Paper coordinates. Canvas frames do not enter geometry bounds or clip ownership.

A uniquely available active Model window may be selected; multiple unqualified windows survive with unspecified activation. Paper current viewport/UCS association remains unavailable on the codec surface. Model/Paper mode and the active Paper tab are retained through the unique reserved *Paper_Space block and its consistent LAYOUT association, including multiple sheets. Export synchronizes BLOCK_RECORD names, existing BLOCK begin names and the reserved header handle together; setting the header cache alone does not change the active role. Unknown choices remain omitted with located loss. Skipped viewports receive no workspace references.

Dot grid style and frequencies beyond CAD i16 receive field-specific substitutions. VIEWPORT grid beyond-limits/adaptive/subdivision/follow-workplane flags stay in CadDocument but its pinned DXF/DWG routes do not retain them; target diagnostics identify each nondefault field and Reject refuses that portability loss. Model VPORT grid flags survive both routes. Unrepresented display/icon/base/orthographic/plot/visual state remains diagnosed. Numeric and structural failures remain fatal under both policies.
