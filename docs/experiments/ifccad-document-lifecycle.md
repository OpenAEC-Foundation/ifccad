# IFCCAD document lifecycle

Native model/validation/storage sections remain active. Direct `ifccad-convert`
exchange and codec qualification below describe the [archived reference](../converter-parking.md);
the [shared Document route](../architecture/explorer-document-route.md) has its own coverage.

`IfccadDocument` is the owned logical CAD projection used by validation,
encoding and direct CAD conversion. It contains typed entities, ordered owner
contents, layers, layouts, shared blocks, named patterns, units and allocation
history. The active root package exports these types directly from `ifccad`;
implementation lives under `src/ifccad`. The core remains independent of
opencadcodec. OCDraw is [parked](../ocdraw-parking.md).

## Provisional standalone CAD scope â€” 2026-10-21

Near-term development targets standalone CAD-profile documents in IFCX: one
supported drawing with its geometry, tables, styles, layouts, workspace state,
required IFCX envelope/schema context and approved preservation content. IFCX
remains the logical foundation; the typed drawing API and physical encodings
represent the same supported drawing semantics. This is a development-scope
decision, not a new reader restriction or a published compatibility guarantee.

Native editing and fresh encoding target that supported drawing content.
Projection output does not promise the original fragment distribution,
overwritten contributions, JSON spelling or a complete mixed IFCX source graph.
Unchanged original IFCX download continues to retain source bytes. Existing
fragment composition, immutable source retention, source-aware conversion
audits and Allow/Reject loss policies remain in place. Mixed input must not
silently be presented as a lossless edited source document; projection export
has its existing narrower scope. Unknown core fields remain rejected.

General editing of CAD inside a broader IFCX document is deferred until a
concrete application supplies source examples, non-CAD relationships and
required edits. That design must address contribution targeting, identity,
ownership, deletion, foreign references, origin/import context and validation.
No full-source-plus-CAD-cache container, general IFCX CBOR mapping, fragment
authoring system or source-graph writeback is required for current CAD work.
Reading existing fragments does not imply these editing capabilities.

CAD semantic coverage and qualified exchange retain the sequence in
[ROADMAP.md](../../ROADMAP.md). Keep JSON, CBOR and shared validation aligned as
semantics evolve, with production readback. The next optional storage
investigation is a separately requested, bounded comparison of whole-document
JSON, CBOR and native CBOR + Zstd; see [CBOR follow-up](ifccad-cbor.md#next-investigation).
This decision neither starts measurements nor changes mapping/transport revisions.

Indexed chunks, columnar encoding and partial save remain evidence-driven
follow-ups. A host that fully loads IFCCAD, converts the complete drawing to
`CadDocument` and reconstructs it on save does not itself exploit partial
access or establish source-preserving updates. Revisit chunking when a named
consumer can use selected content or local updates, or full-document profiling
demonstrates another benefit sufficient to justify its costs. Complete drawing
validation remains distinct from independent chunk decoding.

## Complete source and CAD projection

The [standalone CBOR candidate](ifccad-cbor.md) encodes the same editable
document directly, with a validated binary wrapper that has no source graph.
It preserves supplied vector order and float bits. Original IFCX fragments,
foreign information and JSON-source auditing remain specific to the route below.

Native [drawing defaults](../drawing-defaults.md) are applied after fragment
composition. Compact XY line endpoints expand to XYZ, and omitted native
appearance properties become ByLayer. Source bytes and opaque/authoring presence
semantics remain independent of these physical omission rules.

`load_ifccad_bytes(bytes, IfccadReadOptions)` returns
`Result<ValidatedIfccad, IfccadReadError>`. Options default to LaterWins and
can select RejectConflicts. Read errors expose the existing `IfccadReport`;
`load_ifccad_file(path, options)` separately distinguishes IO and read failures.
A valid result pairs a validated CAD projection with a separate immutable
`LoadedIfccadGraph`:

- `document()` borrows the typed CAD projection.
- `graph().source_bytes()` retains the exact input, including numeric tokens,
  whitespace and fragment order.
- `graph().composed_ifcx()` exposes the complete composed envelope and nodes,
  including information outside the CAD profile.
- `graph().composition_policy()` records LaterWins or RejectConflicts.
- `into_document()` consumes the result and extracts its editable CAD projection.
- `into_parts()` extracts both owned objects independently.

The graph's JSON backing belongs to the current loader; typed CAD conversion
has no JSON bridge.

The snapshot records the input. Editing an extracted CAD document does not
update its source bytes or composed graph. Keeping both objects does not
provide an automatic editing session or synchronization mechanism.

`LoadedIfccadGraph` does not establish general IFCX conformance. The reader retains
its current alpha composition, single supported CAD drawing and limited schema
import resolution. General inheritance evaluation, multi-drawing loading and
remote import resolution remain outside this implementation.

## Editing and validation

Borrow and clone a document to edit an independent copy, or use consuming
accessors to take ownership. Public mutable fields are not a validation guarantee.
Call `validate_ifccad_document(&document)` after changes.

Shared logical validation checks header metadata, identities and watermarks,
units, geometry, placement/transforms, appearances, patterns, layout metadata,
references and block cycles. Entity IDs are unique across Model, Paper and all
block contents. Owner vectors define both membership and draw order. Validation
does not repair content, reorder definitions, renumber identities or infer
allocation history. It runs without an encoding/readback bridge or CAD runtime.

The reader separately checks JSON token/field backings, envelope and schema
requirements, node paths, CAD attribute combinations and graph relationships.
After projection it calls the same logical validator used by encoding and
direct conversion. Neutral geometry and unit helpers belong to the active core;
the parked OCDraw implementation is not an active dependency.

The uint64 allocation watermarks must exceed existing IDs in their domains.
Editors retain them when deleting content, including when a domain becomes empty.
See [ID management](ifccad-id-management.md).

## Encoding and original download

`encode_ifccad_document(&document)` validates the typed document, maps it to
the current IFCX CAD profile, and requires strict production-reader readback
with semantic equivalence, returning `EncodedIfccad`. Its `bytes()` and
`into_bytes()` accessors expose the encoding; `write_file()` creates a new file
and refuses to overwrite an existing one. Existing paths, uint64 values,
ordered entities and supported native Paper semantics are retained. Paper
collections normalize by explicit tab index, independently of IDs; Model is tab
0 and Paper tabs are contiguous from 1. Effective plot mappings determine Paper
coordinate meaning independently of optional medium dimensions; see
[layout output](../layout-output.md). Missing required metadata in earlier
experimental files is rejected; examples migrate directly without a version
layer. Definition collections retain their documented readback normalizations.

`EncodedIfccad`, `EncodedIfccadCbor` and feature-enabled `EncodedIfccadCborZstd`
implement `AsRef<[u8]>`, `PartialEq` and `Eq`. Equality compares exact encoded
bytes; different encodings can represent equivalent logical drawings. Only
validated encoding constructs these values. Replacement, backup and editor-session
save policies belong to applications.

Encoding creates a **new CAD-profile file** with its normal schema import.
It receives no original source graph. Foreign nodes, extra imports/schemas,
foreign relationships and original fragment history are outside that output.
It does not merge an updated CAD projection into a larger IFCX graph.

For an unchanged original download, use the exact source bytes. The explorer
continues providing those bytes for native IFCX download and uses the composed
graph for inspection. Filesystem replacement, backup and save policies remain
application responsibilities.

## Direct CAD conversion

The companion converter exposes these document routes, each requiring explicit
directional options for Allow/Reject behavior:

- `cad_document_to_ifccad_document`: constructs and validates a logical
  projection without encoding or reading an IFCX file. The outcome exposes
  `document()`, `into_document()`, diagnostics and source-handle mappings.
- `ifccad_document_to_cad_document`: validates the supplied logical document
  before creating the CAD result. It assesses CAD content and target limitations
  within that input.

`cad_document_to_encoded_ifccad` calls logical conversion, then core encoding
and production loading. `ifccad_source_to_cad_document` takes `ValidatedIfccad`
and additionally checks the full composed source for foreign content and lossy
numeric projection before using the common typed conversion implementation.
These source-aware checks remain subject to the same Allow/Reject policy;
precision failures remain fatal under both policies.

A projection-only call has no source graph from which to diagnose omitted
foreign content or original numeric tokens. Applications assessing loss from
a complete IFCX input must use the source-aware route. A geometry integer that
rounds during projection remains detectable there even when the projected f64
value alone looks valid. Full-width allocation integers remain exact in typed
state and native output.

Fresh CAD imports allocate fresh native IDs; a CAD-runtime roundtrip does not
preserve native allocation history automatically. Multiple authored Paper
layouts, tab order, optional media, supported effective plot settings and
viewport state have their own qualified mappings. See the
[converter contract](../../legacy/ifccad-convert/reference/crates/ifccad-convert/README.md),
[workspace state](../workspace-state.md) and [layout output](../layout-output.md)
for current coverage and codec limits. A historical viewport repair or practice
conversion does not establish coverage for a later dependency.

## Responsibilities and public API conventions

| Responsibility | Active owner |
| --- | --- |
| Owned drawing and shared semantic validation | Core `logical/`, `IfccadDocument` and checked ID allocators |
| IFCX composition and immutable source | Core `source/`, `LoadedIfccadGraph` |
| Physical mapping and backing validation | Core `codec/json/`, `codec/cbor/` and the separate Zstd transport |
| Validation, encoding and production readback | Core `encode.rs` |
| Encoded bytes and standalone file IO | Core `storage.rs` and `storage/` |
| Active application CAD mapping | Shared `opencaddocument`, through viewer/browser |
| Historical source audits, certified CAD preparation and snapshot restoration | Archived `ifccad-convert` and its six `cad-*` adapters |

Keep construction, logical validation, conversion, physical encoding and file
storage distinct. Public mutable records permit editing but do not certify
validity. No encode/load bridge or CAD runtime is required for logical validation.
No deprecated OCDraw or former `IfcxCad*` facade is exported.

The archived reference API's four converter routes name both direction and backing explicitly:

| Route | Result and scope |
| --- | --- |
| `cad_document_to_ifccad_document` | Validated logical drawing, diagnostics and identity mappings |
| `cad_document_to_encoded_ifccad` | Logical conversion followed by production-checked IFCX encoding |
| `ifccad_document_to_cad_document` | Validated drawing projection to CAD |
| `ifccad_source_to_cad_document` | Loaded IFCX source to CAD, including foreign-content and numeric-source audits |

CAD-to-IFCCAD metadata is supplied through `IfccadTargetMetadata`. Both
directions require their own options: `CadToIfccadOptions` and
`IfccadToCadOptions`; `Default::default()` selects Allow. Outcomes expose
`document()`/`into_document()` or `encoded()`/`into_encoded()`, diagnostics and
mappings. Directional coverage remains in the converter's
[FROM-CAD](../../legacy/ifccad-convert/reference/crates/ifccad-convert/docs/FROM-CAD-COVERAGE.md) and
[TO-CAD](../../legacy/ifccad-convert/reference/crates/ifccad-convert/docs/TO-CAD-COVERAGE.md) inventories.
Located recovery, certified rounding and semantic loss remain distinct.
Unit-aware hard numerical limits are explained in
[shared geometry](../geometry/shared-geometry.md).

CBOR uses `load_ifccad_cbor_bytes`, `load_ifccad_cbor_file` and
`encode_ifccad_cbor_document`, with `ValidatedIfccadCbor` and `EncodedIfccadCbor`.
The optional Zstd transport unwraps to that same validated CBOR document. These
routes have no IFCX source graph; their byte/file error types and limits remain
separate from the JSON route. The [CBOR guide](ifccad-cbor.md) documents the
mutable revision-1 mapping, optional transport and exact API signatures.

Entity construction uses `IfccadEntity::Native(IfccadNativeEntity { ... })` or
`IfccadEntity::Opaque(IfccadOpaqueEntity { ... })`. `id()`, `as_native[_mut]()`
and `as_opaque[_mut]()` expose the appropriate variant. Preservation records,
text/dimension styles, symbol sources, UCS and Model windows have their own ID
domains and persistent watermarks. Whole workspace attributes compose as IFCX
values; omitted choices and dormant authored state are retained.

Concrete/symbolic colors, typed line-pattern markers and active/dormant Hatch
gradients have their own construction constraints. Use
[presentation](../presentation.md#foreground-color),
[line patterns](../line-patterns.md), [Hatch](../hatch.md) and
[preservation](../preservation.md) for those values rather than treating prior
API migration examples as an alternative current contract.

## Editing viewport references

Model, Paper and block definitions have optional supplied bounds. Native load
and encode preserve valid supplied boxes, including oversized boxes. Call
`recompute_ifccad_document_bounds(&mut document)` explicitly after geometry
editing; preparation validates and computes all scopes before changing any box.
Failure leaves the document unchanged. Empty scopes have no box. CAD imports
explicitly prepare fresh bounds rather than trusting CAD cached extents.
See [shared geometry](../geometry/shared-geometry.md) for enclosure and tolerance
boundaries; conversion preferences do not alter native reader validation.

Paper viewports use ordinary entity IDs and owner order. Reordering does not
change the referenced Model, clip boundary or frozen layers. Moving a viewport
requires its stored boundary to remain in the same Paper owner, including when
clipping is disabled. Copying a viewport needs a new entity ID and, if it has a
boundary, an independently identified boundary: sharing one boundary between
viewports is invalid. Editors must update those references before validation.
The JSON encoding canonicalizes frozen-layer sets by numeric ID while preserving
all other viewport payload semantics and strict whole-attribute composition.
The [viewport contract](../../schemas/ifccad/experimental-contract-0.2.0.md#paper-viewports)
and candidate conformance fixtures define the current independent profile.

## Typed failures

`IfccadReport` retains the full ordered diagnostic list. `IfccadReadError`
exposes it through `report()` and `std::error::Error::source()`.
`IfccadEncodeError` distinguishes logical `InvalidDocument`, JSON
`Serialization`, production-reader `Readback` and valid-reader
`SemanticMismatch` failures. Every variant retains its original typed cause;
`report()` remains available when the phase has a validation report.

Conversion distinguishes `CoreValidation(IfccadReport)`,
`CoreEncoding(IfccadEncodeError)`, `CoreReadback(IfccadReadError)` and
`IdAllocation(IfccadIdAllocationError)`. Validation applies to logical input;
encoding includes its own strict readback; conversion readback identifies the
additional production load after encoding. Matching variants or following
`source()` avoids parsing presentation strings. Existing loss policies and
native file semantics are unchanged.

## Deferred source writeback

Updating the original IFCX graph from an edited CAD projection requires a later
design covering fragment authorship, deletion/replacement, foreign references,
shared nodes and unknown content. This implementation adds no partial merge
function. CAD editing-session identity mapping, collaborative editing and
further encodings remain independent follow-ups. The standalone CBOR candidate
already described above provides no source-graph writeback either.

## Evidence

Core lifecycle tests cover full source retention, separate ownership, invalid
direct edits, all supported fixtures and full-width/empty-domain counters.
Converter lifecycle tests compare logical and encoded routes, invalid direct
input, foreign information and source precision under both loss policies.
Existing pinned DXF/DWG exchange and browser original-download checks remain
the end-to-end evidence. See the converter's two coverage contracts for its
supported subset and loss boundaries.
