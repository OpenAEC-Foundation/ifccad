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
`fe69506cb99dea6f4c4a73b690a27fdf04403ea0`. Default Allow returns supported content
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
| Paper | Allocate layouts in tab order, optional media and ordered backing block contents; unitless/in/mm coordinates supported. Other coordinate units omit the entire layout with located evidence. CAD-name collisions and indices exceeding i16 are fatal |
| Layers | Retain names, unused declarations and concrete appearance; synthesize missing CAD layer 0 with diagnosed explicit fallback; reject collisions under opencadcodec's normalized name lookup, including multiple layers named 0, before CAD allocation under both policies |
| Layer/entity appearance | True RGB, resolved named patterns, supported hundredth-mm weight, exact decoded opacity byte; preserve independent inherited/explicit modes on entities |
| Lines | Direct finite XYZ endpoints |
| Circles | Radius and origin retained; standard XY basis only |
| Polylines | Ordered straight vertices/closure; exact BigRational check on translated XY sums; Z becomes elevation |
| Definitions | Allocate all targets before contents; retain base, insertion unit, shared/nested references and unused definitions |
| Instances | Standard XY placement, rotation and signed scales; compare setter getters to requested scales even for empty targets |
| Draw order | Owner vector order, independent of ID numbering |
| Invalid ownership/frames/cycles | Core validation; invalid projections never reach conversion |
| Incompatible names | Reserved/anonymous definitions and referring inserts omitted with diagnostics; other construction errors remain fatal |
| Other IFCX graph/envelope information | Canonical-envelope loss diagnostic; includes extra schemas/imports/relations; source graph remains intact |

Polyline local-origin decomposition is canonicalized without changing its exact
path. Oblique parameterization and certified numerical tolerances are deferred.
The target starts with pinned default table/layout/style infrastructure, including
empty Paper scaffold when no authored Paper is emitted. The first authored sheet reuses this owner. Explicit owned BLOCK/ENDBLK markers preserve secondary Paper ownership through DWG; default overall viewports remain runtime infrastructure. CAD workspace settings are deferred. ByLayer/ByBlock modes are not resolved. Unsupported geometric
parameterization omits the whole entity. Normal local definitions can retain
partial contents, with a loss diagnostic on each affected instance, propagated
through nesting. Only emitted objects receive mappings.

Nonstandard weights are quantized to the nearest supported CAD entry (ties
prefer the lower entry), opacity to the nearest decoded byte (ties prefer higher
transparency). Each adaptation
has Modified evidence. Missing layer 0 is white/opaque/Continuous/0.25 mm with no
source mapping. Reject refuses all these losses. No source preservation is
implemented.

Both policies reject translated-coordinate rounding, integer-to-binary64
geometry/pattern/scale projection and setter scale clamping, including empty definitions.
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

CAD paper dimensions are always millimetres. Positive finite dimensions map to
an optional medium; invalid or partial dimensions omit only that medium with
loss evidence. Explicit fixed 1:1 inch/mm mappings establish Paper coordinate
units. Fully unconfigured unsized defaults are unitless; other authored mappings
retain numeric coordinates as unitless with a loss diagnostic. Media never
establish coordinate units and block instances are never implicitly rescaled.

Native media convert through exact rational millimetre factors. Any binary64
rounding is fatal under both policies (5 inches maps exactly to 127 mm; 1 inch
cannot exactly map to binary64 25.4 mm). Unsupported factors such as parsecs omit
only media under Allow. Printer/media names, margins, rotation, plot limits,
authored overall canvases and workspace state remain deferred losses. Definition-content
losses propagate to Paper instances through shared and nested definitions.
