# Hatch

OCDraw and IFCCAD independently support native Solid, LinePattern and Gradient Hatch: an ordinary entity
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

Each loop can additionally reference one complete closed circle, full ellipse
or closed planar polyline in the same owner. OCDraw uses a uint64 entity ID;
IFCCAD uses a complete entity path. IO checks existence, kind, closure and owner
without geometry comparison or regeneration. Stored contours remain authoritative
after source edits. Deleting a referenced source is invalid; explicitly detaching
the reference keeps the contour. There are no source copies, stream-index links,
fingerprints, inverse indices, block-occurrence references or native reactors.
CAD conversions allocate fresh identities and return mappings. Reference binding
happens after allocation, independently of draw order. A future editor/BIM module
can process these dependencies; synchronization is outside this implementation.

## Qualified CAD subset

The codec pin is `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`, with only the
existing [viewport-off patch](../patches/opencadcodec-viewports/README.md).
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
refuses loss. No supported contour is partially discarded. Unsupported source
relations detach only the relation with located evidence. Inactive pattern or
gradient settings, seed points, pixel size and provenance/nesting hints are
diagnosed when omitted. Annotative Hatch markers and unqualified extension dictionaries also omit the
whole Hatch; other common metadata follows each converter's coverage.
EXTERNAL/DERIVED/OUTERMOST hints are not inferred from the first loop; output
uses default/polyline encoding flags. CAD does not persist the native join
limit: a nondefault limit is located loss, rejected by Reject. Native invalidity
and hard numeric failures remain errors under both policies.

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
linetype gates. Inactive classic pattern/context data receives editing-state
loss; known inactive gradient state is retained natively, including incomplete
preferences. Unknown definitions, nonzero reserved data, non-0/1 positions,
extra stops and inherited colors remain unqualified: Allow omits the active
Hatch or only its inactive state; Reject refuses loss. Missing active data,
nonfinite scalars/positions and incomplete endpoint pairs are hard errors.
Inactive ACAD pattern-origin metadata is left for common loss auditing; it
does not become a gradient anchor and source records are not mutated.

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
fields and pixels remain separately unassessed. Example writer:
`write_hatch_gradient` with route `ocdraw` or `ifccad`.
Its focused samples are under `tests/fixtures/ocdraw` and `tests/fixtures/ifccad`.

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
offering a download. Gradient-state exports to other versions are explicitly
unqualified and refused. Empty diagnostics do not certify renderer fidelity.
