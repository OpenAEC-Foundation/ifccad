# IFCX-CAD document lifecycle

`IfcxCadDocument` is the owned logical CAD projection used by validation,
encoding and direct CAD conversion. It contains typed entities, ordered owner
contents, layers, layouts, shared blocks, named patterns, units and allocation
history. It remains independent of `OcdrawDocument` and opencadcodec.

## Complete source and CAD projection

`load_ifcx_cad_bytes(bytes, IfcxCadReadOptions)` returns
`Result<ValidatedIfcxCad, IfcxCadReadError>`. Options default to LaterWins and
can select RejectConflicts. Read errors expose the existing `IfcxCadReport`;
`load_ifcx_cad_file(path, options)` separately distinguishes IO and read failures.
A valid result pairs a validated CAD projection with a separate immutable
`LoadedIfcxGraph`:

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

`LoadedIfcxGraph` does not establish general IFCX conformance. The reader retains
its current alpha composition, single supported CAD drawing and limited schema
import resolution. General inheritance evaluation, multi-drawing loading and
remote import resolution remain outside this implementation.

## Editing and validation

Borrow and clone a document to edit an independent copy, or use consuming
accessors to take ownership. Public mutable fields are not a validation guarantee.
Call `validate_ifcx_cad_document(&document)` after changes.

Shared logical validation checks header metadata, identities and watermarks,
units, geometry, placement/transforms, appearances, patterns, layout metadata,
references and block cycles. Entity IDs are unique across Model, Paper and all
block contents. Owner vectors define both membership and draw order. Validation
does not repair content, reorder definitions, renumber identities or infer
allocation history. It runs without an encoding/readback bridge or CAD runtime.

The reader separately checks JSON token/field backings, envelope and schema
requirements, node paths, CAD attribute combinations and graph relationships.
After projection it calls the same logical validator used by encoding and
direct conversion. Existing geometry and unit-registry sharing with OCDraw
continues; the drawing models and encoding routes stay separate.

The five uint64 allocation watermarks must exceed existing IDs in their domains.
Editors retain them when deleting content, including when a domain becomes empty.
See [ID management](ifcx-cad-id-management.md).

## Encoding and original download

`encode_ifcx_cad_document(&document)` validates the typed document, maps it to
the current IFCX CAD profile, and requires strict production-reader readback
with semantic equivalence, returning `EncodedIfcxCad`. Its `bytes()` and
`into_bytes()` accessors expose the encoding; `write_file()` creates a new file
and refuses to overwrite an existing one. Existing paths, uint64 values, ordered entities and supported native
Paper semantics are retained. Paper collections normalize by explicit tab index, independently of IDs; Model is tab 0 and Paper tabs are contiguous from 1. Paper coordinate units are distinct from optional physical medium units. Missing metadata in older experimental files is rejected; examples are migrated directly without a version layer. Definition collections retain their existing
unordered/readback normalization rules.

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

- `cad_document_to_ifcx_cad_document`: constructs and validates a logical
  projection without encoding or reading an IFCX file. The outcome exposes
  `document()`, `into_document()`, diagnostics and source-handle mappings.
- `ifcx_cad_document_to_cad_document`: validates the supplied logical document
  before creating the CAD result. It assesses CAD content and target limitations
  within that input.

`cad_document_to_encoded_ifcx_cad` calls logical conversion, then core encoding
and production loading. `ifcx_cad_source_to_cad_document` takes `ValidatedIfcxCad`
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

The paperspace slice extends conversion coverage; existing source recoveries and appearance adaptations remain in force. Fresh CAD imports allocate fresh native IDs; a CAD-runtime
roundtrip does not preserve native allocation history automatically. Multiple authored Paper layouts, supported geometry, tab order and optional media convert. Authored viewport/view state and full plot settings remain deferred, with located loss diagnostics.

## Typed failures

`IfcxCadReport` retains the full ordered diagnostic list. `IfcxCadReadError`
exposes it through `report()` and `std::error::Error::source()`.
`IfcxCadEncodeError` distinguishes logical `InvalidDocument`, JSON
`Serialization`, production-reader `Readback` and valid-reader
`SemanticMismatch` failures. Every variant retains its original typed cause;
`report()` remains available when the phase has a validation report.

Conversion distinguishes `CoreValidation(IfcxCadReport)`,
`CoreEncoding(IfcxCadEncodeError)`, `CoreReadback(IfcxCadReadError)` and
`IdAllocation(IfcxCadIdAllocationError)`. Validation applies to logical input;
encoding includes its own strict readback; conversion readback identifies the
additional production load after encoding. Matching variants or following
`source()` avoids parsing presentation strings. Existing loss policies and
native file semantics are unchanged.

## Deferred source writeback

Updating the original IFCX graph from an edited CAD projection requires a later
design covering fragment authorship, deletion/replacement, foreign references,
shared nodes and unknown content. This implementation adds no partial merge
function. CAD editing-session identity mapping, collaborative editing and
additional encodings remain independent follow-ups.

## Evidence

Core lifecycle tests cover full source retention, separate ownership, invalid
direct edits, all supported fixtures and full-width/empty-domain counters.
Converter lifecycle tests compare logical and encoded routes, invalid direct
input, foreign information and source precision under both loss policies.
Existing pinned DXF/DWG exchange and browser original-download checks remain
the end-to-end evidence. See the converter's two coverage contracts for its
supported subset and loss boundaries.
