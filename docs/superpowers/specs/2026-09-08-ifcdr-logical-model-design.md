# Encoding-neutral IFCDR logical model

Date: 2026-09-08

Status: Implemented and verified on 2026-09-08. The approved design is
implemented on `ifcdr-logical-model`; repository integration remains a separate
user-authorized step.

The [implementation plan](../plans/2026-09-08-ifcdr-logical-model.md) describes
the execution sequence and verification for this approved design.

## Purpose and scope

Give the reduced IFCDR resource one language-neutral meaning and one shared
semantic validation path. Reading, building, and encoding must agree on that
meaning without sharing a mandatory storage layout or depending on JSON values.

This is the second implementation slice of milestone 2, following the
[implemented base contract](2026-09-08-ifcdr-base-contract-design.md). It covers
lines, straight XY polylines, scopes, layers, appearances, identity, draw order,
defaults, and geometric bounds. It does not complete every milestone 2 exit
criterion in the [roadmap](../../../ROADMAP.md).

Keep the one-drawing, one-model-layout package builder and current converter
coverage. Preserve the reader's multiple-scope capability and all supported
resource fields when re-encoding a resource. A resource encoder must not inherit
the package builder's single-scope limitation.

Inline resources, drawing-representation renaming, broader compatibility report
APIs, and size measurements remain separate milestone 2 work. New entity
families, paperspace conversion, spatial placements, and full IFCPR preservation
remain milestone 3 work. Binary codecs, chunking, lazy loading, incremental
editing, deletion, and ID-reservation APIs are not implementation requirements
of this slice.

## Chosen architecture

Retain separate read and write representations. Give them a shared read-access
trait for the complete logical resource, and use that trait in semantic
validation and encoding. This shares interpretation without requiring the
builder to use reader storage or requiring the reader to retain builder state.

Two alternatives were considered:

- One owned model for reader and writer would be simpler initially, but would
  couple construction and read access to the same allocation and ownership
  choices. There is no evidence that this should be a permanent requirement.
- Sharing only scalar types would retain independent rules for complete
  resources, leaving validators and future encoders to duplicate traversal and
  interpretation. It does not establish the required common boundary.

The shared trait is ordinary composition, not a parent class. Each backing has
its own implementation. Start with a crate-internal trait named
`IfcdrResourceAccess`; keep the public validated resource facade simple. A
public third-party backing/codec extension API is not required now.

```text
JSON bytes -> JSON shape decoding -> decoded typed columns -----+
                                                               |
builder state -> semantic preparation -> prepared write model --+
                                                               |
                                             shared resource access
                                                               |
                                             shared semantic validation
                                                               |
                                             validated logical resource
                                                    /                 \
                                       typed navigation        JSON encoder
```

IFCX package validation adds graph, layout, and external binding checks to the
resource proof. Filesystem loading and storage remain outside this boundary.
The core crate remains independent of cadcodec.

### Logical columns and shared access

The contract retains typed columns grouped by entity kind. Lines have equal
row counts across their logical properties. Polylines have row properties and
an ordered vertex sequence per row. Scope order is a separate sequence of
entity references, allowing line and polyline rows to interleave in draw order.

Access covers resource identity, unit, next entity ID, optional bounds, all
four supporting tables, line and polyline columns, and scope order. It provides
counts, safe indexed access, and borrowed iteration. The exact Rust iterator
signatures belong in the implementation plan; consumers must not require
`Vec<T>`, slices of JSON values, JSON pointers, or a physical chunk type.

Group entity access into typed collections: `resource.lines().len()` and
`resource.lines().get(index)`, with equivalent polyline collection access.
Polyline views provide their own vertex access. Keep entity-specific details
off the resource trait; a future supported family adds a collection accessor
and its own typed views. Each backing implements these views over its own
storage without materializing a new collection. Mixed entity iteration in
scope draw order remains available alongside access by entity kind. The
internal trait enumerates all scope-order records through `orders()` as well
as providing `order(scope)` lookup, so invalid or duplicate scope declarations
can also be diagnosed.

The reader decodes JSON into typed columns once. Its semantic views no longer
cast values out of a JSON tree on each access. The builder may retain its
construction-oriented entity storage and expose it through a prepared view
with row indexes. It need not duplicate all vertex data into reader storage.
Both implementations expose the same logical column organization.

Resource construction lives in `ifcdr/write`, alongside the read backing in
`ifcdr/read`. `PreparedIfcdrResource` owns an `IfcdrWriteInput` containing
resource metadata, resolved binding identities and owned entity data. It has
no dependency on package builder state or IFCX path generation. The adapter
in `package/write/prepare.rs` resolves package-specific keys and transfers
entity data, including vertex vectors, into that input without cloning geometry.
The package builder retains only the package information needed for IFCX
assembly. Resource preparation derives per-kind indexes, per-scope order and
bounds; the codec remains responsible only for physical representation.

The names `ValidatedIfcdr<R>` and `ValidatedIfcdrResource` are retained. Their
definitions document the distinction between the shared logical proof and
the reader's source-aware result containing such a proof.

Polyline pool offsets and order-entry offsets are JSON mapping details. Their
logical meaning is the selected sequence; repacking pools without changing
those sequences does not change entity meaning. Access must not require a
materialized point vector for every polyline.

### Preparation, validation, and proof

Preparation freezes construction state, establishes references and defaults,
collects per-kind rows and scope order, and determines bounds and the next ID.
It runs before the encoder. Decoding preserves supplied identity, order, and
bounds; it does not repair invalid input by recomputing them.

The access trait alone proves neither reference integrity nor geometry
validity. Structurally readable candidate data can contain duplicate IDs,
missing references, non-finite coordinates, or invalid bounds. Candidate access
must be safe before validation: use representable scalar values and checked
lookups, not assumptions that only hold after successful validation. Shared
domain conversion/check helpers own constraints such as a nonzero entity ID
and the allowed appearance modes; JSON decoding must not independently define
them. Validated public access can then expose constrained domain types.

Adapt the existing internal `Validated<T>` proof mechanism. Successful shared
validation produces immutable resource data plus indexes/evidence. An encoder
accepts this proof, not an arbitrary implementation of the access trait. No
mutable access may invalidate the evidence after construction.

Resource validation checks local references and typed values. It does not
prove that an IFCX identity resolves to the correct package node. Package
validation resolves those identities, checks layout/scope relationships and
appearance sources, and remains necessary for a strict package result.

### Builder completion and errors

Builder state may be incomplete before completion. This does not require new
cyclic-reference or reservation features for lines and polylines. Retain useful
immediate errors, such as a foreign builder key or a malformed point.

Completion consumes the builder and returns a completed package or a collection
of diagnostics. It need not return a recoverable builder. Collect independent
final errors in deterministic order, and suppress checks whose prerequisites
failed. For example, a missing appearance target must not also become a claim
that the target has an invalid color.

Logical diagnostics identify resource, entity/table row, and property where
available. JSON adapters may add source pointers; builders may add construction
context. Both paths use the same rules and codes, without requiring identical
physical locations or message formatting. Codec syntax, directory, and packing
errors remain separately testable.

## Logical semantics

### Identity, scope, and draw order

Carry forward resource identity, resource-wide nonzero entity IDs, local table
IDs, references, and one canonical entity sequence per scope. Each entity
occurs exactly once in its own scope's order. Type-grouped row order and entity
ID magnitude do not determine draw order. Do not add an `entityOrderId` field.

Encoding the same logical resource preserves entity IDs, references, scope
membership, scope metadata, and order. An encoder never assigns or renumbers
IDs. Gaps are allowed. `nextEntityId` is positive and greater than every present
entity ID; preserve a larger supplied value. Reject arithmetic exhaustion
rather than wrapping. An empty newly built resource starts at 1.

Keep automatic ID assignment during construction. Support caller-supplied
entity IDs for workflows that already have a source-to-entity mapping; that
mapping belongs to the caller/converter, not to the encoder. Assigned IDs must
be unique, and automatic allocation must advance past supplied IDs. This
limited input facility does not introduce editing, ID recycling, or a source
identity database. Resource ID plus entity ID identifies an entity in a package.

Do not infer historical ID reuse from a single file: `nextEntityId` only proves
the current snapshot invariant. Deletion/history policies need their own future
API and evidence.

Preserve existing scope `kind`, name, base coordinates, and flags. This slice
does not reinterpret the numeric kind/flag domains or infer placements from
them. Scope base coordinates remain metadata; encoding does not add them to
entity coordinates. Expanded coordinate-frame semantics belong to milestone 3.

### Geometry

All line endpoints and polyline vertices use finite XY coordinates in the
resource unit. The existing unit vocabulary remains unchanged.

- A line retains its start and end, even when they coincide.
- Every open or closed polyline has at least two vertices.
- Repeated vertices and zero-length segments are allowed. Do not remove or
  merge them or change the entity type.
- `closed` means an implicit final-to-first segment. It does not append a
  stored vertex or require a repeated first vertex.
- Preserve supplied vertex order, duplicate final vertices, and the explicit
  closed flag. Open `[A, B, A]` stays open `[A, B, A]`.
- A closed two-vertex polyline is allowed; no nonzero-length or area requirement
  is introduced.

The minimum of two vertices becomes a shared format rule. It was previously
enforced by the builder but not by the reader. Autodesk's
[AcDbPolyline reference](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-AcDbPolyline.html)
states that zero/one-vertex polylines should not remain in the database. This
supports the chosen persisted profile; permissive parsing or writing by a CAD
library does not establish format validity. This design does not claim that
every encountered DWG/DXF file conforms to that rule.

### Bounds

Bounds describe stored geometry, including invisible entities. They exclude
lineweight and other display effects. A renderer must account for those effects
separately when choosing painted extents or culling margins.

A nonempty resource has finite bounds with `min <= max` on both axes, enclosing
every line endpoint and polyline vertex. Conservative larger bounds are valid;
exact minimal bounds are not required. Coincident geometry permits point-sized
bounds. An empty resource has no bounds.

For this XY profile, bounds enclose the stored coordinate values across all
scopes without adding scope base coordinates or IFCX placements. They are not
a promise of a combined display viewport for independently arranged layouts.
Do not introduce a coordinate transformation while checking bounds.

The builder computes tight bounds during preparation unless a valid enclosing
bound is supplied through a supported input path. The decoder preserves and
checks supplied bounds; the encoder preserves validated bounds. Validation
requires one traversal of endpoints and vertices, with linear cost in their
number. No incremental-performance guarantee is made.

### Defaults and appearance

Omitted visibility has logical value `true`, equal to explicit `true`. Missing
optional tables represent empty collections. The JSON mapping may omit these
defaults deterministically; access to the logical model always exposes their
meaning.

Preserve independent appearance modes for color, opacity, line pattern, and
lineweight: `ByLayer`, `Explicit`, and `ByBlock`. The JSON codes remain 0, 1,
and 2 respectively. An explicit value equal to the current layer value is
semantically different from `ByLayer`. Retain the layer link and inheritance
mode instead of materializing inherited values. Keeping `ByBlock` does not
add block entities.

Replace IFCDR `jsonValue` color with a typed color containing required RGB
channels (integers 0 through 255), optional indexed metadata (nonempty system,
unsigned 64-bit index), and optional named metadata (nonempty catalog and name).
Both metadata forms may coexist and must survive encoding. This is not an enum
that selects one representation and discards the others.

Opacity is finite in [0, 1]; lineweight is finite and nonnegative. Preserve the
existing line-pattern distinction between a name in IFCX appearance data and
an IFCX identity in an IFCDR override. Do not invent a new line-pattern payload
or change unit conventions in this step.

For an explicit property, preserve current precedence: a non-null override
supplies the value before the referenced IFCX appearance. An invalid override
does not silently fall back. The package validator checks that explicit
properties have usable sources and that external links resolve correctly.

Concrete proposal for the typed boundary: validate every non-null stored
override value, including values not selected by the current appearance mode.
Preserve valid unused values during re-encoding. This deliberately tightens
cases the existing validator may ignore, and requires negative conformance
cases; it is not merely a code refactor.

The new IFCDR color object has closed known fields, including its metadata
objects. Unknown fields receive a diagnostic rather than being silently lost.
This does not close IFCX extension points. Typed IFCX appearance access is a
projection of understood properties; the original IFCX graph remains available
with permitted extension fields. Re-encoding an IFCDR resource must not rebuild
that graph from only its typed color projection. This slice does not claim a
new lossless arbitrary-package editing API.

## Published contract and JSON mapping

The implementation must publish logical rules under `schemas/ifcdr/` and copy
the applicable assets into `conformance/next`. This design document explains
the decision; it is not a replacement for those normative assets.

Split the current registry into:

1. A versioned logical registry with named types, supported enum values,
   columns/tables, logical defaults, nullability, references, and constraints.
   References use logical names such as `layerBinding.id`, not JSON paths.
2. A versioned JSON mapping describing header constants, payload keys, stream
   directory metadata, omission rules, numeric representations, and the pool
   offsets/counts used for sequences.
3. A versioned semantic contract document defining relational and geometric
   rules and their meaning, linked to named invariants and conformance cases.

Publish `registry-0.7.0.json` against a new registry meta-schema v2,
`json-mapping-0.7.0.json` against a dedicated JSON-mapping meta-schema v1, and
`logical-contract-0.7.0.md`. Validate mapping completeness against the registry
so fields, defaults, and constraints cannot drift into two competing sources.
Implement supported checks directly where appropriate; no general-purpose rule
interpreter or code generator is required.

The reusable mapping forms are normatively defined in `json-mapping-v1.md`,
referenced by both the mapping and its meta-schema and copied into the
candidate collection. This document specifies zero-based half-open ranges,
XY pool component order, allowed pool sharing and unused points, and complete
contiguous child-range coverage. The meta-schema also enforces two XY pools.

The logical registry describes line columns and each polyline's ordered vertex
sequence. The mapping describes `lineStream`, `polylineStream`, `scopeTable`,
`vertexOffset`, `vertexCount`, and order-entry packing. Directory counts,
equal physical column lengths, checked offset arithmetic, and pool range
validity are codec checks. Referential integrity, minimum vertices, appearance
domains, order coverage, and bounds are shared logical checks.

JSON numbers representing IDs must be integral and preserve the full supported
integer range without routing through floating point. Codec-specific size or
packing limits are explicit encoding errors; they must not silently truncate
otherwise valid logical content.

### Version proposal

Use a new IFCDR 0.7.0 resource contract and IFCX composite overlay 0.7.0. This
records the shared polyline minimum, optional bounds, typed override values,
and registry/mapping split rather than retroactively changing 0.6.0.

| Artifact | Proposed version/behavior |
| --- | --- |
| IFCDR resource | `ifccad.ifcdr.resource.v0.7.0` |
| Logical registry meta-schema | `ifccad.ifcdr.registry.v2` |
| JSON mapping meta-schema | `ifccad.ifcdr.jsonMapping.v1` |
| Polyline definition | `ifccad.ifcdr.polyline.v3` |
| Appearance binding definition | `ifccad.ifcdr.appearanceBinding.v2`, with explicit mode domains |
| Appearance override definition | `ifccad.ifcdr.appearanceOverride.v2`, with typed values |
| Line, scope, layer binding, order definitions | Retain current IDs where their semantics are unchanged |
| JSON stream directory | Retain v1 structure; bind entries to the active definitions |
| IFCX drawing core / IFCPR | Retain 0.2.0 and existing validation scope |
| Candidate conformance collection | Continue unpublished 1.1.0 under `conformance/next` |

In JSON, `bounds` remains a required property: `null` means no bounds, and an
object supplies the four coordinates. Empty means zero drawable entities,
regardless of whether supporting tables are empty.

Keep historical active schema files and all frozen numbered collections intact.
Renew current-reader fixtures and candidate assets. Unsupported older versions,
including 0.6.0, produce an explicit unsupported result and no strict view;
there is no migration layer. Keep the existing distinction between unsupported
content and known invalidity, including suppression of dependent diagnostics.

## Encoding and package integration

Move JSON-specific parsing, physical validation, registry-mapping loading, and
serialization behind a codec module. Shared resource access and semantic
validation must not import `serde_json::Value` or JSON source-location types.
Diagnostic adapters may translate logical locations into JSON pointers.

The JSON encoder consumes every supported field through shared validated
access, including overrides, multiple scopes, supplied IDs, next ID, bounds,
visibility, and scope order. Codec helpers may pack arrays and compute physical
checksums. They must not resolve inheritance, normalize geometry, allocate
identities, or decide draw order.

The package builder uses preparation and shared validation before encoding.
Package binding checks use the assembled IFCX definitions through the existing
package validation boundary. Physical resource descriptors and checksums are
assembled from the encoded artifact. Directory writing remains a separate
operation with its existing overwrite policy.

Keep converter entry points and supported CAD coverage aligned with the
updated facade. Geometry/bounds edge cases follow this contract; no new source
entity coverage or preservation transfer is implied. Every produced package
must pass the production reader's strict package validation in integration
tests.

Re-encoding preserves logical values, not source whitespace, optional-default
syntax, pool packing, or file bytes. Repeated encoding of the same prepared
resource must be deterministic. Physical checksums are recomputed after
serialization. Existing canonical-value and fingerprint algorithms remain
unchanged; this slice does not introduce a whole-resource fingerprint algorithm
or infer semantic equality from identical checksums.

## Verification and acceptance

Exercise the same resource contract suite against the decoded read backing and
the prepared write backing. Check equivalent semantic access and validation,
not their private memory layouts. Include direct decoded-resource re-encoding,
so the proof is not limited to the narrower package builder's output.

Required cases include:

- Empty, line-only, polyline-only, and mixed resources; retained multi-scope
  metadata and per-scope mixed entity order.
- Nonconsecutive IDs, a next ID greater than max+1, supplied-ID allocation,
  duplicate/zero IDs, range exhaustion, and missing references.
- Omitted versus explicit default visibility and independent ByLayer, ByBlock,
  and Explicit modes.
- Color with RGB, indexed and named metadata together; explicit overrides,
  valid unused overrides, invalid unused values, and allowed IFCX extensions.
- Zero-length lines, duplicate polyline vertices, open `[A, B, A]`, implicit
  closing segments, and rejection of zero/one-vertex polylines on both paths.
- JSON `null` bounds on empty resources; point-sized and conservative bounds
  on nonempty resources; rejection of a missing JSON `bounds` property,
  bounds on an empty resource, absent bounds on a nonempty resource, and
  inverted, non-finite, or insufficient bounds, including invisible geometry
  outside them.
- Malformed JSON packing and unsupported versions/streams with no partial
  proof or misleading dependent errors; independent diagnostics aggregated.
- Semantic decoded-resource roundtrips, deterministic repeated JSON output,
  production-reader validation of builder/converter output, and retained
  IFCPR checks with unchanged documented limitations.
- Logical registry/mapping consistency, active/candidate equality, and
  unchanged frozen collections.

Completion requires JSON-free semantic resource APIs and validation, distinct
codec checks, immutable validation evidence, and published contract material
covering the agreed rules. Update affected API docs, converter documentation,
coverage notes where behavior changes, and the candidate compatibility matrix
to the actually verified state. Keep milestone 2 marked Current.

Before reporting implementation complete, run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Use `cargo test --doc --workspace` during public documentation/API work. This
specification alone changes no implementation and does not claim these checks
have run.

## Issue alignment

Relevant open issues were reviewed during design:

- [#7](https://github.com/OpenAEC-Foundation/ifccad/issues/7): this slice implements
  the shared logical/validation and codec boundaries. The explicit decision to
  renew fixtures supersedes its earlier old-JSON compatibility suggestion.
- [#5](https://github.com/OpenAEC-Foundation/ifccad/issues/5): express supported
  appearance modes and units in language-neutral contract data. Do not define
  speculative enum vocabularies for future entity families.
- [#1](https://github.com/OpenAEC-Foundation/ifccad/issues/1) and
  [#8](https://github.com/OpenAEC-Foundation/ifccad/issues/8): inline/external
  normalization and vocabulary alignment follow this resource boundary; they
  remain required milestone 2 work.
- [#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2) and
  [#4](https://github.com/OpenAEC-Foundation/ifccad/issues/4): future irregular
  payloads and spatial semantics are not designed or constrained by new
  implementations in this slice.
- [#3](https://github.com/OpenAEC-Foundation/ifccad/issues/3) and
  [#6](https://github.com/OpenAEC-Foundation/ifccad/issues/6): the boundary permits
  later physical choices but provides no chunking implementation, benchmark
  result, or performance claim.

These issues are design context. The roadmap remains authoritative for
sequencing; schemas and conformance assets remain authoritative for the
published format. Issue edits, commits, and implementation are separate from
this specification-writing task.
