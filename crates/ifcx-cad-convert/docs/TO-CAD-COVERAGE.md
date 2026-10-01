# IFCX-CAD → CadDocument coverage

Profile `urn:example:ifccad:0.1.0`; cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. Unsupported content prevents output.

| Source | Treatment |
| --- | --- |
| Composition/profile rules | Core reader composes first, default LaterWins; converter accepts validated projection |
| Header/drawing/node identity | Source remains intact; mappings report identities; reverse header/drawing ID is caller supplied |
| Drawing unit | Map all 25 declared tokens to CAD code, without scaling coordinates |
| Model | Default Model layout, ordered backing block membership |
| Paper | Located diagnostic, reject |
| Layers | Retain names, unused declarations and concrete appearance; require source layer 0 |
| Layer/entity appearance | True RGB, Continuous, supported hundredth-mm weight, exact decoded opacity byte; preserve independent inherited/explicit modes on entities |
| Lines | Direct finite XYZ endpoints |
| Circles | Radius and origin retained; standard XY basis only |
| Polylines | Ordered straight vertices/closure; exact BigRational check on translated XY sums; Z becomes elevation |
| Definitions | Allocate all targets before contents; retain base, insertion unit, shared/nested references and unused definitions |
| Instances | Standard XY placement, rotation and signed scales; compare setter getters to requested scales even for empty targets |
| Draw order | Owner vector order, independent of ID numbering |
| Invalid ownership/frames/cycles | Core validation; invalid projections never reach conversion |
| Incompatible names | CAD construction error/reserved-block diagnostic; no guessing |
| Other IFCX graph/envelope information | Canonical-envelope difference diagnostic, reject; includes extra schemas/imports/relations |

Polyline local-origin decomposition is canonicalized without changing its exact
path. Oblique parameterization and certified numerical tolerances are deferred.
The target starts with pinned default table/layout/style infrastructure, including
empty Paper scaffold; that does not assert conversion of Paper semantics or CAD
workspace settings. No ByLayer/ByBlock resolution, preservation or approximation
is implemented. Failure returns all diagnostics collected before the failed
boundary, never a successful partially mapped document.

Evidence: `tests/conversion.rs`, `tests/blocks.rs`, `tests/exchange.rs`.
Real DXF/DWG tests use the same primitive corpus and strict IFCX readback;
DWG nonzero-base marker conflicts are expected rejected transfers.
