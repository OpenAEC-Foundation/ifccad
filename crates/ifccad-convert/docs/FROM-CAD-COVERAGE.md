# CadDocument â†’ IFCCAD coverage

The active core package is `ifccad`, exporting its model/IO API directly. OCDraw
is parked and excluded from active verification. This ownership migration changes
no CAD coverage, geometry policy or source/target loss classification.

Standalone CBOR uses the same validated logical document boundary. Its direct
core mapping changes storage only; the IFCX source audit is not available from
a binary document. No CAD coverage or loss-policy expansion is implied.

The logical `cad_document_to_ifccad_document` routes return an independently
validated typed document with diagnostics and mappings, without an IFCX byte
bridge. Existing encoded routes call them, then core profile encoding and
production loading. Coverage, loss acceptance, recoveries and fresh ID
allocation are shared. Encoding creates a new CAD-profile file; no source graph
writeback is performed.

Core encoding may omit exact [native defaults](../../../docs/drawing-defaults.md)
and writes XY line endpoints when Z is +0. Production readback reconstructs the
complete logical document; supported CAD source semantics and loss classification
are unchanged by the compact representation.

Logical document validation returns `CoreValidation(IfccadReport)`; encoded
routes retain core encoder phases in `CoreEncoding(IfccadEncodeError)`.
Their additional production load returns `CoreReadback(IfccadReadError)`.
Checked allocation exhaustion returns `IdAllocation(IfccadIdAllocationError)`
with the affected domain. Payloads and error sources remain typed; these
categories do not change Allow/Reject or the supported source subset.

Pinned semantic inventory V1, opencadcodec
`bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`. Exhaustive matches classify every
inventory category. Serde residual checks cover fields plus typed comparisons
for skipped-serde common/marker/object state. Data not exposed by the codec has
no asserted coverage. Default Allow returns the supported subset with located
loss diagnostics; explicit Reject prevents output for any semantic loss.
The [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-23.md)
records the added public fields and their treatment in both converters.

The 2026-10-05 additions to associative subentity identifiers, edge curves and
embedded modeler-body profiles remain inside unsupported associative objects
and solid/surface/history families. They are diagnosed at those existing
boundaries. The previous pin lost viewport clipping activation and
the DXF boundary reference; the separate viewport development patch is explicit.

## Inventory categories

Fresh imports allocate entity, layer, layout, block-definition and pattern IDs
from 1 using the core's independent checked uint64 domains. The drawing stores
all resulting next-ID watermarks, including empty domains. Allocation
exhaustion is fatal under Allow and Reject. Numeric IDs are not CAD handles or
table positions; mappings preserve source associations for this conversion.
Reimporting CAD does not restore a previous IFCCAD allocation history.

| Category | Treatment |
| --- | --- |
| Header | Map insertion units, linetype scale and plot-style mode; compare other settings/names with pinned default. Derived extents, seed and explicitly listed control/dictionary/name-cache handles are excluded in `source.rs` |
| Layers | Map identity/name/concrete appearance, independent on/off, global freeze, lock, plottability, freeze-in-new-viewports and description. Complete concrete named identity is mapped; incomplete/inherited color names, material/XREF metadata and other residual fields remain classified separately |
| Header PDMODE/PDSIZE | Supported glyph/enclosure bits and finite default/absolute/view-relative sizes become native PointDisplay. Default zero remains distinct from explicit negative-five-percent. Unsupported bits/nonfinite size omit only the setting with located point-display loss; point geometry remains live |
| Local BlockRecords | Map name/base/unit/ordered contents, description, anonymous flag, explodability and signed-uniform policy, including unused ordinary anonymous definitions. Consistent begin-description duplicates are consumed; conflicting extra marker metadata retains loss evidence. Begin/end identity and reverse insert handles are structural/derived. Other flags, preview, unfamiliar insert-count bytes and layout associations remain diagnosed |
| Model/Paper BlockRecords | Validate role/ownership; metadata including unit/base must remain default. A layout-free reserved primary `*Paper_Space` record is codec bookkeeping only when its header role, default state and absence of drawable content or layout/INSERT references agree; it creates no Paper layout. Other orphan Paper records remain structural errors |
| LineType | Actual simple and qualified text/SHX definitions, including unused records, preserve signed lengths, parameters and relative/absolute/upright rotation. Ordinary and shape STYLE references resolve by role and handle; missing/wrong-role targets are fatal. Unsupported active dependencies replace the entire named definition under Allow with one Modified diagnostic; Reject refuses. Invalid periods/scalars and cache disagreements retain independent checks/diagnostics. |
| TextStyle | Qualified ordinary styles retain independent native identities. Qualified shape STYLE records become independent SHX sources, including anonymous records, duplicate filenames and unused resources. File paths retain basenames with one locator-loss diagnostic per record. Active unsupported shape properties and inactive source name/history metadata are diagnosed separately. Wrong-role text lookup and duplicate/nonfinite structures remain invalid or explicitly unsupported. |
| DimStyle/AppId/View/VPort/Ucs/Vx | Only default records, record handles normalized; additional/changed records diagnosed |
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
Activation, DXF boundary references and degree/radian angle mapping are now
upstream capabilities. The development configuration adds the independent off
bit; those off-state results do not qualify the unmodified pin or certify AutoCAD rendering.

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
| Common appearance | Independent ByLayer/ByBlock/Explicit modes. Concrete RGB/ACI identity, complete named identity, resolved patterns, entity pattern scale and nonnegative weights are retained. Default weight uses the qualified effective 0.25 mm convention. Unrepresentable required color/weight omits the dependent item, not an invented white/default appearance |
| Common linetype handle/entity mode/raw record/data-store flag | Typed-name/owner or encoding caches, not persisted separately; name/linetype handle agreement is validated |
| Common visibility | Native common visible maps independently of layer status and viewport on/off/lock; supported hidden geometry, instances, Text/MText and viewports retain ownership/order and geometry bounds |
| Other common fields | Typed residual check includes XDATA/raw EED, unmapped/incomplete color metadata, graphics, material/plot/shadow/visual styles, references, reactors and dictionaries |
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
attributes. Unsupported common metadata (for example XDATA)
is omitted with a diagnostic while representable geometry is retained. No
flattening, block explosion or preservation is performed.

Normal local definitions retain their supported children in source order.
Reserved CAD Model/Paper roles, XREF/external/unloaded flags or paths, and dynamic
objects directly owned by a definition cause definition omission along with
referring inserts. Each instance whose target loses content receives
`block-content-loss`, propagated through nested definitions. Metadata omissions
on a definition also participate in this propagation. Mappings cover emitted
objects only.

Concrete colors retain RGB, qualified ACI and complete named identity. Unavailable
required layer/entity color or weight omits the dependent item with located loss;
there is no white fallback. Layer ByLayer opacity uses the qualified opaque
constructor convention; Layer ByBlock is not a concrete opacity. Default weight
uses effective 0.25 mm, while explicit nonnegative source weights remain native
values; standard-table quantization occurs on CAD output with loss. Complex text/shape definitions become empty under their original name and ID, including unused definitions; each has one `line-pattern-complex` modification. Finite source period disagreements are diagnosed and derived from elements. Entity ByLayer/ByBlock modes remain
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
under its actual name, matched using ASCII-case-insensitive CAD dictionary lookup.
Duplicate membership or colliding case variants (including keys targeting another
object) remain fatal; Unicode case folding is not introduced. A null, missing or wrong-type header cache is recovered
without mutating the caller's document, with `layout-dictionary-cache-recovered`
action Recovery. A cache resolving to a different existing Dictionary, ambiguous
or missing named targets, and actual membership/ownership conflicts remain fatal
under both policies. This also governs scaffold comparison, so cache recovery
does not produce a false dictionary-object loss or make Reject fail by itself.

A Model layout's last-active viewport and viewport-list references must resolve
VPORT table records, rather than Paper VIEWPORT entities. Native Model windows and optional current selection now exist. The extra
layout-cache selection/list relation remains unqualified and receives
`model-viewport-selection` loss; independent active VPORT settings are retained. Paper references still require same-owner VIEWPORT entities;
missing, wrong-kind or foreign targets remain structural errors. Tests in
`tests/layout_references.rs` cover both logical and encoded routes, source
immutability, relocated-dictionary DXF and Model-VPORT DWG readback, plus negative
identity, ownership, ambiguity and target-kind cases. The workspace update below adds Model state and qualified overall roles without
turning unqualified cache pointers into current selections.

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
[2026-10-05 audit](../../../docs/geometry/opencadcodec-update-2026-10-23.md) separates
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

Both routes expose per-Paper output-mapping assessments. Default is exactly 1e-9
coordinate per domain. Explicit physical requests fail without fixed mappings
unless an independent coordinate fallback is explicitly supplied,
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

The converter now has `geometry_tolerance`, shared with OCDraw: default exactly
1e-9 in each coordinate domain, with exact/drawing-unit/physical
caller choices. Proven geometric rounding within the selected limit is reported
and accepted under Allow/Reject; exceedance and incomplete proof return typed
Geometry errors without output. Raw source-projection precision, semantic losses
and scale clamping remain distinct. Model/definition limits use drawing units;
Explicit physical Paper limits use the fixed output mapping, not sheet dimensions. Local definitions,
including unused ones, and every retained nested occurrence are assessed.
Full conics and bulged segment-circle enclosures supplement sampled witnesses.

Output explicitly prepares conservative optional scope bounds before strict
native validation/readback. CAD cached extents are not asserted as native bounds.
Active viewport boundaries additionally support full ellipses and closed bulged
planar polylines using the shared exact analytic Paper containment predicate.
Codec development-patch requirements and their unmodified-upstream boundary
above remain in force; these native/direct mappings do not certify arbitrary
CAD file serialization or application rendering.

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

## Bounded opaque SPLINE preservation

An independent opt-in IFCCAD typed-spline pilot now captures all public Spline
variants/common fields before native geometry admission. Core transport has its
own envelopes/opaque sum type and strict IFCX storage; only the audited codec byte
DTO is shared through cad-preservation. Native common availability is exact and
optional. Capture defaults Disabled; RestoreSupported defaults on export, while
Skip diagnoses omitted live content. Source ownership conflicts remain fatal;
unsupported valid owners may retain detached archives with independent placement loss.
Native fields remain authoritative; original baselines are never refreshed.

IFCX readback canonicalizes preservation records by numeric ID, including mixed
ID lengths; record collection order is independent of opaque/native entity draw
order. Multi-record capture passes the same strict semantic readback as a single
record. See `tests/spline_record_order.rs` for 10/121-spline capture, both public
CAD export routes and physical DXF readback.

Restore checks actual mandatory predicates, provider/payload versions, source/body
identity and required constructed references, including forward typed XDATA.
Model/block units and block insertion unit are guarded; Paper mapping uses exact
reduced rational physical meaning, with explicit Unknown. Unsupported attached/raw
context stays stored and is not replayed. Numeric evidence remains unassessed for
opaque curves and occurrences, and complete bounds are absent. Source IFCX foreign
content still receives graph-aware loss; no writeback is added.

Production save/reopen and native editing are tested, including actual DXF and
AC1032 DWG plus filesystem close/reopen. Literal source/control-point assertions
are independent of the direct codec chain. On 063c106, fit-only parameterization
1/2 restores exactly in typed CAD and DWG, but DXF readback returns 0; application
physical readback reports TARGET_CODEC_SPLINE_PARAMETERIZATION_LOSS separately.
No #99 local repair, new pin, viewer update or raw_record replay is applied.
See the active experimental contract and docs/preservation.md for the full boundary.

## Text and MText

The IFCCAD adapter independently imports ordinary Text/MText through `cad-text`.
Style names resolve to qualified ordinary roles; anonymous SHX shape resources
do not enter the ordinary lookup namespace. Supported styles preserve name,
font-file/TrueType/big-font requests, width, shear, vertical and creation-mirror
properties, fixed-height omission for source zero and explicit last-used zero.
Unused definitions and ordered Model/Paper/local-block content remain present.
Current-style name/settings outside this model remain header loss. No implicit
native Standard style or font substitution is introduced.

Text represents active OCS anchors, layouts, rotations/mirrors, shear, signed
thickness and literal decorated runs. MText represents WCS anchors/direction,
nominal height, attachment/flow, wrap or columns, paragraph/character overrides,
closed inlines and qualified background state. Cached extents are not glyph
proof. Manual column final finite cache values map to Auto; earlier heights stay
positive. A malformed source/reference/scalar remains fatal before annotation or
unsupported-style omission; unsupported active layout/content skips the whole
entity under Allow and Reject refuses located loss. Generic common metadata
continues to receive its independent residual classification.

Finite paragraph factors may exhaust or exceed remaining wrap/column space,
including first-line space. They retain native authored overflow state rather
than triggering invalid-number rejection or whole-text omission. Positive
height/explicit width and finite products/derived arithmetic remain required.
CAD paragraph numbers remain initial-height factors; they are not guessed to be
raw distances. Glyph/layout fidelity remains unassessed.

IFCCAD accepts exactly version-1 `AcadAnnotative` flag 0 as inactive ordinary
text. Active/unknown context data, annotative styles and MText's active annotative
flag remain unsupported. Remaining XDATA method data is still common-metadata
loss. Dynamic fields, attributes, advanced stack/spacing/decimal-tab settings and
unqualified line-break file semantics keep the restrictions in `cad-text`.
Native rich values are broader than this CAD subset. Inline/background concrete RGB and ACI identity use IfccadColor; unqualified named/custom text colors remain whole-text restrictions.

Layer typed ByLayer means the codec's opaque default and maps to opacity 1.
Known raw `AcCmTransparency` integer 0 is a duplicate of that default; disagreeing
or unknown raw tags remain XDATA loss. Layer ByBlock is unrepresentable required appearance; the layer and dependent items are skipped with located loss. Entity inheritance is independent. Unknown raw tags already collapsed
by the codec cannot be reconstructed from the typed mode alone; see upstream #104.

`text_assessment()` reports active Text anchors or MText WCS anchors and separate
unassessed glyph coverage. Geometry assessment is incomplete for nonempty glyphs,
including unused definitions and occurrences. Estimates never certify contours.
Production strict native readback and actual DXF/AC1032 DWG tests cover simple
Text/MText in all three owners, explicit paragraph RGB, Unicode/trailing empty
paragraphs and manual Auto-tail columns. File angle conversion may round the last
bit; native storage remains exact. These tests use the pinned ab2eecd base with the explicit
viewport-off development repair and do not establish every font or file version.

## Workspace and upstream update — 2026-10-08

Both converters select opencadcodec 0.6.0 at `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` plus the explicit viewport-off repair [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). Merged spline DXF parameterization and DWG Paper owner/overall-role repairs now come from upstream. See the [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-08.md) and [workspace contract](../../../docs/workspace-state.md) for the current field/transport boundary.

UCS definitions, Model windows, canvas frame/grid/snap/UCS and authored Paper viewport aids map through shared ID-free scalar helpers. Native identities, ownership, source classification and located losses stay format-specific. Disabled zero snap spacing and stored-UCS activation survive independently of current choices. Model-viewport aids retain Model coordinates inside Paper; canvas aids use Paper coordinates. Canvas frames do not enter geometry bounds or clip ownership.

A uniquely available active Model window may be selected; multiple unqualified windows survive with unspecified activation. Paper current viewport/UCS association remains unavailable on the codec surface. Model/Paper mode and the active Paper tab are retained through the unique reserved *Paper_Space block and its consistent LAYOUT association, including multiple sheets. Export synchronizes BLOCK_RECORD names, existing BLOCK begin names and the reserved header handle together; setting the header cache alone does not change the active role. Unknown choices remain omitted with located loss. Skipped viewports receive no workspace references.

Dot grid style and frequencies beyond CAD i16 receive field-specific substitutions. VIEWPORT grid beyond-limits/adaptive/subdivision/follow-workplane flags stay in CadDocument but its pinned DXF/DWG routes do not retain them; target diagnostics identify each nondefault field and Reject refuses that portability loss. Model VPORT grid flags survive both routes. Unrepresented display/icon/base/orthographic/plot/visual state remains diagnosed. Numeric and structural failures remain fatal under both policies.

## Hatch field coverage

| Field/state | Classification and native result |
| --- | --- |
| elevation/normal, all stored supported paths | Contour conversion assessed against the actual CAD OCS, including nested occurrences; finite/parameter/join failures are hard errors |
| is_solid/style | Solid/explicit LinePattern/qualified Gradient plus exact normal/outer/ignore; MPOLYGON and unqualified active fill profiles omit the whole Hatch with SkippedLoss/Omitted |
| line/circular/elliptic edges, closed straight/bulged polyline | Every loop retained; exact full-turn qualification, signed sweeps, no snapping/tessellation; spline/open/mixed-polyline backing unsupported |
| is_associative/boundary_handles | Ordered whole-object source groups and independent enabled/disabled/unbound switch; same-owner Line/Circle/Arc/Ellipse/Polyline/Text/MText/BlockInstance/Paper Viewport. Order/duplicates retained after allocation. A missing/skipped/wrong-owner/unsupported entry omits its entire group under Allow with entry/group loss, never its contour; Reject refuses |
| path flags | POLYLINE is backing-derived; EXTERNAL/DERIVED/OUTERMOST/TEXTBOX retained as closed provenance. TEXT_ISLAND retained as independent active text-loop fill classification; no nesting/font inference. NOT_CLOSED/SELF_INTERSECTING/DUPLICATE/unknown flags omit whole Hatch |
| pattern name/description/lines/type/angle/scale/double | Solid inactive state diagnosed; explicit effective families, literal metadata, origin/angle/scale retained for Pattern. Library/type dependency diagnosed; UserDefined requires a resolved fixed local continuous linetype and one continuous family; ByLayer layer-0 block dependencies are unqualified. Double only the documented axis-aligned origin-anchored profile, otherwise whole-Hatch omission |
| seed points/pixel size, inactive MPOLYGON fields | Located editing/construction-state loss when nondefault |
| common attachments/context | AcadAnnotative markers and non-null extension dictionaries require separate qualification and omit whole Hatch; qualified top-level ACAD origin is consumed once, nested/unrelated metadata uses existing format-specific audit; no opaque Hatch preservation; only inverse reactors confirmed against complete qualified Hatch source groups are accounted; unrelated attachments remain loss |

Producer join requests use local units or a known physical mapping with explicit
fallback policy. They do not change the hard conversion accuracy budget.
Actual AC1032 DXF/DWG tests qualify stored contour/area/reference data, including
hole-first ordering, multiple regions, tilted planes and nonuniform blocks.
Fill evaluation and interactive reactor updates remain unassessed; this is not
a rendered-pixel guarantee. See [Hatch support](../../../docs/hatch.md).
## IFCCAD validation rule organization

Native output and input continue through the production reader and shared domain
validation. The [profile rules](../../../schemas/ifccad/profile-rules-0.1.0.md)
link ordinary IFCX declarations with bundled local-value supplements and precise
graph/calculation requirements. This organization does not widen the supported
CAD subset or change Allow/Reject, numerical limits, identity or source retention.
## Presentation parity — 2026-10-08

See the [presentation contract](../../../docs/presentation.md) for independent
native color metadata, common/layer state, PointDisplay, ordinary block metadata,
exact transparency bytes, standard-weight quantization, relational viewport
rows and mode-only ShadePlot. Unsupported indexed metadata preserves authored
RGB with loss; unrepresentable required source appearance never invents white.
Semantic quantization remains independent of geometric tolerance and Reject.

Known RGB/ACI, alpha, pattern and explicit-weight sections map with format-owned
reference resolution. Equal duplicates are consumed; contradictory values are
invalid structure. Unknown/incomplete/mixed XRecord content remains classified;
only fully consumed qualified containers are exempted. Source/current IDs are
never copied as target handles. Skipped viewports emit no override/workspace refs.
Named/custom override identity stays unqualified and diagnosed.

Orthographic and perspective authored Paper cameras are qualified through strict
native loading and actual DXF/AC1032 DWG readback with independent hand-authored
camera/landmark constants. The physical profile establishes the overall Paper
canvas separately; no active Paper choice or external renderer accuracy is inferred.
Viewport ShadePlot maps independently of render mode and layout quality. Memory
and DWG mode values 0–3 are qualified; DXF group 170 remains gated by codec PR
#107, with outgoing browser readback loss reported explicitly. Paper grid flags
remain gated by #106. Both PRs were still open at qualification, so the audited
ab2eecd plus explicit viewport-off configuration remains selected.

## Ordinary Dimension slice — 2026-10-13

Linear and aligned dimensions now have an independent native conversion route.
Named DIMSTYLE records, including qualified unused styles, bind local textstyle,
marker-block and pattern references. The neutral helper's complete public-field
[audit](../../../docs/dimension/cad-field-audit-2026-10-09.md) is normative for this
bounded adapter profile. Successfully mapped styles and DIMASSOC objects are
removed from generic unsupported inventory; every residual field/object keeps
its existing diagnostic contract. Default unused bootstrap styles remain scaffold.

DIMLTEX/DIMLTEX1/DIMLTEX2 null or canonical ByBlock references inherit the
Dimension entity's line pattern; canonical ByLayer references retain native
ByLayer. Names use ASCII-case-insensitive matching after resolving the actual
linetype handle. Ordinary linetypes remain explicit references; dangling and
wrong-kind handles remain fatal, including in unused styles. Native outer
occurrence ByBlock is a distinct mode. CAD output's existing restriction on
component ByLayer patterns remains a located policy-controlled loss; this import
fix does not qualify that output or add DSTYLE pattern-override coverage.

Stored definition points and explicit text anchors use the existing hard
owner/occurrence tolerance. Measurements are recomputed from the definition;
CAD cache disagreement is located evidence. Display, fonts, extension lines,
markers, picture regeneration and host automatic update remain unassessed.
Conversion exposes separate dimension assessments and style mappings.
Qualified ACAD DSTYLE preserves sparse false/zero/empty/equal resets. Invalid
structure or numeric excess fails under both policies; unsupported active profile
fields skip the affected style/entity with diagnostics, never default geometry.

Single-object same-owner line start/end, circle center and point position
relations bind after entity allocation, including forward references. Definition
points never synchronize from their sources. Unsupported selectors/owners or
unrepresented sources detach with located loss, retaining the stored definition.
Native full-width identities remain exact; CAD handles are fresh conversion IDs.

Native type overrides can materialize as complete CAD $0/$1 substyles with
explicit inheritance/presence loss. Reimport retains complete named styles;
no exact sparse parent hierarchy is claimed. Per-entity arrow flips and line
spacing materialize inherited dependencies. Fixed-height font dependencies,
aligned oblique extensions, annotation contexts and more complex marker,
background, tolerance or dynamic-field profiles remain unqualified.

Actual DXF and AC1032 DWG tests compare definition/style/override/text/relation
semantics after production native readback. They do not establish full CAD-host
fidelity. Output has no generated anonymous picture block; visibility requires
host regeneration. The pinned DXF clone reader loses group 12, DXF omits flips
and DIMSTYLE true colors, and epsilon omission limits certain near-default values.
Other source picture definitions remain independently audited, never discarded
by anonymous-looking names. See [Dimension support](../../../docs/dimension.md).

## Foreground / ACI 7

Native Foreground is a symbolic background-dependent black/white color, with no
RGB fallback or separate ACI identity. Qualified ACI 7 imports as Foreground
before palette lookup and exports as ACI 7, retaining CTB key 7. Literal RGB
white and black remain true color. The old concrete `indexedColor: ACI/7` native
object is invalid; current provisional examples and candidate fixtures use the
closed `{"kind":"Foreground"}` choice.

This applies to layer/entity colors, qualified Text/MText inline and explicit
background colors, and C3/index-7 viewport-layer overrides. Named metadata on
source ACI 7 cannot attach to Foreground and gets located loss under Allow;
Reject refuses the loss. Editable native common appearance on preserved entities
supports Foreground while provider payload bytes remain unchanged.

The [independent exchange tests](../tests/foreground_exchange.rs) exercise native
production readback and actual DXF/AC1032 DWG readback for this route, with fixed
RGB controls. These tests prove data exchange, not external-app visual/CTB-output
certification. Additional C5/ByPen/None/257 methods and broader plot-style contents
remain unqualified. See [presentation](../../../docs/presentation.md#foreground-color)
and [issue #16](https://github.com/OpenAEC-Foundation/ifccad/issues/16).

Marker footprints are unassessed per affected native curve and block occurrence, alongside the numerical host-curve proof. External font/SHX availability is not probed or diagnosed as semantic loss. Producer bounds claims remain distinct from derived Partial/Estimated evidence.

AC1032 complex DXF/DWG qualification uses `tests/complex_line_pattern_exchange.rs` and the independently authored `tests/fixtures/line-patterns/complex-literal-ac1032.dxf` at the unmodified pinned dependency. STYLE roles, filename identities, marker values, relative/absolute flags and degree/radian conversion are checked semantically. Native upright mode retains the exclusive selector and angle for both text and symbols. Conflicting absolute/upright selectors and unknown flags remain unsupported. Physical DXF before AC1027 loses upright in the selected codec; Explorer preflight and physical readback report that loss and Reject refuses it. Model conversion itself retains upright independently of the eventual file target.

## Gradient field coverage

Enabled gradients override classic fill flags before pattern qualification.
Nine standard names map to native type; values 0/1 and two RGB/ACI colors,
angle, shift, one/two mode and tint are retained independently. Disabled
known settings survive as partial/complete native state; fully default means
absence. Unknown type/reserved/positions/extra colors or inherited endpoint
colors omit active Hatch or only dormant state under Allow. Reject refuses.
Missing active type/colors, one-entry pairs and nonfinite values are hard errors.
Source ACAD pattern origin is inactive and remains common-audit loss, not an anchor.

Original count, malformed-value, RGB/ACI-order and color/book identity information
may already be unavailable in pinned file readback; coverage starts at exposed
CadDocument values. Actual AC1032 tests and codec characterization are separate;
legacy recovery, arbitrary fill fields/pixels and editor updates are unqualified.

Gradient endpoints stay concrete after the Foreground color migration. Source
ACI 7 endpoints are unqualified (active whole-Hatch or dormant-state omission
under Allow; Reject refuses). Native Foreground endpoints fail validation;
no fixed background or RGB replacement is invented.


## Upstream pin update — 2026-10-16

The current exact codec pin is `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`
(upstream package version remains 0.6.0), with only the explicit
[Dimension text-position patch](../../../patches/opencadcodec-dimension/README.md).
Viewport off-state #103, paper grid flags #106, DXF ShadePlot #107 and gradient
state #109 are upstream. Paper VIEWPORT grid behavior now survives DXF/DWG;
its former target-loss diagnostics are removed. ShadePlot modes 0–3 survive
both routes, with physical readback still detecting unexpected loss.

The [public-model audit](../../../docs/geometry/opencadcodec-update-2026-10-16.md)
records added fields on existing types. Spline captures use payload v3 to retain
R12 common metadata, while audited v1/v2 sources remain readable. R12 viewport
context is retained in snapshots but is not eligible for typed restoration;
ordinary conversion diagnoses it. Packed nondefault TrueType flags, unknown
complex-linetype flag bits and named gradient endpoint identity are explicitly
unqualified. Additional header, viewport, object and unsupported-entity fields
remain subject to source loss classification. No native format semantics or
measurement claims are expanded by this dependency update.

### Exact coincident planar segments

Both vertices and their finite signed outgoing bulges remain stored, including
active and dormant values, closure, placement/elevation and pattern generation.
Exactly coincident endpoints describe a point-shaped segment; no deduplication,
bulge reset/reassignment or proximity epsilon is used. A nonzero source bulged chord
newly collapsed by target construction remains a hard error under either loss
policy and any tolerance. Contour eligibility counts only nondegenerate arcs.
See [the shared contract](../../../docs/geometry/coincident-polyline.md).
## SOLID/TRACE CAD import

Both native kinds retain four ordered finite corners, signed thickness and the existing frame. Crossing, repeated, overlapping and collapsed slots are supported. Unequal CAD OCS-Z remains outside the planar mapping; scalar/frame failures stay separate from semantic loss. Eight mandatory base/top position pairs, optional common surface cells and unassessed host display are distinct. ASCII/binary DXF and AC1032 DWG type identity and production strict readback are qualified in `filled_exchange`; no polyline-width or Dimension-graphics coverage is implied. See the [converter contract](../README.md#native-solid-and-trace).
