# Hatch

Native model/validation/storage sections remain active. Direct `ifccad-convert`
exchange and codec qualification below describe the [archived reference](converter-parking.md);
the [shared Document route](architecture/explorer-document-route.md) has its own coverage.

IFCCAD is the active track. OCDraw descriptions below retain historical context;
its implementation and fixtures are parked. See [parking](ocdraw-parking.md).
Only IFCCAD code, schemas and current converter coverage define active behavior.

Current dependency qualification (2026-10-23): unmodified codec `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`
(package `acadrust` 0.6.6, local alias `opencadcodec`). Dimension and viewport repairs are upstream.
The [current audit](geometry/opencadcodec-update-2026-10-23.md) supersedes dated
pin/transport limitations below, including paper grid and DXF ShadePlot gates.

IFCCAD supports native Solid, LinePattern and Gradient Hatch: an ordinary entity
with identity, owner/order, appearance, one placed local XY plane, stored loops,
an area rule and an authored join tolerance. OCDraw uses one `hatchStream`;
IFCCAD uses placement plus one `ifccad::hatch` attribute. IFCX composition
replaces the whole attribute, including loops. Nested boundaries are typed
values, not separate entities or nested attribute invocations.

## Geometry and validation

Boundaries support closed straight/bulged polylines, full circles/ellipses and
ordered line/circular-arc/elliptic-arc edges. Coordinates are local XY; angles
and signed sweeps are radians. Outgoing bulges include the closing segment;
an absent wire bulge array means zero, expanded to one value per logical vertex.

Area rules are `normal` (alternating nested regions), `outer` (outermost areas
up to first islands) and `ignore` (through islands). Loop order and winding do
not impose hole labels. IO does not derive nesting or calculate fill. Basic
validation checks primitive parameters, loop cardinality, joins and references;
it does not audit intersections, touching, overlap, self-crossing or positive
filled area. Bounds conservatively enclose contours and possible temporary
joins. Spline boundaries and MPOLYGON remain outside the current profile. There is no fallback to Solid.

## Tolerance and lifecycle

`joinTolerance` is finite/nonnegative in local coordinate units. Default is the
nearest binary64 value to decimal `1e-9`; zero requires exact proven joins.
Rational and bounded trigonometric evidence proves joins. Excessive gaps and
incomplete proof fail. IO never snaps endpoints, inserts bridges or changes
the limit. A viewer may use a temporary join within the endpoint segment.

Creation requests accept coordinates, metres or millimetres. Physical requests
need known drawing units or a fixed physical Paper plot mapping; media size
alone is insufficient. Unknown mapping/unitless coordinates require an explicit
coordinate fallback. Physical limits round down; nonzero underflow or overflow
fails. Invalid known scales are errors. Plot edits do not rewrite stored limits.
This policy is independent of conversion accuracy. Use
`resolve_ocdraw_hatch_join_tolerance`, `resolve_ifccad_hatch_join_tolerance`,
the OCDraw builder resolver, or converter `hatch_join_tolerance` options.

Each loop can retain an ordered group of whole source objects in the same
direct owner: Line, Circle, Arc, full/partial Ellipse, open/closed PlanarPolyline
and SpatialPolyline, Text, MText, local BlockInstance and Paper Viewport.
OCDraw uses exact uint64 `sourceEntityIds`; IFCCAD uses canonical full `sources`
paths. Order and duplicate IDs survive without implying segment, edge or block
component selection. Point, Hatch, opaque/preserved objects and external sources
are ineligible. Ordinary document validation still applies to each source.

`associationEnabled` defaults false independently of the source lists. Disabled
groups remain stored and valid references remain required. True without groups
is valid unbound state. The inspector distinguishes active, disabled and unbound.
IO never compares current source geometry or regenerates the contour. Source
edits retain the cached Hatch; deletion requires explicit whole-group detach or
rebind. A BlockInstance is the complete placed object, not an invented leaf or
bounding box. Text bounds never replace the stored contour.

Optional loop `provenance` retains fromSourceObjects, derivedFromPickPoint,
outermostHint and textBox (default false); a present all-false record is invalid.
Independent `textIsland` defaults false and retains special text-loop fill
classification. Autodesk describes text loops surrounded by an even number of
outer loops and avoided during Solid fill. It is not an ordinary ignored hint,
generic hole or areaRule replacement. All fills retain the flag; IO does not
infer nesting, font geometry or source agreement. Pattern/gradient evaluation
remains unassessed. No source copies, precise selectors, inverse indices,
fingerprints, occurrence chains or native reactor administration are added.
CAD conversions allocate fresh identities and return mappings. Reference binding
happens after allocation, independently of draw order. A future editor/BIM module
can process these dependencies; synchronization is outside this implementation.

## Qualified CAD subset

The codec pin is `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`, without local source patches.
Shared preparation audits fields and registers full contour curve evidence
with nested occurrence assessment. Exact source spans qualify full turns;
no epsilon promotes near-full arcs. Frames project to the actual CAD OCS;
signed traversal and bulges are retained without tessellation. Target ratios
and spans must remain structurally representable: numerical tolerance cannot
turn an ellipse axis ratio into zero or a partial arc into a full turn.

Actual AC1032 DXF/DWG readback covers all three area rules, hole-first ordering,
disconnected/nested contours, circles/ellipses, negative circular sweeps, bulges,
a tilted plane, a retained small edge gap, associative handles and nested
nonuniform block instances. This qualifies stored data and contour accuracy;
it does not certify arbitrary editor gap interpretation, pixels, topology or
interactive reactor updates. Fill and its occurrences remain unassessed.

Unsupported active fill/edges/flags omit the whole Hatch under Allow; Reject
refuses loss. Supported contours are never partially discarded. If any source
entry is missing, skipped, wrong-owner or unsupported, Allow omits the entire
affected group, retaining its contour, flags, switch and other complete groups.
Entry and group loss are located separately; Reject refuses. Exact inverse
source reactors to qualified complete groups are accounted as derived relation
bookkeeping; unrelated reactors/attachments retain their existing audits.

EXTERNAL/DERIVED/OUTERMOST/TEXTBOX map to provenance; TEXT_ISLAND maps to active
textIsland semantics. POLYLINE remains backing-derived. No flags are inferred
from first-loop order. NOT_CLOSED, SELF_INTERSECTING, DUPLICATE and unknown active
flags remain unsupported. Annotative contexts and unqualified dictionaries
remain outside this slice. Classic editing and boundary construction now have independent optional native
records; unrepresentable model state and physical losses remain explicit.
Nondefault joinTolerance is located loss; native invalidity and hard numerical
failures remain fatal under both policies.

The retired reference-viewer integration verified the actual target CAD model after
DXF/DWG write/read, including source-group order, duplicates, association switch,
provenance and TextIsland, before offering a download. All six offered versions
are qualified independently; see [version profiles](hatch-export-versions.md). Missing
Hatches, invalid owners, missing source correspondence or confirmed loop reorder
fail with HATCH_READBACK_UNCONFIRMED. Unexpected nonempty changed source groups
also fail, so physical codec changes cannot deliver a partial/substituted group.
Complete physical group omission remains a located source-group loss.
The codec clears enabled-without-sources
state: Allow reports TARGET_CODEC_HATCH_ASSOCIATION_STATE_LOSS; Reject refuses.
Other physical codes are TARGET_CODEC_HATCH_SOURCE_GROUP_LOSS,
TARGET_CODEC_HATCH_LOOP_PROVENANCE_LOSS and TARGET_CODEC_HATCH_TEXT_ISLAND_LOSS.
TextIsland loss is active fill-semantic loss. This verification qualifies stored
parameters, not rendering or automatic CAD-editor updates.

The active Explorer uses a different shared mapping and readback boundary; see
[current migration coverage](architecture/explorer-document-route.md). The
reference converter’s physical qualifications do not transfer to it.

## Explicit line patterns

`fill.kind = linePattern` embeds ordered line families, independent of a PAT
library or drawing line-pattern resource. Optional name/description are literal
metadata, including empty strings. Authored origin defaults to (0,0), rotation
to zero radians and positive dimensionless scale to one. Contour/source edits
never move the origin. Each family stores angle, basePoint, offset=(along,
perpendicular), and ordered Dash/Gap/Dot values. Signed perpendicular spacing
must be nonzero; no minimum spacing is imposed. Empty dashes means continuous.
Dash/gap lengths are positive and finite, dots have no length. A nonempty period
is summed exactly and must be positive and within finite binary64 range;
all-dot is invalid, all-gap is valid. No sorting or alternation repair occurs.

For family direction d=(cos(angle),sin(angle)) and normal n=(-sin,cos), effective
base is origin + scale*R(rotation)*basePoint, offset is
scale*R(rotation)*(along*d + perpendicular*n), and lengths scale once.
CAD families are already effective: import applies the inverse once; export
evaluates once in the actual target OCS. Literal CAD names and descriptions are
retained on import. Export writes a Custom pattern with explicit lines; absent
native name/description requires diagnosed target metadata synthesis.
The typed CadDocument retains a nonempty description, but the pinned DXF/DWG
backings do not persist it. Viewer/browser physical readback reports that change
as TARGET_CODEC_HATCH_DESCRIPTION_LOSS; it is separate from parameter conversion.

Predefined/custom source families are authoritative; pattern-library/type
editing dependencies are diagnosed. UserDefined is admitted only after the
adapter resolves a local continuous linetype (explicit or ByLayer), with one
explicit continuous family. Unresolved ByBlock, xref or noncontinuous active
linetypes omit the whole Hatch. ByLayer on layer 0 inside a local block is also
unqualified because its insertion layer can change the active linetype. See
[Autodesk block property rules](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-Core/files/GUID-25E9F20C-D146-426C-8815-37DF48D2D33F.htm).
A double UserDefined profile is qualified only
for one continuous family at angle/rotation zero, base equal to origin, zero
along offset, and signed perpendicular spacing of magnitude patternScale. Its
exact quarter-turn produces a second literal family with certified rounding
error. Other active double cases are unsupported. Double flags on predefined
or custom source patterns are inactive and diagnosed. This activation distinction
follows [Autodesk setPatternDouble](https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-AcDbHatch__setPatternDouble_bool.html).

Only the exposed top-level ACAD origin Point3D (finite XY, zero Z) is consumed
by common auditing. Nested/unrelated XDATA remains diagnosed; source records
are never mutated. Export uses record_pattern_origin, not the moving setter.

Pattern numerical evidence bounds potential integer line and repeat indices
using conservative contour enclosures and exact rational arithmetic. Affine
residual envelopes are certified at domain corners and propagated through the
existing nested block engine together with contour evidence. No pattern lines,
clipped regions or pixels are generated. Dense patterns have no density cap;
unrepresentable values or an incomplete accuracy proof remain hard errors.
Fill and its occurrences remain separately unassessed.

When the exact rational sum of stored native family angle and pattern rotation
equals the stored effective CAD angle, the certificate retains their shared
direction. It subtracts base, offset and along-parameter terms before interval
evaluation, using the trigonometric addition identity for the native offset.
Periodic residuals include period, cumulative dash/gap and segment-length drift.
This identity changes the proof expression only; stored families, source
parameters, contour domains and tolerance are unchanged. A rounded binary64
angle-sum match alone is insufficient; other angles use independent conservative
enclosures. The qualified UserDefined double quarter-turn retains its separate
proof semantics.

The world residual retains all three origin components and actual axis
differences as `S*(s-t) + (S-T)*t`, alongside the target position needed by nested
block maps. This applies to oblique and negative-normal planes as well as +Z.
Point assessment uses both squared-distance bounds: a lower bound beyond the
limit proves exceedance; a straddling interval is a hard incomplete-proof error.
Neither case is accepted by the semantic Allow policy.

Solid was integrated and verified on main before this pattern slice. The
provisional contract is not published by these slices. Intersection auditing,
rendering and runtime associative updates remain separate work. Example writers:
`write_hatch_solid`, `write_ifccad_hatch_solid`, and `write_hatch_pattern` (route
argument `ocdraw` or `ifccad`).

## Clockwise CAD edge angles

The pinned codec exposes clockwise HATCH start/end values in clockwise wire
coordinates. Import negates the start angle and uses a negative sweep of the
positive wire span; export applies the inverse. Numerical curve evidence uses
the same interpretation. This applies to circular and elliptic edges, without
changing the plane, vertices or join limit. The original foundation-repair DXF
Hatch 9AA and corresponding DWG Hatch 9A7 provide independent regressions; the
original 160-unit false gap is the diameter of a radius-80 corner arc.
See [ezdxf's wire convention](https://github.com/mozman/ezdxf/blob/master/src/ezdxf/entities/boundary_paths.py).

## Gradient authorship and storage

One optional `gradient` state lives next to the selected fill. `fill.kind=gradient`
requires its type and ordered pair of concrete colors (implicit endpoints 0/1).
Solid/LinePattern may retain incomplete inactive state: type or colors may be
absent, but a supplied pair always has both colors. The same stored state is
retained after changing fill; there is no second dormant copy or whole-Hatch
opaque payload. All-default state without type/colors is invalid; omit it.

Types are linear, cylinder, inverseCylinder, spherical, inverseSpherical,
hemispherical, inverseHemispherical, curved and inverseCurved. Finite rotation
is local CCW radians; no modulo normalization. Finite shift/tint lie in [0,1].
Defaults: rotation=0, shift=0, colorMode=twoColor, tint=0. Shift blends shifted
and unshifted shape definitions; it is not a displacement. Mode oneColor or
twoColor and tint are independent of both stored colors. Equal colors do not
change mode, and IO never derives color 2 from tint.

Each model uses its own concrete RGB plus optional indexed/named identity.
Endpoints require the Concrete color variant. Symbolic Foreground has no fixed
RGB endpoint and is invalid here; source ACI 7 endpoints remain unqualified,
with active-Hatch or dormant-state loss under Allow and refusal under Reject.
No per-stop alpha, inherited endpoint colors, free stops, HSL/HSV authored
space, tint algorithm or rendering contract is added. Existing common opacity
and visibility apply independently; colors are not replaced by entity color.

OCDraw registers one optional `hatchStream.gradient` column. Omission means
absence for every row; a present column is count-sized, with null only for
absent rows. IFCCAD stores an optional `ifccad::hatch.gradient` member, omitted
when absent; explicit null is invalid. Both closed mappings forbid nested
null/unknown/mixed fields. Whole IFCCAD Hatch-attribute replacement includes
state. Models/builders/readers/writers share intrinsic parameter validation,
without evaluating fill or changing the contour/source/join/bounds contract.

## Qualified gradient CAD boundary

Enabled gradient is selected before classic pattern flags or UserDefined
linetype gates. Inactive classic pattern/context data is captured in classicSettings; known
inactive gradient state is retained independently, including incomplete preferences. Unknown definitions, nonzero reserved data, non-0/1 positions,
extra stops and inherited colors remain unqualified: Allow omits the active
Hatch or only its inactive state; Reject refuses loss. Missing active data,
nonfinite scalars/positions and incomplete endpoint pairs are hard errors.
Inactive ACAD XY pattern-origin metadata belongs to classicSettings; it does not
become a gradient anchor and source records are not mutated.

Native placements remain valid independently of CAD coverage. Export requires
exact numerical equality of local X/Y with the actual target arbitrary-axis
axes. No geometric epsilon permits a basis change or angle repair. Failure
omits active Hatch or only dormant state under Allow with
HATCH_GRADIENT_FRAME_UNSUPPORTED; Reject refuses. Some oblique imported frames
may therefore not qualify for export after normal reconstruction.

Actual AC1032 DXF/DWG tests cover all nine types, both modes, RGB/ACI endpoints,
all area rules, relationships and independently authored gradient tags.
Additional exact tilted frames and nested nonuniform occurrences retain their
local data; contours retain the existing numerical evidence. Fill, gradient
fields and pixels remain separately unassessed. Active examples and focused
samples use IFCCAD under `examples/ifccad` and `tests/fixtures/ifccad`.

Memory CadDocument supports disabled state; pinned DWG retains it, pinned DXF
omits it. Native named identity exceeds GradientColorEntry and produces located
model loss with RGB retained. The reader can already have discarded DWG book
names, DXF declared counts or malformed defaults; no downstream recovery or
original-file losslessness is promised. Independent characterization tests
record those limits; [upstream #109](https://github.com/HakanSeven12/opencadcodec/issues/109)
tracks them. No unreviewed codec update is selected.

Viewer/browser file exports compare mapped target state with physical readback,
using scope/order and emitted conversion-local identity to confirm matching.
Changed parameters receive TARGET_CODEC_HATCH_GRADIENT_PARAMETER_LOSS;
disappeared inactive state receives TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS.
Unconfirmed matching fails verification. Optional conversion `lossPolicy`
defaults to allow; reject is enforced at model and physical-file phases before
offering a download. Active gradients require AC1018 or later; inactive settings
have format/version-specific losses. See [actual output qualification](hatch-export-versions.md).
Empty diagnostics do not certify renderer fidelity.

## Classic editing and construction records

Optional `classicSettings` retains preferred solid/linePattern mode, creation
type userDefined/predefined/custom, optional literal name/description/origin,
rotation (default 0), positive scale (1), double (false), and optional ordered
`definitionLines`. These direct local XY values have an unwrapped angle, basePoint,
XY offsetVector and signed dashLengths (positive dash, negative gap, zero dot;
empty means continuous). They are already effective stored CAD definitions, not
unscaled PAT families. Import copies them before UserDefined/double materialization,
without inverse rotation, scale division or along/perpendicular projection.
The normalized active `HatchLinePattern` remains independent and unchanged.

Some(empty string), Some([0,0]) and Some([]) are authored presence. Direct values
must be finite, offsetVector nonzero and a nonempty dash list must have a positive,
finite absolute period. Duplicate lines, signed zeros and partial/inactive lists
are retained. A nonzero XY vector alone does not qualify active transverse spacing.
The all-default record must be absent.

Optional `boundaryConstruction` retains ordered finite XY seedPoints (default
empty) and finite nonnegative pixelSize (0). Duplicate and outside seeds are
valid. Its all-default record must be absent. Pixel size is neither joinTolerance
nor HPGAPTOL, and never fabricates DERIVED provenance.

Neither record synchronizes with changes to literal fill/contours. CAD import
captures the recipe before qualified UserDefined/double materialization. Export
restores compatible discrete structure, dash/gap/dot order and literal metadata,
then certifies the actual original-parameter CAD candidate against the active fill
within the selected coordinate-domain tolerance, over complete conservative contour
enclosures and all relevant nested/nonuniform occurrences. No fingerprint or exact
normalized-float gate is used. A deliberate tiny numeric edit may also qualify;
this is conversion compatibility, not proof that the fill was never edited.
Exact/zero tolerance adds no minimum margin.

A trial is not included in final evidence or diagnostics. An optional recipe that
is incompatible or unqualified falls back with located semantic loss under Allow;
Reject refuses that loss. The finally selected candidate always needs its own hard
proof, including the active-fill fallback. Unrelated numeric failures remain hard.
An upper bound above the limit is not by itself a proven geometric exceedance.
Absent stored origin remains absent, including when definition points are translated;
offset vectors are never translated. Only the managed origin slot is replaced,
leaving allocated identity, source/reactor links, construction and other XData intact.
Frame-bound state requires exactly matching OCS axes; representable translations
project XY points explicitly. Equal oblique bases preserve local XY values;
translations must remain exactly representable. Finite unrepresentable records are omitted whole
with `hatch-classic-settings-loss` or `hatch-boundary-construction-loss`; Reject
refuses. Known nonfinite/invalid state is hard even while inactive.

Closed IFCX optional members reject null/unknown keys. LaterWins replaces the
whole Hatch attribute. Native CBOR revision 1 adds fields 7/8 in place, with the
same shared validation and existing optional Zstd envelope. Classic slot 9 stores
direct definition lines; retired draft classic slot 8 and IFCX `families` are
unknown and rejected, without a historical decoder or revision bump. Physical transport
and the original AC1024 practice regression are documented in
[Hatch output versions](hatch-export-versions.md).
