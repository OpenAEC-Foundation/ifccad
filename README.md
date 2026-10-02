# OCDraw and IFCX-CAD

**Website: [Open CAD Drawing explorer](https://ifccad-explorer.open-aec.com)**

Two experimental, open drawing formats developed in parallel within OpenAEC.
This repository keeps its historical IFCCAD name.

| Format | Direction | Contract and implementation |
| --- | --- | --- |
| **OCDraw** | A standalone, application-independent CAD drawing model, with direct control over storage and performance | [Contract](schemas/ocdraw), [core](src/ocdraw), [CAD conversion](crates/ocdraw-convert/README.md) |
| **IFCX-CAD** | A CAD drawing profile using IFCX structure and composition, exploring a place for drawings in the evolving IFCX ecosystem | [Experimental contract](schemas/ifcx-native-cad/experimental-contract-0.1.0.md), [core](src/ifcx_cad), [CAD conversion](crates/ifcx-cad-convert/README.md) |

Both routes develop typed drawing content, strict validation and explicit CAD
conversion diagnostics. Their models, schemas and encoding routes stay separate.
OCDraw opens independently of IFCX. Current capabilities differ; support in one
route does not establish support in the other. IFCX-CAD is a provisional profile,
not an official buildingSMART CAD standard.

An OCDraw drawing contains typed geometric entities, layers, layouts, entity appearance
choices, named simple line patterns, shared block definitions, and drawing-bound state. It opens without
IFCX. The JSON format marker is `open_cad_drawing`. The initial 0.1.0 contract
and entity schema v1 are provisional until contract completion and verification.

## Architecture

- `schemas/ocdraw`: language-neutral logical registry, logical rules, JSON
  mapping and document schema.
- `conformance/next/ocdraw`: candidate valid and invalid drawings.
- `src/ocdraw/logical`: typed model and shared semantic validation.
- `src/ocdraw/codec/json`: physical fields, column packing, decoding and encoding.
- `src/ocdraw/read.rs`, `build.rs`, `build/`: validated access and typed construction.
- `src/ifcx_cad` and `schemas/ifcx-native-cad`: separate IFCX-CAD model,
  source graph, validation, profile and encoding.
- Both cores separate `encode.rs` orchestration, `codec/` physical mapping and
  `storage.rs` file IO; see [API conventions and migration](docs/model-io-conventions.md).
- `crates/ocdraw-convert`: conversion between OCDraw and pinned opencadcodec
  `CadDocument`; DXF/DWG IO remains a CAD codec responsibility. Both converters
  use pinned upstream opencadcodec under its upstream name.
- `crates/ifcx-cad-convert`: the independent IFCX-CAD conversion route.
- `crates/ocdraw-viewer`, `crates/ocdraw-browser`: file inspection/conversion
  adapters. The inspector presents data and can embed Open CAD Studio to view
  original CAD and generated CAD through either route; see [website](format-explorer/README.md).

Reader and writer use shared logical validation. Generated outputs are loaded
through the production reader. CAD conversion consumes typed drawing values,
without JSON inspection. Core has no CAD runtime dependency.

`OcdrawDocument` is the complete mutable logical drawing. The builder can return
it with `build_document()`; `finish()` uses that document and the common encoder.
Reading exposes an immutable document snapshot, which can be consumed for edits.
`encode_ocdraw_document()` retains IDs, allocation watermarks and valid supplied bounds;
bounds recomputation is explicit. See the [document lifecycle](docs/ocdraw-document-lifecycle.md).

Current geometry includes XYZ lines, oriented points, placed circles/arcs and
ellipses, planar polylines with bulges, direct XYZ spatial polylines, and local
shared block instances. Layouts, appearances, plot state, named/current UCS,
model windows, paper canvases and paper viewports have typed records.
Named simple patterns retain their definitions, unused records, local references,
scale and polyline generation. Complex text/shape patterns become named continuous
patterns with loss evidence under Allow; Reject refuses this fallback.
Unsupported conversion semantics are diagnosed or rejected. Numerical accuracy
has a hard unit-aware tolerance, including nested block occurrences. See
[export coverage](crates/ocdraw-convert/docs/FROM-CAD-COVERAGE.md) and
[import coverage](crates/ocdraw-convert/docs/TO-CAD-COVERAGE.md) for limits.

## Parallel development

OCDraw and IFCX-CAD are active development tracks. CAD semantics are considered
together and implemented against each format's own contract. Reference drawings,
strict readback and comparable retained-content measurements inform both tracks.
Each route can expand at its own pace while keeping coverage differences visible.

OCDraw focuses on standalone CAD functionality and measured storage/performance
improvements. IFCX-CAD explores drawing integration with IFCX identity,
relationships and composition. Future IFC associations, incremental collaboration
and efficient IFCX storage require concrete designs and upstream coordination;
they are not current capability claims.

Compact, directly openable files are an adoption goal for both routes. Physical
encoding choices follow measured size, opening time and memory use. Binary storage
and compression per part remain options to investigate, not selected contracts.

## Development sequence

The typed model/JSON boundary, standalone parity and package retirement are
implemented. The complete document lifecycle supports direct logical conversion,
shared validation and identity-preserving core encoding. Scope lists now define
ownership and order; the JSON mapping uses
present stream names without a stream directory. Rust gates, candidate
conformance and the standalone primitive exchange experiment pass. Recording
0.1.0 and schema v1 as the first supported version remains a separate step.
[ROADMAP.md](ROADMAP.md) defines sequencing and exit criteria.

Preservation and external semantic links such as IFC associations are future
concrete designs; no generic extension protocol is standardized now.
IFCX-CAD is integrated on main as an independent experiment alongside OCDraw,
with separate models, schemas, validation and direct conversion routes.
The explorer opens either native format and converts DWG/DXF through the chosen
route. This integration does not freeze the experimental IFCX-CAD contract.
Old package IFCX/IFCDR/IFCPR readers, writers and active schemas are retired. Numbered conformance is immutable;
[historical design](docs/legacy-ifccad.md) and Git history retain its context.

## Using the implementation

```text
cargo run --example write_ocdraw -- drawing.ocdraw.json
cargo run -p ocdraw-viewer -- drawing drawing.ocdraw.json
cargo run -p ocdraw-convert --example minimal_to_dxf -- drawing.ocdraw.json drawing.dxf
```

New drawing files refuse to overwrite existing files. The crate and companion
README documents describe public APIs. Implementations can independently use
the published schemas and conformance materials.

## Verification

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Controlled measurements have separate reproduction instructions under
`docs/benchmarks`; historical IFCCAD ratios do not measure standalone OCDraw.
The repeatable [four-format experiment](benchmarks/size-exchange/README.md)
compares OCDraw, IFCX-CAD, DXF and DWG with identical retained content. See the
[synthetic/foundation report](docs/benchmarks/common-subset-size-exchange-v1.md)
and [large DXF report](docs/benchmarks/common-subset-test-dxf-v1.md).
Size measurements are run only on explicit user request, for the full experiment
or a requested selection. They are never automatic checks for a new main revision
or converter/encoding/dependency changes. Focused correctness tests still apply.
Licensed under [MPL-2.0](LICENSE).


## IFCX-CAD capabilities and resources

The independent IFCX-CAD proof is available in
`src/ifcx_cad`, with its [experimental contract](schemas/ifcx-native-cad/experimental-contract-0.1.0.md),
[sample](examples/ifcx-native-cad/hello-cad.ifcx) and
[evaluation](docs/experiments/ifcx-native-cad.md). Its direct
[converter](crates/ifcx-cad-convert/README.md) connects the supported subset to
opencadcodec `CadDocument`, including named simple line patterns, scales and polyline
pattern generation. See the [line-pattern sample](examples/ifcx-native-cad/hello-line-patterns.ifcx). Geometry validation and length-unit tokens reuse OCDraw;
the two drawing models, schemas and encoding routes remain separate. The
[Open CAD Drawing explorer](format-explorer/README.md) now opens IFCX-CAD and
roundtrips through DXF/DWG using its own converter. Paper layouts can be stored
natively; authored paperspace and viewport conversion remain deferred.

## Local maintenance

Run `pwsh -NoProfile -File scripts/cleanup_local.ps1` to audit local worktrees
and build caches. Removal requires an explicitly selected, unused worktree;
the script retains local changes and unfamiliar files for review.
