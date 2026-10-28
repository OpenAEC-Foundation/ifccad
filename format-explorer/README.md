# IFCCAD Explorer

Explorer inspects native IFCCAD and converts CAD through the same
`opencaddocument::Document` / IFCX mapping as Open CAD Studio. All drawing
processing stays in the browser. No drawing-upload API or automatic feedback
attachments exist. OCDraw remains parked.

## Inspection and preview

One validated opening owns the native snapshot. The inspector, IFCX download
and Studio preview use those same bytes. CAD input opens its original DWG/DXF
beside the generated `.ifcx`; native IFCX input opens its original bytes.
Studio internally imports IFCX into Document, so host rendering is distinct
from stored native content and core validation. Preview does not require a
DXF/DWG export. Changing CAD output format/version does not repeat CAD import.

Document structure, authored draw order, composed storage nodes, source
contributions, stored values, relationships and full-width string identities
remain available. The two curated examples are documented in
[examples](../examples/README.md). Dutch/English, pane navigation, bounded lists,
search, layer filters and settings remain local application state.

Selection uses explicit native-ID → CAD-handle mappings from the pinned IFCX
import, verified against that imported Document. This is **not** a physical-CAD
readback qualification. Both directions require the current generated document
and exact layout; shared block contents and unconverted entities are unmapped.
Dirty Studio original/generated tabs survive source/preview replacement. Camera
and session survive pane navigation. Restarting the viewer is an explicit action.
Cancellation and source changes invalidate late results and selection observations.

## Downloads and conversion boundary

IFCX/JSON keeps original native bytes, including foreign nodes and fragments.
CAD-generated IFCX keeps the exact validated generation. CBOR and CBOR + Zstd
use the core production codecs, with strict readback; they encode the validated
drawing projection. Same-encoding binary downloads retain validated source
bytes. Binary inspection and IFCX preview are labelled decoded projections,
never original IFCX fragments. Input and decompressed CBOR are capped at 64 MiB.
Compact storage remains CBOR + Zstd; Studio interchange is currently IFCX/JSON.

DXF/DWG export is optional. It uses the selected version, the shared converter,
actual CAD serialization/readback, and strict native readback of the returned
Document. Readback currently compares entity kinds/counts; it does not certify
all physical field values or complete semantic fidelity. Failure retains the
valid inspected native snapshot. No automatic version or tolerance substitution
occurs.

The migration is deliberately bounded. See the detailed
[migration coverage](../docs/architecture/explorer-document-route.md):

- Core native validation, source inspection and binary codecs remain available.
- Shared mapping diagnostics are retained verbatim; upstream losses are often
  aggregated without source locations. Reader notifications remain separate.
- The backend uses fixed parameter rounding. Certified numerical intervals and
  rendered appearance are **unassessed**. Old converter proofs are not borrowed.
- Adjustable/Exact tolerance, spline capture, partial recovery and
  Reject policy are unavailable. UI controls are hidden with an explanation;
  explicit API requests are rejected, never silently ignored.
- Unknown/unreported losses cannot be ruled out. The observed foundation-model
  dimension-text rendering discrepancy remains open host evidence. Counts of
  stored graphics nodes alone do not qualify correct rendering.

The archived `ifccad-convert` crate remains a reference for research and its own
regression suite. Explorer and viewer have no dependency on it. The old viewer
conversion APIs, codec helpers and route-specific tests have been retired; see
[retirement and recovery](../docs/architecture/retired-viewer-converter.md).
Native core validation, composed source inspection and binary codecs remain active.

## Development and verification

The merged typed API is pinned to Document
`47d7888ca6ae1a13b80c8cf5076f5efdbc5a5f2b` (upstream PR #4). The workspace-root
Cargo patch selects this checkout's core for the shared Rust model identity.
No local Document API patch or preparation script is needed.

```text
python scripts/check_explorer_dependencies.py
wasm-pack build crates/browser --target web --out-dir ../../format-explorer/wasm-build --release
node format-explorer/tests/wasm-smoke.mjs
npm --prefix format-explorer test
npm --prefix format-explorer run build
npm --prefix format-explorer start
```

The browser enables core Zstd and requires a WebAssembly-capable C compiler.
Use `PORT` for a separate loopback development port and `OCS_ROOT` for a pinned
self-hosted viewer bundle. `ocs-source.json` records Studio, Document and core
provenance; preserve Studio's license. A copied existing bundle must keep its
recorded byte hashes and explicitly record any HTML mount-path rewrite.
The workflow builds Studio independently at the pin and public URL.

Before completing Rust changes, run the repository full gate in AGENTS.md.
Frontend, WASM and packaged checks remain additional requirements. A development
server or startup smoke is not a completed migration or visual-fidelity claim.
Pushing main starts the verified production deployment workflow.

## Optional feedback

Feedback sends only explicitly submitted category, title, description and optional
reply email to Open Feedback Studio. The form warns that accepted reports become
public GitHub issues in OpenAEC-Foundation/ifccad. Drawings, names and screenshots
are never attached automatically. Successful responses show the returned issue
link; failures retain the draft. Deploy the corresponding service mapping and
spam protection before any separately authorized production publication.
