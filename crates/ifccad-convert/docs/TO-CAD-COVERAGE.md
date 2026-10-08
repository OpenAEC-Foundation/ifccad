# IFCCAD → CadDocument coverage

The projection-only `ifccad_document_to_cad_document` routes validate typed
input before using the shared CAD conversion implementation. They do not inspect
or encode IFCX bytes. The existing loaded-source routes also assess the complete
composed graph for foreign information and exact-source numeric projection.
Those additional diagnostics and their Allow/Reject enforcement remain intact;
applications assessing a full IFCX source must use the loaded-source routes.
Original input bytes and composed graph remain immutable in `LoadedIfccadGraph`.

Invalid logical input returns `CoreValidation(IfccadReport)` with its original
diagnostics. The loaded-source route's canonical encoder failures return
`CoreEncoding(IfccadEncodeError)`, retaining validation, serialization,
production-readback and semantic-mismatch phases. Error sources remain typed;
this changes Rust error handling, not graph-aware loss or precision assessment.

Profile `urn:example:ifccad:0.1.0`; opencadcodec revision
`ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`. Default Allow returns supported content
with losses; explicit Reject refuses diagnosed omissions or modifications.

The [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-05.md)
records the current public-model review and the remaining upstream clipping
defects. A development patch is not part of the default dependency.

The validated source includes persistent next-ID watermarks. They remain in
the source native drawing and govern subsequent native editing; they are
allocation bookkeeping, not CAD entity semantics or a new preservation payload.
This route does not encode that allocation history into DWG/DXF. A subsequent
fresh CAD import assigns new IDs and watermarks with explicit outcome mappings.

| Source | Treatment |
| --- | --- |
| Composition/profile rules | Core reader composes first, default LaterWins; converter accepts validated projection |
| Header/drawing/node identity | Source remains intact; mappings report identities; reverse header/drawing ID is caller supplied |
| Drawing unit | Map all 25 declared tokens to CAD code, without scaling coordinates |
| Line patterns | Allocate all native named simple/empty definitions, including unused; preserve names/descriptions/lengths; retain required missing Continuous target scaffold with Modified evidence |
| Pattern scale/generation | Drawing/entity scales to header/common fields; planar polyline perSegment/continuous to plinegen |
| Model | Default Model layout, ordered backing block membership |
| Paper | Allocate layouts in tab order, optional media and ordered backing block contents; independent numerical coordinates and effective plot mappings. Unsupported medium units omit media/plot state, not the layout. CAD-name collisions and indices exceeding i16 are fatal |
| Layers | Retain names, unused declarations and concrete appearance; synthesize missing CAD layer 0 with diagnosed explicit fallback; reject collisions under opencadcodec's normalized name lookup, including multiple layers named 0, before CAD allocation under both policies |
| Layer/entity appearance | True RGB, resolved named patterns, supported hundredth-mm weight, exact decoded opacity byte; preserve independent inherited/explicit modes on entities |
| Lines | Direct finite XYZ endpoints |
| Point/Circle/Arc/Ellipse | Position, radii, full/signed partial spans and valid oriented plane through shared CAD preparation |
| Polylines | Ordered straight/bulged planar or straight spatial vertices, dormant bulges, closure and generation; geometric translation residual checked against hard tolerance |
| Definitions | Allocate all targets before contents; retain base, insertion unit, shared/nested references and unused definitions |
| Instances | Valid oriented placement, rotation and signed scales; definition and all nested occurrence proof; compare setter getters even for empty targets |
| Draw order | Owner vector order, independent of ID numbering |
| Invalid ownership/frames/cycles | Core validation; invalid projections never reach conversion |
| Incompatible names | Reserved/anonymous definitions and referring inserts omitted with diagnostics; other construction errors remain fatal |
| Other IFCX graph/envelope information | Canonical-envelope loss diagnostic; includes extra schemas/imports/relations; source graph remains intact |

Polyline local-origin decomposition may be canonicalized; its path must remain
within the configured hard tolerance. Oblique parameterization uses shared preparation.
The target starts with pinned default table/layout/style infrastructure, including
empty Paper scaffold when no authored Paper is emitted. The first authored sheet reuses this owner. Explicit owned BLOCK/ENDBLK markers preserve secondary Paper ownership through DWG; default overall viewports remain runtime infrastructure. CAD workspace mappings and qualified limitations are described below. ByLayer/ByBlock modes are not resolved. Unsupported geometric
parameterization omits the whole entity. Normal local definitions can retain
partial contents, with a loss diagnostic on each affected instance, propagated
through nesting. Only emitted objects receive mappings.

Nonstandard weights are quantized to the nearest supported CAD entry (ties
prefer the lower entry), opacity to the nearest decoded byte (ties prefer higher
transparency). Each adaptation
has Modified evidence. Missing layer 0 is white/opaque/Continuous/0.25 mm with no
source mapping. Reject refuses all these losses. No source preservation is
implemented.

Both policies reject translated-coordinate residual beyond the configured limit,
inexact raw integer-to-binary64 geometry/pattern/scale projection and setter scale
clamping, including empty definitions.
Structural failures remain errors. Failure returns collected diagnostics before
the failed boundary. Diagnostics cover conversion to CadDocument; external
DXF/DWG codecs require independent semantic readback and can introduce additional
angle rounding during degree/radian encoding.

Evidence: `tests/conversion.rs`, `tests/blocks.rs`, `tests/exchange.rs`, `tests/line_patterns.rs`, `tests/layer_names.rs`.
Real DXF/DWG tests use the same primitive corpus and strict IFCX readback;
Nested/shared nonzero-base blocks now pass both DXF and DWG transfers.

The shared codec development configuration is documented in
[patch provenance](../../../patches/opencadcodec-viewports/README.md). OCDraw and
IFCCAD retain independent native geometry and clip contracts. The
[2026-10-05 audit](../../../docs/geometry/opencadcodec-update-2026-10-05.md) separates
patched from unmodified upstream evidence.
## Paper viewport conversion

Viewport output requires the same explicit codec development configuration as
import; see the [patch provenance](../../../patches/opencadcodec-viewports/README.md).
The unmodified dependency fails the required status-bit capability gate and
omits the viewport with located loss. Native core storage has no codec dependency.

The Paper frame maps to CAD center Z=0 and width/height. Model camera target,
unnormalized direction, DCS center Z=0, height and radian twist map directly.
Perspective, positive millimetre lens, signed depth planes, seven render modes,
visibility, enabled, zoom locking and frozen-layer handles remain independent.
Absent dormant lens/depth values use CAD constructor defaults; explicitly stored
zero and signed values are copied. CAD requires bit 0x8000, while effective disabled
state sets 0x20000. Front AtDistance sets not-at-eye only when appropriate;
nonrectangular activation sets 0x10000 independently of the stored boundary handle.
DXF angle degree conversion belongs to the repaired codec, not the converter.

Ordinary owner geometry is prepared first and receives final CAD handles before
viewport references resolve. Boundaries may follow their viewports in authored
order. Analytic circles remain Circle entities; no sampled polyline substitutes
for them. Supported native placements that the geometry converter cannot emit
still omit both the boundary and referencing viewport according to loss policy,
retaining supported siblings. Only emitted objects receive mappings.

Authored viewport numbers start at 2 per Paper owner and are checked through
`i16::MAX`; native uint64 IDs are never cast to runtime numbers. Overflow omits
the viewport with `viewport-number` loss under Allow and Reject refuses that loss.
Insertion uses preallocated handles and original owner order. Overall canvas and
workspace scaffolding remain separate CAD infrastructure; no renderer or print
equivalence is implied. Source graph/numeric loss checks continue to apply through
the loaded-source route.

Evidence: `tests/viewports.rs`, `tests/viewport_exchange.rs` and the numbering
helper unit test; newly encoded native files undergo production strict readback.
`tests/viewport_source.rs` checks exact-source frame/view numbers under both loss
policies and proves frozen-layer array order alone does not cause semantic loss.
The production browser smoke additionally exercises IFCCAD opening/export and
DXF/DWG readback for perspective, circle clips and independent display states.

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


## Shared geometry and adjustable accuracy (provisional 0.1.0)

Every current native primitive family maps through the same prepared CAD geometry
and certified deviation engine as OCDraw, including oriented points, signed arcs,
full/partial ellipses, bulged planar paths and straight spatial polylines. Valid
oblique/rotated planes and signed local block transforms are no longer limited
to standard XY. Unsupported semantic fields/layout coordinates/definition names
retain their independent loss rules; tolerance never permits dropping a field.
Source-aware conversion still checks original graph projection numbers exactly,
including present scope bounds/media. Explicit zero-bulge arrays normalize to
the same default semantics; nonzero dormant bulges remain retained.

Both directions expose `geometry_tolerance` and per-domain `geometry_assessment()`.
Default is exactly 1e-9 in each numerical domain; exact, drawing-unit,
metre and millimetre choices are public. Full conics and bulged segment-circle
pairs plus all retained definition/nested occurrence evaluations must be proved
within the hard limit. Explicit physical Paper root checks use fixed output mappings
or an explicitly requested independent coordinate fallback; no rescaling is inferred
from media. Definition-local acceptance is insufficient
when occurrence scaling or a different root unit makes the deviation too large.

Within-limit geometric rounding remains reported loss evidence but is exempt
from semantic Reject and missing-content propagation. Exceedance/incomplete
proof return Geometry errors with coordinate domain and source/occurrence path,
not successful skipped entities. Raw numeric projection and actual CAD setter
scale clamping remain hard failures under both policies, including empty targets.
Native optional bounds are derived metadata: CAD output does not promise their
exact restoration, and a fresh import prepares its own bounds. CAD file codecs
are verified separately; an isolated conversion report is not an end-to-end
certificate for arbitrary serialization/rendering.

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
## Workspace and upstream update — 2026-10-08

Both converters select opencadcodec 0.6.0 at `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` plus the explicit viewport-off repair [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). Merged spline DXF parameterization and DWG Paper owner/overall-role repairs now come from upstream. See the [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-08.md) and [workspace contract](../../../docs/workspace-state.md) for the current field/transport boundary.

UCS definitions, Model windows, canvas frame/grid/snap/UCS and authored Paper viewport aids map through shared ID-free scalar helpers. Native identities, ownership, source classification and located losses stay format-specific. Disabled zero snap spacing and stored-UCS activation survive independently of current choices. Model-viewport aids retain Model coordinates inside Paper; canvas aids use Paper coordinates. Canvas frames do not enter geometry bounds or clip ownership.

A uniquely available active Model window may be selected; multiple unqualified windows survive with unspecified activation. Paper current viewport/UCS association remains unavailable on the codec surface. Model/Paper mode is available, while the active Paper tab among several sheets is not: the reserved Paper-block handle identifies infrastructure and is never rewritten as a tab selector. Unknown choices remain omitted with located loss. Skipped viewports receive no workspace references.

Dot grid style and frequencies beyond CAD i16 receive field-specific substitutions. VIEWPORT grid beyond-limits/adaptive/subdivision/follow-workplane flags stay in CadDocument but its pinned DXF/DWG routes do not retain them; target diagnostics identify each nondefault field and Reject refuses that portability loss. Model VPORT grid flags survive both routes. Unrepresented display/icon/base/orthographic/plot/visual state remains diagnosed. Numeric and structural failures remain fatal under both policies.
