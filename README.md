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
- the initial IFCDR registry and IFCPR schema; and
- structured package diagnostics and internal directory-package foundations.

The public package loader, a complete IFCCAD vocabulary within IFCX,
production IFCDR codecs, the future `.ifccad` container, CAD conversion, and
conventional IFC integration are still under development.

## Using the current API

```toml
[dependencies]
ifccad = { git = "https://github.com/OpenAEC-Foundation/ifccad.git" }
```

```rust
use ifccad::canonicalization::{fingerprint, CanonicalValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = CanonicalValue::String("drawing-001".to_owned());
    let digest = fingerprint(&value)?;
    println!("{digest}");
    Ok(())
}
```

The API is still evolving. Canonicalisation, conformance support, and the
stable diagnostic vocabulary are public; directory-package loading remains
internal until its contract is mature.

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

The [Python prototype](https://github.com/OpenAEC-Foundation/IFC-CAD) remains
the model-first reference implementation and format laboratory.

This primary crate deliberately does not provide `CadDocument` or DWG/DXF I/O.
A future `ifccad-cad-document` companion crate will own conversion between a
loaded IFCCAD package and the CAD runtime model while keeping this format crate
independent of CAD codecs and geometry engines.

See [PROVENANCE.md](PROVENANCE.md) for the history of the clean repository
transition.

## License

MPL-2.0 — see [LICENSE](LICENSE).
