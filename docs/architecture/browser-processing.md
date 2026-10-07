# Drawing inspection in the browser

The IFCCAD & OCDraw Explorer accepts one OCDraw, IFCCAD, DXF or DWG file.
A browser worker calls the byte-oriented `browser` WASM exports. Files
stay on the device; the static website has no upload or native-processing API.
Worker cancellation terminates processing and stale results remain ignored.
All file paths/selections and 64 MiB limits are checked by the worker.

OCDraw and IFCCAD use separate production readers, typed models and companion
converters. The UI initially selects IFCCAD; the compatibility worker default
when `drawingFormat` is omitted remains OCDraw. `drawingFormat: "ifccad"` selects direct
IFCCAD conversion. Native IFCCAD has request kind `ifccad` and accepts `.ifcx` or
`.ifcx.json`. The request export format must match the selected native format
or be DXF/DWG; no cross-native projection is introduced. Original native download
bytes remain intact after strict validation, including IFCX foreign graph data.

The shared adapter only handles CAD format/version selection, physical IO,
size limits and downloads. Format-specific converters own semantic mapping and
loss diagnosis. IFCCAD uses LaterWins composition and its bundled experimental
schema import. Unknown graph information remains visible during inspection and
is diagnosed if omitted in a CAD projection. CAD-to-IFCX header provenance uses
the source name, explorer author and caller-supplied browser UTC timestamp.

CAD exports are decoded and version checked before download. IFCX additionally
requires conversion of the actual CAD readback to a production-readable IFCX
file. Structural/profile success does not establish exact geometric equality
through the external DXF/DWG codec. Conversion and readback diagnostics remain
separate and visible. A failed check exposes no download. Known nonzero DWG
BLOCK-base marker inconsistencies remain errors; there is no silent repair.

Open CAD Studio previews original CAD and generated CAD as separate documents.
Generated names identify the chosen `via-ocdraw` or `via-ifccad` route. Both use
one CAD version/format selection for preview and download. Native nodes are
inspected as information; the viewer receives only generated DXF/DWG bytes.
Hosted publication is independent of this local feature implementation.
