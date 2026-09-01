# ifccad-convert

`ifccad-convert` connects the validated, typed IFCCAD model from the core
`ifccad` crate to cadcodec's `CadDocument`. It is a companion crate: the core
format implementation remains usable without cadcodec.

The initial conversion is drawing-centric:

```rust,no_run
use ifccad::package::load_directory_package;
use ifccad_convert::convert_drawing;

# fn example(package_directory: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
let inspected = load_directory_package(package_directory)?;
let package = inspected
    .validated_package()
    .ok_or("the IFCCAD package is not strictly valid")?;
let drawing = package.drawings().next().ok_or("the package has no drawing")?;
let outcome = convert_drawing(drawing)?;

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
