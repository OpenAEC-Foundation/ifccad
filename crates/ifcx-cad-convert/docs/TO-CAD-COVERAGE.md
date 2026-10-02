# IFCX-CAD → CadDocument coverage

The projection-only `ifcx_cad_document_to_cad_document` routes validate typed
input before using the shared CAD conversion implementation. They do not inspect
or encode IFCX bytes. The existing loaded-source routes also assess the complete
composed graph for foreign information and exact-source numeric projection.
Those additional diagnostics and their Allow/Reject enforcement remain intact;
applications assessing a full IFCX source must use the loaded-source routes.
Original input bytes and composed graph remain immutable in `LoadedIfcxGraph`.

Profile `urn:example:ifccad:0.1.0`; opencadcodec revision
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`. Default Allow returns supported content
with losses; explicit Reject refuses diagnosed omissions or modifications.

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
| Paper | Omit each layout and its entities with located diagnostics |
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
empty Paper scaffold; that does not assert conversion of Paper semantics or CAD
workspace settings. ByLayer/ByBlock modes are not resolved. Unsupported geometric
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
