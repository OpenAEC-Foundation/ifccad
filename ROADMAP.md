# IFCCAD development roadmap

Status reviewed on **29 October 2026**. This document owns development order,
dependencies and completion criteria. Schemas/conformance define the format;
current code and tests define implementation support. Dated designs, issue
checklists and historical conversion reports do not override those sources.

## Current baseline

IFCCAD development now has two cooperating implementation boundaries:

- **Native core:** the CAD drawing model, construction, validation, source
  inspection/composition, IFCX JSON, CBOR and optional CBOR + Zstd. The core
  remains independent of CAD runtimes.
- **CAD exchange and applications:** DWG/DXF conversion through the shared
  `opencaddocument` mapping; viewer/browser and Explorer consume it. Open CAD
  Studio opens the actual IFCX snapshot for preview. A regenerated DWG/DXF is
  an optional export, not the required preview intermediate.

| Area | Implemented status | Remaining boundary |
| --- | --- | --- |
| Native source alpha | `2026.44.1`; profile `0.2.0`, CBOR mapping `2`, Zstd transport `1` | Release contains the native core only; alpha compatibility is not guaranteed; published format versions are immutable and format changes receive new identifiers |
| Drawing foundations | Primitive/path geometry, shared/nested blocks, Model/Paper layouts, presentation, Text/MText, line patterns, workspace and viewport data | Native storage does not establish complete CAD mapping or rendering |
| Hatch | Solid, line-pattern and gradient fills; source/association state; classic editing and boundary-construction records | Fill evaluation, automatic updates and shared-route coverage remain separate |
| SOLID/TRACE | Native kinds, planar corner state and extrusion thickness | Surface and display evidence remain distinct from stored coordinates |
| Dimensions | Type expansion completed: all nine native definitions, measurement, formatting and storage in `2026.44.1`; paired conversion on official Document PR #7 merge `e6135a7`, adopted in the published Explorer | CAD and local Explorer review accepted; grouped catalogue verified; shared annotation contexts and bounded update interoperability remain follow-ups; editor regeneration/undo is a separate responsibility |
| Planar polyline widths | Constant, PerVertex and classic Defaulted modes; ordinary entities and Dimension graphics | Extrusion thickness for polylines and complete joins/display qualification remain later work |
| Explorer/CLI migration | Integrated and published; active conversion no longer depends on `ifccad-convert` | Shared-route losses, numerical fidelity, physical CAD readback and host display have their own evidence limits |
| Reference-converter retirement | Archived outside ordinary workspace builds/tests, with source/test/fixture hashes and complete-checkpoint reproduction | Historical contracts remain reference evidence; no second product route |
| OCDraw | Parked outside ordinary development | No parity or compatibility-restoration requirement |

Hatch B, SOLID/TRACE, stored Dimension graphics and polyline widths are completed
native prerequisites, not a queue of unfinished implementation candidates.
Their earlier direct-converter/file/host results qualify the recorded route and
cases only. See [Hatch](docs/hatch.md), [Dimensions](docs/dimension.md),
[SOLID/TRACE](docs/geometry/solid-trace-design.md) and
[polyline widths](docs/geometry/polyline-widths.md).

Shared-route consolidation and the seven-kind Dimension expansion are complete,
not pending milestones. The archived converter has reconstructible provenance
and is excluded by the active dependency guard. The published Explorer adopts
Document PRs #5/#6/#7 through pin `e6135a7`, with integration regressions and
accepted CAD/sample_AC1032 review. All nine Dimension kinds are included in
[native release 2026.44.1](docs/releases/2026.44.1.md). Known mapping/display
limits remain in the active coverage; Studio's grip/undo problems are separate
editor work under
[OCS #2325](https://github.com/HakanSeven12/OpenCADStudio/issues/2325).
Future dependency updates are maintenance: audit intervening model/mapping
changes, record Explorer/host pins and run applicable regressions before adoption.

<a id="planned-polyline-width-follow-up"></a>

The original linked red sample_AC1032 Dimension/tick case has bounded
reference-route acceptance evidence; it is not an unfinished zero-width-marker
workaround or a universal display pass. Retain its selected input/resource
closure and provenance when reusing that case for the shared route. Polyline
extrusion thickness remains a separate future slice.

The [active exchange coverage](docs/architecture/explorer-document-route.md)
records the current mapping, pins and unsupported requests. Do not attribute the
reference converter's adjustable hard tolerances, numerical certificates,
Allow/Reject policies, partial opening or spline restoration to the shared route.
Native opaque storage remains independent of CAD capture/restoration support.

## Development sequence

Only remaining work is listed below. Completed foundations stay in the baseline
above and in their topic/release documentation rather than as planned steps.

### 1. Add shared annotation contexts and bounded association updates

Build on the established definition/selector boundary to design common annotation
scale contexts for Text/MText and Dimensions; assess Hatch participation and
drawing scale lists explicitly. Cover mm, m and inch drawings and multiple
viewport scales. Existing text height, Dimension factors and Paper linetype
scaling do not already provide this feature.

Stored source relations and automatic updates are distinct. IFCCAD owns the
stored definitions, role-specific source references/selectors and their shared
validation. Dimensions retain optional associations without adding a separate
enabled/disabled flag; the role-specific links already identify the associated
points. Ordinary IO does not refresh definitions or graphics.

An editor such as Open CAD Studio, or a separate headless update service, owns
reaction to source edits: resolving affected relations, refreshing linked
definition points, recomputing measurements and graphics/bounds, and recording
the dependent changes in the same undo/redo operation. Core measurement and
formatting helpers are reusable; a running dependency engine is not required in
the format reader/writer. `opencaddocument` translates supported stored relations
in both directions rather than supplying the editor's change-processing loop.

Treat viewport/trans-space selection, stable generated-geometry identity and
explicit refresh behavior as bounded follow-ups under
[#13](https://github.com/OpenAEC-Foundation/ifccad/issues/13), including reusable
processing for BIM/headless applications. Agree the application/core boundary
before selecting an update engine. If retaining links while disabling their
updates needs portable file semantics, specify that policy separately; no such
Dimension flag is currently defined. Do not make a general dependency engine a
prerequisite for all annotation storage.

**Exit criteria:** a bounded context/identity contract, native readback, shared
validation and qualified exchange for each selected use case; host regeneration
and rendering claims tested independently.

### 2. Expand resources and selected drawing semantics

Establish font/symbol-source identity and application-owned resolution under
[#15](https://github.com/OpenAEC-Foundation/ifccad/issues/15), with explicit missing
or substitute status. The core describes resources without resolving filesystem
or network locations. A bounded PAT/NLCS importer can be a narrow independent
contribution where existing semantics suffice.

Select block attributes and multileaders before tables or dynamic-block authoring
as separate use-case-backed slices under
[#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2). Native splines need
their own profile under [#4](https://github.com/OpenAEC-Foundation/ifccad/issues/4);
the historical opaque spline pilot is not a native evaluable spline model or a
general preservation protocol. Layer states/groups/filters also need a bounded
contract rather than an unqualified extension of layer flags.

**Exit criteria per slice:** logical meaning and dependencies agreed; shared
validation, native mappings, source-detail/loss rules and production-reader/CAD
regressions implemented together. No universal payload framework is required.

### 3. Qualify plotting and external drawing workflows

Build on resources, annotations and rendering evidence before claiming faithful
PDF or batch publishing. Resolve native color/palette and CTB/STB authority under
[#16](https://github.com/OpenAEC-Foundation/ifccad/issues/16), including indices
251–255. Stored page sizes, ratios and plot-style names do not prove scale-correct
output. PDF generation and its UI belong to a selected host/integration.

Xrefs additionally need document ownership, origin, cycles, reload/bind and
archive rules. Dynamic-block retention requires a concrete dependency-preservation
design and codec support. Neither follows automatically from the spline pilot.

**Exit criteria:** selected workflows have explicit ownership/resource rules and
tests in named readers/hosts, including physical output where claimed.

## Construction-workflow priorities from the October 2026 review

The [requirements review](docs/user-research-2026-10-review.md) of
[#17](https://github.com/OpenAEC-Foundation/ifccad/issues/17) remains useful input,
not evidence that all market requirements or its old converter claims are met.
The sequence above replaces its stale migration/prerequisite status.

Alongside the selected semantic slice, improve a public feature/version/reader
matrix and an understandable retained/changed/omitted/unassessed report. Separate
native validity, semantic mapping, numerical error, physical CAD readback and
host display. Reader recovery cannot promise that every corrupt record is safe
to skip. Internal codec readback is not independent-reader certification.

The optional file-set evaluation proposal
[#20](https://github.com/OpenAEC-Foundation/ifccad/issues/20) can provide common
input identity and reporting for this work. It is not a prerequisite framework
for the next feature or authorization for full-corpus/benchmark runs. Numerical
research in #11/#12/#18 remains route-specific evidence; adopting its techniques
into the shared route requires a selected, justified scope.

## Later physical encodings and performance

Compact, directly openable storage remains an adoption goal. Current native
formats are IFCX JSON, standalone CBOR and optional CBOR + Zstd; Studio interchange
currently uses IFCX JSON. CBOR encodes the typed drawing projection, not arbitrary
IFCX fragments/composition. Whole-file compression supplies neither indexed
access nor partial save. See the [storage boundary](docs/experiments/ifccad-cbor.md).

Under [#6](https://github.com/OpenAEC-Foundation/ifccad/issues/6), a separately
requested investigation can compare equal retained content, bytes, loading/
writing time and peak/resident memory on agreed representative inputs and hardware.
Disclose source-graph retention, omissions and failure coverage. Do not improve a
size ratio by silently dropping content or quantizing geometry.

Indexed chunks/partial access
([#3](https://github.com/OpenAEC-Foundation/ifccad/issues/3)), columnar packing and
further placement/proof optimizations wait for a demonstrated consumer or workload
benefit. The separate xformop experiment is not an adopted production placement
contract. Existing benchmark reports remain evidence for their recorded checkpoints.
Measurements require an explicit request, fresh outputs and applicable provenance;
they are never automatic CI or task-completion gates. `test.dxf` remains outside
the currently selected practice-file scope.

## Release and completion rules

The first source alpha is already published. Follow
[release/version policy](docs/releases.md) for subsequent software releases and
normative profile/encoding changes; do not edit a tagged contract in place.
Maintain the single active mapping and `conformance/next` together. Historical
readers/migrations are optional during alpha; numbered conformance collections
remain immutable. No fixed weekly release cadence is required.

Every implementation slice needs strict production readback, shared validation,
semantic tests and honest coverage/loss disclosure. A conversion fix should have
a portable regression; reader/version/render claims need their own evidence.
Run the [verification gate](AGENTS.md#verification) on the final integration
candidate, including frontend/WASM checks when applicable. Prose-only updates
need content/link checks, not a Rust rebuild.

Keep one broad structural task active at a time. Narrow independent work may
proceed with explicit dependencies and coordinated expensive verification.
Publishing, committing or merging still requires an explicit request. Mixed-IFCX
editing, IFC-object links, source writeback, collaboration, application command/UI
parity and a general CAD editor are not implicit requirements of this roadmap.
