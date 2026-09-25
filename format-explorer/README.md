# IFCCAD Format Explorer

The explorer supports the current IFCDR 0.11 structure, including points,
planar and spatial polylines, circles, arcs, ellipses and ellipse arcs. It also
shows scopes, local blocks and paper-space viewports.
Model, paper and block-definition scopes have distinct labels. Layouts link to
their selected scope, while viewport entities link separately to their owning
paper scope and viewed model scope. Inline plot settings, viewport frames and
views, clipping references and relational layer overrides are inspectable.
Block instances still link separately to their owning `scopeId` and shared
`definitionScopeId`; definition metadata and placement/rotation/scale remain
inspectable without expanding CAD geometry.
The **One shared block, four placements** example is stored under
`examples/blocks-demo`. It deliberately remains an IFCDR 0.10.0 compatibility
example with rotation, mirroring and non-uniform scale.
The **A paper space with viewports** example uses the active conformance fixture,
including two viewports and their child override rows. This remains a structure
explorer, not a drawing renderer.

An independent educational demo website for exploring the **structure** of an
IFCCAD package. The graph is the main interface; the adjacent inspector explains
the selected object and exposes its fields and relationships. An optional drawing
preview embeds Open CAD Studio; it does not change the IFCCAD graph or add a
second CAD renderer. Local file opening delegates validation and CAD conversion
to the repository's production Rust libraries.

## Run locally

Requires Node.js 22 or newer. No dependency installation is needed.

For local package/DXF/DWG opening, first build the reader from the repository root:

```sh
cargo build -p ifccad-viewer
```

Rebuild it after changing Rust code. Examples also work without this executable.

For large CAD files, use an optimized reader. The debug reader can exceed the
local two-minute processing limit on files such as the 33 MB `test.dxf` practice
file. From the repository root, run `cargo build --release -p ifccad-viewer`.
Then set `IFCCAD_VIEWER_BIN` to the resulting `target/release/ifccad-viewer`
executable before starting the explorer. In PowerShell, after entering
`format-explorer`:

```powershell
$env:IFCCAD_VIEWER_BIN = (Resolve-Path ..\target\release\ifccad-viewer.exe).Path
npm start
```

```sh
cd format-explorer
npm start
```

Open the local URL printed by the server (default `http://127.0.0.1:4173`).
The development server binds only to loopback. Set `PORT` to choose another port.
Changes in `src/` rebuild automatically and reload the local page. Restart the
server after changing build scripts or source fixtures. Live reload is injected
only by the development server and is absent from the standalone build.

The drawing preview needs the pinned Open CAD Studio web bundle in the ignored
`format-explorer/ocs-build/` directory. The deployment workflow builds it from
release `v2026.38` at commit `0d023d267bc5b7afeca3b54e98875b0efd4f3926`
with Trunk and the `wasm-bindgen` version in upstream `Cargo.lock`, using
`--public-url /ocs/app/`. Copy the build's `dist/app/` contents and upstream
`LICENSE` into `ocs-build/` for local preview work. The graph, package opening,
validation and exports still run locally when those optional assets are absent;
the preview then offers the generated CAD download for manual opening.

```sh
npm test
npm run build
```

The build creates `dist/` and copies a locally available Open CAD Studio bundle
under `dist/ocs/app/`. The graph itself does not need Rust, cadcodec, an
application backend, external font services or a CDN. File opening and preview
exports do need the processing service. Production serves the viewer on the
explorer origin; no drawing bytes are sent to a separate viewer host. Opening
`index.html` directly with `file://` is not supported because examples are loaded
as a separate JSON resource.

## Explore

- Select a graph node to see its explanation, fields and incoming/outgoing links.
- Native entity nodes briefly explain their meaning, including the IFCDR 0.11
  point, curved-entity and two polyline families.
  Their collection nodes explain IFCDR stream storage; field nodes explain each
  value and its stored columns. Adding another registered native object family
  requires matching inspector explanations and translations.
- The graph discovers additional object streams from the IFCDR stream directory.
  Until a family has its own explorer explanation, the inspector shows its
  stored rows, fields and references as a structural preview, without assigning
  geometric meaning or claiming converter support. Shared x/y and x/y/z vertex
  pools are paged separately from their entity rows. This does not bypass the
  package reader's validation or add support for an unfinished format contract.
- Current IFCX packages list available Layers and Appearances as direct Drawing
  children. Their nodes follow Drawing expansion; a single drawing with more
  than 10 definitions uses an explicitly labelled display group for paging.
  The current examples all use this structure. Older IFCDR 0.9 / IFCX 0.11
  packages opened by the user can lack these lists; their definitions remain
  independent in the graph. The viewer does not invent missing IFCX links.
- Use the adjacent `+` / `−` controls to expand or collapse structure. Shared
  IFCX nodes have one identity even when several other nodes reference them.
  Expanding and collapsing preserve the current pan and zoom; use “Passend”
  explicitly when you want to include newly revealed branches in the overview.
- The graph starts in focus view: package hierarchy and IFCPR-to-drawing bridges
  stay visible, while layer, appearance, scope, block-definition and preservation
  references appear when an endpoint is selected. Up to eight selected references
  are drawn; the inspector pages through the loaded relations and indicates their
  count. “Alle relaties” restores every connection between visible nodes.
- IFCDR resources with more than 10 scopes or block definitions present their
  tables as paged display groups. These groups and their `item` links do not
  add nodes or relationships to the IFCCAD package. Each graph page shows up to
  10 members; selecting an item in the inspector reveals its page.
- Follow any inspector relationship to reveal and select the linked graph node.
- Drag the graph background to pan; use the wheel or buttons to zoom. “Passend”
  fits visible nodes; “Overzicht” restores the initial collapsed view.
- Graph nodes and expansion controls work with Enter and Space. Drag the divider
  to resize the inspector, use Left/Right (Shift for larger steps), or double-click
  to reset its width. On narrow screens, graph and inspector stack within the
  viewport. Only panel content scrolls; the page stays within the window.
- Expand an entity to inspect its typespecific fields as separate presentation
  nodes. Point lists expand to ordered points; placement expands to origin, X
  and Y. Vectors show their component values. The placement inspector compares
  local XY with evaluated XYZ using `P = O + x*X + y*Y` and distinguishes stored
  placement from the implicit XY default. Field nodes are not additional IFCX
  objects, entities or stored records. Existing nodes retain their positions
  while newly opened field branches find free space.
- Full node names and identifiers wrap using measured font widths. Relation
  captions reserve space outside nodes, expansion controls and other captions;
  they prefer their own connection near the destination, after sibling branches
  diverge. Wider column gaps and wrapped long captions keep them close to that
  connection. Displaced captions use a leader. Fit includes captions.
  Zoom in to read individual records when a large graph is fitted to the screen.
- Inspect IFCDR streams as aligned property columns, with links from entity IDs
  to entity records. Inspect IFCPR as source records, dependencies, projection
  bindings, attachments and byte ranges. The optional drawing preview is a
  separate view; it never replaces this structural explanation.
- Explore a paper layout's `scopeId`, effective inline plot settings, native
  `viewportStream` rows, `viewScopeId` back to model space, optional paper clip
  boundary, and `viewportLayerOverrideStream` child rows. The overview keeps
  reference edges focused to avoid covering the structure.

## Settings

The settings dialog follows the OpenAEC template with General, Appearance and
About tabs. Language choices are Nederlands, English and Automatic (browser
language, with English as fallback). Appearance offers light, dark and automatic
(system preference). Changes preview immediately; Save remembers them in this
browser, while Cancel, Escape and the close button restore the saved settings.
Selection, expanded branches and graph camera stay in place when preferences
change. Technical field names, identifiers, JSON and source bytes stay unchanged.
About includes application and contract versions, repository links and licenses.

“What works already, what is concept?” opens a modal with independently scrolling
content, a close button and Escape support. Closing restores focus to its button.
New reading/validation reports open in the full available work area. “Show graph”
restores the graph and inspector; “Expand report” temporarily hides them again.
Following a report's node link restores the graph and selects the target. Graph
state and inspector width are retained while switching views.

## Open your own files locally

Choose **Open file**, then an IFCCAD folder containing `package.ifcx.json`, or
one DXF/DWG file. Files are sent only to the loopback development service on
your computer. Selected inputs are staged temporarily; originals are unchanged.
The application removes staged/generated files after processing or cancellation.
It does not support a ZIP or finalized `.ifccad` container.

The package report keeps validity, completeness, supported strict loading,
diagnostics and assessment gaps separate. Only strictly loaded packages enter
the normal explorer. IFCPR can be explorable while its preservation semantics
remain **not fully assessed**. A failed opening leaves the previous graph visible
and says so explicitly.

DXF/DWG is read by the pinned cadcodec reader, converted with the existing
Allow policy, written temporarily and loaded through the production package
reader. The report lists emitted, partially emitted, skipped and unclassified
source entities, with source-to-target graph links. Partial entities are included
in emitted totals. Document/layer/table/object losses remain visible separately.
These counts describe the public model returned by the reader; they are not a
percentage of original-file fidelity. Unsupported CAD content is not automatically
preserved in IFCPR. Parser and conversion failures retain their own stage.

Initial viewer limits: 1000 files, 64 MiB combined input, one active job and 120
seconds processing time. Results expire after five minutes. Large entity, point,
layer, appearance and preservation collections show at most 10 members per graph
window. Paging replaces the visible members rather than continually adding nodes.
Large IFCX definition collections use explicitly labelled presentation groups;
these are not `LayerTable`/`AppearanceTable` nodes or new format relationships.
Inspector columns and lists share the graph's 10-item window and a single
previous/next and direct-page control. Entity columns share their index range
and horizontal scroll position; polyline x/y pools browse their own 10-value
range because pool indexes differ from entity rows. Compact cells visually
truncate long values; hovering shows the full scalar value without rounding.
Selecting an
item reveals its graph window and ancestors, preserving zoom. Source JSON
previews show up to 12000 characters and blob previews up to 4096 bytes.
These are display limits, not validation rules. Large
integer identities remain exact across the browser boundary; original JSON text
is retained separately from its presentation model.

The standalone static build keeps repository examples and settings. File opening
requires the processing service; it is not a browser-only Rust/WASM implementation.

## Preview in Open CAD Studio

Choose **View drawing** after opening a valid package or CAD file, or while
exploring a repository example. The viewer starts only when requested, in one
Open CAD Studio session inside the explorer. An imported DXF/DWG opens as
**Original** using the selected file's exact bytes; **Via IFCCAD** opens a new
DXF/DWG export from the strictly loaded package as a second drawing document.
An IFCCAD package or example without original CAD has only the generated
drawing. Open CAD Studio's own start page may also remain as a separate tab.

Select a drawing, intermediate format (DXF or DWG) and optional target version.
The default is 2018 (`AC1032`); supported choices are 2000, 2004, 2007, 2010,
2013 and 2018. Changing these replaces an unchanged generated document while
leaving the imported original in place. If you edited a generated document in
Open CAD Studio, that tab remains open and the next roundtrip opens beside it,
so the viewer never discards your work during a format switch. The generated
file and its diagnostics can be
downloaded. The preview reads the written DXF/DWG; it cannot open the in-process
Rust `CadDocument` or IFCCAD package directly. A visually similar result does
not prove that the original file was preserved without loss. The read/import
report and the export report remain distinct. If export fails, the original
document stays available where present.

## Export DXF / DWG / IFCCAD packages

Choose **Export** beside **Open file**, select a drawing and DXF or DWG, then
**Create export**. The dialog shows conversion diagnostics and a scoped fidelity
and geometry assessment before offering the download. It uses the existing
production `drawing_to_cad_document` converter and cadcodec writers with the
selected target version (2018 by default). The written file is reopened and
its version checked. This does not extend native format or converter coverage.

Export works for repository examples and successfully opened packages/CAD files.
Concept examples cannot be exported. Multi-drawing packages require selecting one
drawing; the converter currently requires exactly one model layout in that drawing.
IFCPR restoration and package-wide IFCX metadata transfer remain outside coverage.
An empty diagnostic list does not establish losslessness. The written CAD bytes
are read back before enabling download; this checks readability, not semantic
equivalence of every field. Conversion, writing and readback failures are reported
separately and offer no download.

The browser keeps the original selected input bytes in memory until another source
is opened or the page closes. For export it submits them again to the same processing
service; CAD inputs pass through native IFCCAD before producing the download.
This never downloads the original CAD file under an export label. The initial
opening report remains unchanged; export has its own report and includes the
preceding CAD-to-IFCCAD assessment when relevant. Example exports use exact fixture
bytes, preserving resource checksums and complete preservation blobs.

Exports use the same authenticated jobs, cancellation, timeouts, staging cleanup
and result expiration as opening. Downloads are limited to 64 MiB, transferred
inside the job result and made available as browser-local Blob URLs. Changing
source or export options discards the previous download. Public hosting
processes these inputs on the server; local development keeps processing on this
computer. No CAD files or uploads are retained in the repository.

**IFCCAD package (ZIP)** downloads the complete strictly readable directory
package, including all drawings, external resources and existing IFCPR blobs.
Drawing selection is hidden for this option. Existing package bytes are archived
unchanged, including their checksums; CAD sources are converted through the
production package writer and strict reader first, with their conversion losses
included in the report. Invalid packages produce no download.

The ZIP is a transport wrapper with `package.ifcx.json` at its root, not a new
`.ifccad` container contract or production codec. Extract it and choose the
resulting folder to reopen it. Entries use standard ZIP storage without additional
compression, with UTF-8 names and CRC checks. The same 64 MiB limit applies to the
finished archive. Full IFCPR semantic validation remains incomplete even though
existing preservation files are included byte for byte. The ZIP interoperability
test uses Python's independent standard-library reader when Python is available.

## Public hosting and automatic deployment

The [live Format Explorer](https://ifccad-explorer.open-aec.com/) runs on the
shared OpenAEC demo server, including package/DXF/DWG processing and exports.
Every push to `main` starts the **Deploy format explorer** GitHub Actions workflow.
It can also be started manually on `main` from the Actions tab.

Before deployment, the workflow checks the Rust workspace, builds the pinned
Open CAD Studio web bundle, and builds/tests the website and production reader.
The viewer bundle is cached by source revision and served from the same origin;
the production build fails if its WebAssembly is missing. The build job exercises
package opening, versioned DXF/DWG export and readback, IFCCAD ZIP download,
and the viewer asset path through the real HTTP service.
The Rust build uses Debian 12 to match the existing Node 22 production container.
The exact tested artifact is transferred using the shared `DEPLOY_SSH_KEY` secret
and `DEPLOY_HOST`, `DEPLOY_PORT`, `DEPLOY_USER` organization variables. The server
SSH host key is pinned in `deploy/known_hosts`; private keys stay in Actions secrets.

Releases live under `/opt/ifccad-explorer/releases/`, identified by commit and run.
The `current` symlink selects the release. A small `deployment.yml` Compose override
mounts it at `/app`; the original `compose.yml`, image, private network, non-root
user, resource limits, nginx and HTTPS configuration are preserved. The original
installation under `app/` is retained for recovery from the first deployment.
Only the explorer container is recreated, causing a brief interruption; active
file jobs may need to be retried. Deployments are serialized.

After switching, deployment verifies the running revision, repeats the opening
and export checks, and checks the public HTTPS revision. Failure restores the
previous mount and recreates the previous service. A failed workflow still needs
operator attention, especially if the server itself or rollback is unavailable.
The deployed commit is visible at [/version.json](https://ifccad-explorer.open-aec.com/version.json).
Old releases are retained; no automatic deletion affects recovery copies.

For a manual rollback, point `current` to the desired retained release, then run
`docker compose -p ifccad-explorer -f compose.yml -f deployment.yml up -d --no-deps --force-recreate explorer`
from the installation directory and verify `/version.json` and `/api/capabilities`.
To return to the original installation, use only `compose.yml` and remove the
managed `current` link and `deployment.yml` before the next automatic deployment.

Public hosting must include package/DXF/DWG processing, not only the static demo.
`npm run serve:production` provides the prepared application service: it serves
`dist/` without development rebuilding or live reload and binds to loopback on
port 4183 (override with `PORT`). Set `PUBLIC_ORIGIN` to the exact HTTPS origin
and `IFCCAD_VIEWER_BIN` to the production reader executable. Build the website
and reader before starting it. A reverse proxy must terminate HTTPS and preserve
the original Host header.

The service enforces the configured origin for uploads and cancellation, requires
a separate secret token for each job, and limits concurrent processing and retained
results. The browser explains that public uploads are processed on the server;
the local development service continues to process files on the user's computer.
Temporary inputs and outputs are removed after processing or cancellation.
Results expire after five minutes or are removed when the client finishes.

The existing container runs without root privileges, with a read-only filesystem,
bounded temporary storage, memory/CPU limits and no published ports. Nginx applies
upload and request limits and rate limiting. The container entry point listens on
its private network; the regular production entry point keeps its loopback binding.
The shared OpenAEC static-site workflow alone cannot update the Rust service.

For reproducible DXF/DWG smoke inputs (generated under ignored `target/`):

```sh
cargo run -p ifccad-viewer --example viewer_inputs -- target/viewer-inputs
```

## Examples and boundaries

`scripts/fixtures.mjs` curates nine examples. Eight are read without
changing their conformance source:

- `conformance/next/packages/valid/unrepresented-packed`
- `conformance/next/packages/valid/tilted-plane`
- `conformance/next/packages/valid/planar-bulges`
- `conformance/next/packages/valid/spatial-polyline`
- `conformance/next/packages/valid/ellipse-family`
- `conformance/next/packages/valid/shared-drawing-definitions`
- `conformance/next/packages/valid/layout-viewport-plot`
- `conformance/next/packages/valid/inline-both`

The ninth is `format-explorer/examples/blocks-demo`, a separate 0.10.0 demo
package with four block placements. The selected examples cover IFCPR source
records, placed and bulged planar polylines, spatial polylines, points and
curves, shared IFCX definitions, paper-space viewports and plot settings,
embedded resources, and block instances. They are all shown
with the current Drawing layer/appearance membership.

The additional **Concept · extra CAD-entiteittypen** example adds visibly marked
view model objects separately from the original fixture. Dimension remains
an illustrative future entity type, with its own proposed collection under
IFCDR; circle is native in 0.11. The example illustrates possible native data, a shared dimension style and
related source preservation. These
objects are not registered IFCDR streams, do not constitute a valid new package,
and must not be described as supported conversion or approved schema design.
Field names and graph relationships for concepts are illustrative. No schema or
conformance collection is changed by this demo.

The active native reference is IFCDR 0.11.0: XYZ lines, points, planar and
spatial polylines, circular and elliptical entities, local blocks and
paper-space viewports. IFCPR 0.2.0 schema and
fixtures exist, but production checks are limited;
full preservation validation and converter preservation transfer are not complete.
See `../conformance/next/COMPATIBILITY.md`, the converter coverage contracts and
`../ROADMAP.md`. Issue #2 discusses typed payloads as an option; it does not settle
the storage of future dimension variants.

The browser model is a structural presentation, not a replacement for the
production reader. Unknown IFCDR streams/versions follow that reader's strict
support boundary; permitted unrelated IFCX nodes receive a generic inspector.
Unresolved, unassessed preservation links are not invented as valid targets.

## Source layout

- `src/model.mjs`: read-only graph model, identities, folding and reveal.
- `src/fields.mjs`: expandable typed values and explicit implicit-placement display.
- `src/graph.mjs`: SVG graph, pan/zoom, pointer and keyboard selection.
- `src/layout.mjs`: text wrapping and collision-free caption placement.
- `src/inspector.mjs`: structural explanations, columns, fields and source bytes.
- `src/app.mjs`: loading, example selection and interaction coordination.
- `src/open-files.mjs`, `src/job-client.mjs`, `src/reports.mjs`: local opening and assessment UI.
- `src/export-files.mjs`: source retention, export dialog and browser downloads.
- `src/bundle.mjs`, `src/collections.mjs`: exact identity adaptation and lazy paging.
- `src/preferences.mjs`, `src/settings.mjs`: persisted preferences and settings dialog.
- `src/i18n.mjs`: Dutch/English presentation translations, separate from source data.
- `src/index.html`, `src/styles.css`: independent page shell and presentation.
- `scripts/`: fixture extraction, static build and loopback development server.
- `scripts/package-zip.mjs`: bounded ZIP download packaging with exact input bytes.
- `../crates/ifccad-viewer/`: application adapter for the production reader and converter.
- `tests/`: focused model and build-contract tests using the Node test runner.

Generated output and local review artifacts stay outside version control.

## Visual identity

The shell follows the palette and typography in the official
[OpenAEC style book](https://github.com/OpenAEC-Foundation/OpenAEC-style-book)
and its Tauri+React application template: Deep Forge, Blueprint White, Concrete,
amber accents, Space Grotesk, Inter and JetBrains Mono. Domain colors still
distinguish IFCX, IFCDR and IFCPR. This remains an independent web application.

Fonts and the unmodified OpenAEC symbol are bundled locally. Sources,
attribution and asset-specific licenses are included in `src/THIRD-PARTY.txt`
and copied into the standalone build, with a link under “Over deze demo”.
