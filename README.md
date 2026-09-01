# IFCCAD

Rust implementation and language-neutral format contract for the open IFCCAD
exchange format.

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

## Current status

The crate currently provides:

- language-neutral canonical value encoding and SHA-256 fingerprints;
- conformance manifests, vectors, fixtures, and verification helpers;
- versioned IFCX resource and drawing-core overlays, the IFCDR registry, and
  the IFCPR schema;
- public directory-package loading with structured diagnostics and a strict,
  typed model for validated drawings, layouts, layers, appearances, and IFCDR
  entities; and
- the `ifccad-convert` companion crate for an initial validated IFCCAD drawing
  to cadcodec `CadDocument` conversion.

A complete IFCCAD vocabulary within IFCX, production IFCDR codecs, the future
`.ifccad` container, broader bidirectional CAD conversion, and conventional IFC
integration are still under development.

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
`ValidatedIfccadPackage` is available only when the schema, graph, resources,
bindings, and supported IFCDR content satisfy the current contract. Its typed
views do not expose raw IFCX JSON or physical IFCDR stream columns.

The public API is still evolving while the format contract matures.

## Format contract and versioning

The active language-neutral schemas live in `schemas/`. The `conformance/next`
directory, when present, tests the active contract. A numbered directory such
as `conformance/1.0.0` is an immutable, self-contained release of fixtures,
vectors, expected outcomes, and the schemas applicable to that collection.

Active schemas may move ahead of the latest released conformance collection.
When a new collection is released, its applicable schemas are copied into the
numbered directory and frozen with the rest of that collection.

## Repository layout

- [`src`](src) contains the Rust implementation.
- [`schemas`](schemas) contains the active language-neutral schemas.
- [`conformance`](conformance) contains versioned conformance collections.
- [`tests`](tests) verifies the public Rust API and bundled format assets.

The [Python prototype](https://github.com/OpenAEC-Foundation/ifccad-prototype) remains
the model-first reference implementation and format laboratory.

This primary crate deliberately does not provide `CadDocument` or DWG/DXF I/O.
The workspace's `ifccad-convert` companion crate owns conversion between a
validated IFCCAD package and the CAD runtime model while keeping this format
crate independent of CAD codecs and geometry engines.

See [PROVENANCE.md](PROVENANCE.md) for the history of the clean repository
transition.

## License

MPL-2.0 — see [LICENSE](LICENSE).
