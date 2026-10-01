# Open CAD Drawing explorer

The current application opens one standalone OCDraw or experimental IFCX-CAD
file, or converts one DXF/DWG file to the selected drawing format. It displays
validated records and diagnostics and can download the selected native format,
DXF or DWG. Drawing preview embeds Open CAD Studio. The former
IFCX package explorer is retired on this branch.

All inspection and conversion runs in the browser through WebAssembly. Selected
files stay on the device; the website exposes no upload or native processing API.
The HTTP service only serves the website, WASM and viewer assets.

If the selected file cannot be read (for example, another application locks it),
the inspector reports the failure and clears the previous drawing selection.
Close the file in the other application and select it again before retrying.

A valid native result confirms the converted content meets that drawing contract;
it does not confirm lossless CAD conversion. Consult the conversion diagnostics:
unsupported entities can be skipped. The IFCX-CAD route retains named simple
patterns; complex text/shape patterns get a diagnosed whole-pattern fallback.

Run the app: `npm start` in this directory. A matching wasm-bindgen build of
`ocdraw-browser` must be placed in `wasm-build/` for browser processing.
No hosted deployment is performed by development commands.

Tests: `npm test`. File and CAD accuracy checks use the production Rust
reader/converter; the browser worker transport is tested independently.

## Drawing preview

Choose **Tekening bekijken** after opening a file. Original DXF/DWG bytes open
as the original document; the strictly validated OCDraw conversion is exported
back to DXF/DWG as a distinct `via-ocdraw` or `via-ifcx` document. A native
OCDraw/IFCX-CAD file has only the converted document. One shared CAD format/version selection controls both preview regeneration and
CAD download. OCDraw download is a separate action next to the drawing contents.
Changing a selection regenerates a visible preview. The viewer uses the available
page width and can enter full screen; leaving full screen restores inspection. An edited generated document remains open when it is replaced;
no automatic save/discard is performed. Preview diagnostics are shown separately
from the opening report. The original can still be viewed if conversion fails.
Hiding the preview keeps its document session; opening a new source or explicitly
restarting the viewer replaces the session.

Open CAD Studio reads the generated CAD bytes, not OCDraw or IFCX directly. Viewing a
similar drawing does not prove lossless conversion. Named simple line patterns,
local references and scales are inspectable and exported. Complex text/shape
patterns use a diagnosed named continuous fallback. Spatial pattern generation
has a known limitation in the pinned DWG codec; see the converter coverage.
The two converters retain their own coverage contracts and diagnostics.

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

The smoke check exercises standalone fixtures and actual OCDraw/IFCX-CAD/DXF/DWG
output and production readback. Deployment checks exercise the static HTTP service,
WASM and framed viewer assets. Linux deployment activation/rollback tests are
skipped on Windows. Pushing to main triggers the existing deployment workflow;
local development commands do not publish the website.

## IFCX-CAD workflow

Open `.ifcx` or `.ifcx.json` using the same file picker. The production IFCX-CAD
reader composes fragments with LaterWins and validates the experimental profile;
the contents show named patterns, layers, layouts, entities, blocks and the
composed graph, including foreign attributes/nodes. Its bundled profile schema
resolves the experimental import offline. A native IFCX download preserves the
original bytes and fragments, including foreign information.

For a roundtrip, download DXF/DWG, select that file and choose **IFCX-CAD** under
**DXF omzetten naar** or **DWG omzetten naar**, then open it and download IFCX-CAD.
This choice appears only for DXF/DWG input; native OCDraw/IFCX files determine
their own route. Changing the selection after opening reruns the
original CAD input with the chosen converter. IFCX uses `ifcx-cad-convert`
directly, without OCDraw as an intermediate. The current supported CAD profile
is documented [here](../schemas/ifcx-native-cad/experimental-contract-0.1.0.md);
this route does not claim arbitrary IFCX building-geometry rendering.

Allow conversion returns the supported subset and exposes all located loss
messages. CAD output is read back and version checked. The IFCX route additionally
converts that actual readback to IFCX and strict-reads it before exposing download
bytes. This is structural/profile readback, not a claim of exact semantic or
numeric equality through the external codec. Readback diagnostics are displayed.
The pinned DWG codec's nonzero BLOCK-base marker conflict blocks IFCX downloads
of that generated DWG; zero-base blocks and DXF are covered by tests. No repair
or block explosion is applied. Paper CAD conversion remains deferred.

A small fixture is included in built assets at
`examples/hello-line-patterns.ifcx`. Browser worker, UI and WASM smoke tests
exercise both native formats. Publishing remains a separate repository action.
