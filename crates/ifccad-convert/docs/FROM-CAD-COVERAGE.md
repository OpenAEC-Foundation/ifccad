# CadDocument → IFCCAD coverage

The logical `cad_document_to_ifccad_document` routes return an independently
validated typed document with diagnostics and mappings, without an IFCX byte
bridge. Existing encoded routes call them, then core profile encoding and
production loading. Coverage, loss acceptance, recoveries and fresh ID
allocation are shared. Encoding creates a new CAD-profile file; no source graph
writeback is performed.

Logical document validation returns `CoreValidation(IfccadReport)`; encoded
routes retain core encoder phases in `CoreEncoding(IfccadEncodeError)`.
Their additional production load returns `CoreReadback(IfccadReadError)`.
Checked allocation exhaustion returns `IdAllocation(IfccadIdAllocationError)`
with the affected domain. Payloads and error sources remain typed; these
categories do not change Allow/Reject or the supported source subset.

Pinned semantic inventory V1, opencadcodec
`fe69506cb99dea6f4c4a73b690a27fdf04403ea0`. Exhaustive matches classify every
inventory category. Serde residual checks cover fields plus typed comparisons
for skipped-serde common/marker/object state. Data not exposed by the codec has
no asserted coverage. Default Allow returns the supported subset with located
loss diagnostics; explicit Reject prevents output for any semantic loss.
The [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-05.md)
records the added public fields and their treatment in both converters.

The 2026-10-05 additions to associative subentity identifiers, edge curves and
embedded modeler-body profiles remain inside unsupported associative objects
and solid/surface/history families. They are diagnosed at those existing
boundaries. Unmodified upstream still loses viewport clipping activation and
the DXF boundary reference; the separate viewport development patch is explicit.

## Inventory categories

Fresh imports allocate entity, layer, layout, block-definition and pattern IDs
from 1 using the core's independent checked uint64 domains. The drawing stores
all five resulting next-ID watermarks, including empty domains. Allocation
exhaustion is fatal under Allow and Reject. Numeric IDs are not CAD handles or
table positions; mappings preserve source associations for this conversion.
Reimporting CAD does not restore a previous IFCCAD allocation history.

| Category | Treatment |
| --- | --- |
| Header | Map insertion units, linetype scale and plot-style mode; compare other settings/names with pinned default. Derived extents, seed and explicitly listed control/dictionary/name-cache handles are excluded in `source.rs` |
| Layers | Map identity/name/concrete appearance. Diagnose changed flags, descriptions, named colors, plotting/material/XREF metadata or other residual fields |
| Local BlockRecords | Map name/base/unit/ordered contents. Begin/end identity and reverse insert handles are structural/derived. Diagnose flags, description, preview, insert-count bytes, scaling/explodability restrictions and layout association |
| Model/Paper BlockRecords | Validate role/ownership; metadata including unit/base must remain default |
| LineType | Actual signed-length definitions, names/descriptions and unused records; canonical ByLayer/ByBlock are mode scaffolding. Whole text/shape pattern fallback to named empty under Allow, one Modified diagnostic per definition; invalid values/alignment/name-handle targets fail both policies |
| TextStyle/DimStyle/AppId/View/VPort/Ucs/Vx | Only default records, record handles normalized; additional/changed records diagnosed |
| Classes | Default definitions only; normalize derived numbering/instance counts and version metadata |
| Entities | Supported fields below; every other family diagnosed. Full inventory includes structural markers hidden by `entities()` |
| Layout objects | Unique bidirectional Model/Paper block links and named-root ACAD_LAYOUT dictionary membership; names and tab order mapped. Recover a missing/wrong-type derived layout-dictionary cache only through a unique consistent named relationship. Media dimensions are millimetres, independently of plot units; optional media, effective plot state, limits/checking and saved PSLTSCALE retained. Model viewport references resolve VPORT records; Paper references resolve same-owner VIEWPORT entities. Model view selection is diagnosed loss; exclude derived extents and structural handles. Other plot/UCS/layout fields diagnosed |
| Other typed/unsupported objects | Compare pinned scaffold by named dictionary roles and typed values; additional/modified/unsupported objects diagnosed |
| Summary/Preview | Changed summary or any preview diagnosed |
| Relationships | Entity/marker/Layout ownership and typed drawing references remain structural checks. Unresolved ownership of omitted non-Layout objects is located loss, including object kind and owner handle. Reactors/extension dictionaries, including unresolved endpoints, are diagnosed losses rather than global structural failures |
| Non-entity extended data | Accept only resolved layer-owned canonical `AcCmTransparency` with one Integer32 exactly duplicating mapped explicit transparency. All other records, including undecodable payloads, are diagnosed; no preservation in this slice |

Defaults come from fresh CadDocument for in-memory/DXF, and one cached
unmodified fresh-DWG write/read for DWG. Roles follow named dictionaries,
never handle coincidence/count alone. Standard style handle/name caches and
duplicate raw DXF style records are normalized beside their typed comparisons;
authored style fields remain compared. Source identity/reference collisions fail.
The Model layout's `plot_flags.model_type` is also derived role bookkeeping:
the DXF writer sets it from the Model name. Other plot flags remain diagnosed.

An overall viewport is excluded only for a Paper-layout-linked ID-1 viewport with
all constructor-default values apart from verified owner/identity/storage fields.
Authored view settings or dimensions fail that classification. The pinned DWG reader leaves non-active viewport numbers at zero; a linked, otherwise fully default viewport is also scaffold in that backing. Authored non-overall Paper viewports convert with the explicit development codec configuration as specified below; without its required status support they are omitted with a located viewport-codec diagnostic. Supported sibling entities and additional empty/populated layouts are retained.
This comparison includes the newly exposed `Viewport.off_screen`; true prevents
the viewport from being silently classified as default scaffold.

## Paper viewport conversion

This route requires the explicit codec configuration in
[`patches/opencadcodec-viewports`](../../../patches/opencadcodec-viewports/README.md).
Default manifests select unmodified upstream; their status-bit capability gate
omits authored viewports with `viewport-codec` loss instead of claiming support.
The development configuration preserves activation, DXF boundary references,
degree/radian angle mapping and the independent off bit. These repaired results
do not qualify the unmodified pin or certify AutoCAD rendering.

| VIEWPORT fields | Native treatment |
| --- | --- |
| Frame center/width/height | Paper XY coordinates and dimensions; nonzero frame Z omits the whole viewport |
| View center/target/direction/height/twist | DCS XY and Model XYZ, original non-unit target-to-camera vector, Model view height and radians; nonzero DCS Z or invalid native parameters omits the whole viewport |
| Perspective and lens length | Orthographic/Perspective plus stored millimetre lens, including dormant Orthographic zero |
| Front/back clips | Disabled, front AtCamera or AtDistance, back AtDistance; preserve signed exposed distances even when dormant; native plane-order validation applies |
| Render mode | Explicit mapping of all seven modes; numeric 4 is smooth without edges, 5 is flat with edges |
| On/off, zoom lock, common invisibility | Effective enabled = is_on and not bit 0x20000; locked and visible are independent; mapped invisibility is excluded from common residual loss |
| Nonrectangular activation and boundary | Bit 0x10000 and handle independently; three states: absent, dormant reference, active reference |
| Frozen layers | Resolve names/identities into native layer references; deduplicate and sort numeric IDs; unresolved targets are individually diagnosed and omitted |
| Runtime viewport number | Infrastructure role only, never a native identity; authored zero numbers remain authored unless the complete overall-canvas role applies |
| Snap/grid/UCS/plot/visual/off-screen and other state | Typed residual comparison against constructor defaults; changed unmapped fields produce partial viewport-state loss |

Ordinary geometry is prepared before viewports. A stored boundary must be retained
in the same Paper owner, even when dormant. Missing or unsupported boundary geometry
omits the whole viewport, retains supported siblings, and leaves no viewport ID
mapping. Active clips additionally require native eligibility and full frame
enclosure: analytic Circle, full Ellipse or closed straight/bulged planar path
in Paper Z=0. Spline clips remain unsupported. No rectangle fallback or tessellation is performed.
Contradictory boundary ownership or shared retained boundaries fail both policies.

Model and all Paper IDs are allocated before owner contents. After eligibility,
retained entities receive native IDs in original owner order, and forward boundary
references resolve through the complete retained map. Partial losses retain the
supported viewport under Allow; Reject refuses every diagnosed loss. Invalid
structural relationships remain errors under both policies.

Evidence: `tests/viewports.rs`, `tests/viewport_codec.rs` and
`tests/viewport_exchange.rs`, with production native/DXF/DWG readback. The
perspective reference uses independent arithmetic and hand-authored DXF values;
it is not an AutoCAD-produced interoperability sample.

## Entity fields

| Fields | Treatment |
| --- | --- |
| Common identity/owner/layer | Non-null unique identity; one ordered owner; existing layer; return mappings |
| Common appearance | Independent ByLayer/ByBlock/Explicit modes. True RGB, resolved named patterns, entity pattern scale and supported weights. Indexed/named color and Default weight diagnosed |
| Common linetype handle/entity mode/raw record/data-store flag | Typed-name/owner or encoding caches, not persisted separately; name/linetype handle agreement is validated |
| Other common fields | Typed residual check includes XDATA/raw EED, visibility, named colors, graphics, material/plot/shadow/visual styles, references, reactors and dictionaries |
| Line start/end | Direct XYZ; diagnose thickness/nondefault normal |
| Point/Circle/Arc/Ellipse | Position, radii and supported signed parameter spans in interpreted CAD planes; diagnose thickness and source normal normalization |
| LwPolyline/ordinary Polyline2D | Ordered vertices, outgoing/dormant bulges, closure, elevation and interpreted plane; map plinegen; diagnose widths, vertex metadata, thickness and unsupported fitted/mesh state |
| Ordinary generic/3D polylines | Straight XYZ vertices, closure and exposed generation; unsupported vertex/curve-fit metadata remains omission |
| Insert target/point/rotation/scales | Existing local target, valid interpreted plane and signed transforms with nested accuracy proof; diagnose attributes, arrays/MINSERT, spacing, view/sequence handles and other residual fields |
| BLOCK/ENDBLK | If present, record handle/type/name/owner/base must agree; diagnose delimiter/common changes. Missing DXF/programmatic markers are normal |

Unused definitions participate in cycle/target validation. Case-insensitive
duplicate layer/block names, duplicate/null handles and multiple owners fail.
A stale missing model cache is recovered only through unique consistent
BlockRecord/Layout agreement, with diagnostic; conflicting existing cache fails.

## Partial conversion policy

Unsupported families and incompatible geometry residuals omit the whole entity,
including bulges, widths, nondefault normals/thickness and Insert arrays or
attributes. Unsupported common metadata (for example XDATA or invisibility)
is omitted with a diagnostic while representable geometry is retained. No
flattening, block explosion or preservation is performed.

Normal local definitions retain their supported children in source order.
Reserved/anonymous names, XREF/external/unloaded flags or paths, and dynamic
objects directly owned by a definition cause definition omission along with
referring inserts. Each instance whose target loses content receives
`block-content-loss`, propagated through nested definitions. Metadata omissions
on a definition also participate in this propagation. Mappings cover emitted
objects only.

Indexed color is projected through opencadcodec's canonical ACI palette to RGB,
with index/context loss reported. Unavailable layer color becomes white;
inherited layer opacity becomes opaque; unavailable/default weight becomes
explicit 0.25 mm. Unsupported numeric weights use the closest standard CAD
weight. Complex text/shape definitions become empty under their original name and ID, including unused definitions; each has one `line-pattern-complex` modification. Finite source period disagreements are diagnosed and derived from elements. Entity ByLayer/ByBlock modes remain
stored. Unknown drawing or block unit codes become unitless without scaling,
with modification evidence. Unsupported table/header/object/source metadata is
diagnosed and omitted. No full-source preservation claim follows from Allow.

Both policies reject essential drawing reference/owner/marker conflicts, cycles,
invalid scalar values in known geometry and numeric precision failures. A
unique model-cache repair has action Recovery and does not count as a loss.
Unresolved ownership of unsupported objects and optional reactor/extension
relationships is omitted with loss diagnostics under Allow; Reject refuses that
loss. This does not repair the source or preserve the unsupported metadata.
Essential dictionary-entry and Layout/block consistency checks remain
in place. `tests/metadata_relationships.rs` covers the metadata/structure boundary
through logical and encoded conversion and production-reader readback.

Layout dictionary identity comes from the root named dictionary's unique
`ACAD_LAYOUT` entry, with matching dictionary map key, payload identity and root
ownership. Every layout must be owned by that dictionary and occur exactly once
under its actual name. A null, missing or wrong-type header cache is recovered
without mutating the caller's document, with `layout-dictionary-cache-recovered`
action Recovery. A cache resolving to a different existing Dictionary, ambiguous
or missing named targets, and actual membership/ownership conflicts remain fatal
under both policies. This also governs scaffold comparison, so cache recovery
does not produce a false dictionary-object loss or make Reject fail by itself.

A Model layout's last-active viewport and viewport-list references must resolve
VPORT table records, rather than Paper VIEWPORT entities. Their selection is not
stored by the native model: `model-viewport-selection` is located loss under
Allow and refused by Reject. Individual VPORT settings retain existing table
classification. Paper references still require same-owner VIEWPORT entities;
missing, wrong-kind or foreign targets remain structural errors. Tests in
`tests/layout_references.rs` cover both logical and encoded routes, source
immutability, relocated-dictionary DXF and Model-VPORT DWG readback, plus negative
identity, ownership, ambiguity and target-kind cases. These corrections do not
add Model view state, plot support or an overall-viewport identity heuristic.

Metadata is caller supplied. New IDs follow source table/owner enumeration and
remain u64; layer/definition vectors follow current core lexical path ordering.
Entity vectors retain owner order. Layout mappings use actual Layout handles;
definitions use BlockRecord handles. No handle or cross-file ID persistence claim.

Output goes through the production writer and strict reader. Tests cover large
IDs, different handles, unused definitions, cycles, missing references, tiny
scales, common-field loss and semantic DXF/DWG exchange, including nested/shared
nonzero-base DWG blocks after upstream #52 was fixed. `tests/loss_policy.rs` covers
partial owner order, missing mappings, nested loss propagation, explicit
appearance adaptations, foreign graph loss and the Allow/Reject boundary.

Line-pattern and layer/definition vectors follow lexical path ordering. Signed
lengths and drawing/entity scale values are included in exact projection checks.
Tests in `tests/line_patterns.rs` include real pinned DXF/DWG exchanges, optional
defaults, Unicode/target lookup collisions and whole complex-pattern fallback.

The shared codec development configuration is documented in
[patch provenance](../../../patches/opencadcodec-viewports/README.md). OCDraw and
IFCCAD retain independent native geometry and clip contracts. The
[2026-10-05 audit](../../../docs/geometry/opencadcodec-update-2026-10-05.md) separates
patched from unmodified upstream evidence.
## Paper layout conversion boundary

Native Paper names use full Unicode case folding; target CAD lookup additionally
rejects uppercase collisions before allocation. Source tabs must be distinct and
nonnegative, with Model at zero. Gaps normalize to contiguous Paper tabs with a
Recovery diagnostic, preserving relative order. Native IDs and watermarks are
independent of tab order. Only the fully default initial Layout1 scaffold is
excluded; additional empty sheets remain authored.

CAD dimensions, margins and offsets are millimetres independently of the plot
unit selector. Layout media are retained without complete plot settings. Effective
plotUnit/page/area/mapping/output/options and layout limits/checking/PSLTSCALE map
through typed fields; unsupported complete plot state is diagnosed with valid
media retained. Active scale selectors govern Fixed/Fit interpretation; conflicting
standard preset/factor values are not guessed. Unknown/raster/Fit mappings never
borrow Model units. Native Paper lengthUnit and paper spellings are removed.

Exact finite binary64 physical scalar conversion is required under both policies:
5 inches maps to 127 mm; an exact 1-inch-to-mm conversion fails with PlotNumeric.
Unsupported exact medium factors omit medium and dependent plot under Allow;
Reject refuses loss. Unqualified pixel calibration, printable-relative offsets,
transparency and active external style-table contents retain located restrictions.
Media never infer geometry rescaling or containment. Fully default CAD plot fields
canonicalize to absence; authored identical-default intent is indistinguishable.

Both routes expose per-Paper output-mapping assessments. Fixed physical output
resolves one-micrometre default accuracy per layout; unknown/Fit/pixel uses zero
coordinate residual. Explicit physical requests fail without fixed mappings,
including empty layouts and targets that lost their configuration. Definition-local
and signed/nonuniform nested-root proofs stay mandatory. Native/typed-CAD/file
exchange qualification and source-graph projection checks remain distinct.
See [layout output](../../../docs/layout-output.md) and tests/plot_settings.rs,
tests/plot_exchange.rs and tests/paper_plot_accuracy.rs.


## Geometry and accuracy expansion (provisional 0.1.0)

Point, Circle/Arc, full/partial Ellipse, straight/bulged LwPolyline and ordinary
Polyline2D, ordinary generic/3D spatial polylines now map through shared CAD
geometry preparation. Arbitrary valid CAD planes are interpreted through the
pinned helper; source normal normalization is separately diagnosed. Native
family identity, outgoing/dormant bulges, closure, pattern generation, owner
order and fresh uint64 mapping are retained. Widths/thickness, fitted/mesh flags,
nondefault per-vertex metadata and unsupported families retain explicit omission.
Zero/full/multiple-turn ARC and unsupported elliptic spans are not synthesized
as another kind. Invalid scalar values in every recognized family remain fatal.

The converter now has `geometry_tolerance`, shared with OCDraw: default one
micrometre in known units, zero for unitless, with exact/drawing-unit/physical
caller choices. Proven geometric rounding within the selected limit is reported
and accepted under Allow/Reject; exceedance and incomplete proof return typed
Geometry errors without output. Raw source-projection precision, semantic losses
and scale clamping remain distinct. Model/definition limits use drawing units;
Paper limits use the fixed physical output mapping, not sheet dimensions. Local definitions,
including unused ones, and every retained nested occurrence are assessed.
Full conics and bulged segment-circle enclosures supplement sampled witnesses.

Output explicitly prepares conservative optional scope bounds before strict
native validation/readback. CAD cached extents are not asserted as native bounds.
Active viewport boundaries additionally support full ellipses and closed bulged
planar polylines using the shared exact analytic Paper containment predicate.
Codec development-patch requirements and their unmodified-upstream boundary
above remain in force; these native/direct mappings do not certify arbitrary
CAD file serialization or application rendering.
