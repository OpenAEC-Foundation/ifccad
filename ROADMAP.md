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

IFCDR has encoding-neutral logical types, validation, and public APIs. IFCX and
Rust terminology describe an IFCDR resource as a complete drawing resource,
independent of whether its content is JSON, inline, external, chunked, or later
binary encoded.

### Why this precedes broader entity coverage

Adding many entity families while public views and validation still depend on
JSON structure would multiply migration work and make a second encoding much
harder. The common semantic boundary should be credible before the model grows.

### Scope

- Separate logical IFCDR resources and semantic validation from physical JSON
  decoding and encoding ([issue #7](https://github.com/OpenAEC-Foundation/ifccad/issues/7)).
- Align IFCX, IFCDR, Rust, and documentation terminology around complete drawing
  resources ([issue #8](https://github.com/OpenAEC-Foundation/ifccad/issues/8)).
- Add language-neutral enum constraints to the registry
  ([issue #5](https://github.com/OpenAEC-Foundation/ifccad/issues/5)).
- Complete a common abstraction for logically identified external and inline
  resources ([issue #1](https://github.com/OpenAEC-Foundation/ifccad/issues/1)).
- Preserve compatibility and deterministic output for the JSON reference
  encoding while moving JSON-specific behavior behind codec boundaries.

### Exit criteria

- Public semantic resource and entity APIs do not expose JSON values, property
  paths, file extensions, or chunk layout.
- Physical codec validation and shared semantic validation are separately
  testable.
- The logical registry distinguishes semantic constraints from JSON mappings.
- Drawing-resource terminology is consistent across active schemas, code,
  conformance material, and documentation.
- Supported inline and external resources normalize into the same validated
  logical model.
- Existing supported JSON packages remain covered by compatibility and
  determinism tests.

## 3. Native CAD semantics and preservation

**Status: Next**

### Outcome

IFCCAD represents useful complete drawings beyond the initial 2D proof while
retaining a precise account of anything that is not yet native. CAD-only,
BIM-aware, imported, and generated drawings use the same logical model.

### Scope

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

### Exit criteria

- Representative drawings with model and paper layouts pass semantic
  IFCCAD/CAD roundtrips with complete, structured loss reporting.
- Planar and spatial geometry have explicit logical invariants independent of
  their physical encoding.
- Irregular supported entities expose typed semantic views rather than raw
  storage payloads.
- Preserved content is distinguishable from native content and prevents false
  loss reports when it guarantees the intended roundtrip meaning.
- The conformance suite covers the expanded native and preservation contracts.

## 4. Scalable physical encodings and packaging

**Status: Later**

### Outcome

Large IFCCAD packages can be stored and accessed efficiently without changing
their logical meaning or public semantic APIs. Encoding choices are supported
by reproducible measurements rather than assumed benefits.

### Dependencies

This milestone needs a stable encoding-neutral model and a representative
corpus of regular, irregular, planar, spatial, layout, and preservation data.

### Scope

- Separate logical streams from physical chunk boundaries
  ([issue #3](https://github.com/OpenAEC-Foundation/ifccad/issues/3)).
- Establish reproducible component-size, performance, and memory benchmarks
  against comparable DXF and DWG inputs
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
models.

### Direction

- Publish stable conformance collections for independently implementable
  profiles.
- Validate interoperability with additional producers, consumers, and CAD/BIM
  applications.
- Generate drawing resources from building elements while retaining optional
  links to their source products.
- Distinguish generated, cached, imported, and CAD-edited drawing resources so
  applications can make safe regeneration decisions.
- Develop conventional IFC import and export where it serves the wider workflow
  without conflating IFC models with drawing exchange.

Progress in this milestone depends on implementation experience and ecosystem
participation rather than a fixed feature checklist.

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
