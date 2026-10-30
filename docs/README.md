# Documentation

Start with the [project overview](../README.md), or choose a reading route below.
[ROADMAP.md](../ROADMAP.md) is authoritative for development order and status.
Schemas/conformance define the language-neutral contract; current Rust code and
tests define the Rust API and implementation behavior. These pages explain and
connect those sources without replacing them.

## Read routes

| I want toâ€¦ | Read |
| --- | --- |
| Understand the purpose and direction | [Vision](vision.md), then [roadmap](../ROADMAP.md) |
| Understand alpha compatibility and releases | [Alpha policy](alpha-compatibility.md), [release/version policy](releases.md), [first candidate notes](releases/2026.44.0.md) |
| Try the Explorer or reproduce an example | [Explorer guide](../format-explorer/README.md), then [example recipes](../examples/README.md) |
| Load, edit, validate and save drawings | [Lifecycle and API conventions](experiments/ifccad-document-lifecycle.md), [ID management](experiments/ifccad-id-management.md) and [drawing defaults](drawing-defaults.md) |
| Implement or validate the format | [Profile contract](../schemas/ifccad/experimental-contract-0.3.0.md), [rule catalog](../schemas/ifccad/profile-rules-0.3.0.md) and [candidate conformance](../conformance/next/README.md) |
| Understand CBOR and optional Zstd | [Encoding guide](experiments/ifccad-cbor.md), [CBOR mapping](../schemas/ifccad/cbor-mapping-3.md) and [Zstd transport](../schemas/ifccad/cbor-zstd-transport-1.md) |
| Assess active DXF/DWG exchange | [Shared Document route coverage](architecture/explorer-document-route.md) |
| Reproduce historical direct-converter proofs | [Converter archive](../legacy/ifccad-convert/README.md), [reference coverage](../legacy/ifccad-convert/reference/crates/ifccad-convert/README.md) |
| Investigate historical converter failures | [Conversion diagnosis](conversion-diagnostics.md) |
| Work on browser processing and inspection | [Browser architecture](architecture/browser-processing.md), then the [Explorer development guide](../format-explorer/README.md#development-and-verification) |
| Contribute or maintain a checkout | [Working agreements](../AGENTS.md), [verification](../README.md#verification) and [local maintenance](local-maintenance.md) |

## Drawing topics

| Topic | Explanation and supporting detail |
| --- | --- |
| Geometry and numerical guarantees | [Shared geometry](geometry/shared-geometry.md), [units/tolerance](geometry/unit-tolerance.md), [block transforms](geometry/block-transform.md) and [trigonometric proof](geometry/block-trigonometry.md) |
| Filled primitives and Hatch | [SOLID/TRACE](geometry/solid-trace-design.md) and [Hatch](hatch.md) |
| Text and dimensions | [Text/MText](text.md), [Dimensions](dimension.md) and [Dimension graphics mapping](dimension/graphics-design.md) |
| Appearance and resources | [Presentation](presentation.md) and [line patterns](line-patterns.md) |
| Layouts and saved drawing state | [Layout output](layout-output.md) and [workspace state](workspace-state.md) |
| Retained source semantics | [Preservation](preservation.md) |

## Research and retained evidence

- **Requirements:** [October 2026 review](user-research-2026-10-review.md).
- **IFCX questions:** [Assumptions and upstream gaps](experiments/ifcx-upstream-gaps.md)
  and the [composition probe](experiments/ifcx-composition-probe.md).
- **Practice investigations:** [SOLID/TRACE host behavior](geometry/solid-trace-cad-behaviour.md),
  [Dimension reference elevations](dimension/reference-elevations-investigation-2026-10-20.md)
  and [coincident polyline segments](geometry/coincident-polyline.md).
- **Measurements:** [IFCCAD CBOR reports and data](../benchmarks/ifccad-cbor/README.md),
  [common-subset exchange](../benchmarks/size-exchange/README.md),
  [historical package/OCDraw size reports](../benchmarks/size/README.md) and
  [placement preparation and practice inventory](benchmarks/placement-preparation-v1.md).
  Follow each report's matching results and provenance. Dates/checkpoints bound
  the evidence; new measurements require a separate explicit request.

## History

Use [project provenance](../PROVENANCE.md), [OCDraw parking](ocdraw-parking.md)
and the [frozen source reference](../legacy/ocdraw/README.md) for historical work.
[Numbered conformance provenance](../conformance/1.0.0/PROVENANCE.md) belongs to
its immutable collection; it does not define the current IFCCAD profile.
