# OCDraw document lifecycle

`OcdrawDocument` is the complete, owned logical drawing: typed entities,
ordered scope membership, definitions, layouts and authored drawing state.
It has no JSON columns, stream directory, geometry pools or opencadcodec dependency.
A different physical encoding can reuse this model and its semantic validator;
its encoder, decoder, mapping and conformance evidence must implement the same
logical contract. `OcdrawDocument` and IFCX-CAD remain separate domain models.

## Construction, reading and editing

`OcdrawBuilder::build_document()` consumes a fresh builder, moves its geometry
into an `OcdrawDocument`, prepares scope bounds and validates the result.
`finish()` remains a convenience wrapper over `build_document()` followed by
`encode_ocdraw_document()`. Both paths use the same final encoder.

The production reader retains a validated immutable snapshot. Use
`ValidatedOcdraw::document()` to borrow the logical document, `clone()` to edit
an independent copy, or `into_document()` to consume the snapshot without a
whole-document clone. `load_ocdraw_bytes()` returns
`Result<ValidatedOcdraw, OcdrawReadError>`; invalid and unsupported-version input
retain structured diagnostics through the error. `load_ocdraw_file()` separately
distinguishes storage failures from read failures. Typed reader accessors remain available.

Before checking individual encoded rows, the reader checks that stream counts
have exact unsigned integer backings in the addressable range, required columns
exist and row-column lengths match. It rejects malformed structures before
traversing their declared rows, including tiny inputs with enormous counts.

Public document and entity fields can be edited directly. Raw mutable content
is not a validation guarantee: call `validate_ocdraw_document()` after changes.
This checks typed scalar values, identities, references, ownership, block
cycles, geometry, layouts, saved state and declared enclosures without encoding.
Errors expose structured diagnostics. Tagged enum choices and checked frame
and transform types prevent some malformed states by construction.

## Identity and allocation history

Entity IDs are drawing-wide across geometry and viewports. Scope membership
is the authoritative ordered ownership list. Table IDs, layout IDs, scope IDs
and layout tab indices have distinct roles: IDs need not be contiguous or equal
to a vector position. Model identity is determined by its kind, not ID zero.

The document retains all four allocation watermarks: `next_entity_id` (u64),
`next_layer_id`, `next_layout_id` and `next_line_pattern_id` (u32).
Encoding retains them exactly, including empty tables and integers above the
exact range of a floating-point JSON number. The reader requires an exact
unsigned integer backing for watermarks, rejecting tokens such as `6.0` or
numbers beyond the namespace range. Each watermark must exceed every
existing ID in its namespace. Scopes, UCS definitions and model windows do not
acquire new watermarks in this slice.

An editor must carry forward these watermarks when deleting entities. For
IDs `{1, 3, 4}` and next entity ID `6`, deleting `4` leaves the next ID at `6`.
Assign `6` to a new entity and advance the watermark to `7`. Rewriting the
drawing preserves this caller-supplied history. Validation cannot detect
previously deleted IDs if the caller has already discarded their history.
There is no automatic editing allocator, copy-merge identity protocol or
conflict resolution in this API. A maximum watermark is valid but indicates
that allocation of another representable ID needs an explicit exhaustion policy.

## Explicit bounds preparation

`encode_ocdraw_document()` validates the supplied document and retains valid bounds,
including conservative oversized bounds. It does not recompute or repair them.
Missing, unordered, nonfinite or insufficient bounds are validation errors;
an empty scope has no bounds. Call `recompute_ocdraw_document_bounds(&mut doc)` when
new enclosures are wanted after a geometry change.

Preparation ignores the old bounds but validates other content first. It derives
leaf and viewport enclosures, resolves nested blocks from their newly computed
definition bounds and unions each scope. All results are staged before any
bounds are replaced. A reference, cycle, semantic or numerical error leaves
every supplied bound intact. Other records, IDs, membership and watermarks stay
unchanged. An empty block instance retains the insertion-origin point enclosure
used by the fresh builder.

## Encoding and storage

`encode_ocdraw_document(&doc)` borrows and validates the complete document, maps it to
the current JSON encoding, and requires strict production-reader readback before
returning `EncodedOcdraw`. Geometry is not cloned solely for column grouping.
Repeated encoding of the same document is deterministic. Different vector
orders are allowed and are not promised to serialize to the same bytes.

`OcdrawEncodeError` distinguishes invalid input, serialization failure and
unexpected readback failure. Storage stays separate: `EncodedOcdraw::write_file`
creates a new file and refuses to overwrite an existing file. Applications own
replacement, backup and editor-session policies.

## CAD conversion and future editing sessions

`cad_document_to_ocdraw_document` and its `_with_id` variant return the logical
document together with export diagnostics, source-handle/entity-ID mapping and
geometry assessment. Existing encoded exports call this route and then the core
encoder. `ocdraw_document_to_cad_document` validates raw input before CAD
construction; `ocdraw_source_to_cad_document` uses the reader's immutable snapshot.
Both imports share one typed conversion implementation without a JSON bridge.
The pinned source coverage, loss policies and numerical tolerances are unchanged.

A CAD export constructs a fresh drawing and allocates fresh OCDraw IDs. A CAD
import allocates fresh CAD handles. An identity-preserving save after editing
in a CAD runtime will need a future explicit session context linking those
identities and retaining unmapped authored content and allocation history.
Giving a changed `CadDocument` a drawing ID alone does not provide that context.
An editor working directly on `OcdrawDocument` can retain these values itself;
core editing commands are not required to write its updated document.
