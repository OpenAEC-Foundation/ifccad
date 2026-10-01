# ocdraw-convert

`ocdraw-convert` connects standalone OCDraw to cadcodec's `CadDocument` through
`cad_document_to_drawing` and `ocdraw_to_cad_document`. These implementations
live in [`src/ocdraw`](src/ocdraw). Pure CAD source classification lives in
[`src/source`](src/source), and numerical kernels in [`src/geometry`](src/geometry).
The core format implementation remains usable without cadcodec.

## Standalone conversion

```rust,no_run
use ocdraw::ocdraw::load_drawing_bytes;
use ocdraw_convert::{
    cad_document_to_drawing, ocdraw_to_cad_document, ExportOptions, ImportOptions,
};
use ocdraw_convert::cadcodec::CadDocument;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let source = CadDocument::new();
let exported = cad_document_to_drawing(&source, ExportOptions::default())?;
let inspected = load_drawing_bytes(exported.drawing().bytes());
let drawing = inspected.validated_drawing().ok_or("invalid OCDraw")?;
let imported = ocdraw_to_cad_document(drawing, ImportOptions::default())?;
for diagnostic in imported.diagnostics() {
    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
}
let cad_document = imported.into_document();
# let _ = cad_document;
# Ok(())
# }
```

The converter consumes the validated typed logical model. It does not inspect
JSON columns or construct JSON. The core writer encodes every output and loads
it through the production reader before returning it. Filesystem storage is a
separate application responsibility.

The standalone route supports modelspace, paperspace, shared local blocks,
lines, oriented points, placed circles/arcs/ellipses, placed planar polylines
including bulges, direct XYZ spatial polylines, signed block transforms, layers,
appearance choices, layouts/plot settings, named UCS definitions, current UCS,
model windows, paper canvases and paper viewports. Source order is retained
across supported entity families. An open polyline's dormant final bulge is
stored without treating it as an active segment.

Conversion diagnoses unsupported source properties and target limitations;
`Reject` rejects semantic loss. Numerical accuracy is a separate hard limit.
The default is one micrometre for known units and exact conversion for unitless
drawings. An explicit physical tolerance requires a known unit. Assessment
covers nested block occurrences as well as definition-local geometry; scale
can amplify local rounding. Outcomes expose `geometry_assessment()` and
source/target entity mappings.

See [standalone coverage](src/ocdraw/COVERAGE.md) for scope and limitations, and
[block codec limits](../../docs/geometry/block-cad-boundary.md) for upstream
DXF/DWG restrictions. Conversion through the real pinned DXF and DWG readers
and writers is tested without dependency patches.
