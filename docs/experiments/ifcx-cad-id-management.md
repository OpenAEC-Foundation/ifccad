# Persistent ID allocation for IFCX-CAD

Date: 2026-10-02
Status: implemented and verified, 2026-10-02.

## Intent and decision

Support a native `read -> edit typed document -> write -> read` lifecycle
without changing existing node paths or reusing previously allocated IDs.
Keep the experiment's compact drawing-local uint64 paths and persist one
allocation watermark per ID domain. Do not add an entity ID attribute: the
complete IFCX path remains the node identity.

This is a CAD-profile convention, not a general IFCX allocator or global
identity protocol. It follows OCDraw's persistent-watermark principle while
keeping IFCX-CAD's own model, schema and path domains separate.

OCDraw uses uint64 entity IDs and uint32 layer/layout/line-pattern IDs; its
block definitions are identified through uint32 scopes. This design retains
IFCX-CAD's existing uint64 domains rather than adopting those OCDraw widths.
The user confirmed this distinction on 2026-10-02.

## Current behavior

`IfcxCadDocument` already carries explicit numeric IDs, and native writing
preserves them in paths. It does not carry allocation watermarks. The direct
CAD importer assigns fresh IDs from source enumeration on each conversion.
Deterministic enumeration does not establish identity across DWG/DXF saves.

The current path contract uses `/cad/dN`, `/cad/dN/eN`, and drawing-local
`layer/N`, `layout/N`, `block/N` and `linePattern/N` paths. Model and Paper
layouts share a domain. Entities across every layout and block definition
share one entity domain. Child keys encode order, independently of identity.

## Stored allocation state

The existing `ifccad::drawing` object has five required fields:

| Field | Objects covered |
| --- | --- |
| `nextEntityId` | All drawable entities, including block instances and block contents |
| `nextLayerId` | Drawing layers |
| `nextLayoutId` | Model and Paper layouts together |
| `nextBlockId` | Block definitions |
| `nextLinePatternId` | Named line-pattern definitions |

Each field is an IFCX `Integer` with profile validation requiring an exact
unsigned 64-bit value. Each watermark must exceed every currently present ID
in its domain. New empty domains start at 1; existing canonical ID 0 remains
permitted by the current path contract. There is no `nextDrawingId`: the
reader handles one drawing and its drawing ID remains caller supplied.

Keep the fields together on `ifccad::drawing`; do not introduce a separate
allocation node or a second entity identity. For example, existing entities
`e1` and `e3`, with a deleted `e4`, may have `nextEntityId = 5`. Reading and
rewriting must preserve 5 even though the current maximum is only 3.

JSON must retain exact integer values, including above JavaScript's safe
integer range. Core allocation and persistence use u64, not floating point.
Browser inspection must not turn rounded JavaScript numbers back into saved
allocation state; test the production Rust/WASM read/write route with such
values. This design adds no binary encoding or new IFCX datatype.

## Identity and allocation rules

1. Preserve the drawing prefix and every existing object's numeric ID during
   native edits and rewrites. Renaming a file or a named definition, editing
   geometry/appearance, reordering, or moving an entity between owners in the
   same drawing does not change its path.
2. A newly created object, including a copy, receives its domain's next ID.
   Increment the watermark with checked arithmetic before reporting a
   successful allocation. Failure must leave allocation state unchanged.
3. Deleted IDs are not reused. Deletion, undo of an allocation, sorting and
   rewriting never reduce watermarks. Reserved but unused IDs are acceptable.
   Undo may restore the same deleted object with its original identity.
4. Applications introducing explicit IDs must ensure they are fresh within
   the drawing's history and advance the relevant watermark if necessary.
   Snapshot validation can detect current duplicates, but cannot prove that
   an absent ID was never used before.
5. Exhaustion is an error, never wraparound or renumbering. A watermark of
   `u64::MAX` can be stored but cannot allocate another ID, since advancing
   it would overflow. A current object with ID `u64::MAX` cannot satisfy the
   new watermark rule and must be reported as unrepresentable on transition.
6. References continue to contain complete paths. Ordered child keys may
   change without changing their target identities. No new owner or draw-order
   attribute is introduced.

The writer copies supplied IDs and watermarks exactly. It does not compact
IDs, infer them from collection position or recompute watermarks as `max + 1`.

## Core and conversion boundary

`IfcxCadDocument.id_counters` holds `IfcxCadIdCounters` with the five u64
watermarks. Its checked `allocate_*_id` methods reserve IDs without inserting
objects; failures return `IfcxCadIdAllocationError` identifying the domain.
There is no general editing engine. Reader and writer use shared watermark
checks alongside existing profile validation. Successful writer output still
passes production strict readback and compares the complete typed meaning,
including allocation state.

For a new CAD import, the converter initializes and advances allocation state
while assigning new IDs. Source mappings continue to associate those IDs with
CAD handles in the conversion outcome. Exporting to DWG/DXF and importing the
result remains a new conversion; these watermarks do not preserve IDs across
that boundary. Retaining such identity requires a separate approved design.

OCDraw's model and conversion contract are unchanged. Reuse the allocation
principle rather than introducing a shared format-dependent allocator.

## Composition and validation

Apply the selected IFCX composition policy before checking the final drawing.
Under `LaterWins`, a later `ifccad::drawing` value replaces the entire earlier
attribute object, as it does today. A fragment updating allocation state must
therefore include the complete drawing attribute. The five fields are not
independently merged, and no special maximum-of-counters rule is introduced.
`RejectConflicts` retains its existing diagnostic meaning.

Reject missing fields, negative/fractional/out-of-range values and any watermark
not above the corresponding current maximum. Preserve valid sparse IDs and
watermarks above the minimum. A snapshot cannot detect a historically lowered
watermark that still exceeds the current maximum; monotonic history is an
editor obligation, not a guarantee made by the reader.

## Transition

The bundled provisional schema, profile contract, examples, constructors,
direct CAD importer and reader/writer tests were updated together. There is
no experimental version-number increase for this slice.
The updated strict profile requires all five fields; it does not silently
derive missing counters on every load.

Existing experimental fixtures can be explicitly initialized from their
known authored IDs. For an older external file with missing allocation
history, `max + 1` cannot recover deleted IDs. Any one-time migration must
declare that limitation and obtain known watermarks or explicitly establish
a new allocation baseline. A general migration tool is outside this slice.

This is an independent IFCX-CAD model-lifecycle follow-up to the integrated
experiment in ROADMAP. It does not reopen OCDraw's completed milestones or
depend on deferred paperspace/viewport conversion.

## Collaboration and upstream context

Paper viewports share the ordinary entity domain and allocation watermark.
The Model path, stored clip boundary and frozen layers use existing layout,
entity and layer domains; there is no viewport counter. Native uint64 IDs are
independent of CAD viewport runtime numbers. A copied viewport requires fresh
entity allocation; its boundary must also receive its own identity because
active and dormant boundary claims are exclusive. Moving the viewport alone
across Paper owners invalidates a still-referenced boundary in the old owner.
Native encode/readback preserves these IDs and sorts frozen-layer sets by exact
numeric IDs; CAD reimport still allocates a new native identity baseline.

This allocator assumes one writer or a centrally coordinated allocator per
drawing. Independently edited copies can allocate identical IDs and child
positions. `LaterWins` does not make that safe. Offline collaboration, UUID
paths, drawing namespaces, conflict detection, revision tracking, node
deletion protocols and concurrent draw-order editing are separate designs.

G1 in the [upstream gap record](ifcx-upstream-gaps.md) covers local-path
identity, uniqueness domains and cross-file resolution. [IFCX issue #63](https://github.com/buildingSMART/IFC5-development/issues/63)
documents disagreement between the path pattern and examples. The allocation
state is a profile choice, not a claim that those upstream questions are
settled. G3 and [issue #132](https://github.com/buildingSMART/IFC5-development/issues/132)
cover composition/deletion differences; this slice does not change them.

The repository's open issues #2, #3, #4, #6 and #10 were reviewed by title on
2026-10-02. They concern payload storage, logical/physical separation, splines,
benchmarks and numerical tolerance; none requests ID allocation or supersedes
this scope. No physical-encoding or benchmark change is proposed here.

## Acceptance checks for implementation

- Preserve sparse IDs and all five watermarks through strict native readback.
- Delete the highest allocated entity, save/reopen, then allocate: the deleted
  ID must not be reused. Repeat for every definition domain.
- Reorder and move an entity between Model, Paper and block ownership while
  retaining its path and references; copying receives a fresh ID.
- Preserve an advanced watermark when its domain is empty.
- Reject missing/invalid/stale counters after composition, including a later
  drawing fragment whose counter falls below a still-present object's ID.
- Preserve the existing whole-attribute `LaterWins` and `RejectConflicts`
  behavior; do not claim historical monotonicity from snapshot checks.
- Exercise IDs/counters above 2^53 and checked exhaustion without mutation.
- Confirm fresh CAD imports produce strict-readable allocation state, without
  claiming native identity preservation through CAD serialization.
- Run the repository Rust completion gates and focused browser/WASM checks
  if the exercised adapter path changes. Use semantic readback comparisons.

## Implementation evidence (2026-10-02)

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` and `cargo test --doc --workspace` pass. Cargo used
  the existing lockfile and cached dependencies offline. The workspace test
  run reports 331 passed including doctests and one existing ignored test.
- Eleven native ID-management tests cover stored counters, every ID domain,
  block-only entity maxima, sparse identities, deleting/reopening/allocating,
  reordering/moving/copying, empty domains, composition and full-width numbers.
  Three allocator tests cover checked advancement and unchanged state on
  exhaustion. Converter and viewer routes pass their existing and new tests.
- A fresh release WASM build passes the actual browser smoke: OCDraw and
  IFCX-CAD opening, real DXF/DWG roundtrips and byte-exact native download of
  IFCX-CAD containing counters above 2^53. Node tests report 31 passed and
  four Windows-skipped deployment tests.
- Native editing also exposed an existing writer comparison defect: unordered
  layer/block collections were not canonicalized like reader output. Comparing
  those collections in the reader's lexical path order now permits appending
  a definition without false semantic-change errors. IDs and draw order stay
  unchanged.

Review was performed by the implementing agent without subagents, as required
by the task's working convention. Global uniqueness, upstream full-width
Integer interoperability and offline collaboration remain unproven.
