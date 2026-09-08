# IFCCAD

Rust implementation and language-neutral format contract for the open IFCCAD
exchange format.

This repository develops the primary, recommended IFCCAD implementation.
Applications can build on its Rust crates directly or through language bindings
as those become available. The format itself remains independently
implementable from the published schemas, registries, and conformance material;
using this implementation is not a requirement for producing or consuming
IFCCAD.

## What is IFCCAD?

IFCCAD brings IFC-style project and building semantics together with CAD
drawings in an IFCX-based package architecture. A package combines:

- one **IFCX** document containing the semantic graph and resource references;
- one or more **IFCDR** resources containing drawing data; and
- optional **IFCPR** resources preserving source-format information that
  cannot yet be represented natively.

The IFCX graph can contain only a CAD drawing set, or a broader project,
building, and product model from which drawing resources are generated. IFCDR
resources may therefore be authored directly, imported from CAD, generated
from building elements, or retained as a cache.

IFCCAD sits between IFC semantics and CAD exchange. It is not the same as
support for conventional IFC files, though conventional IFC import and export
can become part of the wider workflow.

## Why IFCCAD?

CAD drawings remain essential project deliverables, but the dominant exchange
options leave a gap. DWG is compact and widely used but proprietary. DXF is
documented and accessible, but verbose and governed by a single vendor. IFC is
open and semantically rich, but a building model and a CAD drawing are different
artifacts.

CAD and BIM files are therefore often maintained side by side. When a drawing
is generated from BIM and exported to conventional CAD, relationships to the
products and semantics that produced it are commonly lost. At the same time,
many valid CAD workflows do not have or need an underlying BIM model.

IFCCAD addresses both cases: CAD-only drawings can stand on their own, while
BIM-aware drawings can retain optional links into a wider IFCX project graph.
IFCX carries meaning and relationships, IFCDR carries typed high-volume drawing
content, and IFCPR can preserve source information that is not yet represented
natively. Logical drawing semantics remain separate from JSON, compression,
chunking, or a future binary encoding.

The goal is an open, independently implementable and fidelity-aware drawing
model—not a clone of DWG and not a requirement to turn every drawing into BIM.
See the [IFCCAD vision](docs/vision.md) for the use cases, architectural model,
design principles, and long-term success criteria.

## Current status

The crate currently provides:

- language-neutral canonical value encoding and SHA-256 fingerprints;
- conformance manifests, vectors, fixtures, and verification helpers;
- versioned IFCX package-header, resource, and drawing-core overlays, the IFCDR
  registry, and the IFCPR schema;
- public directory-package loading with structured diagnostics and a strict,
  typed model for validated package metadata, drawings, layouts, layers,
  appearances, and IFCDR entities;
- a deterministic package builder and safe new-directory writer for one
  model-space drawing with layers, appearances, lines, and polylines; and
- the `ifccad-convert` companion crate for bidirectional conversion between a
  validated IFCCAD drawing and cadcodec `CadDocument`, including deterministic
  loss diagnostics and source-to-target entity mappings.

A complete IFCCAD vocabulary within IFCX, production IFCDR codecs, the future
`.ifccad` container, broader native CAD entity coverage and preservation, and
conventional IFC integration are still under development.

The active reader and writer use the encoding-neutral IFCDR **0.7.0** contract with
lines, straight polylines, and their supporting data. Older IFCDR versions and
other entity schemas do not produce a strict typed package; there is no legacy
migration path. Reader and writer retain separate storage behind shared typed
collection access and semantic validation; JSON encoding is a separate boundary.
IFCPR 0.2.0 retains its existing limited checks. See the
[candidate compatibility matrix](conformance/next/COMPATIBILITY.md) for operation
support and validation limits.

## Development roadmap

Development follows an incremental sequence:

1. **Working format foundation** — typed packages, validation, deterministic
   writing, and an initial bidirectional 2D CAD roundtrip are established.
2. **Stable logical drawing-resource model** — stabilize terminology,
   encoding-neutral boundaries, language-neutral rules and conformance,
   compatibility reporting, registry semantics, and resource identity, while
   beginning reproducible size measurements. Start from a reduced line/polyline
   contract with its supporting data; older IFCDR file compatibility is not
   required.
3. **Native CAD semantics and preservation** — expand geometry, layouts,
   entities, drawing relationships, and IFCPR-backed fidelity using a growing
   corpus of representative CAD workflows, including an initial CAD-BIM link.
4. **Scalable physical encodings and packaging** — evaluate chunking,
   compression, binary encodings, and a container using representative
   benchmarks.
5. **Interoperability and wider IFC workflows** — demonstrate practical
   exchange between applications using the primary implementation, then add
   independent validation for stable profiles and expand generated-drawing and
   IFC integration workflows.

The order expresses architectural dependencies rather than release dates.
Milestones may overlap, but later work should not become coupled to boundaries
that earlier work still needs to resolve. See [ROADMAP.md](ROADMAP.md) for the
authoritative status, scope, dependencies, exit criteria, and related issues.

## Using the current API

```toml
[dependencies]
ifccad = { git = "https://github.com/OpenAEC-Foundation/ifccad.git" }
```

```rust
use ifccad::package::load_directory_package;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let outcome = load_directory_package("project")?;

    for diagnostic in outcome.report().iter() {
        eprintln!("{}: {}", diagnostic.code, diagnostic.message);
    }

    let Some(package) = outcome.validated_package() else {
        return Ok(());
    };

    let header = package.header();
    println!(
        "package {} data version {} by {} at {}",
        header.package_id(),
        header.data_version(),
        header.author(),
        header.timestamp()
    );

    for drawing in package.drawings() {
        let entity_count = drawing
            .layouts()
            .map(|layout| {
                layout
                    .representation()
                    .resource()
                    .entities(layout.scope().id())
                    .count()
            })
            .sum::<usize>();
        println!(
            "{}: {} entities",
            drawing.path(),
            entity_count
        );
    }
    Ok(())
}
```

Opening a directory is intentionally separate from obtaining a strict model.
`PackageLoadOutcome` always retains diagnostics for an inspectable package;
`ValidatedPackage` is available only when the schema, graph, resources,
bindings, and supported IFCDR content satisfy the current contract. Its typed
views do not expose raw IFCX JSON or physical IFCDR stream columns.

`PackageHeaderRef` exposes the required package ID, `ifcxVersion`, mutable
`dataVersion`, author, and the original validated timestamp. Timestamps must be
valid RFC 3339 UTC values ending in `Z` or `+00:00`; their source spelling is
preserved by the reader.

### Writing a directory package

```rust
use ifccad::ifcdr::{IfcdrLengthUnit, Point2};
use ifccad::package::{
    AppearanceColor, AppearanceDefinition, DrawingOptions, EntityAppearance,
    LayerDefinition, LineDefinition, LinePatternDefinition, PackageBuilder,
    PackageOptions,
};
use ifccad::{PackageId, ResourceId};

fn write_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("building-a")?,
        data_version: "1".into(),
        author: "Example application".into(),
        timestamp: "2026-09-03T10:00:00Z".into(),
    })?;
    let mut drawing = package.add_drawing(DrawingOptions {
        model_layout_name: "Model".into(),
        representation_resource_id: ResourceId::new("geometry-main")?,
        length_unit: IfcdrLengthUnit::Millimetre,
    })?;
    let style = drawing.appearances().add(AppearanceDefinition {
        name: "Wall style".into(),
        color: AppearanceColor::rgb(255, 0, 0),
        opacity: 1.0,
        line_pattern: LinePatternDefinition::named("continuous"),
        line_weight: 0.25,
    })?;
    let walls = drawing.layers().add(LayerDefinition {
        name: "A-WALL".into(),
        visible: true,
        appearance: style,
    })?;
    drawing.model_space().add_line(LineDefinition {
        start: Point2::new(0.0, 0.0),
        end: Point2::new(1000.0, 0.0),
        layer: walls,
        appearance: EntityAppearance::ByLayer,
        visible: true,
    })?;

    package.finish()?.write_directory("building-a")?;
    Ok(())
}
```

The current writer deliberately requires exactly one Drawing with one model
layout and one external IFCDR model-space resource. `model_layout_name` names
that layout; it is not a drawing name. The declared length unit belongs to the
individual IFCDR resource. The writer supports layers, explicit appearances,
lines, polylines, visibility, and global entity order. It does not yet write paper
space, blocks, IFCPR, inline resources, or `.ifccad` containers, and it never
overwrites an existing target directory. Mapping a cadcodec `CadDocument` into
this builder is the responsibility of `ifccad-convert`. Its exporter currently
supports exact finite 2D lines and straight lightweight polylines, represents
mixed ByLayer/ByBlock/explicit appearance inheritance, and reports every
detected unsupported source semantic according to an allow-or-reject loss
policy.

To preserve existing identities, use `add_line_with_id` or
`add_polyline_with_id` with `EntityId::new(value)`. Automatic allocation continues
above the greatest assigned ID and does not recycle gaps. Encoding retains IDs
and draw order. `finish()` consumes the builder and returns an encoded package
only after resource and package validation; final errors are available together
in `PackageBuildError::Validation`.

The public API is still evolving while the format contract matures.

## Format contract and versioning

The active language-neutral schemas live in `schemas/`. The mutable
`conformance/next` collection currently targets suite `1.1.0` and tests the
minimal package-header contract alongside explicit resource identity: a
logical resource ID is independent of its external URI. IFCX overlay `0.7.0`
requires the top-level `header`, `imports`, and `data` fields and the known
header fields, while still allowing additional top-level and header fields and
unknown IFCX node types for forward-compatible extension. It remains a
development candidate until it is frozen as a numbered release. A numbered
directory such as `conformance/1.0.0` is an immutable, self-contained release
of fixtures, vectors, expected outcomes, and the schemas applicable to that
collection.

Active schemas may move ahead of the latest released conformance collection.
When a new collection is released, its applicable schemas are copied into the
numbered directory and frozen with the rest of that collection.

The [logical registry](schemas/ifcdr/registry-0.7.0.json),
[normative rules](schemas/ifcdr/logical-contract-0.7.0.md), and
[JSON mapping](schemas/ifcdr/json-mapping-0.7.0.json) define the drawing contract.
The [mapping language](schemas/ifcdr/json-mapping-v1.md) specifies the meaning
of its encoding forms and physical range rules.
Polylines require at least two vertices; repeated vertices and coincident line
endpoints are valid. Empty resources have no bounds; nonempty bounds must enclose
all geometry, including invisible entities, and may be conservative.

An initial compatibility matrix exists. Richer reporting remains planned to
distinguish three separate questions:
whether a package is valid, whether an implementation supports its content for
a stated operation, and whether that operation can transfer the relevant
meaning without loss. The conformance collections will grow an explicit matrix
covering readable and writable versions, understood extensions, and handling of
unknown content; see [ROADMAP.md](ROADMAP.md) for the staged work.

## Repository layout

- [`src`](src) contains the Rust implementation. `package` is the public
  package facade with private `read` and `write` implementations; `ifcdr`
  exposes shared drawing-resource types. Its private `logical` module owns
  semantic access and validation, `read` owns decoded storage and validated
  views, `write` owns resource preparation, and `codec/json` owns physical
  interpretation and encoding. `package/write` resolves package-specific
  references and supplies owned resource data to `ifcdr/write`.
- [`schemas`](schemas) contains the active language-neutral schemas.
- [`conformance`](conformance) contains versioned conformance collections.
- [`tests`](tests) verifies the public Rust API and bundled format assets.
- [`ROADMAP.md`](ROADMAP.md) defines development sequencing and milestone
  status; [`docs/vision.md`](docs/vision.md) describes the long-term purpose and
  design principles.

The [Python prototype](https://github.com/OpenAEC-Foundation/ifccad-prototype)
remains a model-first prototype and format laboratory.

This primary crate deliberately does not provide `CadDocument` or DWG/DXF I/O.
The workspace's [`ifccad-convert`](crates/ifccad-convert) companion crate owns
import from a validated IFCCAD drawing to the CAD runtime model and export from
`CadDocument` to an encoded IFCCAD package. Package encoding and safe directory
storage remain separate steps, keeping this format crate independent of CAD
codecs and geometry engines.

See [PROVENANCE.md](PROVENANCE.md) for the history of the clean repository
transition.

## License

MPL-2.0 — see [LICENSE](LICENSE).
