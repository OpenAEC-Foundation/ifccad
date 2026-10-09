# Solid Hatch

OCDraw and IFCCAD independently support native Solid Hatch: an ordinary entity
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
joins. Patterns await slice 2; gradients, spline boundaries and MPOLYGON remain
outside this slice. There is no fallback to Solid.

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

## Slice checkpoint

Solid storage, conversion and inspection form slice 1. Patterns begin after
integration, verification and user acceptance on main. This does not publish
the provisional contract, add an intersection engine or rerun measurements.
Example writers: `write_hatch_solid` and `write_ifccad_hatch_solid`.

## Clockwise CAD edge angles

The pinned codec exposes clockwise HATCH start/end values in clockwise wire
coordinates. Import negates the start angle and uses a negative sweep of the
positive wire span; export applies the inverse. Numerical curve evidence uses
the same interpretation. This applies to circular and elliptic edges, without
changing the plane, vertices or join limit. The original foundation-repair DXF
Hatch 9AA and corresponding DWG Hatch 9A7 provide independent regressions; the
original 160-unit false gap is the diameter of a radius-80 corner arc.
See [ezdxf's wire convention](https://github.com/mozman/ezdxf/blob/master/src/ezdxf/entities/boundary_paths.py).
