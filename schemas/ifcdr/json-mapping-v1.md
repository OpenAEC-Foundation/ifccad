# IFCDR JSON mapping language v1

This document normatively defines `ifccad.ifcdr.jsonMapping.v1`. It applies to
mapping documents using that identifier, including
[the IFCDR 0.7.0 mapping](json-mapping-0.7.0.json). The
[meta-schema](json-mapping-meta-schema-v1.json) checks the structure of a mapping
document; this document defines its interpretation. MUST, MUST NOT and MAY
express requirements and permitted choices.

The mapping identifies physical representations of logical properties. Their
types, defaults, references and semantic constraints belong to the applicable
logical registry and contract. A structurally correct mapping or physically
well-formed resource does not establish logical or package validity.

## Names and direct fields

`logical` names a property as `<collection>.<property>`, or
`resource.<property>` for resource metadata. `name` identifies a logical table
or stream. These names MUST resolve in the registry for `ifcdrVersion`.
Every stored logical property MUST have one mapping entry; a range entry maps
the complete sequence property. Physical offset/count fields do not introduce
additional logical properties.

`payload` identifies the physical location. At resource level, or on a table,
stream or directory, dot-separated segments select object members from the
resource root. Within a table's `fields`, it names a member of each row object;
within a stream's `fields`, it names a column in that stream object. These are
fixed member locations, not expressions, JSONPath, filesystem paths or URLs.
Mapping v1 provides no wildcard, array-index or escaped-dot path syntax.

Tables are arrays of row objects. Streams are objects with a `count` and named
column arrays. Each ordinary row column MUST contain exactly `count` values;
array index `i` identifies the same logical row across those columns. Pool
columns are the exception described below. Resource fields are single values.

`resource.constants` assigns required literal values at root-relative member
locations, such as `header.format` and `header.version`. `valueMappings` maps
logical enum members to stored JSON values. For appearance modes these are
`ByLayer = 0`, `Explicit = 1`, and `ByBlock = 2`; other codes are invalid.

## Omission and null

`omission: forbidden` means the mapped member MUST be present, even when its
logical value is null or the containing table/stream is empty.
`omission: logicalDefault` permits absence only by applying the default from
the logical registry. It does not declare another default. For a missing
defaultable row column, the default is applied independently to every row.
For a missing defaultable table, the table default applies (currently `[]`).

For example, absent `line.visible` means every line is visible. An explicitly
stored `false` remains false. Null is not omission: it is accepted only where
the logical field is nullable. Required empty-resource bounds are stored as
`"bounds": null`. Optional members inside logical records follow the record's
own optional/nullable declarations. Record members retain their logical names;
RGB is an array of three channels.

## Common range rules

`offset` and `count` name row columns in the containing stream. Their values
MUST be unsigned 32-bit integers. Indexing starts at zero. For parent row `i`,
let `o = offset[i]` and `n = count[i]`. Its ordered sequence selects the
half-open interval `[o, o + n)` in the target storage, preserving that order.

Implementations MUST calculate the endpoint without wrapping or truncation
(for example using checked or sufficiently wide arithmetic). Both `o` and
`o + n` MUST be within the target length. An empty range MAY start at the
target's end. A physically valid empty range may still violate a logical
minimum such as a polyline's two-vertex requirement.

Negative, fractional or out-of-domain offsets/counts, missing required
columns, wrong column lengths, and ranges exceeding the target MUST be
rejected. Integer identity values MUST retain their full declared precision;
decoding through a floating-point intermediate that rounds an ID is invalid.

## `pointPoolRange`

This form represents a sequence of XY points using two parallel numeric pool
columns in the same stream. `pools` MUST name exactly two distinct columns:
the first supplies X and the second supplies Y. Both arrays MUST have the
same length. Every stored coordinate MUST be a finite number, including
coordinates that no range selects.

For sequence element `j` of row `i`, where `0 <= j < count[i]`:

```text
k = offset[i] + j
point[j] = (firstPool[k], secondPool[k])
```

For the 0.7.0 polyline mapping these names are `vertexOffset`, `vertexCount`,
and `pools: ["x", "y"]`. For example:

```json
{
  "vertexOffset": [0, 3],
  "vertexCount": [3, 2],
  "x": [0, 10, 10, 20, 30],
  "y": [0, 0, 10, 20, 20]
}
```

The first sequence is `(0,0), (10,0), (10,10)`; the second is
`(20,20), (30,20)`. This is a fragment, not a complete IFCDR stream.

Ranges MAY overlap, be shared, or occur in a different physical order from
their parent rows. Pool points MAY remain unselected. For example offsets
`[1,1]` and counts `[2,3]` select overlapping sequences; the first pool point
is unused. Sharing creates no logical vertex identity or edit linkage between
polylines. Unselected points are physical storage, not drawing geometry, and
do not contribute to logical bounds.

A writer MAY repack pools, duplicate shared points or omit unselected points,
provided every logical point sequence is preserved exactly. Pool offsets and
unused values have no logical roundtrip-preservation guarantee. Closure is a
separate logical property; decoding MUST NOT append or remove a vertex based
on the `closed` flag or endpoint equality.

## `childRange`

This form represents a sequence through rows of a child stream. `target`
identifies the logical child property, for example `entityOrderEntry.entityId`.
Resolve its stream and field mapping to locate the target column. The range
selects that column's values in row order. The child column MUST have the
child stream's declared row count.

In mapping v1, child ranges MUST form a complete contiguous partition of
the child rows in **parent stream row order**:

```text
offset[0] = 0
offset[i + 1] = offset[i] + count[i]
sum(count) = child row count
```

Thus overlap, gaps, reordering of ranges, and unselected child rows are
invalid. With no parent rows the child row count MUST be zero. Empty ranges
are permitted and do not advance the offset. A nonempty range requires its
target payload to be present.

For example offsets `[0,2]`, counts `[2,1]`, and child entity IDs `[10,5,11]`
produce the sequences `[10,5]` and `[11]`. Offsets `[0,1]` overlap; offsets
`[0,3]` leave a gap. Both are invalid for those counts and three child rows.
Whether each selected entity exists and belongs to the parent scope is a
separate logical reference/order check.

## Directory and validation responsibilities

`directory` locates the stream-directory object and identifies its schema and
required/optional metadata fields. Directory v1 lists present streams, their
logical names, schema IDs, roles, row counts and physically present columns.
Counts MUST match their payloads, and the declared column set MUST match the
present payload columns. Parent/child declarations MUST agree with registered
relationships. An omitted defaultable column is also absent from that set.

The JSON codec checks this physical representation before constructing typed
resource data. Shared validation then checks logical rules such as vertex
minimums, entity identity, references, bounds and order coverage. Package
validation checks external IFCX links. Unsupported encoding forms or stream
schemas cannot be silently interpreted as a supported form.

The meta-schema alone does not verify logical-name resolution, mapping
completeness, ranges in resource files, or semantic correctness. Implementers
must apply the applicable registry, these representation rules, and the
logical contract together. The mapping is a format specification, not a
requirement to implement a general-purpose mapping interpreter.
