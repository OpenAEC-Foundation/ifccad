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
- Placement and derived values use the upstream finite-arithmetic mapping without
  automatic parameter quantization. Certified numerical intervals and rendered
  appearance remain **unassessed**. Old converter proofs are not borrowed.
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

Explorer pins official upstream Document
`3a3676b12e824292bcf46b29f4eaf02d9264c3a4` (including merged PR #8),
with codec `ef27529c868a81bac0872e9260f597a2c6f85665`. Studio uses the same
Document and codec pins with compatible native core `07e1ef1c4e975afcab0a07012bed8a82a56202cb`.
The workspace root shares its current core for Rust model identity. No local
Document patch is needed. Host source `3d2c1bf4` (OCS 2026.44) uses kernel `b2656bd` (matching codec ef27529c).
Both readers use profile 0.3.0 / CBOR3. Software release and file identifiers are
independent: the host core pin is a compatible source commit, not the release tag.
Studio grip/undo picture behavior is tracked separately in OCS #2325.
Newly decoded header settings outside the shared mapping are explicitly
reported as conversion losses when they differ from defaults.

Prepare the Studio checkout at the commit in `ocs-source.json` with
`python scripts/prepare_explorer_ocs.py <studio-checkout>` before building it.
This applies the main/save/worker/plugin dependency overrides and frozen
`ocs-inputs/Cargo.lock`. Build its main and parser-worker bundles, complete the
LICENSE/supporters assets, then record/verify the bundle with the same script's
`--record-build` / `--verify-build` options. CI performs these steps explicitly.
The cache key includes dependency, lock and build configuration; changing only
attribution JSON cannot qualify old binaries. The actual asset hashes and resolved
reader identities are retained in `IFCCAD-BUILD.json`. Explorer's WASM is a separate
build. To assemble a local candidate, provide the host bundle as `OCS_ROOT` to
the development server; the build rejects stale host assets.

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

## Initial extrusion release integration

Profile 0.3.0 / CBOR 3 uses official Document merge 28f1ad1, codec 0a5542b and
compatible host core 07e1ef1. The pins require no local source patch. Inspection
shows signed thickness and owning-scope direction separately from widths and
lineweight. Paper uses its separate numeric domain. The seven existing catalogue
objects have thickness 15 mm and base Z=0. The separate gallery is not shipped.
That initial host lacked the Circle/Arc display fix from OCS #2361; the current
upstream set below supersedes its application pins.

The host kernel pin 3a77b22 aligns its codec with Document. Compared with
faf2ee7, it also includes the upstream periodic-rim seam fix in B-rep meshing;
this dependency adoption does not claim additional native CAD coverage.

## Current upstream rendering fix

OCS source 3d2c1bf4 contains 4a637fdbe, the fix for circle/arc thickness
wires staying on the regular wire path instead of the base-only analytical GPU
path (OCS #2361). Document 3a3676b, codec ef27529c and kernel b2656bd are pinned
together; the native 2026.45.0 release remains unchanged. The kernel also adds
a bounded finest meshing retry and centerline selection/span corrections.



### Local annotation candidate

The annotation development worktree uses profile0.4.0/CBOR4. Its native/browser
candidate pins public Document and codec development revisions, with one native
model identity supplied by the workspace. It is labelled
`localDevelopment: annotation-contexts` in ocs-source.json and the processor's
capabilities; the official base revision alone does not identify these sources.
The separately accepted host's development build record includes its actual resolved local packages,
source/config/lock digests and asset hashes. This candidate is for the agreed
user review. That existing bundle predates the rebase to native main11390fb6;
coordinated host metadata/lock integration and a matching build remain required
before public deployment. Public development source pins do not qualify that build.
Annotation scale/view resources have their own navigation groups and typed
relations. Existing sample objects expose saved mode, default and contexts in
NL/EN, alongside the independent base values. Host display/editing and final
workspace gates remain separately qualified.
