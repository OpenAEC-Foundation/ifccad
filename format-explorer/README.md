# CAD Format Explorer

Selecting an IFCCAD element in the tree also selects its mapped object in the
generated Open CAD Studio document. The link comes from conversion and is
qualified against the written CAD readback; IDs/handles remain strings. Selection
waits for existing preparation and never triggers an additional conversion.
Camera position and document edits are retained. The relevant CAD layout must
already be active. Shared block-definition contents and skipped/foreign items
have no direct selectable link; block instances are selectable. Unavailable
links produce a brief viewer notice. This is tree-to-viewer selection only.

The document/storage inspector, integrated IFCX navigation, overview and panel navigation
and adjustable conversion tolerance are specified in
[CAD Format Explorer interface design](../docs/architecture/format-explorer.md).

The application starts in **Overview / Overzicht** with **IFCCAD · Drawing
overview**: file contents on the left, Open CAD Studio on the right and both
conversion stages beneath them. Drag the separators or use their arrow keys to
adjust the split. Each pane's expand icon opens it across the workspace; the
restore icon replaces that button and returns to Overview. Tooltips and accessible
labels follow the selected language. Returning to Overview retains the tree
selection, settings and existing CAD session.

Native download uses an icon beside the contents heading, with the actual
IFCCAD/OCDraw format in its tooltip. CAD download is beside Output format and
Version, with the selected CAD format/version in its tooltip. Keyboard activation
and disabled/stale-result guards remain available.

Narrow screens stack the panels. Inspection starts in the **Bestandsboom / File tree**
view. Choose **Document structure** for logical contents and **Storage
structure** for IFCX node/source inspection or OCDraw streams/columns. Entities
are grouped by type within their layout or block by default; **Draw order**
shows the original sequence and each item retains its original draw position.
The native contents heading follows the selected IFCCAD/OCDraw result and shows
the source filename. Validation/readback stay in the report; redundant document
and footer status bars are omitted. Actionable errors or settings reminders
appear beside the file controls, and invalid contents remain explicitly labelled.
Composed IFCX node JSON in Source and JSON of selection uses `path`, optional
`children`, then `attributes`, followed by any remaining fields. Original source
fragments retain their stored order and values; full-file JSON retains its source text.
Group counts appear directly beside each group name. Opening another file or
selecting an example retains the active workspace view. A visible loading panel
shows the filename, operation and current phase, with cancellation available
while reading, fetching examples or converting.

**Conversion / checks** owns tolerance and output settings; CAD input exposes
its IFCCAD/OCDraw target next to the file picker before processing. Default,
exact and explicit physical/drawing-coordinate tolerances reach both converters
and generated CAD output/readback. The numerical evidence remains available
separately from semantic loss and strict native validity. The existing optional
OCDraw supported-SPLINE capture/restoration route is retained.
Its **Input conversion / Invoerconversie** tab shows source reading, CAD-to-native
diagnostics and native validation. **Output conversion / Uitvoerconversie**
shows export, source restoration and written CAD readback, with its own status
and diagnostic JSON. Both show their actual direction (for example DWG → OCDraw
and OCDraw → DWG); output identifies the written version. Opening a native file
explicitly reports that no input conversion was needed. An output failure does
not change input validity, and native downloads retain the existing input/output
reports. The nested tab survives file/example changes and preferences changes,
and supports arrow keys and Home/End. Output format/version and CAD actions are
on output in the full view; source capture is on input; tolerance and Apply settings remain shared.
Overview also exposes a compact settings strip with tolerance, CAD format/version,
Apply settings and CAD download, plus source capture when available. Hide/show
reports with the header chevron, or drag the row separator down to the settings
strip. Both reports disappear together while the settings remain available;
expanding Conversion shows its reports regardless of that overview preference.
For newly opened DWG/DXF files, CAD output automatically follows the original
format and the version reported by the CAD reader. The background roundtrip and
CAD download share these settings. Subsequent manual choices are retained when
applying settings. A source version outside the supported writer choices keeps
a supported output version and is explicitly identified beside the controls.

**Drawing** keeps the Open CAD Studio session across views. **Settings** offers
Dutch/English and light/dark appearance, remembered locally; changing them does
not reopen the source or reset the CAD session. **About** explains the formats,
local processing and source/licenses. The [representative example inventory](../examples/README.md)
describes the bundled demonstrations and their production-reader verification.

Generating CAD for the drawing viewer updates the same **Conversion / checks**
report as an explicit export. Reading/native validation, conversion messages,
tolerance evidence and CAD readback stay there; Drawing has no duplicate
diagnostic block. CAD viewer startup/session status remains beside the viewer.
After successful opening, CAD roundtrip output is prepared automatically in a
background worker using the applied CAD format/version and tolerance. Its
progress and cancel control are available in Conversion / checks, and its
readback/evidence is available without opening Drawing. Drawing shares an
ongoing job or reuses the prepared output; Open CAD Studio itself starts when
Drawing is visible in Overview or its own tab. Viewer startup does not block file
opening or native inspection. Opening a new source or changing settings invalidates
old pending output; applying settings starts preparation again.
Drawing shows a compact filename and expand/restore icon above Open CAD Studio;
its full view gives the remaining height to CAD. No separate browser-fullscreen
action is added to the explorer toolbar.

The current application opens one standalone OCDraw or experimental IFCCAD
file, or converts one DXF/DWG file to the selected drawing format. It displays
validated records and diagnostics and can download the selected native format,
DXF or DWG. Drawing preview embeds Open CAD Studio. The selectable routes support
parallel OCDraw and IFCCAD development, with their own models, validation and
coverage. Support in one route does not imply support in the other. The former
IFCX package explorer is retired.

All inspection and conversion runs in the browser through WebAssembly. Selected
files stay on the device; the website exposes no upload or native processing API.
The HTTP service only serves the website, WASM and viewer assets.

If the selected file cannot be read (for example, another application locks it),
the inspector reports the failure and clears the previous drawing selection.
Close the file in the other application and select it again before retrying.

A valid native result confirms the converted content meets that drawing contract;
it does not confirm lossless CAD conversion. Consult the conversion diagnostics:
unsupported entities can be skipped. The IFCCAD route retains named simple
patterns; complex text/shape patterns get a diagnosed whole-pattern fallback.

OCDraw Text/MText appears in draw order with links to text styles, literal content
and authored nested formatting. Layout/block inspection shows independently
derived bounds quality, the stored producer declaration and whether enclosure is
verified. Conversion reports distinguish text anchor coverage from unassessed
letter shapes/layout. Estimated/partial bounds never certify negative spatial
queries. The chooser includes a standalone Text/MText example; see
[text support](../docs/text.md) for native semantics and the narrower CAD profile.
IFCCAD text integration is deferred until the OCDraw slice has been tested on main.

Run the app: `npm start` in this directory. A matching wasm-bindgen build of
`browser` must be placed in `wasm-build/` for browser processing.
No hosted deployment is performed by development commands.

The dev server supports an alternate `PORT` and an optional `OCS_ROOT` for an
existing pinned viewer bundle. It stays on loopback and reloads when frontend
source changes. Rebuilding WASM or adding example assets requires a website
rebuild. This live development view is independent of hosted publication.
Website assembly compares file contents and writes only changed files. Existing
WASM, viewer runtime, fonts and bridge files retain their output files on an
unchanged build. Changed bundle files are synchronized and removed source files
are pruned; a missing converter/viewer bundle clears its stale output. The viewer
HTML is regenerated from its source with one bridge injection. Compilation of
Rust/WASM and Open CAD Studio remains a separate operation.

The production Rust-check cache includes repository `examples/` alongside Rust,
schemas and conformance inputs, so edits to the authored examples invalidate a
previous passing-check marker. The corrected cache key uses a new version;
converter and pinned Open CAD Studio caches remain independent.

Tests: `npm test`. File and CAD accuracy checks use the production Rust
reader/converter; the browser worker transport is tested independently.

For a DXF/DWG input on the OCDraw route, **Spline-brongegevens bewaren** explicitly
controls typed spline preservation. It is on by default for a newly selected CAD
file and can be switched off before opening; it is not an IFCCAD option.
Every interpreted spline variant can be stored; this does not add native
spline geometry or certified bounds. Capture, unavailable native geometry and
qualified restoration are reported separately. Current native layer/appearance/
visibility/order edits are authoritative. Unknown/raw/attached contexts may remain
stored without safe restoration. Native save/open preserves those bytes.

The CLI equivalent is `viewer cad FILE --preserve-splines` or
`viewer export-cad FILE FORMAT [VERSION] --preserve-splines`. Rust callers select
`OcdrawPreservationCapture::SupportedTyped`; existing entry points remain disabled
wrappers. Actual DXF/AC1032 DWG readback and the WASM smoke fixture cover the bounded
pilot; neither snapshots nor a successful native readback prove whole-file fidelity.

## Drawing preview

Choose **Tekening bekijken** after opening a file. Original DXF/DWG bytes open
as the original document; the strictly validated OCDraw conversion is exported
back to DXF/DWG as a distinct `via-ocdraw` or `via-ifccad` document. A native
OCDraw/IFCCAD file has only the converted document. One shared CAD format/version selection controls both preview regeneration and
CAD download. OCDraw download is a separate action next to the drawing contents.
Changing a selection regenerates a visible preview. The viewer uses the available
page width and can enter full screen; leaving full screen restores inspection. An edited generated document remains open when it is replaced;
no automatic save/discard is performed. Preview conversion diagnostics appear
in Conversion / checks alongside the original reading and validation sections.
The original can still be viewed if conversion fails.
Leaving the Drawing tab keeps its document session; opening a new source or explicitly
restarting the viewer replaces the session.
Compound inspector values expand into JSON beneath their label/count row, using
the full information-panel width. Their label, count and chevron align centrally
on the same summary row.

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

Build the browser processor from the repository root after preparing and
selecting the remaining [viewport-off codec repair](../patches/opencadcodec-viewports/README.md).
The deployment workflow uses the same pinned base and patches for its Rust
checks and browser processor; both cache keys include the patch recipe.

```text
wasm-pack build crates/browser --target web --out-dir ../../format-explorer/wasm-build --release
node format-explorer/tests/wasm-smoke.mjs
```

The smoke check exercises standalone fixtures and actual OCDraw/IFCCAD/DXF/DWG
output and production readback, including active/dormant DXF clip references
and viewport angles in radians. Deployment checks exercise the static HTTP service,
WASM and framed viewer assets. Linux deployment activation/rollback tests are
skipped on Windows.

Pull requests targeting main run the same Rust formatting, Clippy, workspace
tests, WASM readback smoke checks, website tests, asset assembly and local release
smoke checks as main. Unchanged converter and pinned Open CAD Studio bundles reuse
the existing build caches. The Rust verification cache includes repository-root
examples and the workflow recipe. The **CAD Format Explorer checks** status fails
when a required job fails, is cancelled or is skipped; it can be selected as a
required check in GitHub's branch rules. PR updates cancel only that PR's old run.

Only a push to main or a manual workflow run on main can deploy, after all checks
succeed. PR runs use read-only repository permissions and never enter the
production deployment job. Local development commands do not publish the website.

Deployment currently retains the existing Docker service: a checksummed archive
is tested before transfer, activated as one release, checked for the expected
revision, and rolled back on activation or service-check failure. OpenAEC's
[shared site deployment workflow](https://github.com/OpenAEC-Foundation/github/blob/main/.github/workflows/deploy-site.yml)
currently syncs static files directly to a web directory and can configure nginx.
The explorer retains its own deployment workflow and the existing release
activation and rollback behavior; migration to the shared workflow is outside
the current scope.

## IFCCAD workflow

Open `.ifcx` or `.ifcx.json` using the same file picker. The production IFCCAD
reader composes fragments with LaterWins and validates the experimental profile;
the contents show named patterns, layers, layouts, entities, blocks and the
composed graph, including foreign attributes/nodes. Its bundled profile schema
resolves the experimental import offline. A native IFCCAD download preserves the
original bytes and fragments, including foreign information.

For a roundtrip, download DXF/DWG, select that file and choose **IFCCAD** under
**DXF omzetten naar** or **DWG omzetten naar**, then open it and download IFCCAD.
This choice appears only for DXF/DWG input; native OCDraw/IFCCAD files determine
their own route. Changing the selection after opening reruns the
original CAD input with the chosen converter. IFCCAD uses `ifccad-convert`
directly, without OCDraw as an intermediate. The current supported CAD profile
is documented [here](../schemas/ifccad/experimental-contract-0.1.0.md);
this route does not claim arbitrary IFCX building-geometry rendering.

Allow conversion returns the supported subset and exposes all located loss
messages. CAD output is read back and version checked. The IFCX route additionally
converts that actual readback to IFCX and strict-reads it before exposing download
bytes. This is structural/profile readback, not a claim of exact semantic or
numeric equality through the external codec. Readback diagnostics are displayed.
The processor uses pinned opencadcodec under its upstream Rust name `opencadcodec`.
Nonzero BLOCK bases now pass both real DXF and DWG routes, including the browser
smoke fixture, without marker repair or block explosion. Paper CAD conversion
retains multiple sheets, separate coordinate units and expanded clips through
the documented codec repairs. Full plot conversion remains deferred. The
separately pinned Open CAD Studio viewer is unchanged.

A small fixture is included in built assets at
`examples/hello-line-patterns.ifcx`. Browser worker, UI and WASM smoke tests
exercise both native formats. Publishing remains a separate repository action.

Expanded IFCCAD geometry and supplied scope bounds appear in composed nodes.
Import reports expose `conversion.geometryAssessment`; CAD exports expose
`export.geometryAssessment` and independent readback evidence under
`export.fileCheck.geometryAssessment`. Domains show units, resolved limits,
counts and source/occurrence evidence. Identity fields in these records are
decimal strings to preserve uint64 values. Numerical failures expose a structured
`failure.geometry` reason and available limit/deviation. Each report qualifies
its own conversion boundary, rather than equality through an arbitrary codec.
The browser request applies the selected default, exact or explicit tolerance,
including a drawing-coordinate limit for unitless domains, to both conversion
routes. Native validation retains its own rules; no UI setting is stored in
native geometry. See [shared geometry](../docs/geometry/shared-geometry.md).
