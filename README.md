# Open CAD Drawing (OCDraw)

An open, application-independent information model and exchange format for CAD
drawings. This experimental repository keeps the IFCCAD name and develops
standalone OCDraw.

A drawing contains typed geometric entities, layers, layouts, entity appearance
choices, shared block definitions, and drawing-bound state. It opens without
IFCX. The JSON format marker is `open_cad_drawing`. The initial 0.1.0 contract
and entity schema v1 are provisional until contract completion and verification.

## Architecture

- `schemas/ocdraw`: language-neutral logical registry, logical rules, JSON
  mapping and document schema.
- `conformance/next/ocdraw`: candidate valid and invalid drawings.
- `src/ocdraw/logical`: typed model and shared semantic validation.
- `src/ocdraw/codec/json`: physical fields, column packing, decoding and encoding.
- `src/ocdraw/read.rs`, `write.rs`, `write/`: validated access and typed construction.
- `crates/ocdraw-convert`: conversion between OCDraw and pinned cadcodec
  `CadDocument`; DXF/DWG IO remains a CAD codec responsibility.
- `crates/ocdraw-viewer`, `crates/ocdraw-browser`: file inspection/conversion
  adapters. The inspector presents data and can embed Open CAD Studio to view
  original CAD and generated CAD via OCDraw; see [website](format-explorer/README.md).

Reader and writer use shared logical validation. Generated outputs are loaded
through the production reader. CAD conversion consumes typed drawing values,
without JSON inspection. Core has no CAD runtime dependency.

Current geometry includes XYZ lines, oriented points, placed circles/arcs and
ellipses, planar polylines with bulges, direct XYZ spatial polylines, and local
shared block instances. Layouts, appearances, plot state, named/current UCS,
model windows, paper canvases and paper viewports have typed records.
Unsupported conversion semantics are diagnosed or rejected. Numerical accuracy
has a hard unit-aware tolerance, including nested block occurrences. See
[converter coverage](crates/ocdraw-convert/src/ocdraw/COVERAGE.md) for limits.

## Development sequence

The typed model/JSON boundary, standalone parity and package retirement are
implemented. Scope lists now define ownership and order; the JSON mapping uses
present stream names without a stream directory. Rust gates, candidate
conformance and the standalone primitive exchange experiment pass. Recording
0.1.0 and schema v1 as the first supported version remains a separate step.
[ROADMAP.md](ROADMAP.md) defines sequencing and exit criteria.

Preservation and external semantic links such as IFC associations are future
concrete designs; no generic extension protocol is standardized now. IFCX
integration is also planned as an independent CAD-native experiment on main
while that experiment runs. The next integration will bring the existing
experimental branch and its browser inspection route alongside OCDraw, with
clearly separate models and contracts. Old IFCX/IFCDR/IFCPR readers,
writers and active schemas are retired. Numbered conformance is immutable;
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
Licensed under [MPL-2.0](LICENSE).


## IFCX-native experiment and local maintenance

The independent IFCX-CAD proof is available in
`src/ifcx_cad`, with its [experimental contract](schemas/ifcx-native-cad/experimental-contract-0.1.0.md),
[sample](examples/ifcx-native-cad/hello-cad.ifcx) and
[evaluation](docs/experiments/ifcx-native-cad.md). Its direct
[converter](crates/ifcx-cad-convert/README.md) connects the supported subset to
cadcodec `CadDocument`. Geometry validation and length-unit tokens reuse OCDraw;
the two drawing models, schemas and encoding routes remain separate. Browser
support and integration onto main are follow-up work.

Run `pwsh -NoProfile -File scripts/cleanup_local.ps1` to audit local worktrees
and build caches. Removal requires an explicitly selected, unused worktree;
the script retains local changes and unfamiliar files for review.
