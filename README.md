# IFCCAD and OCDraw

**Website: [IFCCAD & OCDraw Explorer](https://ifccad-explorer.open-aec.com)**

OpenAEC develops two experimental, open CAD drawing formats in parallel.
IFCCAD is a CAD drawing profile built on IFCX structure and composition.
OCDraw is a standalone, application-independent drawing model and exchange
format, with direct control over storage and performance.

| Format | Direction | Contract and implementation |
| --- | --- | --- |
| **OCDraw** | A standalone, application-independent CAD drawing model, with direct control over storage and performance | [Contract](schemas/ocdraw), [core](src/ocdraw), [CAD conversion](crates/ocdraw-convert/README.md) |
| **IFCCAD** | A CAD drawing profile using IFCX structure and composition, exploring a place for drawings in the evolving IFCX ecosystem | [Experimental contract](schemas/ifccad/experimental-contract-0.1.0.md), [core](src/ifccad), [CAD conversion](crates/ifccad-convert/README.md) |

Both routes develop typed drawing content, strict validation and explicit CAD
conversion diagnostics. Their models, schemas and encoding routes stay separate.
OCDraw opens independently of IFCX. Current capabilities differ; support in one
route does not establish support in the other. IFCCAD is a provisional profile,
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
- `src/ifccad` and `schemas/ifccad`: separate IFCCAD model,
  source graph, validation, profile and encoding.
- `src/geometry_kernel`: shared primitive geometry, placement, validation,
  bounds and analytic clip calculations, independent of CAD runtimes.
- Both cores separate `encode.rs` orchestration, `codec/` physical mapping and
  `storage.rs` file IO; see [API conventions and migration](docs/model-io-conventions.md).
- `crates/ocdraw-convert`: conversion between OCDraw and pinned opencadcodec
  `CadDocument`; DXF/DWG IO remains a CAD codec responsibility. Both converters
  use pinned upstream opencadcodec under its upstream name.
- `crates/ifccad-convert`: the independent IFCCAD conversion route.
- `crates/cad-geometry-convert`: shared CAD geometric preparation, tolerance
  resolution and primitive/nested occurrence accuracy proofs for both routes.
- `crates/viewer`, `crates/browser`: file inspection/conversion
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
Active viewport clips support circles, full ellipses and closed planar
polylines with straight or bulged segments. Direct CAD conversion resolves clip
references independently of draw order, preserving activation separately from
stored boundaries. The new upstream pin includes clipping and DXF angle repairs;
the [remaining local codec patch](patches/opencadcodec-viewports/README.md)
retains the independent viewport-off bit. The DWG profile includes a paper canvas.
See the [dependency audit](docs/geometry/opencadcodec-update-2026-10-07.md) and the
converter coverage contracts for limits.
Named simple patterns retain their definitions, unused records, local references,
scale and polyline generation. Complex text/shape patterns become named continuous
patterns with loss evidence under Allow; Reject refuses this fallback.
Unsupported conversion semantics are diagnosed or rejected. Numerical accuracy
has a hard unit-aware tolerance, including nested block occurrences. See
[export coverage](crates/ocdraw-convert/docs/FROM-CAD-COVERAGE.md) and
[import coverage](crates/ocdraw-convert/docs/TO-CAD-COVERAGE.md) for limits.

## Parallel development

OCDraw and IFCCAD are active development tracks. CAD semantics are considered
together and implemented against each format's own contract. Reference drawings,
strict readback and comparable retained-content measurements inform both tracks.
Each route can expand at its own pace while keeping coverage differences visible.

OCDraw focuses on standalone CAD functionality and measured storage/performance
improvements. IFCCAD explores drawing integration with IFCX identity,
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

OCDraw's first bounded preservation slice stores complete typed spline snapshots
with editable native common properties, ordinary ownership/order, durable readback
and separately qualified CAD restoration. Capture is opt-in; it does not add
native spline geometry. Broader providers, raw/private/shared codec storage and
external links such as IFC associations remain future work. See
[preservation](docs/preservation.md). No generic foreign-semantic extension
protocol or IFCPR resource is introduced.
IFCCAD is integrated on main as an independent experiment alongside OCDraw,
with separate models, schemas, validation and direct conversion routes.
The explorer opens either native format and converts DWG/DXF through the chosen
route. IFCCAD geometry now includes points, signed arcs, full/partial ellipses,
bulged planar and straight spatial paths, optional scope bounds and expanded
viewport clips. Both converters share adjustable accuracy with independent
coordinate-domain evidence. This remains a provisional 0.1.0 revision;
see [shared geometry](docs/geometry/shared-geometry.md).
Both formats now retain layout media independently of plot settings. IFCCAD adds
effective plot output, limits, plot-style mode and Paper-space linetype-scaling
parity; both CAD routes assess Paper accuracy through each fixed physical plot
mapping. The provisional contract removes IFCCAD's independent Paper coordinate
unit. See [layout output](docs/layout-output.md) for numeric/raster restrictions.
The next IFCCAD slice covers UCS, model windows, paper canvases, grid/snap and
active workspace choices against this output boundary.

## Using the implementation

```text
cargo run --example write_ocdraw -- drawing.ocdraw.json
cargo run -p viewer -- drawing drawing.ocdraw.json
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
compares OCDraw, IFCCAD, DXF and DWG with identical retained content. See the
[synthetic/foundation report](docs/benchmarks/common-subset-size-exchange-v1.md)
and [large DXF report](docs/benchmarks/common-subset-test-dxf-v1.md).
Size measurements are run only on explicit user request, for the full experiment
or a requested selection. They are never automatic checks for a new main revision
or converter/encoding/dependency changes. Focused correctness tests still apply.
Licensed under [MPL-2.0](LICENSE).


## IFCCAD capabilities and resources

The independent IFCCAD proof is available in
`src/ifccad`, with its [experimental contract](schemas/ifccad/experimental-contract-0.1.0.md),
[sample](examples/ifccad/hello-cad.ifcx) and
[evaluation](docs/experiments/ifccad.md). Its direct
[converter](crates/ifccad-convert/README.md) connects the supported subset to
opencadcodec `CadDocument`, including named simple line patterns, scales and polyline
pattern generation. See the [geometry sample](examples/ifccad/hello-geometry.ifcx)
and [line-pattern sample](examples/ifccad/hello-line-patterns.ifcx). Geometry validation and length-unit tokens use shared neutral helpers;
the two drawing models, schemas and encoding routes remain separate. The
[IFCCAD & OCDraw Explorer](format-explorer/README.md) now opens IFCCAD and
roundtrips through DXF/DWG using its own converter. Model and multiple Paper layouts convert with explicit tab order, optional media and explicit output mappings. Paper viewports now retain orthographic/perspective camera, depth/display state and circle/full-ellipse/closed straight-or-bulged path clipping. CAD viewport exchange uses the explicit development codec repairs described in the converter; effective plot settings retain the documented CAD limits.

## History and naming

The current IFCX-native CAD profile was previously called IFCX-CAD and is now
named IFCCAD. Its Rust module, public types, converter crate, schema files and
browser APIs use IFCCAD names. See the [API migration](docs/model-io-conventions.md#ifccad-naming-migration)
for the previous-to-current names. The underlying IFCX syntax, `.ifcx` file
extension and `ifccad::` schema namespace retain their meaning.

The IFCCAD name also belonged to an earlier package experiment combining an
IFCX semantic graph with IFCDR drawing resources and optional IFCPR source
preservation. That package architecture, its readers and writers, and its active
schemas are retired. Today's IFCCAD and OCDraw are independent drawing models
with their own schemas, validation and conversion routes.

Numbered conformance collections remain immutable. The
[historical package design](docs/legacy-ifccad.md) and Git history retain the
earlier experiment's context.

## Local maintenance

Run `pwsh -NoProfile -File scripts/cleanup_local.ps1` to audit local worktrees
and build caches. Removal requires an explicitly selected, unused worktree;
the script retains local changes and unfamiliar files for review.
