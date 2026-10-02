# ocdraw-convert

`ocdraw-convert` connects standalone OCDraw to opencadcodec's `CadDocument` through
`cad_document_to_ocdraw_document` and `ocdraw_document_to_cad_document`.
The encoded `cad_document_to_encoded_ocdraw` and validated `ocdraw_source_to_cad_document`
entry points are convenience wrappers over logical conversion. These implementations
live in [`src/from_cad`](src/from_cad) and [`src/to_cad`](src/to_cad),
with native mapping helpers under [`src/mapping`](src/mapping). Pure CAD source classification lives in
[`src/source`](src/source), and numerical kernels in [`src/geometry`](src/geometry).
The core format implementation remains usable without opencadcodec.
The dependency and public Rust reexport use the upstream name `opencadcodec`
at a shared fixed revision with `ifcx-cad-convert`. The
[dependency audit](../../docs/geometry/opencadcodec-update-2026-10-02.md) records
the current public-model classification and exchange fixes.

## Standalone conversion

```rust,no_run
use ocdraw::ocdraw::load_ocdraw_bytes;
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, ocdraw_source_to_cad_document, CadToOcdrawOptions, OcdrawToCadOptions,
};
use ocdraw_convert::opencadcodec::CadDocument;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let source = CadDocument::new();
let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default())?;
let drawing = load_ocdraw_bytes(exported.encoded().bytes())?;
let imported = ocdraw_source_to_cad_document(&drawing, OcdrawToCadOptions::default())?;
for diagnostic in imported.diagnostics() {
    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
}
let cad_document = imported.into_document();
# let _ = cad_document;
# Ok(())
# }
```

The converter consumes typed `OcdrawDocument` content. Raw input is validated
before CAD construction; a reader snapshot already carries that guarantee. It does not inspect
JSON columns or construct JSON. Logical export returns a document and the same loss, mapping and accuracy
evidence without encoding. The encoded wrapper uses the core encoder and strict
production readback. Filesystem storage is a
separate application responsibility.

The standalone route supports modelspace, paperspace, shared local blocks,
lines, oriented points, placed circles/arcs/ellipses, placed planar polylines
including bulges, direct XYZ spatial polylines, signed block transforms, layers,
appearance choices, named simple line patterns and scales, layouts/plot settings, named UCS definitions, current UCS,
model windows, paper canvases and paper viewports. Source order is retained
across supported entity families. An open polyline's dormant final bulge is
stored without treating it as an active segment.

Complex text/shape line patterns retain their names, descriptions and local
references but become continuous under Allow, with one loss diagnostic per
definition. Reject refuses this fallback, including unused complex definitions.
Simple unused records, entity scales and polyline generation are retained.

Conversion diagnoses unsupported source properties and target limitations;
`Reject` rejects semantic loss. Numerical accuracy is a separate hard limit.
The default is one micrometre for known units and exact conversion for unitless
drawings. An explicit physical tolerance requires a known unit. Assessment
covers nested block occurrences as well as definition-local geometry; scale
can amplify local rounding. Outcomes expose `geometry_assessment()` and
source/target entity mappings.

See [standalone coverage](docs/COVERAGE.md) for scope and limitations, and
[block codec limits](../../docs/geometry/block-cad-boundary.md) for upstream
DXF/DWG restrictions. Conversion through the real pinned DXF and DWG readers
and writers is tested without dependency patches.

## Logical conversion without serialization

```rust
use ocdraw::ocdraw::{encode_ocdraw_document, validate_ocdraw_document};
use ocdraw_convert::{cad_document_to_ocdraw_document, ocdraw_document_to_cad_document,
    opencadcodec::CadDocument, CadToOcdrawOptions, OcdrawToCadOptions};
# fn example() -> Result<(), Box<dyn std::error::Error>> {
let exported = cad_document_to_ocdraw_document(&CadDocument::new(), CadToOcdrawOptions::default())?;
let drawing = exported.document();
validate_ocdraw_document(drawing)?;
let cad = ocdraw_document_to_cad_document(drawing, OcdrawToCadOptions::default())?;
let bytes = encode_ocdraw_document(drawing)?; // Only when native storage is wanted.
# let _ = (cad, bytes);
# Ok(())
# }
```

These are fresh conversions. CAD handles, OCDraw IDs and allocation history are
not roundtripped through `CadDocument`. A future CAD editor save route needs
explicit session context. See the [core lifecycle](../../docs/ocdraw-document-lifecycle.md).
