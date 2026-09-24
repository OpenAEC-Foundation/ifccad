# IFCDR 0.10.0 logical contract

This document, [the logical registry](registry-0.10.0.json), and the applicable
encoding mapping define the IFCDR 0.10.0 contract. The
[JSON reference mapping](json-mapping-0.10.0.json) specifies physical storage.
This is an unpublished development contract. Its presence does not imply
support by an implementation; consult the candidate compatibility matrix.

## Values and collections

The resource contains its identity, length unit, next entity ID, supporting
tables, typed entity collections, and scope order. Bounds belong to each scope;
there is no aggregate resource bound across independent coordinate domains.
Integers have the unsigned range named by their type (8, 32 or 64 bits).
`float64` denotes finite binary64 values. Strings preserve their contents;
`nonEmptyString` and IFCX identities contain at least one character.
Implementations must not pass integer identities through floating point.

Table IDs are unique within their table. Entity IDs are unique across all
entity kinds and scopes in the resource. Table IDs may be zero. Supporting
tables with no rows have the logical value of an empty collection, including
when the encoding permits their omission. Required physical table presence is
specified by the encoding mapping. There are no implicit table rows.

Line columns describe corresponding rows. Polyline rows have corresponding
entity properties and an ordered vertex sequence. `entityOrder` associates a
scope with an ordered sequence represented relationally by `entityOrderEntry`
rows. Those entry rows reference entities; they are not drawable entities.
Physical offsets, arrays and row packing do not define entity identity or
draw order. Only vertices selected by a polyline sequence form its geometry.

The logical default of visibility is true. Omitted default values and explicit
default values are equivalent. An absent override value is distinct from an
explicit numeric zero. Values and references not currently selected by an
appearance mode remain stored logical content.

## Entity identity

`entity.identity`: Every drawable entity ID is nonzero and occurs once across
the entire resource. Identity does not depend on type, row position or scope.
The pair (resource ID, entity ID) identifies an entity within a package.
Re-encoding a resource preserves those IDs and all references to them.

## Entity next-id

`entity.next-id`: The next entity ID is positive and greater than every present
entity ID. Gaps and a next ID greater than max+1 are valid. An empty resource
may carry any positive next ID; a newly created empty resource starts at 1.
The maximum u64 entity ID cannot occur because its successor is not
representable. Overflow is an error, never wraparound. A snapshot makes no
claim about historical reuse of identities in other versions of the resource.

## Reference local

`reference.local`: Each entity's scope, layer binding and appearance binding
must exist. Each non-null override ID must identify an appearance override.
Table IDs are not interchangeable between tables. IFCX identities are external
references whose resolution is checked at package level, not invented or
replaced by the resource decoder.

## Order coverage

`order.coverage`: Every scope has exactly one order sequence, which may be
empty. Every drawable entity appears exactly once in its scope's sequence and
never in another scope's sequence. Order cannot reference a missing entity.
The sequence may interleave entity kinds independently of their column order
or ID magnitude. An entity's own visibility does not remove it from order.

## Geometry finite

`geometry.finite`: Line endpoints and block base points are finite XYZ values.
Polyline vertices are finite local XY values. Endpoint Z defaults independently
to zero; explicit point3/vector3 records have no component defaults. Model and
definition geometry use resource coordinate units; paper coordinates form their
own numeric domain with physical paper/plot mapping outside this contract.
No bulge, width or curved segment is implied.

Coincident endpoints, repeated vertices and zero-length segments remain valid.
Do not merge, discard, reorder or change entity kinds to normalize geometry.
Scope records have no generic name, base or flags properties.

## Polyline vertex-count

`polyline.vertex-count`: Every polyline has at least two vertices, whether open
or closed. Closed means an implicit last-to-first segment; it does not append
a stored vertex. Preserve the supplied closed flag and any duplicated final
vertex. In particular, open [A, B, A] stays open, and a closed two-vertex
polyline is valid. There is no nonzero-area or nonzero-length constraint.

## Plane frame

`plane.frame`: Each planar polyline has a complete origin O and directions X,Y.
Its whole logical default is O=(0,0,0), X=(1,0,0), Y=(0,1,0). Explicit records
contain all nine finite components; there are no component defaults. Resolve
local point (x,y) to owning-scope geometry as O + x*X + y*Y. The directed normal
is X cross Y. For a polyline inside a definition, apply this plane placement before any
containing block-instance transforms.

For semantic predicates, interpret each stored binary64 as its exact rational
value. Let tau be the binary64 value with bits 0x3d719799812dea11 (nearest to
1e-12). Require inclusively:

```text
abs(dot(X,X) - 1) <= tau
abs(dot(Y,Y) - 1) <= tau
abs(dot(X,Y)) <= tau
```

Use the supplied axes without normalization. Equivalent omitted/explicit whole
defaults have the same meaning. Approximate equality is not logical equality,
and different origins/axes describing the same plane remain different values.
These dimensionless predicates do not depend on units or drawing size. An
implementation may use intervals, but must resolve ambiguous boundaries with
sufficient arithmetic precision to produce the exact predicate result.

## Bounds enclosure

`bounds.enclosure`: Every scope has nullable bounds. Null means no evaluated
native geometry, even if instances of empty definitions occur in the scope.
Otherwise finite ordered XYZ bounds enclose all its evaluated geometry: line
endpoints, placed polyline vertices and recursively instanced contents. An
instance's origin/base point alone is not geometry. Straight segments lie
inside the bounds of their transformed endpoints. The definition base point is
not subtracted when bounding the definition itself; it is subtracted on insertion.

Include invisible geometry. Exclude lineweight/presentation effects and any
union across independent model/paper domains. Loose and point-sized enclosing
boxes are valid without a quality warning. Base points need not lie inside
definition bounds. Preserve supplied valid bounds; readers do not repair them.

Stored scalars mean their exact binary64 values; sin/cos mean mathematical
functions at the stored angle. No positional epsilon excuses under-enclosure.
A rounded point or protruding transformed definition AABB is not proof of
invalidity: the AABB may be loose. A reader may accept a fitting transformed
validated definition AABB, but must inspect actual leaves before concluding a
violation when the shortcut fails.

A reference writer derives bounds from actual leaves using the same outward
interval evaluation order as the reader; store the computed endpoints without
arbitrary padding. Existing exact rational predicates remain authoritative for
rational-only expressions. Fixed certified sin/cos enclosures are allowed;
iterative transcendental precision escalation is not required by this profile.
Implementation-specific unresolved proofs do not change mathematical validity.

A proved-outside leaf is a contract violation. An unresolved proof is blocking
execution/incomplete assessment, not evidence that the file is invalid. No strict
validated result may be issued for either case. Independently proved violations
still take precedence in the validity conclusion. Intermediate overflow alone
does not prove invalidity; proved geometry outside the finite binary64 range
cannot have finite enclosing bounds.

## Appearance values

`appearance.values`: Color contains three RGB channels in [0,255], optionally
an indexed color (nonempty system and u64 index), optionally a named color
(nonempty catalog and name), or both metadata forms. Retain all supplied forms.
IFCDR color and its nested metadata records have only the registered fields;
unknown fields cannot be silently discarded. This rule does not close IFCX
objects whose schema allows extensions.

Opacity is finite in [0,1]. Lineweight is finite and nonnegative, retaining the
existing appearance convention without conversion based on resource unit.
Line-pattern overrides are nonempty IFCX identities. Validate every non-null
stored override value, even if the current mode does not select it.

Each property independently has mode ByLayer, Explicit or ByBlock. The logical
mode is distinct from any currently resolved value: Explicit equal to a layer
value does not become ByLayer. Effective occurrence-style resolution is outside
this contract; retain symbolic ByBlock intent instead of baking resolved styles.

## Appearance source

`appearance.source`: At package validation, resolve IFCX layer/appearance and
line-pattern references to the appropriate existing definitions. Retain the
versioned name-uniqueness rule below for layer names within a resource. Explicit
properties use a non-null local override first, otherwise the referenced IFCX
appearance value. An invalid override does not fall back. An explicit property
without a usable source is invalid. ByLayer/ByBlock inheritance stays symbolic.
Permitted IFCX extension fields remain in the graph; typed appearance access
is not a replacement serialization of that graph.

## JSON reference representation

The [JSON mapping language v4](json-mapping-v4.md) defines payload locations,
omission, nulls, value mappings, point pools, child ranges and directory rules.
The [0.10.0 mapping](json-mapping-0.10.0.json) applies those forms to this
registry. In particular, point-pool ranges may overlap and leave unused
physical points, while child ranges partition the complete child stream in
parent row order. Neither storage sharing nor unused pool points changes
logical geometry. Empty-scope bounds are represented by JSON null.

Repeated rows use separate columns for stable scalar fields that readers,
validators and query engines commonly select independently. A compound value
remains one column only when its members form one inseparable semantic unit and
are normally consumed together (for example a placement). Variable-length
per-entity collections use a relational child stream with explicit ranges, not
an object array embedded in each parent column. This is a construction
convention for evolving mappings, not a universal requirement on other
physical encodings. Here, viewport identity/reference/flags have separate
columns, cohesive frame and view records each have one column, and viewport
layer overrides are complete relational child rows.

Only the registered tables, streams, fields and declared directory metadata
are part of this closed IFCDR contract. An unsupported version or stream
prevents a complete resource proof. Report support limits without claiming
invalidity under an unknown contract. Skip semantic checks that require
unavailable data; independent established errors may still be collected.

## Scope kinds

`scope.kinds`: A resource has exactly one ModelSpace, zero or more PaperSpace
scopes and zero or more BlockDefinition scopes. A scope row contains only id,
kind and bounds. Scope IDs are unique resource-local uint32 values; zero is
valid but not reserved for model space. Identify the model by kind, not ID.
A scope has its own ownership, coordinate domain and explicit draw order.

The pair (resourceId,scopeId) identifies a scope in a package snapshot, not a
durable cross-version identity. Renumbering updates all resource-local and IFCX
layout references atomically without changing persistent entity IDs. IFCX
layouts select scopes through their DrawingRepresentation; scopes have no
stored IFCX backlink. Block definitions have no IFCX node. This profile has no
cross-resource block target and no external-reference scope kind.

## Block definitions

`block.definitions`: Every BlockDefinition scope has exactly one definition
row with the same scopeId, and no model/paper scope has a definition row.
The row has name (nonempty, unique by name.uniqueness), basePoint (default
0,0,0), description (default empty), anonymous (default false), insertionUnit
(default unitless), explodable (default true), and scaling (default Any).
There is no separate definition identity. Empty and unused definitions are valid.

Preserve strings exactly. Anonymous is independent of any name prefix.
Explodable is editing intent, not a geometry validity constraint.
insertionUnit may differ from the resource unit: it describes insertion/copy
intent and never silently rescales existing geometry or bounds.
BasePoint is subtracted once per insertion, never pre-applied to stored children.
Scaling Uniform requires exact signed equality sx == sy == sz in every targeting
instance. (-2,-2,-2) is uniform; (-2,2,2) is not. Any allows unequal components.

## Block transforms

`block.transforms`: BlockInstance is an ordinary drawable entity with
entityId, containing scopeId, target definitionScopeId, transform, layerId,
appearanceId and visible (default true). Its owner may have any scope kind;
its target must be a definition in the same resource. An instance occupies one
position in the owner's order; definition children stay in their own scope/order.

Transform is a required logical record. Its independently defaulted members are
placement (identity PlanePlacement), rotation (zero radians, not normalized),
and scale (dimensionless Scale3, default 1,1,1). Explicit PlanePlacement, Point3
and Scale3 values must be complete records. Every scale component is finite and
nonzero, including negative/subnormal values; rotation is finite. No minimum
CAD-codec scale restriction or near-zero snapping belongs to this contract.

Placement follows the existing plane.frame predicates. Stored axes are
authoritative and are neither normalized nor required to be arbitrary-axis
neutral. Neutral placement plus separate rotation is an authoring/conversion
convention, not a universal format requirement.

For a definition point p, definition base b, placement origin O, axes U,V,
rotation t and scale s:

```text
d = p - b
N = U cross V
Xr = cos(t)*U + sin(t)*V
Yr = -sin(t)*U + cos(t)*V
parentPoint = O + d.x*s.x*Xr + d.y*s.y*Yr + d.z*s.z*N
```

For b=(2,0,0), O=(0,1,0), identity axes/unit scale/zero rotation, b maps to
(0,1,0) and the definition origin maps to (-2,1,0). Evaluate polyline placement
first, then nested instances inner-to-outer. The composite may shear; do not
require decomposition back into one BlockTransform. No additional affine
transform, normal or prepared representation is persisted.

## Block graph

`block.graph`: The complete definition dependency graph is acyclic, including
definitions unreachable from model/paper. Self-cycles, indirect cycles, missing
targets and wrong-kind targets are structural violations. Check references and
cycles before evaluating bounds. Never fabricate an empty definition to repair
a missing target. Shared DAG nodes and instances of genuinely empty definitions
are allowed. Every entity still obeys identity and order.coverage across streams.

## Name uniqueness

`name.uniqueness`: For layer names within a resource and definition names
within that resource, compare Unicode 17.0.0 full default case folds, using C/F
mappings from [the pinned data](unicode/CaseFolding-17.0.0.txt).
Do not apply Turkic tailoring, trim whitespace, remove accents or normalize
Unicode. Preserve the original name. Door/DOOR, Straße/STRASSE and σ/ς collide.
Door and "Door " differ; composed/decomposed accented forms are not merged merely
by canonical equivalence. IDs and case-sensitive unit tokens are unaffected.

## Unit meaning

`unit.meaning`: Resource unit and definition insertionUnit use the same
case-sensitive token domain. CAD integer codes are not the IFCDR representation.
No aliases are accepted. Coordinate evaluation has no hidden insertion-unit,
paper-unit or plotting-scale multiplier.

| Tokens | Exact metres per unit, respectively |
| --- | --- |
| unitless | No physical factor |
| mm, cm, m, km | 1/1000, 1/100, 1, 1000 |
| in, ft, yd, mi | 127/5000, 381/1250, 1143/1250, 201168/125 |
| microin, mil | 127/5000000000, 127/5000000 |
| angstrom, nm, um | 1/10000000000, 1/1000000000, 1/1000000 |
| dm, dam, hm, Gm | 1/10, 10, 100, 1000000000 |
| au | 149597870700 |
| ly | 9460730472580800 (Julian year of 365.25 days) |
| pc | 648000 * 149597870700 / pi |
| usSurveyFoot, usSurveyInch | 1200/3937, 100/3937 |
| usSurveyYard, usSurveyMile | 3600/3937, 6336000/3937 |

Rational factors are exact fractions, not rounded decimal constants. Parsec's
exact definition involves mathematical pi. A converter may use a fixed certified
pi interval for a physical-tolerance proof; unresolved proof blocks conversion
without declaring excessive deviation or invalid native geometry. Scope bounds
remain native-coordinate bounds, independent of a converter's physical tolerance.

## Viewport entities

`viewport.owner`: A viewport is an object entity in exactly one PaperSpace
scope. `viewScopeId` identifies the same resource's unique ModelSpace; it is
not a cross-resource reference. The viewport occupies one place in its
PaperSpace entityOrder and has the standard layer, appearance, visible and
entity identity rules. Its geometric paper footprint is its frame rectangle
at Z=0, not model geometry projected through the view. Scope bounds must
enclose that footprint even if `viewEnabled` or `visible` is false.

`viewport.view`: Frame center is Point2 in paper coordinates and width/height
are finite and strictly positive. View center is Point2 in model view/DCS
coordinates; target is Point3 in model coordinates; direction is a finite,
nonzero target-to-camera Vector3 whose magnitude is retained; view height is
finite and positive; twist is finite radians. Lens length is finite and
nonnegative when present in Orthographic, where zero is permitted as dormant
CAD state. Perspective requires a finite, strictly positive lens length.
The lens may be omitted only in Orthographic. The camera is
`target + direction`. No separate FOV, camera
position or derived custom scale is stored. `paperHeight / viewHeight` is the
derived orthographic height ratio, before plot mapping.

Front clip mode is Disabled, AtCamera or AtDistance. AtDistance requires a
finite stored distance; other modes may retain a finite dormant distance.
AtDistance's plane is perpendicular to normalized direction at
`target + normalize(direction) * distance`, positive toward the camera.
AtCamera's plane is at `target + direction`. Disabled adds no explicit front
plane; perspective projection still excludes geometry behind the eye. Back
clip mode is Disabled or AtDistance with the same signed distance rule. An
active back plane must lie strictly behind an active front plane on this
axis, with AtCamera at the stored direction norm. Dormant distances do not
participate in this order check.

`viewport.render`: The seven render modes are TwoDimensional, Wireframe,
HiddenLine, FlatShadedWithoutEdges, FlatShadedWithEdges,
SmoothShadedWithoutEdges and SmoothShadedWithEdges. `viewEnabled` controls
model display, `visible` the rectangular border role, and `viewLocked` only
editor changes to the view. The optional plotShadingOverride is a complete
ShadedPlot value. Custom quality requires dpi 100..32767; other quality modes
forbid dpi. Unknown modes are invalid.

`viewport.clip`: PaperClip always has an enabled boolean. When enabled,
boundaryEntityId is required. A present ID, even when disabled, must resolve
to an entity in the same PaperSpace. No boundary ID may be claimed by more
than one viewport. Active boundaries in this version are closed straight
polylines whose placed vertices lie in the paper XY plane, with at least
three distinct transformed vertices and all vertices within or on the frame
rectangle. Self-intersection and zero signed polygon area are not in
themselves invalid. An inactive boundary need not pass the active aperture
shape/enclosure checks. The boundary entity has its own order, layer,
appearance and visibility; those govern its border, not clipping.

`viewport.overrides`: Each logical override is an ordered record of layerId,
frozen and nullable appearanceOverrideId. The referenced patch uses the
existing appearanceOverride table; every non-null stored patch value is
validated even if the layer is frozen. A viewport may override a layer at
most once. `frozen=false` plus null appearanceOverrideId is a no-op and
invalid. Absence or false never thaws a globally frozen layer. Effective
model visibility is layer.visible && !layer.frozen && !viewportOverride.frozen.
Appearance values remain symbolic: ByLayer uses viewport-effective layer
values, including through nested blocks; ByBlock follows the containing
instance recursively. Layer 0 in definition geometry inherits its
containing instance's effective layer before this lookup. Do not mutate
shared definition geometry or scope bounds to resolve a viewport appearance.
