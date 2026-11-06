# IFCCAD development roadmap

Development sequence reviewed on **3 November 2026**. This document owns
development order, dependencies and completion criteria. Schemas/conformance define the format;
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
| Native source alpha | `2026.45.0`; profile `0.3.0`, CBOR mapping `3`, Zstd transport `1` | Release contains the native core only; the annotation candidate is unpublished profile `0.4.0` / CBOR `4`; published contracts remain immutable |
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
extrusion thickness is covered by the isolated contribution described below.

The [active exchange coverage](docs/architecture/explorer-document-route.md)
records the current mapping, pins and unsupported requests. Do not attribute the
reference converter's adjustable hard tolerances, numerical certificates,
Allow/Reject policies, partial opening or spline restoration to the shared route.
Native opaque storage remains independent of CAD capture/restoration support.

The selected geometry contribution before annotation work adds primitive extrusion
thickness and classic 2D POLYLINE mapping. The native release uses
profile 0.3.0/CBOR3. Native/CAD control files and local Explorer integration have
been verified and visually accepted; the catalogue samples use 15 mm thickness
with base Z=0. Native release 2026.45.0 includes this contract. Document PR #8 is merged
and the source pins its official revision 28f1ad1. The Explorer update uses the same profile through a separately checked
browser/host build and deployment. See [extrusion scope and review boundary](docs/geometry/extrusion-thickness.md).

## Development sequence

Only remaining work is listed below. Completed foundations stay in the baseline
above and in their topic/release documentation rather than as planned steps.

Keep native IFCCAD contributions sequential: complete the annotation-context slice,
then add the bounded font/symbol-source foundation, followed by selected drawing
semantics. Qualify each substantial contribution against the preceding verified baseline;
compatible unpublished slices may ship together, as annotation contexts and
font/symbol sources do in2026.45.1. The independent Document preservation work described below can
proceed in parallel; its later IFCCAD integration becomes a separate contribution
between native extensions when the required Document support is ready.

### 1. Add shared annotation contexts and bounded association updates

Build on the established definition/selector boundary to design common annotation
scale contexts for Text/MText and Dimensions; assess Hatch participation and
drawing scale lists explicitly. Cover mm, m and inch drawings and multiple
viewport scales. Existing text height, Dimension factors and Paper linetype
scaling do not already provide this feature.

Complete and qualify the selected stored-context slice before integrating the
resource contribution. General association-update processing is a separate follow-up,
not an additional prerequisite for font/symbol-source storage.

The accepted storage/exchange slice is delivered with font/SHX sources in
release 2026.45.1, profile 0.5.0 / CBOR5: Text/MText, all nine Dimension kinds,
Hatch, Block definitions/references, independent scale/view identities and
saved Model/Paper policies. Compact physical roundtrips and the local catalogue
completed user review. Document PRs #10/#11 are merged and the matching Explorer
update uses their official merged source. A bounded catalogue text edit/grip
smoke covers undo/redo; general association regeneration and all annotation
editing remain outside that claim.

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

After the annotation-context slice, the next selected native contribution is a
bounded font/symbol-source foundation under
[#15](https://github.com/OpenAEC-Foundation/ifccad/issues/15), with explicit missing
or substitute status owned by the resolving application. The core describes
resources without resolving filesystem or network locations. Source identity,
application resolution and correct glyph geometry/layout are separate capabilities;
a complete font renderer is not a prerequisite for this first slice. Select the
fields and loss rules from concrete Text/MText and line-pattern use cases rather
than adopting a universal resource framework. A bounded PAT/NLCS importer can be
a narrow independent contribution where existing semantics suffice.

The selected block-attribute slice implements native Text/MText and paired Document
conversion in release 2026.45.2, profile 0.6.0 / CBOR6. The gallery and sample_AC1032
roundtrips have user acceptance. Native conformance/inspection, full native and
paired gates, local Explorer integration and bounded OCS editing/ATTSYNC checks
pass on the explicit pair. Official Document PR #12 is merged; the thickness
codec support revision remains explicitly pinned in the user fork. Necessary field/context relations are
included; MINSERT and dynamic blocks are deferred. See [attributes](docs/block-attributes.md).

Then select multileaders before tables or dynamic-block
authoring as separate use-case-backed slices under
[#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2); their relative order
and scope remain to be agreed. Native splines need their own profile under
[#4](https://github.com/OpenAEC-Foundation/ifccad/issues/4). They are largely
independent of resource resolution, but are not selected ahead of the resource
foundation without a concrete use case justifying that priority. Neither generic
preservation nor a complete font system is a prerequisite for native splines.
The historical opaque spline pilot is not a native evaluable spline model or a
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

## Parallel preservation track

Develop selective preservation in `opencaddocument` alongside the sequential
native IFCCAD contributions. The upstream
[direction copied from former fork issue #91 into acadrust #307](https://github.com/hakanaktt/acadrust/issues/307)
places typed capture/selection and reconstruction on Document. Reassess the earlier
codec storage and PR3/PR4 proposals against the current Document model before
preparing small PR contributions; those older plans do not define an implemented
Document API or an agreed final PR split.

1. **Document capture and durable storage.** Define a bounded, versioned selection
   of typed source content with its required dependencies, source identity and
   explicit coverage gaps. Prove storage and reopening after discarding the live
   source document. Successful capture alone does not establish safe reconstruction.
2. **Document reconstruction.** Reconstruct explicitly requested selections into
   another Document with qualified identity/reference rebinding and caller-supplied
   bindings to existing content. Test the selected cases through physical DWG/DXF
   write/read where exchange is claimed. Reconstruction and physical writeback
   fidelity remain separate claims.
3. **A separate IFCCAD integration slice when ready.** Once a bounded capture,
   durable storage, reopen and reconstruction chain is usable, insert an IFCCAD
   contribution between the other native extensions. It need not wait for all
   resource/entity work or a universal preservation facility. Implement and verify
   selected native preservation storage and the paired Document mapping together,
   then qualify the applicable Explorer integration and release stages separately.

Document owns the reusable capture/dependency/reconstruction mechanisms. IFCCAD
owns native coverage, preservation policy, storage validation and the rules for
combining restored content with current native values, ownership and draw order.
Keep the core independent of Document and CAD runtimes. The integration must prove
that native edits remain authoritative, deleted entities are not restored and
content is not emitted twice when represented natively. Test durable save/reopen
and subsequent native edits before claiming the selected end-to-end workflow.

Start with a few concrete cases. Raw DWG/DXF records, private replay baselines and
shared AcDs storage remain separate work; do not restore the archived converter
or make broad opaque relocation or full-document restore implicit requirements.
The active route currently has no preservation capture/restoration guarantee;
the [coverage contract](docs/architecture/explorer-document-route.md) changes only
with qualified implementation evidence. Historical native storage and spline-pilot
evidence do not establish Document-route support.

Keep implementation worktrees and dependency pins separate, coordinate changes
to shared Document interfaces, and schedule expensive verification and IFCCAD
integration sequentially. Local testing may use an explicit pinned development
pair before upstream merge. Prefer audited official merged revisions for deployment;
PR submission, integration and publication retain their explicit-authorization rules.

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

Keep one broad native IFCCAD implementation/integration task active at a time.
The bounded Document preservation track may proceed in parallel under the
boundaries above. Narrow independent research or review may also proceed with
explicit dependencies and coordinated expensive verification. The later IFCCAD
preservation slice joins the same sequential integration/release path.
Publishing, committing or merging still requires an explicit request. Mixed-IFCX
editing, IFC-object links, source writeback, collaboration, application command/UI
parity and a general CAD editor are not implicit requirements of this roadmap.

Release 2026.45.1 combines the annotation and font/SHX contributions in profile
0.5.0 / CBOR5 on annotation basis `100deb04`. It preserves exact source
requests and independent SHX records, with bounded AC1032 DWG and ASCII/binary
DXF roundtrips. The gallery and sample_AC1032 have completed user review. This does not
complete issue 15. The local Explorer and embedded host use the pinned development
pair, with existing examples migrated and a bounded text edit/grip/undo smoke.
The release selects official Document merge 47031369 and a matching Explorer
update. Resolution, portable locators, embedding and complete rendering remain
later work. Historical release identifiers above remain unchanged.
