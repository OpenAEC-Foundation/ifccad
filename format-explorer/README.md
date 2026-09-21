# IFCCAD Format Explorer

An independent educational demo website for exploring the **structure** of an
IFCCAD package. The graph is the main interface; the adjacent inspector explains
the selected object and exposes its fields and relationships. This is not a CAD
drawing renderer or Open CAD Studio integration. Local file opening delegates
validation and CAD conversion to the repository's production Rust libraries.

## Run locally

Requires Node.js 22 or newer. No dependency installation is needed.

For local package/DXF/DWG opening, first build the reader from the repository root:

```sh
cargo build -p ifccad-viewer
```

Rebuild it after changing Rust code. Examples also work without this executable.

```sh
cd format-explorer
npm start
```

Open the local URL printed by the server (default `http://127.0.0.1:4173`).
The development server binds only to loopback. Set `PORT` to choose another port.
Changes in `src/` rebuild automatically and reload the local page. Restart the
server after changing build scripts or source fixtures. Live reload is injected
only by the development server and is absent from the standalone build.

```sh
npm test
npm run build
```

The build creates `dist/`, a standalone static website that can be served by any
static HTTP server, including under a subdirectory. It does not need Rust,
cadcodec, an application backend, external font services or a CDN. Building reads the
curated repository fixtures; the resulting website runs independently. Opening
`index.html` directly with `file://` is not supported because examples are loaded
as a separate JSON resource.

## Explore

- Select a graph node to see its explanation, fields and incoming/outgoing links.
- Layers and appearances are real IFCX nodes inside the common IFCX region.
  The viewer adds no artificial definitions node to their relationships.
- Use the adjacent `+` / `−` controls to expand or collapse structure. Shared
  IFCX nodes have one identity even when several other nodes reference them.
  Expanding and collapsing preserve the current pan and zoom; use “Passend”
  explicitly when you want to include newly revealed branches in the overview.
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
  bindings, attachments and byte ranges. There is deliberately no drawing canvas.

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

## Public hosting status

Publication at `https://ifccad-explorer.open-aec.com` is **paused**, pending the
administrator's DNS setup. The intended A record points to `167.235.54.105`, the
shared OpenAEC demo server. No live deployment has been performed.

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

Before publication, provision an unprivileged, isolated service with memory/CPU
limits, HTTPS proxy configuration, upload/request limits and rate limiting. The
shared OpenAEC static-site workflow alone does not install the Rust processing
service. Deployment automation, server installation and end-to-end public checks
remain to be completed when publication resumes.

For reproducible DXF/DWG smoke inputs (generated under ignored `target/`):

```sh
cargo run -p ifccad-viewer --example viewer_inputs -- target/viewer-inputs
```

## Examples and boundaries

`scripts/fixtures.mjs` reads four examples without changing them:

- `conformance/next/packages/valid/unrepresented-packed`
- `conformance/next/packages/valid/multi-drawing-projections`
- `conformance/next/packages/valid/inline-both`
- `conformance/next/packages/valid/tilted-plane`

The additional **Concept · extra CAD-entiteittypen** example adds visibly marked
view model objects separately from the original fixture. Circle and dimension
are two arbitrary examples of additional entity types, each with its own
independent collection under IFCDR. They are not a combined stream or an
exhaustive list of future families; final stream registration is still open.
The example illustrates possible native data, a shared dimension style and
related source preservation. These
objects are not registered IFCDR streams, do not constitute a valid new package,
and must not be described as supported conversion or approved schema design.
Field names and graph relationships for concepts are illustrative. No schema or
conformance collection is changed by this demo.

The active native reference is IFCDR 0.8.0: XYZ lines and placed straight
polylines. IFCPR 0.2.0 schema and fixtures exist, but production checks are limited;
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
- `src/bundle.mjs`, `src/collections.mjs`: exact identity adaptation and lazy paging.
- `src/preferences.mjs`, `src/settings.mjs`: persisted preferences and settings dialog.
- `src/i18n.mjs`: Dutch/English presentation translations, separate from source data.
- `src/index.html`, `src/styles.css`: independent page shell and presentation.
- `scripts/`: fixture extraction, static build and loopback development server.
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
