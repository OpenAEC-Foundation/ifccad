# OCDraw inspector

The current application opens one standalone OCDraw file or converts one
DXF/DWG file to OCDraw. It displays validated records and diagnostics and can
export OCDraw, DXF or DWG. Drawing preview embeds Open CAD Studio. The former
IFCX package explorer is retired on this branch.

All inspection and conversion runs in the browser through WebAssembly. Selected
files stay on the device; the website exposes no upload or native processing API.
The HTTP service only serves the website, WASM and viewer assets.

If the selected file cannot be read (for example, another application locks it),
the inspector reports the failure and clears the previous drawing selection.
Close the file in the other application and select it again before retrying.

A valid OCDraw result confirms the converted content meets the drawing contract;
it does not confirm lossless CAD conversion. Consult the conversion diagnostics:
unsupported entities and explicit non-Continuous line types can be skipped.

Run the app: `npm start` in this directory. A matching wasm-bindgen build of
`ocdraw-browser` must be placed in `wasm-build/` for browser processing.
No hosted deployment is performed by development commands.

Tests: `npm test`. File and CAD accuracy checks use the production Rust
reader/converter; the browser worker transport is tested independently.

## Drawing preview

Choose **Tekening bekijken** after opening a file. Original DXF/DWG bytes open
as the original document; the strictly validated OCDraw conversion is exported
back to DXF/DWG as a distinct `via-ocdraw` document. An OCDraw file has only the
converted document. One shared CAD format/version selection controls both preview regeneration and
CAD download. OCDraw download is a separate action next to the drawing contents.
Changing a selection regenerates a visible preview. The viewer uses the available
page width and can enter full screen; leaving full screen restores inspection. An edited generated document remains open when it is replaced;
no automatic save/discard is performed. Preview diagnostics are shown separately
from the opening report. The original can still be viewed if conversion fails.
Hiding the preview keeps its document session; opening a new source or explicitly
restarting the viewer replaces the session.

Open CAD Studio reads the generated CAD bytes, not OCDraw directly. Viewing a
similar drawing does not prove lossless conversion. Named simple line patterns,
local references and scales are inspectable and exported. Complex text/shape
patterns use a diagnosed named continuous fallback. Spatial pattern generation
has a known limitation in the pinned DWG codec; see the converter coverage.

The pinned viewer bundle must be in the ignored `ocs-build/` directory. Pin:
Open CAD Studio v2026.38, commit `0d023d267bc5b7afeca3b54e98875b0efd4f3926`.
Build the official source with Trunk, its Cargo.lock wasm-bindgen version and:

```text
trunk build --locked --release --public-url /ocs/app/ --dist dist/app --html-output index.html web-app.html
```

Copy `dist/app/` contents plus upstream `LICENSE` into `ocs-build/`. The workflow
also supplies an empty supporters.json to keep that source self-hosted. The site
build copies the bundle and injects the separate same-origin message bridge.
A missing bundle produces a visible message; inspection and download still work.
GPL-3.0 and source revision attribution accompany the bundle. The OpenAEC symbol
keeps its CC BY-SA license; see `src/THIRD-PARTY.txt`.

Build the browser processor from the repository root:

```text
wasm-pack build crates/ocdraw-browser --target web --out-dir ../../format-explorer/wasm-build --release
node format-explorer/tests/wasm-smoke.mjs
```

The smoke check exercises standalone fixtures and actual OCDraw/DXF/DWG output
and readback. Deployment checks exercise the static HTTP service,
WASM and framed viewer assets. Linux deployment activation/rollback tests are
skipped on Windows. Pushing to main triggers the existing deployment workflow;
local development commands do not publish the website.
