# CadDocument to IFCCAD Export Design

**Status:** Proposed

**Date:** 2026-09-04

**Scope:** `ifccad` and `ifccad-convert`

**Related issues:** IFCCAD #7, IFCCAD #8, cadcodec #30

## Summary

This change adds the first supported export direction from a cadcodec
`CadDocument` to an in-memory IFCCAD package. The first version exports one
drawing with one model-space resource, supports lines and straight lightweight
polylines that can be represented exactly in IFCDR 0.5, preserves the complete
representable layer table and supported appearance semantics, and reports every
detectable semantic loss through typed diagnostics.

The exporter is intentionally conservative. It never silently flattens,
approximates, rescales, or fabricates semantic source data. Callers can allow a
lossy result or reject the complete result when any package-level loss is found.

The change also extends the core package writer so each entity appearance
property can independently be inherited by layer, inherited by block, or set
explicitly. This is required to preserve mixed CAD appearance inheritance.

## Goals

- Provide a stable public `CadDocument -> EncodedPackage` conversion API.
- Export one true model-space drawing from a `CadDocument`.
- Exactly export supported `LINE` and `LWPOLYLINE` instances.
- Export all representable layers, including empty layers, in source order.
- Preserve supported units, visibility, color, opacity, line pattern, and line
  weight semantics.
- Preserve mixed per-property entity appearance inheritance.
- Report all safely detectable semantic loss in deterministic diagnostics.
- Support permissive and rejecting loss policies without changing the public
  conversion shape later.
- Return a mapping from emitted cadcodec handles to generated IFCDR entity IDs.
- Keep logical conversion, logical package construction, physical encoding, and
  storage as separate responsibilities.
- Establish controlled end-to-end fixtures for low-loss, high-loss, and
  IFCCAD-originated roundtrips.

## Non-goals

- Paper-space export.
- Multiple drawings or multiple drawing resources.
- Block definitions, inserts, or block expansion.
- Legacy `POLYLINE` export.
- Curves, text, dimensions, hatches, images, meshes, solids, or other entity
  types beyond the initial line and lightweight-polyline subset.
- IFCPR preservation storage.
- A standalone loader or writer facade in `ifccad-convert`.
- ZIP or other physical package codecs. That boundary remains tracked by
  IFCCAD #7.
- Renaming the legacy IFCX `DrawingGeometryRepresentation` terminology. That
  schema change remains tracked by IFCCAD #8 and should happen before
  multi-scope or paper-space work.
- A claim that semantic coverage is exhaustive for cadcodec internals that are
  not exposed to downstream consumers. cadcodec #30 tracks that prerequisite.

## Terminology and responsibility boundaries

- **Import:** IFCCAD drawing to `CadDocument`.
- **Export:** `CadDocument` to a logical/in-memory IFCCAD package.
- **Encode:** logical package data to a physical package representation. The
  current writer still performs this while finishing an `EncodedPackage`; the
  responsibility can move behind a codec later without changing the export API.
- **Write/save:** store an already produced package artifact at a destination.

A direct IFCCAD-to-DXF convenience workflow does not need to expose these
internal direction names. Internally, it imports IFCCAD into a `CadDocument`
and then lets cadcodec write DXF.

## Public API

The export module is the counterpart of the existing `import` module:

```rust
pub fn cad_document_to_package(
    document: &CadDocument,
    package_options: PackageOptions,
    export_options: ExportOptions,
) -> Result<ExportOutcome, ExportError>
```

`PackageOptions` remains the core writer type and contains package identity and
provenance:

```rust
pub struct PackageOptions {
    pub package_id: PackageId,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}
```

`ExportOptions` contains only conversion policy. Resource identity and package
metadata do not belong in it.

```rust
pub struct ExportOptions {
    pub loss_policy: ExportLossPolicy,
}

pub enum ExportLossPolicy {
    Allow,
    Reject,
}
```

`ExportOptions::default()` selects `ExportLossPolicy::Allow`. This makes the
initial exporter useful while its native coverage is deliberately small, while
allowing strict callers to opt in immediately.

The exporter creates the first resource ID internally and deterministically as
`drawing-main`. The current core writer continues to use its fixed resource URI
`resources/model-space.ifcdr.json`. A caller does not choose either value because
the initial export always produces exactly one drawing resource.

`ExportOutcome` owns:

- the completed `EncodedPackage`;
- the ordered `Vec<ExportDiagnostic>`;
- an `ExportEntityMapping`.

It exposes `package()`, `diagnostics()`, `entity_mapping()`, `into_package()`,
and `into_parts()`, mirroring the ergonomics of `ImportOutcome`.

`ExportEntityMapping` maps each emitted cadcodec `Handle` to the IFCDR
`EntityId` assigned by `PackageBuilder`. Only emitted entities occur in the
mapping. Cadcodec handle numbers are not reused as IFCDR IDs.

All new public option, outcome, diagnostic, mapping, and error types are
re-exported by `ifccad-convert` at crate root, while their implementations live
under `export`.

## Loss contract

`ExportLossPolicy` governs irreversible semantic loss in the produced IFCCAD
package:

- `Allow` returns a package together with every detected loss diagnostic.
- `Reject` performs the same complete safe scan but returns `LossRejected`
  instead of a partial package when one or more loss diagnostics exist.

A source handle's numeric value is serialization bookkeeping, not semantic
identity by itself. Replacing it with an IFCDR ID is therefore not export loss.
It is loss when a handle expresses a semantic identity, relationship, or link to
attached content and that meaning is not represented in the output.

Likewise, raw codec bytes, parser bookkeeping, object order used only for file
encoding, caches, and unreferenced numeric IDs are not package-level semantics.
Geometry, rendering, relationships, meaningful metadata, XDATA, materials,
visual styles, and semantically relevant dictionaries or reactors are semantic
and must either be represented or diagnosed.

This version's strict guarantee is intentionally bounded: `Reject` rejects all
semantic loss detectable through cadcodec's typed public model plus the explicit
coverage audit pinned to cadcodec revision
`a0f7d444f1607bc4b2c881060cbe7ea1014253cb`. It cannot prove preservation of
private or raw information that cadcodec does not expose. cadcodec #30 requests
an exhaustive semantic inventory so downstream converters can make that
guarantee stronger and maintain it across future cadcodec changes.

When full IFCPR preservation is added, unsupported data that is preserved
completely will no longer count as package loss, even if it cannot be represented
natively in IFCDR. It may still produce a non-loss informational diagnostic.
That future distinction can add a separate native-coverage policy; it must not
overload the meaning of `ExportLossPolicy`.

## Architecture

### Single staging representation

The exporter does not create a second full `ExportPlan` intermediate model.
`PackageBuilder` already stages layers, appearances, and entities in memory and
encodes them only when `finish()` is called. Duplicating all geometry in another
plan would add memory, conversion code, and another invariant boundary without
improving the first exporter.

Instead, small pure classification functions return one of three conceptual
decisions for each source item:

- `Emit(converted_value)`: the item can be represented under the initial
  contract;
- `Skip(loss_diagnostic)`: the source is coherent, but current IFCCAD export
  coverage cannot represent it without loss;
- `Fatal(structural_problem)`: the source is inconsistent or ambiguous enough
  that a coherent drawing cannot safely be constructed.

Accepted definitions are inserted directly into `PackageBuilder`. A small
`ExportContext` retains only conversion-wide state:

- accumulated diagnostics;
- handle-to-entity-ID mappings;
- deduplication indexes for composite appearances;
- CAD layer name to `LayerKey` lookup.

The context does not own the builder. The orchestrator owns `PackageBuilder`
and passes its scoped `DrawingBuilder` access to the relevant conversion code,
avoiding self-referential Rust lifetimes.

### Processing sequence

1. Construct `PackageBuilder` from `PackageOptions`, validating package metadata
   before expensive source traversal.
2. Run structural preflight and collect every independently detectable fatal
   source problem.
3. If preflight found fatal problems, return `InvalidSourceStructure` before
   opening the drawing. Problems that make source traversal unsafe therefore do
   not lead to speculative conversion.
4. Derive the model layout name and length unit.
5. Open one drawing using resource ID `drawing-main`.
6. Convert appearances and every representable source layer.
7. Scan source entities and other exposed semantic document components,
   classifying each as emit, skip, or fatal.
8. If the content scan found fatal problems, return `InvalidSourceStructure`
   with every such problem that could be collected safely.
9. If the policy is `Reject` and loss diagnostics exist, discard the builder and
   return `LossRejected` with the complete loss list.
10. Otherwise call `PackageBuilder::finish()` and return `ExportOutcome`.

`PackageBuilder` remains responsible for defensively validating target IFCCAD
data. The converter is responsible for understanding source fidelity. Builder
errors are not repackaged as loss diagnostics.

This decision seam can later grow to `EmitNative`, `Preserve`,
`EmitNativeAndPreserve`, and `Skip` when IFCPR exists. A richer full plan should
only be introduced when preservation, multiple resources, or cross-resource
coordination makes it useful. None of those changes require replacing the public
export function.

## Structural preflight and drawing identity

The true model-space block record is identified by
`CadDocument.header.model_space_block_handle`. The exporter then finds `Layout`
objects whose `block_record` relationship points to that handle.

Exactly one matching layout is required. Its name is copied exactly to
`DrawingOptions.model_layout_name`; in ordinary DXF data this is `Model`. The
relationship is authoritative, not the layout name or a paper/model flag.

Zero or multiple matching layouts are fatal source-structure problems. Preflight
also checks every other independently inspectable prerequisite needed to
interpret ownership and layers. It aggregates those problems rather than
returning at the first one. If corruption makes further inspection unsafe, it
stops at that boundary and returns everything found so far.

## Conversion rules

### Units

`CadDocument.header.insertion_units` maps exactly as follows:

| cadcodec unit | IFCDR unit |
| --- | --- |
| Unitless | `unitless` |
| Millimeters | `mm` |
| Centimeters | `cm` |
| Meters | `m` |
| Kilometers | `km` |
| Inches | `in` |
| Feet | `ft` |

An unsupported or unknown source unit maps to IFCDR `unitless`, leaves all
coordinate values unchanged, and creates a loss diagnostic. The exporter never
performs a hidden coordinate rescale.

### Layers

The exporter scans the complete cadcodec layer table in source order, including
empty layers. Every layer whose required IFCDR fields can be represented exactly
is emitted with its name, visibility, and appearance.

IFCDR visibility is false when the source layer is off or frozen. Because IFCDR
does not retain the distinction, the frozen/off state that is not recoverable is
included in that layer's diagnostic. Other unsupported layer semantics, such as
locking, frozen-in-new-viewport state, plot state, material links, XREF state,
or attached semantic metadata, also produce one bundled diagnostic for that
layer while its representable core is retained.

The exporter does not invent a required appearance. If a layer lacks an exactly
representable required color, opacity, line pattern, or line weight, that layer
is skipped with a loss diagnostic, and entities that reference it are skipped
with their own missing-target loss diagnostic. This preserves deterministic
behavior without an arbitrary rendering fallback.

An entity that refers to a missing source layer is skipped under `Allow`; under
`Reject` it contributes to the eventual rejection. The exporter never
synthesizes a replacement layer.

### Appearance model

The core writer's current entity-wide `EntityAppearance` mode is too coarse for
CAD semantics. It is replaced or extended with a composite entity appearance in
which each property independently has one of these modes:

- `ByLayer`;
- `ByBlock`;
- `Explicit(value)`.

The four independently controlled properties are:

- color;
- opacity;
- line pattern;
- line weight.

Layer appearances remain explicit definitions. Indexed CAD colors preserve both
their resolved RGB value and their ACI index; true-color values preserve RGB.
Named line patterns preserve their source name. Exactly representable numeric
opacity and line weight values are preserved without rounding.

Identical explicit definitions and identical composite bindings are deduplicated
deterministically. Mixed inheritance, such as color by layer with explicit line
weight and line pattern by block, survives the writer/reader roundtrip.

Source appearance values that have no exact current representation cause the
affected entity to be skipped rather than approximated. Unsupported but
non-required auxiliary appearance metadata is diagnosed while the representable
appearance remains usable.

### Drawing scope and ownership

An entity belongs to the exported model space only when its `owner_handle`
equals `header.model_space_block_handle`.

- True model-space entities are considered for native export.
- Paper-space entities are skipped with a loss diagnostic.
- Block-definition content and insert semantics are skipped with loss
  diagnostics; blocks are not exploded.
- Missing or unknown ownership is a diagnosed skip when the entity can be safely
  isolated. Ownership ambiguity that prevents identifying the drawing is fatal.

### Entities

Only `EntityType::Line` and `EntityType::LwPolyline` are candidates for native
emission in this version. A supported entity type is still skipped as a whole if
one of its semantic properties cannot be represented exactly. No partial
geometry is emitted.

| Entity | Exact eligibility requirements | IFCDR result |
| --- | --- | --- |
| `LINE` | finite coordinates; start and end Z are zero; thickness is zero; normal is positive unit Z; layer and appearance are representable; no unsupported semantic attachments | one 2D line |
| `LWPOLYLINE` | finite coordinates; elevation and thickness are zero; normal is positive unit Z; every bulge is zero; every start/end width and constant width are zero; PLINEGEN is not set; layer and appearance are representable; no unsupported semantic attachments | one 2D polyline with source closure flag |

Legacy `POLYLINE` is not folded into `LWPOLYLINE`; it is an unsupported entity
type for the initial exporter. Curved bulges, widths, 3D placement, extrusion,
thickness, or other unsupported features skip the entire entity and are listed
together in that entity's diagnostic.

A standalone numeric handle or lightweight-polyline vertex ID is not diagnosed
unless an exposed semantic relationship depends on it. Semantic attachments
addressed by those IDs are diagnosed when not represented.

Entities are visited in cadcodec's stable stored drawing order. Emitted entities
receive sequential IFCDR IDs in that order. Skipped entities do not reserve IDs,
so there are no gaps introduced solely by loss. The returned mapping records the
final IDs for emitted handles only.

### Remaining CadDocument coverage

The implementation includes a versioned coverage matrix for every semantic
component exposed by the pinned cadcodec revision, not only the entity enum. Each
row records whether a component is:

- represented exactly;
- partially represented with a loss diagnostic;
- skipped with a loss diagnostic;
- nonsemantic serialization/bookkeeping data;
- fatal when inconsistent.

The matrix covers at least header semantics, layers and other tables, blocks and
layouts, entities, objects, XDATA and extension data, visual/material links,
dictionaries/reactors, and exposed document-level metadata. It is reviewed
against cadcodec #30 when that API becomes available. This audit is both
implementation input and regression documentation; adding a cadcodec variant
must not silently fall through to an unexamined default.

## Diagnostics

`ExportDiagnostic` is a separate `#[non_exhaustive]` typed enum rather than a
`PackageDiagnostic`. Its source locations are cadcodec concepts such as handles,
layers, tables, objects, or document fields, not IFCCAD resource URIs and JSON
paths.

Diagnostics are deterministic and actionable. Each exposes whether it describes
actual package-level loss, leaving room for future non-loss preservation notices.
The initial exporter emits only loss diagnostics.

Granularity is one diagnostic per affected source item, with all reasons for
that item bundled together:

- a polyline with bulge and width produces one skipped-entity diagnostic listing
  both properties;
- a layer with locking and material semantics produces one partially-exported
  layer diagnostic listing both properties;
- a wholly unsupported top-level collection can produce one summary diagnostic
  containing its kind and count.

Small values that help identify a problem may be included, but diagnostics do
not embed raw byte blobs or complete object dumps. There is no silent semantic
loss.

A lossless successful export returns an empty diagnostic list. A successful
`Allow` export may contain diagnostics, but the produced package itself must
still pass strict IFCCAD loading with zero package-validation diagnostics. Export
loss diagnostics describe the source-to-package conversion; package diagnostics
describe the validity of the produced package. They must not be conflated.

## Error model and stopping behavior

`ExportError` is `#[non_exhaustive]` and separates these categories:

- `InvalidSourceStructure`, containing every independently and safely detected
  fatal structural problem;
- `LossRejected`, containing the complete ordered loss diagnostic list and no
  partial package;
- invalid package/build input;
- target capacity or encoding failure;
- an internal invariant failure.

Fatal source information generally means an inconsistency or ambiguity in the
`CadDocument`. A coherent feature that current IFCCAD cannot yet represent is
normally a diagnosed skip, not fatal. A pure hard target constraint, such as an
unrepresentable range or capacity limit, remains a technical export error.

The scan does not stop at the first skipped item, even in `Reject` mode. It
returns the full safely detectable loss report. Structural preflight similarly
aggregates independent problems. Unexpected technical errors, capacity failures,
or violated internal invariants stop immediately because continuing may not be
safe; such an error is not relabeled as source loss.

Supported source values are classified before insertion into the builder. This
includes finite coordinate checks, minimum point counts, required names,
appearance validity, and other target preconditions. A subsequent ordinary
builder rejection therefore indicates defensive target validation, a hard
capacity/encoding constraint, or an internal mismatch—not a hidden source
diagnostic channel.

## Output and storage failures

Export and `PackageBuilder::finish()` are in-memory operations. They do not
accept a path and do not report filesystem errors.

Storage failures remain `PackageWriteError` values from
`EncodedPackage::write_directory`, including an existing target, invalid or
missing parent, permission denial, exhausted storage, or staging/rename failure.
The package value remains available for retry. Directory writing remains atomic
and does not overwrite an existing target.

A future ZIP encoder should introduce an encoding error at the logical-to-
physical boundary described by IFCCAD #7 rather than folding it into
`ExportError`.

## Determinism

Given the same `CadDocument`, `PackageOptions`, `ExportOptions`, and dependency
versions, export produces:

- the same diagnostic variants and order;
- the same layer, appearance, and entity ordering;
- the same resource and entity IDs;
- the same canonical package bytes.

Source order is retained where semantic order exists. Maps used only for lookup
or deduplication must not leak randomized iteration order into output.

## Test strategy

### Core writer tests

- Every appearance property independently supports by-layer, by-block, and
  explicit modes.
- Mixed modes survive strict writer-to-reader roundtrips.
- Identical explicit definitions and composite bindings deduplicate.
- Existing all-by-layer, all-by-block, and all-explicit behavior remains valid.

### Export classification tests

- Eligible lines and lightweight polylines emit exactly.
- Z coordinates, thickness, non-unit-Z normals, bulges, widths, PLINEGEN, and
  non-finite values cause whole-entity skips with bundled reasons.
- Paper-space, block-owned, unknown-owner, and unsupported entity types are
  diagnosed.
- Unit, layer, visibility, and mixed appearance mappings are exact for the
  supported subset.
- Empty representable layers are retained.
- Missing or unrepresentable layers skip dependent entities without synthesis.

### Policy and error tests

- `Allow` returns the supported package projection and all loss diagnostics.
- `Reject` reports the same complete loss list and returns no package.
- `Allow` is the default.
- Multiple independent source-structure problems are aggregated.
- Technical and builder failures are errors, not loss diagnostics.
- A lossless export returns zero export diagnostics.

### Controlled chain fixtures

The repository owns two small, hand-auditable ASCII DXF fixtures under
`crates/ifccad-convert/tests/fixtures/chains/`, with provenance documented in a
README alongside them. They are authored for this project rather than copied
from an arbitrary external drawing, avoiding licensing and reproducibility
ambiguity.

1. **`supported-model-space.dxf`.** This low-loss fixture exercises as much of
   the first exact subset as possible:
   model-space lines, open and closed straight lightweight polylines, declared
   units, multiple layers including an empty layer, visibility, and mixed
   supported appearances. The chain is `DXF -> CadDocument -> IFCCAD ->
   CadDocument -> DXF -> CadDocument`. Semantic projections are checked at the
   in-memory IFCCAD boundary and after cadcodec serializes and reloads the final
   DXF. Both equal the supported source projection, and export emits no loss
   diagnostics.
2. **`loss-heavy.dxf`.** This deliberately includes supported geometry
   alongside a circle/arc, a 3D or thick line, a bulged/width polyline, block
   content and an insert, paper-space content, and unsupported semantic
   metadata. Under `Allow` the supported projection is emitted and the exact
   expected diagnostic set is asserted, then the emitted projection completes
   the same serialized-DXF chain as the low-loss fixture. Under `Reject` the
   same complete loss set is returned with no package.
3. **`conformance/1.0.0/packages/valid/minimal-no-preservation`.** This stable
   IFCCAD conformance package is used for the IFCCAD-originated chain because it
   already contains both lines and open/closed polylines, two layers, explicit
   appearances, and model-space ordering without preservation data. The test
   imports it to a `CadDocument`, serializes and reloads an intermediate DXF,
   exports it back to IFCCAD, loads the result strictly, and compares the
   semantic drawing projection. Source handles and physical DXF/JSON byte layout
   are deliberately excluded from semantic equality.

Every produced package is also written to a temporary directory and loaded by
the strict package reader. It must produce zero package-validation diagnostics,
including when the export itself legitimately reports source loss under
`Allow`.

### User-review artifacts

The automated assertions may use temporary directories, but completing the
implementation also generates persistent, git-ignored review artifacts under
`target/chain-artifacts/`. These are deterministic outputs of the same chain
logic, not separate hand-edited examples:

| Chain | Start supplied for review | Intermediate supplied for review | End supplied for review |
| --- | --- | --- | --- |
| Supported model space | `crates/ifccad-convert/tests/fixtures/chains/supported-model-space.dxf` | `target/chain-artifacts/supported-model-space/ifccad/` | `target/chain-artifacts/supported-model-space/roundtrip.dxf` |
| Loss-heavy | `crates/ifccad-convert/tests/fixtures/chains/loss-heavy.dxf` | `target/chain-artifacts/loss-heavy/ifccad/` plus its export diagnostics | `target/chain-artifacts/loss-heavy/roundtrip.dxf` |
| IFCCAD-originated | `conformance/1.0.0/packages/valid/minimal-no-preservation/` | `target/chain-artifacts/minimal-no-preservation/roundtrip.dxf` | `target/chain-artifacts/minimal-no-preservation/ifccad/` |

After implementation and verification, the handoff gives the user clickable
absolute paths to each start and end, to both generated IFCCAD package
directories, and to every generated DXF. This lets the source and roundtripped
DXFs be opened and compared independently in OCS. The handoff also states the
expected intentional omissions in the loss-heavy end file so those omissions
are not mistaken for regressions.

### Regression and public API tests

- Repeated export with identical input produces identical bytes and diagnostics.
- The crate-root API compiles using only public imports.
- Workspace formatting, unit tests, integration tests, documentation tests, and
  clippy pass.

## Documentation impact

The implementation updates crate-level documentation and examples to show:

- the import/export terminology;
- construction of `PackageOptions` and default or rejecting `ExportOptions`;
- inspection of diagnostics and entity mappings;
- the separate `EncodedPackage::write_directory` storage step;
- the exact limits of the initial native export subset.

The coverage matrix is kept near the exporter and linked from its module-level
documentation so future cadcodec and IFCCAD changes have an explicit review
point.

## Future evolution

- **cadcodec #30:** replace the pinned manual semantic inventory with an
  exhaustive, compiler-visible inventory where cadcodec makes that possible.
- **IFCPR:** add preservation decisions at the existing classification seam;
  fully preserved data no longer counts as package loss.
- **IFCCAD #8:** rename stale `DrawingGeometryRepresentation` schema terminology
  before adding paper-space or multiple drawing scopes.
- **IFCCAD #7:** move physical JSON/ZIP encoding behind a codec boundary while
  retaining this conversion API.
- **Broader native geometry:** add new exact classifiers incrementally. A new
  entity type does not weaken the no-approximation rule unless a separately
  named approximation policy is designed later.
- **Multiple drawings/resources:** introduce caller-visible resource selection
  only when the export can genuinely produce more than one resource; do not
  expose an internal ID prematurely.

## Accepted decisions

- The initial default is permissive (`Allow`), with a strict mode available now.
- Loss is defined at whole-package semantic preservation level, not merely native
  IFCDR coverage.
- Numeric source handles are not semantic on their own.
- The exporter gathers all safely detectable loss before rejecting.
- Source inconsistency is fatal; coherent unsupported content is normally a
  skip.
- The builder has no diagnostic channel and remains the target validator.
- There is no duplicate full export-plan representation in the first version.
- The model layout name comes from the layout related to the model-space block
  record.
- The initial drawing resource ID is internal and deterministic.
- Controlled low-loss, high-loss, and IFCCAD-originated chains are required
  before the first exporter is considered complete.
