# IFCCAD Explorer

The Explorer inspects IFCCAD and converts DXF/DWG directly through
`ifccad-convert`. IFCCAD is the only active native format. OCDraw files and
explicit old selections report `OCDRAW_PARKED`; see [parking](../docs/ocdraw-parking.md).
The download menu includes experimental CBOR and CBOR + Zstd using the
production writers. The browser explicitly enables Zstd.

## Using the Explorer

The interface defaults to Automatic language selection: the first supported
Dutch or English browser preference, with English as fallback. Settings can
override this with Nederlands or English, or return to Automatic. Saved explicit
choices remain in effect; changing the theme does not pin an automatic language.
About links to the OpenAEC IFCCAD web page at https://open-aec.com/ifccad/.

Feedback opens a local dialog with category, message (10–5,000 characters) and an
optional reply email. It sends only these fields and the `IFCCAD` app identifier
to Open Feedback Studio's `/api/private-feedback` endpoint on its `pages.dev`
host. Drawings, file names and screenshots are never attached. A message is
cleared only after the service confirms private receipt; failed requests retain
the draft in the dialog. Drafts are not saved across page reloads.

Deploy the private endpoint in Open Feedback Studio before publishing this
Explorer integration. It must store IFCCAD feedback for authenticated dashboard
review without automatically creating GitHub issues, even if an app/repository
mapping is enabled. The client requires the `visibility: "private"`
acknowledgement and never falls back to the ordinary submission endpoint or
email. The separate public feedback page uses the same private API for every
selected tool; its deployment does not publish the Cloudflare API.
No mail delivery to an individual recipient
is configured by this integration. Existing service spam checks remain in force;
if Turnstile is enabled there, its client integration must be configured before
launch. No secret or admin key belongs in the Explorer bundle.

Start with Overview: IFCCAD contents, Open CAD Studio and conversion evidence
share resizable panes. Document structure groups entities by type within their
layout/block; Draw order preserves authored sequence. Storage structure shows
composed IFCX nodes and ordered original source fragments. Full-width identities
remain strings. Collections above ten items use a bounded searchable list.
Selection maps to the generated CAD document in both directions when qualified
readback links exist; block-definition contents and skipped items have no direct link.
Camera, workspace tab and prepared CAD output survive navigation.

Two curated IFCCAD examples provide an extensible entity catalogue and a fictitious
house floor plan with an A3 sheet at 1:100. See [example recipes and extension
instructions](../examples/README.md). Dimensions use native entities with compact
millimetre labels; the CAD host regenerates their display.
The native download button opens a popup with IFCX / JSON, experimental CBOR
(`.ifccad.cbor`) and experimental CBOR + Zstd (`.ifccad.cbor.zst`, level 3).
IFCX / JSON retains original IFCX bytes. Binary downloads encode the validated
IFCCAD drawing projection; foreign IFCX nodes/fragments are excluded, as noted
in the menu. Both production writers strictly read back and compare their output.
Opening `.cbor` and `.cbor.zst` / `.zst` selects the corresponding strict
production reader. Input and decompressed CBOR are capped at 64 MiB. The
inspector clearly labels decoded JSON/nodes; these are not original IFCX source
fragments. Binary-to-CAD output uses typed document conversion without an
invented IFCX source audit. JSON download reconstructs IFCX; same-encoding binary
download retains the original validated bytes. CAD download follows the shared
DXF/DWG and version selection. A native file requires no input conversion;
output failure retains input validity and native download. Input/output diagnostics,
numerical evidence, preservation and CAD readback remain separate.

Conversion / checks owns tolerance. Default is exactly 1e-9 per coordinate domain;
Exact is zero. Custom may specify drawing coordinates or physical mm/m, with an
independent coordinate fallback for unitless Model/Paper without a fixed output
mapping. Without an explicit fallback, unknown physical meaning is rejected.
Input and output limits can be selected together or independently and apply on
Apply; effective evidence records the domain and fallback provenance.

Supported typed spline preservation is on for newly selected CAD files and can
be disabled before opening. Capture, opaque native geometry and qualified fresh
CAD restoration have separate evidence. This does not add native spline geometry,
font geometry, evaluated Hatch fill or complete Dimension display.

Overview and the enlarged Contents, Conversion/checks and Drawing workspaces reuse
the same selection, applied settings and CAD session. Expand/restore changes the
pane arrangement without reopening the source. Splitters support pointer and
keyboard resizing. Desktop panes scroll internally; narrow screens stack the
main panes and retain accessible controls.

The contents pane has independently scrolling structure, item-list and inspector
columns. A singleton appears directly under its owner; collections of 2–10
members stay inline, and larger collections open the searchable middle list.
That list stays open until closed and can then show later small selections too.
Search matches identity, label, type, layer and short geometry description;
the layer filter intersects the search. Source order remains unchanged and
opening another source resets collection/filter state. Arrow keys, Home/End
and Page Up/Down select and reveal list items. Large ownership sequences remain
browsable in Draw order. There is no 150-item truncation or Show more step.

Contents columns initially share the available width; a resize makes widths
user-selected. The inspector can collapse and is automatically hidden below
850px pane width when a list is open, or below 520px in any contents mode,
unless explicitly overridden. Below 520px it opens over contents. Source
changes initially open the first property nesting level; deeper objects start
collapsed. Later explicit expansion/collapse persists across selection and
inspector-tab changes. Plain property labels do not change record selection;
typed references remain navigable. Selecting a navigation group of up to ten
members opens it inline, while layouts and block definitions keep their own
explicit expansion controls.

## Development and checks

The [browser and inspection architecture](../docs/architecture/browser-processing.md)
describes source ownership, composition, request cancellation, separate conversion
evidence and readback-qualified CAD selection.

On an entity-local CAD conversion failure the Explorer first shows the normal
error, then offers **Open partially / Gedeeltelijk openen**. Recovery is explicit
per source; selecting a new file restores ordinary import. The retry uses the same
tolerance settings. A persistent notice gives omitted-entity/error counts and a
complete list with source handles, owners, layers and reasons. The downloadable
JSON omission report includes every conversion diagnostic and original failure
evidence. Native and CAD exports use the partial drawing; skipped content is absent.
Shared structure and strict final validation still block output. Older WASM builds
without partial-opening support reject the request instead of silently ignoring it.

```text
wasm-pack build crates/browser --target web --out-dir ../../format-explorer/wasm-build --release
node format-explorer/tests/wasm-smoke.mjs
npm --prefix format-explorer test
npm --prefix format-explorer run build
npm --prefix format-explorer start
```

Run these from the repository root. The browser includes the Zstd feature and needs a WebAssembly-capable C compiler
(e.g. Clang) to build its bundled Zstd dependency. Native code uses the same explicitly selected
codec base/repair as [the converter](../crates/ifccad-convert/README.md).
The browser module exports `open_ifccad`, `convert_cad_to_ifccad`,
`export_ifccad`, option variants and `conversion_capabilities`; the worker defaults
CAD input to IFCCAD and rejects retired route choices before invoking WASM.

The dev server uses loopback, accepts `PORT` and optional `OCS_ROOT` for an existing
pinned viewer bundle, and reloads frontend source. Website assembly writes changed
assets only, prunes removed examples and clears stale missing bundles. WASM and
Open CAD Studio compilation remain separate; no deployment occurs here.

## Drawing preview and deployment

Open CAD Studio reads generated DXF/DWG bytes. Native IFCCAD files have a converted
preview; CAD source can also be opened as its original document. Preview preparation
runs in the background and supports cancellation. Editing CAD does not rewrite
IFCCAD source. The viewer bridge accepts only its same-origin parent/current token.

The pinned Open CAD Studio revision is recorded in `ocs-source.json` and the workflow. Preserve its license and self-hosted bundle. The deployment scripts under `deploy/`
cover release packaging, static smoke, rollback and public verification. Smoke uses
the shipped IFCCAD manifest and examples, including actual CAD readback where
available, using the Explorer's default CAD version so the catalogue's gradient
fills can be retained. The full packaged smoke regression runs with `npm test`
when both browser WASM and Open CAD Studio bundles are present; it does not
require source files in the deployed package. CI builds
only IFCCAD plus the active companions and has native and WASM Zstd checks.
