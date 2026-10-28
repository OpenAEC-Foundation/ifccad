# IFCCAD

**[Web page](https://open-aec.com/ifccad/) Â· [Open the Explorer](https://ifccad-explorer.open-aec.com/) Â· [Discussions](https://github.com/OpenAEC-Foundation/ifccad/discussions)**

IFCCAD is an experimental, open CAD drawing profile built on IFCX. It describes
geometry, layers, blocks, layouts and drawing styles as typed,
application-independent information. The Rust implementation validates drawings,
reads and writes standalone files, and exchanges supported content with DXF/DWG
with conversion messages and explicit coverage limits.

The profile and its encodings are still experimental, not a published standard.
During alpha, later releases may be unable to open earlier IFCCAD files;
backward compatibility and migration are not guaranteed. See the
[alpha compatibility policy](docs/alpha-compatibility.md) and
[release/version policy](docs/releases.md). The first native source alpha is
[2026.44.0](https://github.com/OpenAEC-Foundation/ifccad/releases/tag/v2026.44.0).
Its attached artifacts contain only the native core. Explorer and CAD conversion
are outside that release.
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
model. The first alpha publishes profile 0.1.0, CBOR mapping revision 1 and
optional native Zstd transport revision 1. Those tagged contracts are fixed;
subsequent normative changes follow the release policy.
The `ifccad-cbor-zstd` core feature is optional. The browser enables it for explicit
experimental CBOR + Zstd downloads; ordinary IFCX download remains the default choice.

Neutral geometry, color, text, dimension, line-pattern, plot and workspace kernels
remain in the core. Active DWG/DXF exchange belongs to the shared
`opencaddocument` mapping used by Explorer and the viewer CLI. The older
`ifccad-convert` and its six CAD adapters are [archived](docs/converter-parking.md)
with reproducible source, tests and contracts, outside ordinary workspace builds
and tests. They are not dependencies of the active viewer/browser route.
Conversion, logical construction, encoding and storage remain separate responsibilities.

Current native support includes primitive and planar/spatial path geometry, local blocks,
Model/Paper layouts, appearance, simple and complex line patterns, Text/MText,
viewports, workspace state, native SOLID/TRACE, Solid/LinePattern/Gradient Hatch and ordinary
linear/aligned Dimensions, optional stored Dimension graphics and three-mode planar
polyline widths. Hatch classic editing and boundary-construction records are
implemented. Font contours, complete dimension display and evaluated Hatch fill
remain unassessed. Native opaque spline storage does not imply that the active
shared converter captures or restores splines.

The [active exchange coverage](docs/architecture/explorer-document-route.md)
describes the shared mapping, exact dependency pins and evidence limits. Preview
opens the actual IFCX snapshot in Open CAD Studio; DWG/DXF regeneration is optional.
Native validation, mapping losses, numerical fidelity, physical CAD readback and
host rendering are separate claims. Historical converter guarantees do not carry
over automatically; see the [reference contract](legacy/ifccad-convert/reference/crates/ifccad-convert/README.md).
For IFCX / JSON input, the matching
native download retains original source bytes. Experimental CBOR and CBOR + Zstd
downloads encode the validated IFCCAD drawing projection.

Development provisionally focuses on standalone CAD-profile documents in IFCX.
Editing CAD within a broader IFCX graph waits for a concrete application; existing
source retention and loss diagnostics remain unchanged. See the
[scope decision](docs/experiments/ifccad-document-lifecycle.md#provisional-standalone-cad-scope--2026-10-21).

## Development sequence

[ROADMAP.md](ROADMAP.md) is authoritative; reviewed on **28 October 2026**.
The native alpha and shared Explorer/CLI route are published. Hatch editing,
SOLID/TRACE, Dimension graphics and planar polyline widths are implemented native
foundations. The next steps are:

1. **Consolidate the shared conversion route.** Reference-converter retirement
   is complete. Adopt merged placement/numerical fixes through explicit dependency
   updates, and qualify the actual IFCX preview and representative CAD cases.
   Upstream fixes are not automatically present in deployed dependency pins.
2. **Extend native Dimensions with the seven remaining kinds.** Extend the
   [current Dimension model](docs/dimension.md) as one task with
   internal stages, with native readback and conversion contributions to
   `opencaddocument` in both directions. The native/Document candidate and matching local Explorer are implemented;
   the user accepted CAD and sample_AC1032 Explorer review. The grouped entity
   catalogue includes all nine kinds and native source relationships. Integration/
   release remain separate; see the [checkpoint](docs/dimension/checkpoint-2026-10-28.md).
3. **Add shared annotation contexts and bounded association updates.** Establish
   scale contexts and their exchange before claiming viewport-dependent updates
   or general regeneration.
4. **Expand resources and selected drawing semantics.** Define font/symbol
   resolution boundaries, then bounded attributes, multileaders and other selected
   families; native splines require their own contract.
5. **Qualify plotting and external drawing workflows.** Build on resources,
   annotations, color/plot-style semantics and rendering before claiming faithful
   PDF output, xref workflows or dynamic-block retention.

Feature/version/reader evidence and understandable loss reports accompany this
sequence. The [October requirements review](docs/user-research-2026-10-review.md)
is input to these priorities, not a current-route certification. Compact storage
and large-file performance remain evidence-driven follow-ups; indexed chunks,
columnar encoding and the separate xformop experiment are not adopted production
requirements. Measurements require a separate explicit request.

## Repository layout

| Path | Responsibility |
| --- | --- |
| `src/ifccad`, `src/*_kernel`, `src/text`, `src/dimension` | Active model and neutral kernels |
| `legacy/ifccad-convert` | Historical direct converter, adapters and complete-checkpoint reproduction |
| `crates/viewer`, `crates/browser`, `format-explorer` | IFCCAD CLI, WASM and Explorer |
| `schemas/ifccad`, `conformance/next/ifccad` | Active alpha contract and evolving conformance |
| `tests/fixtures/ifccad`, `examples/ifccad` | Active readback fixtures and curated examples |
| `legacy/ocdraw` | Frozen reference source, tests and reproduction metadata |
| `conformance/1.0.0`, `docs/benchmarks`, `benchmarks` | Immutable releases / retained historical evidence |

## Using the implementation

From a checkout with the pinned shared dependencies, inspect or export the same
example drawings with the CLI:

```text
cargo run -p viewer -- ifccad examples/ifccad/entities.ifcx
cargo run -p viewer -- ifccad examples/ifccad/house.ifcx
cargo run -p viewer -- export-ifccad examples/ifccad/house.ifcx dxf
```

CAD input defaults to IFCCAD. `--drawing-format ifccad` remains accepted;
`--drawing-format ocdraw` reports `OCDRAW_PARKED` before reading the input.
See the [IFCCAD lifecycle and API conventions](docs/experiments/ifccad-document-lifecycle.md),
[experimental contract](schemas/ifccad/experimental-contract-0.2.0.md) and
[CBOR experiment](docs/experiments/ifccad-cbor.md).

## Verification

```text
python scripts/check_active_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On a fresh checkout, run `cargo fetch --locked` once before the offline boundary
audit. These commands cover only the active core, viewer and browser; no reference
codec patch/configuration is needed. Archived converter/adapters are excluded;
historical reproduction uses its recorded complete checkpoint. The Explorer dependency guard can be
run with `python scripts/check_explorer_dependencies.py`. OCDraw remains excluded;
the boundary guard rejects parked packages, dependency paths and active Rust
references to parked fixtures.
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

The [reference diagnosis report](docs/conversion-diagnostics.md) records the old
route's capabilities. Use the [active exchange coverage](docs/architecture/explorer-document-route.md)
for the current Explorer/CLI boundary.
