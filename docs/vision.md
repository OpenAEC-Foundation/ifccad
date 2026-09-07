# IFCCAD vision

## Why IFCCAD exists

CAD drawings remain essential deliverables across architecture, engineering,
construction, manufacturing, and infrastructure. The dominant exchange options
do not provide an open, community-governed path that combines compact drawing
data with extensible project and BIM meaning.

DWG is compact and widely used, but proprietary. DXF is documented and useful
for interchange, but verbose and governed by a single vendor. IFC provides an
open semantic model for the built environment, but a building model and a CAD
drawing are not the same artifact. Projects therefore often maintain IFC and
DWG or DXF files side by side. When a BIM-derived drawing is flattened into
CAD, relationships to the products and semantics that produced it are commonly
lost.

IFCCAD aims to provide an open drawing model within the IFCX architecture. It
should support ordinary CAD-only drawings without requiring a BIM model, while
also allowing drawings to participate in a richer project graph and retain
optional relationships to IFC objects.

## Intended role

IFCCAD is an exchange package and logical drawing model. It is intended to:

- represent drawing structure and content through openly specified contracts;
- support both CAD-authored and BIM-derived drawings;
- retain semantic relationships where they exist without requiring them where
  they do not;
- distinguish native IFCCAD meaning from preserved source-format information;
- permit efficient streaming, compression, partial access, and parallel
  processing as physical encodings mature;
- remain implementable by independent applications and languages.

IFCCAD is not a requirement that every drawing contain building semantics. A
drawing set can stand on its own as CAD data inside an IFCX package.

This repository develops the primary, recommended IFCCAD implementation.
Applications can share that implementation directly or consume it through
language bindings as those become available, providing a practical common
interoperability layer. The published format contract remains authoritative and
language-neutral, so other implementations and focused validators can be built
without depending on the Rust source or a particular CAD library.

## The three-part model

### IFCX: meaning and relationships

IFCX owns the semantic graph and package-level relationships. It can describe
projects, buildings, products, drawing sets, drawings, layout identity, shared
drawing definitions, resource references, provenance, and optional links
between drawing content and IFC objects.

This graph may contain a complete BIM context, a minimal CAD-only drawing set,
or something between those extremes.

### IFCDR: drawing resources

IFCDR carries the high-volume typed content of a drawing. One IFCDR drawing
resource normally represents one drawing and can contain separate scopes for
model space and paper spaces, together with entities, draw order, bounds, and
compact bindings to semantic definitions.

Regular entity families can use typed columnar data. Repeated structured data
can use relational columns, while irregular entity families may use registered,
versioned typed payloads. These are logical choices; they do not prescribe JSON
arrays, chunks, or a binary layout.

IFCDR is not required to become a universal standalone CAD file. Its primary
role is to provide drawing content referenced by the IFCX graph.

### IFCPR: preservation

IFCPR records source semantics or content that cannot yet be represented
natively in IFCX and IFCDR. Preservation allows conversion coverage to grow
incrementally without pretending that unsupported information is irrelevant.

Preserved data remains distinguishable from native drawing meaning. It may be
structured for known source concepts or opaque where exact source bytes are the
only reliable roundtrip representation.

## Primary use cases

### Open CAD exchange

A CAD-only package can contain drawings, layers, appearances, layouts, styles,
and drawing resources without an IFC building model. This provides a path for
open drawing exchange independent of BIM adoption.

### BIM-aware drawings

A drawing can retain links to the IFCX products, spaces, systems, or other
semantic objects that it depicts. Applications may use these relationships for
selection, coordination, provenance, regeneration, or downstream automation.

### Generated and cached drawings

Drawing resources may be generated from building elements by IfcOpenShell or
another drawing engine. Generated output can be stored or cached for efficient
display while IFCX retains its origin and relationships. Generated resources
should remain distinguishable from independently CAD-edited resources so that
regeneration does not overwrite intentional work.

### Progressive migration and preservation

Applications can adopt native IFCCAD concepts incrementally. Unsupported source
semantics can be diagnosed and, where appropriate, carried through IFCPR until
an interoperable native representation exists.

## Design principles

### Semantic model before physical optimization

Entities, IDs, references, scopes, units, placements, and draw order are logical
concepts. Their meaning must not depend on JSON property names, chunk boundaries,
file names, compression, or binary layout.

These logical rules and their meaning must be stated in published schemas,
registries, documentation, and conformance cases. An implementer should not
need to infer the format contract by reading the primary Rust implementation.

JSON is the initial reference and conformance encoding because it is transparent
and straightforward to inspect. Compact encodings should be introduced only
when representative benchmarks justify their complexity.

### Explicit fidelity

Converters must not silently flatten, approximate, or discard source meaning.
They should represent content exactly, preserve it, emit structured diagnostics,
or reject inconsistent input. Technical serialization identities can change;
semantic identities and relationships cannot disappear without an explicit
account.

### Open extension with owned contracts

IFCX should remain extensible where open graph content is expected, while
IFCCAD-owned descriptors and registered payloads have clear versioned contracts.
Unknown extensions should be safely ignorable or preservable rather than making
the entire package unreadable.

### Compatibility is capability-specific

Package validity, implementation support, and lossless transfer are separate
claims. A valid package can contain extensions or semantics that a particular
application cannot convert or edit. Implementations should therefore state
which versions they read and write, which extensions and operations they
support, and how unknown content is handled. Conformance collections should
make those claims explicit through compatibility matrices and testable cases.

### Equivalent meaning across storage modes

Inline, external, packaged, generated, and cached resources can have different
physical locations while retaining the same logical identity and relationships.
Equivalent supported content should validate and fingerprint consistently
across encodings and rechunking.

### Evidence-driven efficiency

The mature format should be capable of efficient storage and access in the same
practical domain as established CAD formats. Exact targets and encodings must be
derived from reproducible measurements. Measurement can begin with the initial
JSON reference encoding and grow with the representative corpus before a
physical encoding is selected. Full source archives in IFCPR are measured
separately because they intentionally add source content to the native
representation.

## What success looks like

- Applications using the primary implementation or its language bindings can
  exchange the same logical drawing consistently.
- Once profiles stabilize, independent implementations or focused validators
  can verify the published contract without depending on the Rust source or a
  particular CAD library.
- CAD-only users gain an open drawing package without being forced into a BIM
  workflow.
- BIM-aware workflows can retain meaningful links between drawings and project
  objects.
- Unsupported source semantics are visible and preservable rather than silently
  lost.
- Public semantic APIs remain stable as physical storage evolves.
- Representative drawings can be processed and stored efficiently using
  evidence-backed encodings.
- Versioned conformance collections and compatibility matrices make validity,
  support, lossless transfer, and interoperability testable.

## Non-goals

- Reproduce DWG internals or every historical CAD implementation detail as the
  IFCCAD logical model.
- Require an IFC building model for CAD-only drawing exchange.
- Treat IFCDR as a mandatory standalone file format.
- Standardize binary encoding, compression, or a container before measurements
  and implementation experience support the decision.
- Hide lossy conversion behind visually plausible approximations.
- Replace conventional IFC exchange immediately or conflate product models with
  drawing deliverables.

## Development direction

The project develops the logical model, reference implementation, conformance
material, conversion coverage, and physical encodings incrementally. See
[`ROADMAP.md`](../ROADMAP.md) for the current milestone order, dependencies,
exit criteria, and linked design issues.
