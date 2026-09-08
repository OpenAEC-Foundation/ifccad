# IFCDR base contract before codec separation

Date: 2026-09-08

Status: Approved and implemented on 2026-09-08. The reduced baseline is verified;
encoding-neutral model design and the remaining milestone 2 work are still open.

## Purpose

Reduce the active IFCDR contract and implementation to the drawing content
already exposed through dedicated typed APIs: lines and straight polylines,
with the resource structure and appearance information they require.

The prototype registry currently describes many more entity families than the
Rust implementation models. Carrying those definitions into an encoding-neutral
resource model would require designing representations for semantics that may
be reconsidered in milestone 3. This step removes that obligation before the
reader and writer are separated from their JSON encoding.

The result is a working, smaller JSON reference implementation with matching
schemas, reader, writer, converter integration, and conformance fixtures. This
step does not by itself establish a stable encoding-neutral contract or complete
milestone 2.

## Agreed decisions

- Reduce the active registry to lines, polylines, and their required supporting
  concepts. Remove other inherited entity definitions from the new contract.
- Remove corresponding prototype-specific implementation assumptions rather
  than merely making them unreachable through the registry.
- Do this reduction before introducing the shared logical resource and codec
  boundaries. Design the sequence together, but verify each change separately.
- Do not require the new reader to accept old IFCDR files. There is no legacy
  reader, migration layer, or old-version writer in this scope.
- Update active test files while retaining the meaning of relevant test cases.
- Keep numbered conformance collections immutable.
- Retain IFCPR 0.2.0 and its existing limited implementation and tests. Full
  preservation design and converter integration remain milestone 3 work.
- Future entity families are introduced through deliberate semantic design,
  implementation, and conformance work. Prototype definitions are reference
  material, not constraints on the new model.

## Scope and sequencing

This is the first implementation slice of milestone 2:

1. Establish this reduced contract and its explicit support boundary.
2. Restore a fully verified JSON reader/writer/converter baseline for it.
3. In a subsequent design, introduce a shared logical IFCDR representation,
   semantic preparation and validation, and explicit JSON codec modules.

The later design must use the smaller baseline rather than preserve discarded
prototype entity structures. It must also complete the language-neutral
semantic rules, logical registry/JSON-mapping separation, compatibility
reporting, and other milestone 2 requirements.

The following are outside this slice:

- changing line/polyline geometry semantics or adding 3D geometry;
- designing text, dimensions, blocks, hatches, viewports, or typed payloads;
- adding paperspace export or widening the existing one-model-layout writer;
- implementing inline resources;
- renaming the IFCX drawing-representation vocabulary under issue #8;
- completing the registry enum system under issue #5;
- implementing a new logical value system or removing every JSON dependency;
- implementing preservation transfer or complete IFCPR semantic validation;
- choosing binary encodings, chunking, compression, or a container;
- implementing the size-measurement baseline.

Scopes remain a resource concept. Keeping the writer limited to one model
layout must not remove existing reader handling of multiple scopes or the
existing IFCX model/paper layout distinction. Dedicated broader layout proofs
and drawing-resource terminology are subsequent milestone 2 work; paperspace
conversion remains milestone 3 work.

## Contract versions

Use the following version boundary for this slice:

| Artifact | Version after this change | Decision |
| --- | --- | --- |
| Active IFCDR registry | `0.6.0` | New reduced resource contract. |
| IFCDR resource schema ID | `ifccad.ifcdr.resource.v0.6.0` | Identifies the changed resource vocabulary. |
| IFCX composite overlay | `0.6.0` | References IFCDR `0.6.0`; otherwise retains the current vocabulary and package rules. |
| IFCX drawing core | `0.2.0` | Unchanged. |
| IFCDR registry meta-schema | `ifccad.ifcdr.registry.v1` | Unchanged; redesign belongs to the subsequent semantic/codec work. |
| IFCDR stream directory | `ifccad.ifcdr.streamDirectory.v1` | Unchanged. |
| IFCPR | `0.2.0` | Unchanged. |
| Conformance candidate | `1.1.0` in `conformance/next` | Remains an unpublished candidate; update its provenance and contract inventory. |

Create `schemas/ifcdr/registry-0.6.0.json` and
`schemas/ifcx/ifccad-overlay-0.6.0.json`, and copy their exact contents into the
corresponding schema directories in `conformance/next`.

Retained stream and table definitions keep their schema IDs because this slice
does not change their field semantics. Removing other definitions changes the
resource contract, not the meaning of `ifccad.ifcdr.line.v2`, for example.

Keep earlier versioned files under `schemas/` as historical artifacts. Remove
superseded IFCX overlays and IFCDR registry copies from the candidate collection
so its schema inventory identifies the current candidate contract. Keep the
shared meta-schema, drawing core, and IFCPR schema there. Do not change any file
under `conformance/1.0.0`, including its registry, fixtures, vectors, and license.

The new overlay continues to use `DrawingGeometryRepresentation`,
`attributes.geometry`, and the existing resource descriptor fields. Naming
changes are explicitly deferred rather than mixed into this reduction.

## Exact registry inventory

The new registry contains exactly four streams:

| Stream | Role | Reason |
| --- | --- | --- |
| `line` | Object | Typed finite XY line content. |
| `polyline` | Object | Typed straight XY polyline content and vertex pools. |
| `entityOrder` | Order | The canonical sequence for each scope. |
| `entityOrderEntry` | Child | Entity references used by that sequence. |

It contains exactly four tables:

| Table | JSON payload | Reason |
| --- | --- | --- |
| `scope` | `scopeTable` | Scope identity and existing scope metadata. |
| `layerBinding` | `layerBindings` | Links entity layer IDs to IFCX definitions. |
| `appearanceBinding` | `appearanceBindings` | Per-property appearance modes and bindings. |
| `appearanceOverride` | `appearanceOverrides` | Already supported explicit overrides used by package navigation and validation. |

Retain the existing header, resource ID, unit, next-entity-ID, bounds, stream
directory, presence rules, defaults, and field definitions needed by this
inventory. Retaining appearances includes `ByLayer`, `Explicit`, and `ByBlock`;
the existence of `ByBlock` does not imply support for block-reference entities.

Remove every other stream, including circle, arc, point, ellipse, dimension
variants, text and MText structures, block references, hatches and their paths
and edges, viewports, and viewport support streams.

Remove the following tables and references that exclusively serve them:

- `textStyleBinding`;
- `dimensionStyleBinding`;
- `hatchPatternBinding`;
- `namedUcsBinding`;
- `dimensionOverride`;
- `characterFormat`;
- `paragraphFormat`;
- `textRun`.

The retained registry must have no dangling references to removed definitions.
Keep the parent/child and range mechanisms needed by entity order and polyline
vertex pools. Do not remove general validation machinery solely because hatch
also used it.

`appearanceOverride.color` currently uses `jsonValue` and is interpreted by
existing appearance validation. Keep that behavior in this slice. Replacing it
with a bounded semantic color type belongs to the subsequent logical model
design. Its temporary retention does not satisfy the encoding-neutral exit
criterion of milestone 2.

## Reader behavior and cleanup

### Supported resource

The reader uses only the new active registry. Successful resource validation
exposes lines and polylines through the existing typed views and iterates them
in scope draw order. Existing ID, reference, scope, unit, appearance, and
ordering checks remain in force.

This slice preserves the current validation guarantees for retained concepts.
It does not silently promote a writer-only restriction into a format rule.
For example, the builder's minimum polyline vertex count must not be assumed
to be a reader rule without explicit contract work. Such reader/writer gaps
are recorded for the subsequent semantic design, not resolved accidentally
while deleting unrelated entities.

### Prototype-specific code

Remove the `text.ownerKind` exception in entity-order validation. All object
streams in the reduced registry contain directly ordered drawable entities.
The retained `entityOrderEntry` child stream is not an object entity stream.

Remove `IfcdrEntityRef::Unmodeled`, `UnmodeledEntityRef`, and their public
re-exports. Remove the corresponding IFCCAD-to-CadDocument skip branch,
diagnostic aggregation, and entity-mapping branches. This is a deliberate
public API change during the current development phase.

Remove `UnmodeledStreamRef` and its enumeration if they have no remaining
production use. Tests of retained order streams should use registry/order
behavior directly; order streams must not be described as unmodeled entities.

Keep ordinary unsupported-source diagnostics in the CadDocument-to-IFCCAD
exporter. An unsupported CAD circle still requires a loss diagnostic even
though the new IFCDR registry has no circle entry. Do not conflate those source
conversion diagnostics with the removed IFCDR `Unmodeled` path.

### Unsupported versions and content

An old or otherwise unsupported IFCDR version produces
`IFCCAD_IFCDR_VERSION_UNSUPPORTED` and no validated resource or strict package
view. Include the actual and supported version in diagnostic context. Do not
attempt old-version decoding with the new registry or infer support from the
presence of line/polyline columns.

For a structurally recognizable directory entry whose stream name or schema ID
is not supported, emit `IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED`. Include the
available stream name and schema ID in diagnostic context. Do not decode that
stream using a prototype definition and do not create an `Unmodeled` entity.

Unsupported content blocks the strict typed package view. The load outcome
still exposes diagnostics through the current inspection API. This slice does
not add partial drawing views, lossless copying of unknown content, or a new
editing/preservation API.

The diagnostic means the implementation cannot support the content. It is not
by itself proof of invalidity under some other contract. If a file claims the
closed base contract while adding forbidden fields, those fields can also
constitute a known contract violation. The documentation must distinguish
unsupported interpretation from an independently established structural error.

After an unsupported version or stream prevents complete resource analysis,
skip semantic checks whose premises require that missing analysis. Do not
report cascaded missing entity-order targets, missing layout scopes, or IFCPR
links as though their absence had been established. Independent physical and
package errors may still be reported.

In particular, avoid the current misleading cascade in which an unknown
directory stream also yields "payload has no directory entry" merely because
it was not registered. Real orphan payloads in otherwise fully understood
resources remain errors.

Removed support tables are not silently accepted, even when empty. The new
contract's closed-field rules apply; only renewed fixtures and new writer
output are required to load successfully.

The existing error-severity diagnostics can continue to block
`validated_package()`. This step does not redesign the public report API into
a complete validity/support/transfer result system. Document that the existing
success flag is an implementation validation result, not a universal verdict
on unsupported contracts. Completing operation-specific reporting remains
part of milestone 2.

## Writer and converter alignment

The package writer emits IFCDR `0.6.0` in both the resource header and the IFCX
resource descriptor, and its output satisfies the new composite overlay.

Remove the currently emitted empty `namedUcsBindings` and
`dimensionOverrideTable`. Do not emit any other discarded support tables or
streams. Existing line/polyline payload organization, deterministic entity
assignment, draw order, bounds preparation, appearance defaults, and JSON
serialization stay unchanged except where the contract/version reduction
requires a difference.

The writer remains limited to one drawing with one model layout and one
external resource. Directory writing remains separate from in-memory package
construction and does not overwrite an existing directory.

Both conversion directions continue to use the current exact native subset
and fidelity policies. Keep the pinned source coverage inventory and update
nearby documentation when removing IFCDR-specific import diagnostics. This
change does not add CAD export coverage or preservation support.

Every new writer/converter package must reload through the production reader
without diagnostics and expose the expected supported semantic content.
Determinism is checked between repeated runs of the new writer, not against
bytes produced under the old contract.

## IFCPR boundary

Keep `schemas/ifcpr/schema-0.2.0.json` and its identical candidate copy. IFCPR
describes source records, payloads, dependencies, and projections generically;
it does not require the discarded hatch/text entity registry definitions.

Retain existing IFCPR JSON-schema checks, descriptor/header resource identity
checks, resource-file checksum checks, and links to successfully validated
IFCDR resource IDs. Preserve their focused tests with updated IFCDR companions
where necessary.

Do not claim that these checks validate the complete preservation graph or
guarantee a lossless source roundtrip. The currently deferred checks for blob
digests, payload ranges, record references, dependency cycles, and projection
resource interpretation stay explicitly documented and tested as deferrals.
The converter still does not transfer IFCPR preservation data.

The base-profile positive reader/writer/conversion fixtures need no IFCPR.
Existing preservation-focused fixtures remain separate coverage in the same
candidate collection. Their supported IFCDR resources are migrated to the new
contract; the IFCPR schema itself is unchanged.

Keeping IFCPR now is not a commitment to its final architecture. Milestone 3
may introduce a new schema or replace its place in the active candidate after
concrete preservation design. Frozen collections remain unchanged.

## Conformance and compatibility

Add `conformance/next/COMPATIBILITY.md` describing the implemented state after
this slice. It must distinguish the new base profile, unsupported IFCDR
versions/content, the writer/converter subset, open IFCX content, and limited
IFCPR validation. A document about intended future capability must not be
presented as current support before implementation is verified.

The initial matrix must cover at least:

| Input or operation | Required result after this slice |
| --- | --- |
| New base-profile JSON resource in a valid package | Typed read and existing supported validation. |
| New writer output | Deterministic, production-reader validated package. |
| Current supported CadDocument subset | Existing conversion behavior and semantic roundtrip. |
| Unsupported source CAD entities/properties | Existing explicit export loss policy. |
| IFCDR `0.5.0` or another unsupported version | Unsupported; no strict view or migration. |
| Unsupported IFCDR stream/schema | Unsupported; no strict view or silent removal. |
| Malformed retained fields or broken known references | Invalid under known rules; no strict view. |
| Open IFCX extension fields and unknown node types | Preserve existing permitted read behavior; no new conversion or editing guarantees. |
| IFCPR `0.2.0` | Existing limited checks only; no complete preservation or transfer guarantee. |

Renew applicable fixtures under `conformance/next`, including embedded package
manifests and test-created resources. Update resource versions and schema
references, remove discarded tables, and recompute resource checksums. Update
fingerprints only if the normative logical projection changed; a physical
checksum change alone is not a reason to change a semantic fingerprint vector.

Preserve the intent of negative cases. For example, an identity-mismatch
fixture must still reach the identity check rather than fail first on an old
version or a discarded table. Retain the explicit IFCPR deferral list rather
than deleting failing preservation cases to make the smaller suite pass.

Tests that intentionally inspect frozen contracts may continue using them.
Tests of the current production reader must use current fixtures except when
testing rejection of old versions. Update candidate asset inventory and
active/candidate equality tests without weakening frozen-byte checks.

## Verification requirements

Use focused tests while implementing. Required scenarios include:

- exact retained registry inventory and valid registry cross-references;
- active/candidate schema byte equality and unchanged frozen assets;
- empty drawing, line-only, polyline-only, and mixed drawing packages;
- open and closed straight polylines and vertex order;
- mixed line/polyline draw order, units, layers, visibility, appearances, and
  existing supported overrides;
- omitted `visible` versus explicit `true` with equal logical results;
- duplicate/zero entity IDs, missing known references, invalid ranges and
  column lengths, and invalid or incomplete draw order;
- unsupported IFCDR version and a small synthetic unsupported hatch directory
  entry, with no dependency on a complete prototype hatch payload;
- mixed supported/unsupported content giving no strict package view and no
  misleading dependent diagnostics;
- removed empty support tables rejected under the new contract;
- permitted IFCX extensions retaining their existing read behavior;
- retained IFCPR validation cases and explicit semantic-validation deferrals;
- production-reader roundtrips and deterministic new JSON output;
- existing exact CAD conversion chains and unsupported-source loss tests.

Entity assertions compare semantic content rather than unstable CAD handles or
serialized layout, except for byte determinism itself. No representative hatch,
text, or other deferred-family positive corpus is required for this slice.

Before reporting implementation complete, run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Use `cargo test --doc --workspace` as an intermediate check when changing public
Rust documentation or examples. The current documentation-only specification
change does not assert that these implementation checks have run.

## Documentation and issue alignment

Update `ROADMAP.md` and its `README.md` summary to state that milestone 2 begins
with a deliberately reduced profile and does not promise old-file support.
Keep milestone names, order, and status unchanged. After implementation,
describe the actual active contract and reader/converter boundaries in nearby
API documentation, conversion documentation, and candidate provenance.

Relevant open issues reviewed during design:

- [#7](https://github.com/OpenAEC-Foundation/ifccad/issues/7): this reduction is
  preparation for codec separation, not completion of that issue. Its proposed
  old-JSON compatibility requirement is superseded by the explicit user decision
  for this milestone.
- [#8](https://github.com/OpenAEC-Foundation/ifccad/issues/8): drawing-resource
  terminology remains subsequent work; reader support for old vocabulary is not
  a requirement of the chosen compatibility policy.
- [#1](https://github.com/OpenAEC-Foundation/ifccad/issues/1): resource IDs already
  exist; inline/external normalization remains later milestone 2 work.
- [#5](https://github.com/OpenAEC-Foundation/ifccad/issues/5): language-neutral enum
  constraints remain required later in milestone 2.
- [#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2) and
  [#4](https://github.com/OpenAEC-Foundation/ifccad/issues/4): future payload and
  geometry decisions should not be constrained by removed prototype definitions.
- [#3](https://github.com/OpenAEC-Foundation/ifccad/issues/3) and
  [#6](https://github.com/OpenAEC-Foundation/ifccad/issues/6): this slice chooses no
  physical optimization and does not replace the later measurement work.

Open issue descriptions are design context, not normative requirements. This
specification and roadmap record the agreed compatibility departure; editing
or closing GitHub issues is not part of this task.

## Acceptance criteria

This slice is complete only when:

1. The new versioned registry contains precisely the four streams and four
   tables listed above, and active/candidate contract assets agree.
2. The reader and writer use the new contract, with no old-version support
   requirement or runtime dependency on the prototype registry.
3. The public entity model contains only typed lines and polylines, and the
   prototype text-order exception and obsolete unmodeled-entity paths are gone.
4. Unsupported input produces explicit support diagnostics without a strict
   typed result or misleading dependent errors; known malformed input still
   fails the relevant validation checks.
5. Writer and converter output reloads and preserves the current supported
   semantics, and repeated new-version output is deterministic.
6. IFCPR schemas and existing limited checks remain, with their limitations
   and deferred checks clearly reported.
7. Active fixtures cover the new baseline, relevant negative scenarios remain
   meaningful, and frozen collections are unchanged.
8. Compatibility documentation and required verification reflect the actual
   implementation. No claim is made that milestone 2 as a whole is complete.

The [implementation plan](../plans/2026-09-08-ifcdr-base-contract.md) details
execution of this approved slice. Design the encoding-neutral model against
the resulting reduced baseline as the next architectural step.
