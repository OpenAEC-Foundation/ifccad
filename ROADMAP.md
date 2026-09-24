# IFCCAD development roadmap

This roadmap is the source of truth for IFCCAD development sequencing and
milestone status. It describes intended outcomes and dependencies, not release
dates or commitments. Milestones may overlap, but later work should not become
coupled to boundaries that an earlier milestone still needs to resolve.

Detailed design questions and implementation tasks belong in linked GitHub
issues. The shorter roadmap in `README.md` summarizes this document and must be
updated in the same change when milestone names, order, or intent change.

## Status vocabulary

- **Established** — the foundation exists and is exercised by the current
  implementation, although its coverage can continue to grow.
- **Current** — the main architectural focus for substantial new work.
- **Next** — depends on the current milestone sufficiently stabilizing.
- **Later** — deliberately deferred until representative semantics and evidence
  exist.
- **Long-term** — strategic direction without a committed delivery window.

## Sequencing principles

- Stabilize logical semantics and public boundaries before expanding many
  entity families on top of storage-specific assumptions.
- Expand and verify semantic coverage before standardizing optimized physical
  encodings.
- Start measuring physical size early, but defer physical encoding choices
  until the logical model and representative corpus provide enough evidence.
- Use deterministic reference encodings and real roundtrips to validate the
  model throughout development.
- Do not silently approximate or discard source meaning. Represent it natively,
  preserve it, diagnose the loss, or reject inconsistent input.
- Keep numbered conformance collections immutable and develop new contracts in
  the active schemas and `conformance/next`.

## 1. Working format foundation

**Status: Established**

### Outcome

A small but complete vertical slice proves that IFCCAD packages can be defined,
validated, written, navigated, and converted without making the core format
crate depend on a CAD runtime.

### Established capabilities

- Versioned IFCX, IFCDR, and IFCPR schemas and conformance assets.
- Canonical values, physical checksums, and semantic fingerprints.
- Strict typed loading and navigation of directory packages.
- Deterministic construction and storage of an initial JSON-based package.
- One model-space drawing with layers, appearances, lines, and straight
  lightweight polylines.
- Bidirectional conversion between validated IFCCAD drawings and cadcodec
  `CadDocument`, with explicit loss diagnostics and entity mappings.
- Production-reader verification of writer and converter output.

This milestone intentionally establishes behavior with limited CAD coverage. It
does not imply that the logical model or physical encoding is complete.

## 2. Stable logical drawing-resource model

**Status: Established** — completed 2026-09-11.

### Outcome

IFCDR has encoding-neutral logical types, validation, and public APIs. Its
logical rules and their meaning are described in the published format contract
and exercised by conformance cases, without requiring an implementer to study
the Rust source. IFCX and Rust terminology describe an IFCDR resource as a
complete drawing resource, independent of whether its content is JSON, inline,
external, chunked, or later binary encoded.

### Why this precedes broader entity coverage

Adding many entity families while public views and validation still depend on
JSON structure would multiply migration work and make a second encoding much
harder. The common semantic boundary should be credible before the model grows.

### Scope

- Begin with a reduced active IFCDR contract for lines, straight polylines,
  and their required resource, scope, binding, appearance, and order data.
  Remove other inherited prototype entity definitions and corresponding
  implementation assumptions before introducing the encoding-neutral model.
  Retain IFCPR and its existing limited checks; full preservation remains
  milestone 3 work.
- Separate logical IFCDR resources and semantic validation from physical JSON
  decoding and encoding ([issue #7](https://github.com/OpenAEC-Foundation/ifccad/issues/7)).
- Align IFCX, IFCDR, Rust, and documentation terminology around complete drawing
  resources ([issue #8](https://github.com/OpenAEC-Foundation/ifccad/issues/8)).
- Describe logical types, constraints, defaults, references, and invariants in
  language-neutral contract material and make their meaning testable through
  conformance cases. Language-neutral enum constraints in the registry are one
  part of this broader requirement
  ([issue #5](https://github.com/OpenAEC-Foundation/ifccad/issues/5)).
- Complete a common abstraction for logically identified external and inline
  resources ([issue #1](https://github.com/OpenAEC-Foundation/ifccad/issues/1)).
- Establish a new versioned JSON reference baseline for the reduced contract,
  then preserve its supported semantics and deterministic output while moving
  JSON-specific behavior behind codec boundaries. Reading or writing older
  IFCDR versions is not required; renew active fixtures and report unsupported
  versions explicitly rather than providing a migration layer.
- Define an explicit compatibility matrix alongside the conformance collections
  for the primary implementation. It must state which contract and profile
  versions the implementation can read and write, which extensions it
  understands, and how it treats unknown content. This matrix supports the open
  extension model and the migration work in
  [issue #8](https://github.com/OpenAEC-Foundation/ifccad/issues/8).
- Begin the reproducible file-size measurement experiments from
  [issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6) using the
  current JSON reference encoding and available fixtures. These early results
  establish a method and baseline; they do not select a physical encoding.

### Compatibility vocabulary

Compatibility must report separate, operation-specific properties:

- **Valid** — the package satisfies the applicable published format contract.
- **Supported** — the implementation understands the content required for the
  stated read, write, conversion, or editing operation.
- **Losslessly transferable** — the operation preserves the relevant meaning,
  either natively or through an approved preservation mechanism.

A valid package need not be fully supported, convertible, or editable by every
implementation. Unknown content must therefore have explicit behavior rather
than being treated as proof that the entire package is invalid.

### Exit criteria

- Public semantic resource and entity APIs do not expose JSON values, property
  paths, file extensions, or chunk layout.
- Physical codec validation and shared semantic validation are separately
  testable.
- The logical registry and associated contract material distinguish semantic
  constraints from JSON mappings, and an implementer can determine those rules
  and their meaning without inspecting the Rust implementation.
- Drawing-resource terminology is consistent across active schemas, code,
  conformance material, and documentation.
- Supported inline and external resources normalize into the same validated
  logical model.
- The new supported JSON baseline remains covered by semantic roundtrip and
  determinism tests. Compatibility reporting explicitly identifies unsupported
  older versions; frozen conformance collections remain unchanged.
- Conformance reporting distinguishes validity, operation support, and lossless
  transfer, and includes an initial explicit compatibility matrix.
- A reproducible size-measurement method and initial JSON baseline are recorded
  without committing the format to a new physical encoding.

An independent implementation is not an exit criterion for this milestone.
The primary implementation and conformance material establish the initial
contract; independent implementations or limited validators become an
additional test once a profile is sufficiently stable.

The base contract, encoding-neutral logical model and drawing-resource
terminology are implemented and verified. Shared semantic access and validation,
separate reader/writer backings, logical registry and JSON mapping establish the
validated-resource codec boundary. The [compatibility matrix](conformance/next/COMPATIBILITY.md)
describes the active contract and representation references across layouts.
The [resource-source contract](schemas/ifcx/resource-source-contract-0.9.0.md)
defines inline/external IFCDR and IFCPR access; the writer explicitly selects
IFCDR storage. The [reporting contract](conformance/next/reporting-contract-v1.md)
defines diagnostic categories, package assessment completeness and scoped
converter summaries, with manifest v2 expectations. The
[size and exchange experiment](docs/benchmarks/size-baseline-v1.md) consolidates
the method, measurements and successful exchange/repeatability checks for
cadcodec 0.5.4 at `2f2cd25` with two local DWG fixes. Compression remains an
explanatory check until a supported production encoding can be measured.
The successful, reproducible run with its explicitly pinned local patches is
accepted as the initial measurement reference for this milestone. Integrating
the two upstream DWG fixes is codec maintenance, not an unresolved logical-model
boundary; it therefore does not block broader native semantics. This replaces
the earlier, unnecessarily strict requirement to wait for upstream integration.
The subsequent scopes/blocks run uses unmodified cadcodec `5b682ed6` and passes
the full primitive corpus, including DWG return chains, with
`baseline_accepted: true`. That report replaced the historical patched
reference without changing milestone 2's closure decision. Block-specific DWG
marker issue #52 remains a separate, explicitly rejected boundary.
The 2026-09-23 layout/viewport candidate rerun retains the same unmodified
cadcodec pin and corpus, updates the active 0.10.0/0.12.0 output measurements,
and again passes full repeatability and IFCCAD/DXF/DWG exchange. Its first
diagnostic runs caught an inert DXF layout scaffold and a redundant DWG model
plot flag, both now fixed and regression-tested; the
[retained report](docs/benchmarks/size-baseline-v1.md)
replaces the earlier measurements without changing milestone 2's closure.

### Closure assessment

| Exit criterion | Evidence |
| --- | --- |
| Encoding-neutral public semantic APIs | Typed resource/entity access and separate reader/writer backings; [logical contract tests](tests/ifcdr_logical_contract.rs). |
| Separate physical and semantic validation | Independent [JSON codec tests](src/ifcdr/codec/json/tests.rs) and [logical validation tests](src/ifcdr/logical/tests.rs). |
| Language-neutral semantics separate from mappings | [Logical contract](schemas/ifcdr/logical-contract-0.8.0.md), registry, JSON mapping and [mapping language](schemas/ifcdr/json-mapping-v2.md). |
| Consistent drawing-resource terminology | [Drawing resource contract](schemas/ifcx/drawing-resource-contract-0.8.0.md), active overlay and conformance checks. |
| Inline/external normalization | [Resource source contract](schemas/ifcx/resource-source-contract-0.9.0.md) and [inline resource tests](tests/inline_resources.rs). |
| Supported JSON roundtrip, determinism and version handling | [Writer roundtrip tests](tests/package_writer_roundtrip.rs), active conformance and compatibility matrix; frozen collection unchanged. |
| Validity, support and transfer distinguished | [Reporting contract](conformance/next/reporting-contract-v1.md), compatibility matrix and package/converter assessment tests. |
| Initial reproducible size reference | [Complete experiment](docs/benchmarks/size-baseline-v1.md), both successful generations and explicit dependency provenance. |

Final workspace verification: formatter and Clippy pass; 335 tests pass with
one existing ignored test. No new encoding, migration layer, native entity
family, full IFCPR implementation or conformance release is implied by closure.

### Follow-up outside milestone 2

- The cadcodec fixes tracked by [#41](https://github.com/HakanSeven12/cadcodec/issues/41)
  and [#42](https://github.com/HakanSeven12/cadcodec/issues/42) are used through
  the unmodified shared `5b682ed` pin; the full corpus passes. Older local
  patches remain only as a historical reproduction recipe.
  [IFCCAD #9](https://github.com/OpenAEC-Foundation/ifccad/issues/9) tracks the
  subsequent semantic-inventory coverage integration. Upstream
  [field-level classification #50](https://github.com/HakanSeven12/cadcodec/issues/50)
  would strengthen coverage but does not block adoption of the current inventory.
- Expand the practical corpus and supported native semantics in milestone 3.
- Measure production compression/other encodings when implemented; the broader
  experiments in issue #6 remain separate.

## 3. Native CAD semantics and preservation

**Status: Current**

### Outcome

IFCCAD represents useful complete drawings beyond the initial 2D proof while
retaining a precise account of anything that is not yet native. CAD-only,
BIM-aware, imported, and generated drawings use the same logical model.

### Scope

#### Reference workflow and coverage corpus

Use representative DXF and DWG drawings from the
[`ifccad-prototype`](https://github.com/OpenAEC-Foundation/ifccad-prototype)
repository to test native CAD coverage and preservation requirements. Begin with
the foundation-repair drawing in `DXF DWG samples/3bm` as a candidate reference
workflow.

Inventory that drawing's entities, relationships, presentation requirements,
and source semantics. Use the inventory to define an initial practical drawing
profile and staged roundtrip expectations. Track which content is represented
natively, preserved through IFCPR, or reported as unsupported.

Expand the reference corpus as implementation progresses to cover workflows
and semantics beyond this initial drawing.

#### Development dependencies

Sequence work by semantic and architectural dependencies. Entity frequency in
the reference drawing supplies test cases and helps choose between equally
ready extensions; it does not override those dependencies.

Begin with coordinate frames, planar/spatial geometry, placement boundaries and
bounds, proved by a small line/polyline implementation. Establish scope ownership
and block instancing before dependent block inheritance and presentation work.
Introduce shared styles and further entity families when their prerequisites
are stable, without making independent geometry wait for unrelated style work.
Define preservation identity and restoration boundaries early, then prove
preservation incrementally. Add the CAD-BIM relationship proof once identity
and placement are ready, without waiting for complete CAD coverage.

The first coordinate-frame slice is implemented in IFCDR 0.8.0: XYZ lines,
placed straight polylines, per-scope bounds and conversion accuracy policy.
The [logical contract](schemas/ifcdr/logical-contract-0.8.0.md),
[compatibility matrix](conformance/next/COMPATIBILITY.md),
[spatial exchange tests](crates/ifccad-convert/tests/spatial_exchange.rs) and
[current preparation measurements and practice-file inventory](docs/benchmarks/placement-preparation-v1.md)
record the boundary and evidence. Milestone 3 remains current: blocks, further
entity families, preservation and CAD-BIM relationships remain separate work.
Internal design notes and implementation plans under `docs/superpowers/` are
local-only; published contracts and this roadmap remain authoritative.

The **scope ownership, local block definitions and instances** slice (B) is
implemented in IFCDR 0.9 / IFCX overlay 0.11, with the explicit codec limitations
recorded below. This is not completion of milestone 3. The following dependency map carries forward the agreed
sequencing; each slice still needs its own concrete design and roundtrip proof.
It is a partial order, not a requirement to finish every row before the next.

| Slice | Prerequisites | Boundary and practical proof |
| --- | --- | --- |
| A. Coordinates and basic geometry — implemented | Milestone 2 logical model | XYZ lines, placed straight polylines, per-scope bounds and conversion accuracy. |
| B. Scopes and instances — implemented | A | IFCDR 0.9/overlay 0.11 model/paper ownership, resource-local definitions and instances, signed transforms and evaluated bounds; shared-reference conversion, strict-readback and real DXF/DWG boundary tests. |
| C. Shared styles and inheritance — candidate graph lists | Existing appearances; B for instance-dependent inheritance | IFCX 0.12 adds direct drawing-level layer/appearance lists and distinct layer On/Freeze/Lock/Plot state. Effective inherited-style rendering through instances remains a separate proof. |
| D. Geometric families | A; additional references where needed | Curves, planar polyline segments and spatial polylines; prove each family independently. |
| E. Annotation and compound entities | A and relevant B/C/D boundaries | Text, attributes, hatches and dimensions; introduce typed payloads when a concrete family needs them. |
| F. Viewports and presentation — candidate native profile | A/B and applicable entity/style support | IFCX effective plot values and IFCDR paper viewports/relational overrides establish paper/model mapping without flattening; broader CAD view-state and rendering equivalence remain open. |
| P. Preservation | Existing identity/reporting and the selected family's references | Source/native correspondence, restoration eligibility and dependencies; prove one bounded preserve/restore case. |
| I. CAD-BIM integration | A and existing identities | One product linked to drawing entities with explicit placement and provenance; no dependency on full CAD coverage. |

D can advance after A without waiting for unrelated styles. C develops with its
first consumers rather than as a speculative universal style system. F need
not wait for every annotation family, but its test profile must state coverage.
Design P's boundaries alongside B and prove restoration with a selected source
case. Exercise I early enough to test placement and identity before later
families depend on them. These dependencies do not authorize parallel agent work.

For B, the [block transform contract](docs/geometry/block-transform.md) separates
placement, rotation and signed scale; base points belong to definitions, not
generic scopes. The F candidate adds effective plot settings and viewport
entities to native paper scopes without changing block geometry. The
[CAD boundary](docs/geometry/block-cad-boundary.md) records DWG marker issue #52,
DXF description loss and scale limitations; these are not silently repaired.
For P, define restoration
eligibility after native edits: retaining source bytes alone does not prove
restoration or justify suppressing loss diagnostics.

#### Drawing-level layer and appearance collections — candidate implemented

Add direct `Layers` and `Appearances` reference lists to the IFCX `Drawing`
node's `children`. These lists make the definitions available to a drawing
discoverable from IFCX without opening its IFCDR resource. Use direct lists,
not separate `LayerTable` or `AppearanceTable` nodes.

- Membership means available to the drawing, not necessarily used by an
  entity; retain unused definitions as well.
- Refer to the existing IFCX `Layer` and `Appearance` nodes. Definitions remain
  shareable across drawings; list membership does not imply exclusive ownership
  or duplicate the definition's values.
- Keep each layer's reference to its default appearance. Keep resource-local
  `layerBindings` and `appearanceBindings` in IFCDR, including per-property
  ByLayer/ByBlock/explicit intent. The IFCX lists do not replace local bindings.
- Define consistency rules between drawing membership, layer-default appearance
  references and IFCDR bindings. Specify whether referenced appearances require
  explicit membership or are included transitively, and validate the chosen
  rule so the two representations cannot contradict each other.

IFCX overlay 0.12 requires the lists and defines one-way membership closure
against IFCDR bindings. Reader, writer and the unpublished `conformance/next`
candidate now expose the contract. Unused definitions are valid; one definition
may be listed by multiple Drawings. Effective ByLayer/ByBlock style resolution
through nested instances remains a presentation-level follow-up, not an
implicit change to block geometry or resource ownership.

#### Format and implementation work

- Define deliberate planar and spatial geometry, coordinate-frame, placement,
  and polyline semantics
  ([issue #4](https://github.com/OpenAEC-Foundation/ifccad/issues/4)).
- Represent model space and paper spaces as scopes of a complete drawing
  resource.
- Expand native support for blocks, text, dimensions, hatches, viewports,
  styles, and other prioritized CAD concepts.
- Use registered typed payloads where irregular entity data does not fit a
  natural columnar or relational-columnar model
  ([issue #2](https://github.com/OpenAEC-Foundation/ifccad/issues/2)).
- Connect IFCPR preservation to conversion so non-native source semantics and
  selected source content can survive a package roundtrip.
- Preserve optional relationships between drawing content and IFCX project or
  product semantics.
- Add a compact CAD-BIM integration test with an IFCX product, linked drawing
  entities, an explicit placement, and recorded BIM-model provenance. This is
  an early vertical proof of the relationship model, not a demand for full IFC
  interoperability.
- Extend the size-measurement baseline as the reference corpus, native entity
  coverage, and preservation payloads grow. Report native and preservation
  contributions separately so later encoding decisions use representative
  evidence.

### Exit criteria

- Representative drawings with model and paper layouts pass semantic
  IFCCAD/CAD roundtrips with complete, structured loss reporting.
- The initial foundation-repair workflow has a published coverage inventory,
  practical drawing profile, and staged roundtrip expectations that distinguish
  native, preserved, and unsupported content.
- Planar and spatial geometry have explicit logical invariants independent of
  their physical encoding.
- Irregular supported entities expose typed semantic views rather than raw
  storage payloads.
- Preserved content is distinguishable from native content and prevents false
  loss reports when it guarantees the intended roundtrip meaning.
- A small CAD-BIM fixture verifies product-to-drawing relationships, placement,
  and provenance through the supported workflow.
- The conformance suite covers the expanded native and preservation contracts.

## 4. Scalable physical encodings and packaging

**Status: Later**

### Outcome

Large IFCCAD packages can be stored and accessed efficiently without changing
their logical meaning or public semantic APIs. Encoding choices are supported
by reproducible measurements rather than assumed benefits.

### Dependencies

This milestone needs a stable encoding-neutral model, the measurement method
begun in milestone 2, and the representative corpus expanded in milestone 3
with regular, irregular, planar, spatial, layout, and preservation data.

### Scope

- Separate logical streams from physical chunk boundaries
  ([issue #3](https://github.com/OpenAEC-Foundation/ifccad/issues/3)).
- Extend the earlier size experiments into representative component-size,
  performance, and memory benchmarks against comparable DXF and DWG inputs
  ([issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6)).
- Compare minified and compressed JSON with experimental binary column and
  typed-payload encodings.
- Evaluate indexing, partial loading, parallel decoding, checksums, and
  corruption isolation.
- Define a distributable `.ifccad` container when its resource and integrity
  requirements are sufficiently understood.

### Exit criteria

- Equivalent logical content validates and fingerprints consistently across
  supported physical encodings and rechunking.
- Benchmarks identify which optimizations justify their complexity and which do
  not.
- Native-only and preservation-inclusive package sizes are reported separately.
- The selected container and codec contracts are versioned, bounded, testable,
  and implementable independently.

## 5. Interoperability and wider IFC workflows

**Status: Long-term**

### Outcome

IFCCAD becomes a practical open interchange layer for CAD-only drawings,
BIM-aware drawings, and drawings generated from IFCX or conventional IFC
models. Applications using the same primary library or its language bindings
can already achieve practical interoperability; independently implemented
producers, consumers, or limited validators provide an additional test of a
published profile once it is sufficiently stable.

### Direction

- Demonstrate end-to-end interoperability between multiple applications using
  the primary implementation or its language bindings.
- Publish stable conformance collections and compatibility matrices for
  independently implementable profiles.
- Validate sufficiently stable profiles with an independent implementation or
  a deliberately limited validator, in addition to testing more producers,
  consumers, and CAD/BIM applications.
- Generate drawing resources from building elements while retaining optional
  links to their source products.
- Distinguish generated, cached, imported, and CAD-edited drawing resources so
  applications can make safe regeneration decisions.
- Develop conventional IFC import and export where it serves the wider workflow
  without conflating IFC models with drawing exchange.

Progress in this milestone depends on implementation experience and ecosystem
participation rather than a fixed feature checklist. An independent
implementation or validator is deliberately not a prerequisite for milestone
2; it is most useful after the logical profile it tests has stabilized.

## Maintaining this roadmap

- Change milestone status only when repository evidence supports the new state.
- Keep outcomes and exit criteria stable; put short-lived task breakdowns in
  GitHub issues.
- Use GitHub issues for bounded problems, design questions, and deliverables
  with their own acceptance criteria. Use roadmap milestones for sequencing and
  shared outcomes; do not turn every future roadmap item into an issue
  prematurely.
- Update the roadmap when an issue changes an architectural dependency, not for
  every implementation detail.
- Keep the `README.md` summary synchronized in the same change.
- Treat roadmap changes as design decisions and explain meaningful reordering.
