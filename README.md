# IFCCAD

**[Web page](https://open-aec.com/ifccad/) · [Open the Explorer](https://ifccad-explorer.open-aec.com/)**

IFCCAD is an experimental, open CAD drawing profile built on IFCX. It describes
geometry, layers, blocks, layouts and drawing styles as typed,
application-independent information. The Rust implementation validates drawings,
reads and writes standalone files, and exchanges supported content with DXF/DWG
while reporting conversion losses and numerical accuracy.

The profile and its encodings are still experimental, not a published standard.
See the [vision](docs/vision.md) for the long-term direction and the
[roadmap](ROADMAP.md) for development priorities.

Use the [documentation reading routes](docs/README.md) to find guides, contracts,
topic explanations and retained research evidence.

## Try it in your browser

The **[IFCCAD Explorer](https://ifccad-explorer.open-aec.com/)** lets you inspect
drawing structure and properties, view the drawing, and compare conversion
results. Open an IFCCAD, DXF or DWG file, or start with a built-in example.
Drawing files are processed locally in your browser.

| Example | What it shows |
| --- | --- |
| [Entity catalogue](examples/ifccad/entities.ifcx) | A numbered overview of all 16 current native entity families, Hatch fills, dimensions, blocks and drawing styles. |
| [House floor plan](examples/ifccad/house.ifcx) | A fictitious house with rooms, openings, reusable blocks, dimensions and an A3 sheet at 1:100. |

The catalogue is generated from an extensible sample registry. See
[example recipes](examples/README.md) to reproduce the drawings or add a sample,
and the [Explorer guide](format-explorer/README.md) for inspection and conversion
details.

## Architecture and capabilities

The root Rust package is `ifccad`; its public API is `ifccad::{IfccadDocument,
load_ifccad_bytes, encode_ifccad_document, ...}`. The model stays under `src/ifccad`.
The core owns construction, shared validation, encoding and file IO and has no CAD
runtime dependency. JSON/IFCX and experimental base-CBOR use the same logical
model. CBOR mapping revision 1 and optional native Zstd transport revision 1
remain experimental; this migration changes neither their bytes nor semantics.
The `ifccad-cbor-zstd` core feature is optional. The browser enables it for explicit
experimental CBOR + Zstd downloads; ordinary IFCX download remains the default choice.

Neutral geometry, color, text, dimension, line-pattern, plot and workspace kernels
remain in the core. The six `cad-*` companions own CAD preparation, numerical
proof, presentation, text, dimensions, workspace conversion and preservation.
`ifccad-convert` connects validated IFCCAD and opencadcodec `CadDocument` directly.
Conversion, logical construction, encoding and storage remain separate responsibilities.

Current support includes primitive and planar/spatial path geometry, local blocks,
Model/Paper layouts, appearance, simple and complex line patterns, Text/MText,
viewports, workspace state, native SOLID/TRACE, Solid/LinePattern/Gradient Hatch and ordinary
linear/aligned Dimensions. Font contours, complete dimension display and evaluated
Hatch fill remain unassessed. The bounded opaque spline preservation route retains
source capture separately from qualified CAD restoration.

The [converter contract](crates/ifccad-convert/README.md) describes supported
semantics, loss policy, source/target restrictions and the explicit codec repair.
The Explorer uses this same conversion route. For IFCX / JSON input, the matching
native download retains original source bytes. Experimental CBOR and CBOR + Zstd
downloads encode the validated IFCCAD drawing projection.

Development provisionally focuses on standalone CAD-profile documents in IFCX.
Editing CAD within a broader IFCX graph waits for a concrete application; existing
source retention and loss diagnostics remain unchanged. See the
[scope decision](docs/experiments/ifccad-document-lifecycle.md#provisional-standalone-cad-scope--2026-10-21).

## Development sequence

[ROADMAP.md](ROADMAP.md) is authoritative. The current order is:

1. The IFCCAD-main-track migration is integrated; OCDraw remains parked.
2. Hatch B adds classic editing/construction state in IFCCAD/IFCX and the mutable
   CBOR revision 1, with version-specific DXF/DWG qualification and Explorer disclosure.
3. The [native SOLID/TRACE precursor](docs/geometry/solid-trace-design.md) is implemented,
   retaining all finite local corner combinations and both CAD object types,
   with coordinate, surface and display guarantees kept separate.
4. Add bounded worked-out Dimension graphics using the currently supported
   zero-width polyline profile; native storage, CAD exchange and Explorer support are implemented and verified.
5. Add polyline widths as a separate follow-up, using the red sample_AC1032
   Dimension and its wide `_ArchTick` marker as an acceptance case.
6. Continue remaining Dimension kinds/selectors and later joint annotation support
   according to their dependencies.
7. Expand supported CAD semantics; assess whole-document storage/performance
   using representative evidence.

Hatch B is resumed on the integrated IFCCAD basis. The xformop experiment stays
separate. One broad cross-cutting task runs at a time; independent narrow work
can proceed with focused checks and coordinated integration verification.

The [October 2026 requirements review](docs/user-research-2026-10-review.md)
checks all 30 recommendations in issue #17 against current support. Within this
sequence, prioritize visible exchange/conformance evidence, construction
annotations, resource/library workflows and selected missing drawing semantics,
then qualified plotting and external drawing exchange. Large-file targets need
representative evidence; measurements remain explicitly requested work. The
[detailed priorities](ROADMAP.md#construction-workflow-priorities-from-the-october-2026-review)
separate core, converter, codec and host-application responsibilities.

CAD coverage and qualified exchange remain the priority. The next optional
storage investigation compares whole-document JSON, CBOR and native CBOR + Zstd
on agreed inputs; indexed chunks and columnar encoding await a demonstrated
need. See the [CBOR follow-up](docs/experiments/ifccad-cbor.md#next-investigation).
Measurements still require a separate explicit request.

## Repository layout

| Path | Responsibility |
| --- | --- |
| `src/ifccad`, `src/*_kernel`, `src/text`, `src/dimension` | Active model and neutral kernels |
| `crates/ifccad-convert`, `crates/cad-*` | CAD adapters and numerical evidence |
| `crates/viewer`, `crates/browser`, `format-explorer` | IFCCAD CLI, WASM and Explorer |
| `schemas/ifccad`, `conformance/next/ifccad` | Active provisional contract |
| `tests/fixtures/ifccad`, `examples/ifccad` | Active readback fixtures and curated examples |
| `legacy/ocdraw` | Frozen reference source, tests and reproduction metadata |
| `conformance/1.0.0`, `docs/benchmarks`, `benchmarks` | Immutable releases / retained historical evidence |

## Using the implementation

From a checkout with the documented codec configuration, inspect or export the
same example drawings with the CLI:

```text
cargo run -p viewer -- ifccad examples/ifccad/entities.ifcx
cargo run -p viewer -- ifccad examples/ifccad/house.ifcx
cargo run -p viewer -- export-ifccad examples/ifccad/house.ifcx dxf
```

CAD input defaults to IFCCAD. `--drawing-format ifccad` remains accepted;
`--drawing-format ocdraw` reports `OCDRAW_PARKED` before reading the input.
See the [IFCCAD lifecycle and API conventions](docs/experiments/ifccad-document-lifecycle.md),
[experimental contract](schemas/ifccad/experimental-contract-0.1.0.md) and
[CBOR experiment](docs/experiments/ifccad-cbor.md).

## Verification

```text
python scripts/check_active_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On a fresh checkout, select the documented codec configuration and run `cargo fetch`
once before the offline boundary audit. These commands cover only the active workspace, including its companions,
converter, viewer and browser. There is no active compatibility feature that
re-enables OCDraw. The boundary guard rejects parked packages, dependency paths
and active Rust references to parked fixtures.
Use focused checks during development, then one full gate for the final candidate
prepared against intended main. Reuse requires comparison of all verification
inputs, including ignored dependency patches/configuration, Cargo.lock, toolchain
and environment. Keep per-worktree caches and build settings stable; coordinate
expensive runs. See [verification rules](AGENTS.md#verification).

When changing native Zstd, also run `cargo test -p ifccad --features ifccad-cbor-zstd
--test ifccad_cbor_zstd` and the corresponding feature-enabled Clippy check.
Explorer changes require `npm --prefix format-explorer test`, the default WASM
build and `node format-explorer/tests/wasm-smoke.mjs` as applicable.
Measurements run only on explicit request and never as completion gates.
Historical reports retain their original scope and checkpoint; they do not
automatically describe later revisions. Numbered conformance collections remain immutable.

## History and naming

IFCCAD was previously called IFCX-CAD. Project-owned APIs use IFCCAD; underlying
IFCX syntax, `.ifcx` extensions and `ifccad::` schema keys retain their meaning.
The earlier package architecture using IFCDR/IFCPR remains retired. Today's IFCCAD
has no OCDraw intermediate and does not revive that architecture.
[OCDraw is parked](docs/ocdraw-parking.md) under `legacy/ocdraw`, outside ordinary
builds and tests. Its complete frozen checkpoint remains recoverable; feature
parity is not a development requirement. Legacy OCDraw files and explicit legacy
Explorer selections return `OCDRAW_PARKED`.
Licensed under [MPL-2.0](LICENSE).

## Local maintenance

Run `pwsh -NoProfile -File scripts/cleanup_local.ps1 -Sizes` to audit local
worktrees, separate codec repositories and their build caches. Sizes are
informational: there are no storage quotas or build limits. Cleanup follows
task completion, with active work and measurement evidence preserved.

Removal requires an explicitly selected, completed and unused worktree. Cache
cleanup defaults to main only; other checkouts must be selected individually.
See [local maintenance](docs/local-maintenance.md) for the completion checklist,
commands and protection rules.

For large CAD inputs, the [report-only conversion diagnosis](docs/conversion-diagnostics.md)
collects independent errors and blocked checks without pre-excluding entities.
