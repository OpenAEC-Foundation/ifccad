# IFCCAD Explorer

The Explorer inspects IFCCAD and converts DXF/DWG directly through
`ifccad-convert`. IFCCAD is the only active native format. OCDraw files and
explicit old selections report `OCDRAW_PARKED`; see [parking](../docs/ocdraw-parking.md).
The download menu includes experimental CBOR and CBOR + Zstd using the
production writers. The browser explicitly enables Zstd.

## Using the Explorer

Start with Overview: IFCCAD contents, Open CAD Studio and conversion evidence
share resizable panes. Document structure groups entities by type within their
layout/block; Draw order preserves authored sequence. Storage structure shows
composed IFCX nodes and ordered original source fragments. Full-width identities
remain strings. Collections above ten items use a bounded searchable list.
Selection maps to the generated CAD document in both directions when qualified
readback links exist; block-definition contents and skipped items have no direct link.
Camera, workspace tab and prepared CAD output survive navigation.

Four curated IFCCAD examples cover overview, Text/MText, Hatches and Dimensions.
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

## Development and checks

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
available; it does not require source files in the deployed package. CI builds
only IFCCAD plus the active companions and has native and WASM Zstd checks.
