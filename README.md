# IFCCAD

IFCCAD is an experimental CAD drawing profile using IFCX. It describes drawings
as typed, application-independent information with explicit validation, ownership,
order and conversion evidence. IFCCAD is the active development track.
[OCDraw is parked](docs/ocdraw-parking.md) as a frozen source reference under
`legacy/ocdraw`, outside ordinary builds and tests. Feature parity is no longer a requirement.

## Architecture and capabilities

The root Rust package is `ifccad`; its public API is `ifccad::{IfccadDocument,
load_ifccad_bytes, encode_ifccad_document, ...}`. The model stays under `src/ifccad`.
The core owns construction, shared validation, encoding and file IO and has no CAD
runtime dependency. JSON/IFCX and experimental base-CBOR use the same logical
model. CBOR mapping revision 2 and optional native Zstd transport revision 1
remain experimental; this migration changes neither their bytes nor semantics.
The `ifccad-cbor-zstd` feature is optional; the default browser build excludes it.

Neutral geometry, color, text, dimension, line-pattern, plot and workspace kernels
remain in the core. The six `cad-*` companions own CAD preparation, numerical
proof, presentation, text, dimensions, workspace conversion and preservation.
`ifccad-convert` connects validated IFCCAD and opencadcodec `CadDocument` directly.
Conversion, logical construction, encoding and storage remain separate responsibilities.

Current support includes primitive and planar/spatial path geometry, local blocks,
Model/Paper layouts, appearance, simple and complex line patterns, Text/MText,
viewports, workspace state, Solid/LinePattern/Gradient Hatch and ordinary
linear/aligned Dimensions. Font contours, complete dimension display and evaluated
Hatch fill remain unassessed. The bounded opaque spline preservation route retains
source capture separately from qualified CAD restoration.

The [converter contract](crates/ifccad-convert/README.md) describes supported
semantics, loss policy, source/target restrictions and the explicit codec repair.
The [Explorer](format-explorer/README.md) inspects IFCCAD and converts DXF/DWG
through this route. Native files retain their original source bytes on download.
Legacy OCDraw files and explicit legacy selections return `OCDRAW_PARKED`.

## Development sequence

[ROADMAP.md](ROADMAP.md) is authoritative. The current order is:

1. Complete and integrate this IFCCAD-main-track migration after the accepted CBOR gate.
2. Resume Hatch B on the integrated basis, updating its scope to IFCCAD and reviewing
   any CBOR revision implications before contract changes.
3. Continue worked-out Dimension graphics, then remaining kinds/selectors and later
   joint annotation support according to their dependencies.
4. Expand semantic coverage and assess physical storage using representative evidence.

Hatch B stays paused during this migration. The xformop experiment stays separate;
combining or rebasing unrelated work is not part of migration execution. One broad
cross-cutting task runs at a time; independent narrow work can proceed with focused
checks and coordinated integration verification.

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

```text
cargo run -p viewer -- ifccad examples/ifccad/overview.ifcx
cargo run -p viewer -- export-ifccad examples/ifccad/overview.ifcx dxf
```

CAD input defaults to IFCCAD. `--drawing-format ifccad` remains accepted;
`--drawing-format ocdraw` reports `OCDRAW_PARKED` before reading the input.
See [model IO conventions](docs/model-io-conventions.md), the
[IFCCAD lifecycle](docs/experiments/ifccad-document-lifecycle.md),
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
Historical reports retain their original scope and checkpoint; they do not measure
this migration. Numbered conformance collections remain immutable.

## History and naming

IFCCAD was previously called IFCX-CAD. Project-owned APIs use IFCCAD; underlying
IFCX syntax, `.ifcx` extensions and `ifccad::` schema keys retain their meaning.
The earlier package architecture using IFCDR/IFCPR remains retired. Today's IFCCAD
has no OCDraw intermediate and does not revive that architecture. OCDraw can be
restored from its complete frozen checkpoint rather than maintained for parity.
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
