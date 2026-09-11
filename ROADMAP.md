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

**Status: Current**

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

The first implementation slice is described in the
[base-contract design](docs/superpowers/specs/2026-09-08-ifcdr-base-contract-design.md)
(implemented and verified). The second slice, the
[encoding-neutral logical model](docs/superpowers/specs/2026-09-08-ifcdr-logical-model-design.md),
is also implemented and verified: shared semantic access and validation,
separate reader/writer backings, logical registry and JSON mapping, and a
validated-resource codec boundary. The third slice,
[drawing resource terminology](docs/superpowers/specs/2026-09-08-drawing-resource-terminology-design.md),
adds IFCX overlay 0.8.0 and consistent representation references across layouts.
See the
[compatibility matrix](conformance/next/COMPATIBILITY.md) for the active 0.7.0
contract. The [inline resource implementation](docs/superpowers/specs/2026-09-09-inline-resource-design.md)
normalizes inline and external IFCDR/IFCPR sources and adds explicit IFCDR writer
storage selection. The [reporting implementation](docs/superpowers/specs/2026-09-11-compatibility-reporting-design.md)
adds diagnostic categories, package assessment completeness and scoped converter
summaries, with manifest v2 expectations. Initial reproducible size measurements
remain required; this milestone is still Current.

## 3. Native CAD semantics and preservation

**Status: Next**

### Outcome

IFCCAD represents useful complete drawings beyond the initial 2D proof while
retaining a precise account of anything that is not yet native. CAD-only,
BIM-aware, imported, and generated drawings use the same logical model.

### Scope

#### Reference workflow and coverage corpus

Use representative DXF and DWG drawings from the
[`ifccad-prototype`](https://github.com/OpenAEC-Foundation/ifccad-prototype)
repository to guide native CAD coverage and preservation priorities. Begin with
the foundation-repair drawing in `DXF DWG samples/3bm` as a candidate reference
workflow.

Inventory that drawing's entities, relationships, presentation requirements,
and source semantics. Use the inventory to define an initial practical drawing
profile and staged roundtrip expectations. Track which content is represented
natively, preserved through IFCPR, or reported as unsupported.

Expand the reference corpus as implementation progresses to cover workflows
and semantics beyond this initial drawing.

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
