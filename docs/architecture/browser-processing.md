# Browser processing for the Format Explorer

The Format Explorer uses the same Rust package reader, validator, converter and
CAD codec for its two processing routes. Browser processing is the default. The
user can explicitly choose the OpenAEC server when browser memory or performance
is insufficient. Repository examples still render without opening a user file.

`ifccad-browser` is a thin WebAssembly binding over byte-oriented functions in
`ifccad-viewer`. The core package loader accepts a map of relative package paths
and bytes, applies the same contract validation as directory loading, and does
not depend on cadcodec. The converter and CAD codec remain in their existing
companion crates. No browser-specific approximation of the format is introduced.

A dedicated Web Worker receives selected file bytes through transferable
`ArrayBuffer`s. It loads the local WASM bundle lazily and performs opening,
validation, conversion and export without HTTP upload. Cancelling a browser job
terminates the worker. The source bytes remain available in the page for another
export or CAD preview until the user chooses another source or closes the page.
The worker returns the bounded explorer presentation and diagnostics. IFCCAD ZIP
is a transport wrapper assembled in the worker from the validated source package;
the ZIP is not a new IFCCAD encoding. DXF/DWG export is read back through the
production CAD reader before a download is offered.

Browser and server routes share the UI and report shape. The server route is an
explicit selection and retains its authenticated job limits, cleanup and
timeouts. The browser enforces 1000 selected files and 64 MiB combined input;
actual peak memory is higher due to conversion and graph presentation. The
Open CAD Studio preview remains a separate, optional viewer of original CAD
bytes and generated DXF/DWG. It does not read IFCCAD directly.

Deployment builds the IFCCAD WASM bundle from this repository, tests package
opening plus DXF export and reimport in Node, and places the bundle at `/wasm/`.
The production smoke check verifies that it is served from the explorer origin
with the WASM MIME type. The pinned Open CAD Studio WASM bundle is built and
deployed separately. Each compiled output has its own source-keyed deployment
cache; website-only changes reuse the browser processor, native reader and pinned
Open CAD Studio bundle. The release is still assembled and smoke-tested on every
push to `main`.
