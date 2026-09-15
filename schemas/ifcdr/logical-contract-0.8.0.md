# IFCDR 0.8.0 logical contract

This document, [the logical registry](registry-0.8.0.json), and the applicable
encoding mapping define the IFCDR 0.8.0 contract. The
[JSON reference mapping](json-mapping-0.8.0.json) specifies physical storage.
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

`geometry.finite`: Line endpoints and scope base coordinates are finite XYZ
values. Polyline vertices are finite local XY values. Coordinate lengths use
the declared resource unit. Endpoint Z defaults independently to zero. No
implicit Z default exists on a general point3/vector3 record. No bulge, width
or curved segment is implied. Scope kind and flags remain unsigned 32-bit
metadata. Scope bases are retained without adding them to entity coordinates
or inferring IFCX placements.

A line with equal endpoints remains a valid line. Repeated polyline vertices
and zero-length segments are valid and must be retained. Encoders do not merge,
discard, reorder, or change entity kinds to normalize geometry.

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
is X cross Y. This frame is distinct from future block/instance transformations.

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

`bounds.enclosure`: Every scope has nullable bounds. A scope with no drawable
entities has null bounds; a nonempty scope has finite min/max bounds on XYZ,
with min <= max on each axis. Bounds enclose the exact rational geometry of
all line endpoints and all placed polyline vertices belonging to that scope.
Straight segments, including a closing segment, lie within their endpoint
bounds. Independent model/paper scopes are never combined into one bound.

Include invisible geometry; exclude lineweight and other presentation effects.
Do not add scope bases, instance transforms or package placements. Conservative
bounds and point-sized bounds are valid. Bounds are checked, not repaired, on
read; an encoder preserves supplied valid bounds. A writer computing new bounds
rounds outward and may use a conservative interval enclosure.

There is no positional epsilon for under-enclosure, including when a converter
previously accepted coordinate rounding. A rounded evaluation point is not a
proof of exact enclosure. Refine arithmetic when a calculation interval crosses
a supplied bound; a loose interval alone does not establish a violation.
Intermediate overflow alone is not invalid input. If exact placed geometry
exceeds the finite binary64 range, no finite bounds can enclose it. An operation
unable to complete a proof must not confer validated status or report numerical
uncertainty as an established contract violation.

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
value does not become ByLayer. ByBlock does not introduce block entities.

## Appearance source

`appearance.source`: At package validation, resolve IFCX layer/appearance and
line-pattern references to the appropriate existing definitions. Retain the
case-insensitive uniqueness rule for layer names within a resource. Explicit
properties use a non-null local override first, otherwise the referenced IFCX
appearance value. An invalid override does not fall back. An explicit property
without a usable source is invalid. ByLayer/ByBlock inheritance stays symbolic.
Permitted IFCX extension fields remain in the graph; typed appearance access
is not a replacement serialization of that graph.

## JSON reference representation

The [JSON mapping language v2](json-mapping-v2.md) defines payload locations,
omission, nulls, value mappings, point pools, child ranges and directory rules.
The [0.8.0 mapping](json-mapping-0.8.0.json) applies those forms to this
registry. In particular, point-pool ranges may overlap and leave unused
physical points, while child ranges partition the complete child stream in
parent row order. Neither storage sharing nor unused pool points changes
logical geometry. Empty-scope bounds are represented by JSON null.

Only the registered tables, streams, fields and declared directory metadata
are part of this closed IFCDR contract. An unsupported version or stream
prevents a complete resource proof. Report support limits without claiming
invalidity under an unknown contract. Skip semantic checks that require
unavailable data; independent established errors may still be collected.
