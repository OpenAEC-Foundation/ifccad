# Drawing workspace state

Workspace describes saved drawing views, construction aids and current choices.
OCDraw and IFCCAD retain independent records, identities and wire formats.
The native and converter contracts below cover the workspace slice and its
qualified codec limits.

## Codec qualification

Base: opencadcodec 0.6.0, revision `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`,
with the explicit viewport-off-state repair documented in
[patch provenance](../patches/opencadcodec-viewports/README.md).

| Field or relation | Evidence | Qualified boundary |
| --- | --- | --- |
| Model VPORT grid, disabled zero snap spacing, quarter-turn snap angle, stored UCS origin and activation flag | Fresh typed document through DXF and AC1032 DWG | Tested values survive; no spacing normalization is required by the codec |
| Unused named UCS origin and elevation | Fresh document through both codecs | Tested definition retained without a live selection |
| Paper viewport grid, disabled zero snap and stored UCS/activation | Two Paper owners through both codecs | Tested values survive independently of canvas versus authored viewport role |
| Nondefault overall canvas center/width/height | Fresh document through both codecs | Tested numeric frame retained; no relation to media or drawable bounds asserted |
| First literal `*ACTIVE` VPORT | Hand-authored DXF with heights 20 then 30 | Reader iteration retains this literal order; DWG current-selection meaning is not established by a set-of-values assertion |
| Paper current-selection/positive stacking rank | Literal VIEWPORT status 68 changed from 1 to 3 with the same viewport ID | Both read to identical public viewport values. Number, visible/off state and draw order cannot reconstruct the lost rank |
| Isometric pair bits | Four raw status-bit combinations | Both pair bits remain separately available; the workspace adapter must interpret their joint meaning |

Tests and the independent literal source live in
[`cad-workspace-convert/tests`](../crates/cad-workspace-convert/tests).
They compare semantic fields rather than unstable handles. The literal fixture
qualifies field parsing and is not a complete native conversion source. Fresh
CadDocument cases separately exercise complete runtime layout infrastructure.
No size/exchange measurement was run and this table does not certify every
possible CAD source or an unavailable active-window association.

The shared `workspace_kernel` supplies identifier-free view, grid, snap and
canvas-frame values and intrinsic predicates. Model views may be perspective;
Paper canvas views remain orthographic. Snap spacing is positive while enabled
and nonnegative while disabled. Nonfinite/negative dormant values stay invalid.
Physical coordinate meaning belongs to each consuming workspace, not these
scalar types; named/current references remain format-specific.

## Native records and ownership

IFCCAD stores drawing-owned UCS definitions and ordered Model-window nodes,
with canonical full path references and separate `nextUcsId` and
`nextModelWindowId` uint64 watermarks. They remain strictly above live IDs;
deleting records never reduces them, and allocation never rebuilds history
from the current set. Canvas and viewport workspace attributes belong to their
existing Paper owners. A viewport may have saved aids without a canvas snapshot.

OCDraw keeps relational records with its existing IDs. `DrawingViewState`
current/active selections and Paper current/context selections are optional.
Windows remain valid without a view-state record, and current Model UCS can
exist without windows. Stored/current UCS and `useStoredUcs` remain independent;
consistency is checked only for known activation with the stored flag enabled.
Legacy missing canvas activation defaults to true. New empty choice records are
omitted rather than inventing World, window zero, Canvas or a first entry.

A canvas frame stores center XYZ and positive width/height. It describes a
workspace frame, not media, geometry bounds, draw order or clip ownership.
Model windows and authored Model-viewport aids use Model coordinates, even when
the viewport is placed on inch Paper. Canvas aids use the layout's Paper
coordinates. No unit comes from media dimensions and no grid/UCS coordinate is
rescaled by a plot mapping. Lens length retains its physical millimetre meaning.

## Qualified conversion boundaries

| Surface | Native/direct CAD | DXF and AC1032 DWG |
| --- | --- | --- |
| Available Model windows and UCS definitions, including unused records | Retained with fresh identity bindings | Tested scalar values and definitions survive; active association is qualified separately |
| Disabled snap, zero spacing, base, quarter-turn angle and isometric plane | Retained without default substitution | Tested values survive; both pair bits encode the left plane and normalize to native Left |
| Canvas XYZ frame and stored-UCS activation | Retained independently of media and geometry | Tested values survive with correct Paper ownership |
| Authored viewport grid/snap/stored UCS | Applied to the existing entity without resetting frame/view/display/clip state | Qualified scalar values survive; file-only grid behavior limit below remains explicit |
| Model VPORT grid behavior | Retained | beyond-limits/adaptive/subdivision/follow-workplane survive |
| VIEWPORT grid behavior | Public fields copied; nondefault target fields receive portability evidence | Pinned routes omit all four GridFlags; source readback has defaults. Reject refuses this known portability loss |
| Active Model window | A single consistent `*ACTIVE` record is unambiguous; otherwise records remain without a choice | Multiple-record ordering has not been qualified as a general current-window guarantee |
| Current Paper context/UCS | Unavailable association receives an owner-located diagnostic | Viewport ID, visibility, off-screen state and source order do not reconstruct lost stacking/current rank |
| Active layout | Model mode is known; current Paper is qualified from the unique `*Paper_Space` record and consistent reciprocal LAYOUT link, independently of sheet count or tab order | DXF and AC1032 DWG retain first and secondary active Paper tabs. Export synchronizes record names, existing BLOCK-begin names and the reserved header cache; handles, entity ownership and tab order remain unchanged |
| Unsupported grid style/frequency | Dots substitute Lines; frequency beyond positive i16 substitutes 5, with exact field evidence | These diagnosed substitutions are not exact fidelity |

Malformed references, collisions and nonfinite/negative workspace values remain
failures. Unsupported viewport omission creates no dangling native workspace or
active reference. Unrepresented icon/display, base/orthographic UCS, separate
unnamed elevation, plot and lighting data remains located loss; mapped families
are not globally exempt from residual coverage.

Loaded IFCCAD conversion compares known workspace values exactly against the
canonical typed projection, separately from geometry accuracy. Scalar rounding
receives a precision failure under both policies. Original native downloads keep
original bytes and source contributions. Inspector links follow only explicit
known references, keep full-width IDs exact, identify construction domains and
show omitted choices as unspecified.

The ordinary example generator produces both `workspace-state` examples;
mutable candidate conformance includes unknown choices, full-width references
and invalid spacing/reference/frame cases. No released numbered collection was
modified and no live CAD editing controls were added.

### Active Paper qualification correction (2026-10-08)

The original foundation DXF retains Paper mode (`$TILEMODE=0`) and associates
Layout1 with `*Paper_Space`, while Layout2 owns `*Paper_Space0`. This role follows
[DXF layout management](https://ezdxf.readthedocs.io/en/stable/dxfinternals/layout_management.html). The initial
workspace slice incorrectly treated all multi-sheet Paper choices as unavailable;
a subsequent export then fell back to the default Model mode. A header-only
selection attempt had failed earlier, but this did not establish that the complete
reserved-block role was unrepresentable. Both adapters now qualify the reciprocal
block/layout association and synchronize the complete target role. Existing BLOCK
begin records must agree with their renamed BLOCK_RECORD: DWG uses their names
to reconstruct the entity spaces on readback. No native contract was extended.

The regression tests exchange two distinct sheets through both physical backings,
check first and secondary selection and retain geometry on its original sheet.
Unknown or inconsistent role associations remain diagnosed; a tab index or sheet
count is not used as a selection substitute.
