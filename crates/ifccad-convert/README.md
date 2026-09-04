# ifccad-convert

`ifccad-convert` connects the validated, typed IFCCAD model from the core
`ifccad` crate to cadcodec's `CadDocument`. It is a companion crate: the core
format implementation remains usable without cadcodec.

The initial conversion is drawing-centric:

```rust,no_run
use ifccad::package::load_directory_package;
use ifccad_convert::drawing_to_cad_document;

# fn example(package_directory: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
let inspected = load_directory_package(package_directory)?;
let package = inspected
    .validated_package()
    .ok_or("the IFCCAD package is not strictly valid")?;
let drawing = package.drawings().next().ok_or("the package has no drawing")?;
let outcome = drawing_to_cad_document(drawing)?;

let document = outcome.document();
for diagnostic in outcome.diagnostics() {
    eprintln!("{diagnostic}");
}
# let _ = document;
# Ok(())
# }
```

The crate re-exports its pinned cadcodec dependency as
`ifccad_convert::cadcodec`. Consumers should use this re-export for
`CadDocument`, handles, entities, and DXF/DWG writers to avoid mixing cadcodec
revisions.

The direction names describe the boundary around `CadDocument`:

```text
IFCCAD -- import --> CadDocument -- cadcodec DXF writer --> DXF
DXF -- cadcodec DXF reader --> CadDocument -- export --> IFCCAD
```

This crate currently implements only the IFCCAD import direction. The export
names are reserved for the inverse `CadDocument`-to-IFCCAD direction; no empty
export API is exposed yet. A direct IFCCAD-to-DXF or IFCCAD-to-DWG converter
can use file-format-oriented names and does not need to expose this internal
import/export terminology.

## Current scope

- exactly one model layout;
- planar lines and lightweight polylines (`z = 0`);
- IFCDR draw order and source-entity-to-target-handle mapping;
- IFCDR length units;
- layers, visibility, color (including named layer colors), line pattern, line
  weight, and ByLayer, ByBlock, or explicit opacity;
- structured, aggregated diagnostics for approximations and skipped content.

The converter accepts only a `DrawingRef` from a strictly validated package.
It does not load package paths or raw JSON and does not repeat package
validation.

Multiple layouts, paperspace, 3D geometry, other entity kinds, preservation
transfer, and conversion from `CadDocument` back to IFCCAD are deliberately
deferred.
