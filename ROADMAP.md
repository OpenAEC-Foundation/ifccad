# IFCCAD development roadmap

This document is authoritative for sequencing and status. Schemas/conformance
and current Rust code define contracts; dated local plans retain their slice scope.

## Current sequence and worktrees

IFCCAD is the active track. OCDraw is parked as a reference archive, without
feature parity, routine builds or tests; see [parking](docs/ocdraw-parking.md).
The native CBOR/Zstd checkpoint precedes this migration; the current CBOR mapping
is revision 1, published with native source alpha 2026.44.0. Later normative
changes follow the [release/version policy](docs/releases.md).

1. The IFCCAD-main-track migration is integrated: workspace/API ownership, archive
   and application routing now use IFCCAD.
2. Hatch B is resumed on that basis: classic editing/construction state belongs to
   IFCCAD/IFCX and CBOR revision 1. Its separate DXF/DWG target profiles
   qualify all six offered versions, disclose losses before export, and retain the
   original AC1024 foundation drawing as local correctness acceptance. This does
   not reopen the completed Dimension graphics slice or deferred spline contours.
3. Native SOLID and TRACE are implemented and verified: two kinds retaining every
   finite local corner combination, native IFCX/CBOR storage and qualified CAD
   type retention, with coordinate/surface/display evidence kept separate.
4. Add bounded worked-out Dimension graphics on that basis, keeping polyline
   widths outside the slice. Graphics and Hatch contract changes require explicit
   dependency assessment when they overlap.
   The implementation is complete and verified: optional owned
   graphics, IFCX/CBOR storage, atomic CAD picture transfer, occurrence proof
   and Explorer support.
5. Native polyline widths are implemented as a separate candidate: Constant,
   PerVertex and classic Defaulted state, segment-band proof, partial bounds and
   qualified exchange. Original red sample_AC1032 storage acceptance passes; final
   verification is recorded separately; bounded unchanged-host comparison passes. Extrusion
   thickness remains a later slice.
6. Continue remaining Dimension kinds/selectors and later joint annotation support
   according to their dependencies.
7. Expand supported CAD semantics; assess whole-document storage/performance
   using representative evidence as described below.

The independent xformop worktree remains separate. One broad structural task runs
at a time. Independent narrow work can use focused checks; coordinate expensive
full gates and verify one explicitly authorized final integration candidate.
Migration execution alone does not authorize merging, pushing or resuming Hatch B.

## IFCCAD development track

Native opening, composition inspection and direct DXF/DWG roundtrip use the
`ifccad` core and `ifccad-convert` with independent validation and codec mapping.
The experiment includes points, lines, circles/signed arcs, full/partial ellipses,
straight/bulged planar paths, straight spatial paths and local block geometry,
native Model/Paper layouts, named simple and complex text/SHX line patterns, scales and polyline
pattern generation, with strict readback and pinned DXF/DWG tests.
Allow/Reject conversion policies expose or reject supported loss classifications.
Optional scope bounds have explicit atomic preparation. Native Text/MText and
independent local text styles now use the same enclosing/estimated/partial
quality meanings as OCDraw, with separate schemas, conversion and file tests.
Font contours and layout remain unassessed; numeric text evidence covers anchors.
Geometry validation and
CAD accuracy are shared with OCDraw through neutral helpers; converters expose
adjustable hard tolerances and per-coordinate-domain numerical evidence.
Model and multiple Paper layouts now convert with explicit tab order, optional media and explicit plot mappings. The viewport slice adds camera, perspective/depth/display state and circle/full-ellipse/closed straight-or-bulged path clipping, with CAD exchange verified using explicit codec development repairs; upstream dependency adoption remains a separate follow-up; qualified complex text/SHX patterns retain independent source/style references through AC1032 exchange. Unsupported active dependencies use a diagnosed whole-definition fallback under Allow; external glyph outlines remain unassessed.
This integration does not freeze IFCCAD compatibility or expand the standalone
OCDraw contract. Continued IFCCAD development expands drawing semantics and
conversion coverage in bounded slices, including later annotations and
authored view/plot state. Its long-term direction is a CAD drawing module
that can participate in the evolving IFCX ecosystem. IFC object associations,
source-graph writeback and collaborative updates require concrete future designs.

The layout-output slice implements standalone medium dimensions and effective
plot settings in both models, including Model/Paper limits and IFCCAD plot-style
mode and Paper-space linetype-scaling parity. Plot mappings replace IFCCAD's
independent Paper coordinate unit; both converters assess physical Paper accuracy
per layout. Native media-only states remain valid. Exact binary64 scalar conversion
and unqualified raster CAD output retain explicit restrictions; see
[layout output](docs/layout-output.md). Current active schemas remain provisional.

The workspace slice adds IFCCAD UCS and Model windows, Paper canvases and
authored viewport aids, with optional current choices, exact references and
independent allocation watermarks. OCDraw conversion uses the same scalar rules.
Active Paper tabs are qualified through consistent reserved block/layout roles.
Paper current-context/UCS selection and VIEWPORT file grid behavior retain
qualified codec limitations; see [workspace state](docs/workspace-state.md).
Other layer/entity/block presentation differences remain separate.

opaque SPLINE preservation pilot now uses this output boundary, including exact
Paper coordinate-meaning predicates, durable IFCX transport and qualified fresh
CAD restoration. It does not depend on workspace state, add native spline geometry,
freeze the profile or expand raw/private/shared storage; see [preservation](docs/preservation.md).

## Construction-workflow priorities from the October 2026 review

[Issue #17](https://github.com/OpenAEC-Foundation/ifccad/issues/17) proposes
30 CAD requirements. The [item-by-item review](docs/user-research-2026-10-review.md)
records implemented subsets, evidence, gaps, ownership and relative effort.
Its Must/Should ordering is an unvalidated market proposal, not a replacement
for the dependencies in the current sequence. The following priorities refine
that sequence without resuming paused work or changing current contracts.

1. **Make qualified exchange visible.** Alongside existing work, publish a
   development compatibility matrix by feature, DXF/DWG version and named reader.
   Distinguish internal codec readback from independently verified opening,
   repair/audit behavior and display. Record candidate, codec pin/patch, fixture
   identity and redistribution provenance; untested cells remain untested.
   Extend existing conversion diagnostics into an accessible retained/changed/
   omitted/preserved/unassessed summary. Retain the explicit fatal boundary for
   codec parsing and shared structure despite entity-local partial opening.
   These are correctness/reporting tasks, not automatic performance measurements.
2. **Finish construction annotations in the queued order.** Keep SOLID/TRACE,
   bounded Dimension graphics and polyline widths before the remaining kinds.
   Within those kinds, prioritize radius, diameter and angular dimensions using
   representative construction drawings. Follow with joint Text/MText/Dimension
   annotation contexts, assessing Hatch participation and drawing scale lists
   explicitly across mm, m and inch. Viewport/trans-space selectors and automatic
   dependent updates need their later contract; stored relations alone are not
   an update engine ([#13](https://github.com/OpenAEC-Foundation/ifccad/issues/13)).
   Hatch changes must respect the paused Hatch B boundary and its migration
   dependency; this review does not authorize resuming it.
3. **Add resource and library workflows, then selected missing semantics.** Use
   [#15](https://github.com/OpenAEC-Foundation/ifccad/issues/15) for font/symbol
   identity, application-owned resolution and explicit missing/substitute status.
   A bounded PAT importer or supported NLCS-library subset can proceed separately
   where current semantics suffice; select exact source/version and diagnose gaps.
   After queued annotation work, prioritize block attributes and multileaders as
   separate [#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2) slices,
   ahead of tables and dynamic-block authoring. Layer states/groups/filters need
   their own contract. Upstream codec #108 was completed in `8d995b8` on
   2026-10-20; upright line-pattern rotation now needs an audited dependency
   adoption and native mapping, including explicit older-DXF restrictions.
   Existing relative/absolute markers remain implemented.
4. **Qualify plot fidelity and external drawing exchange.** Work out CTB/STB
   authority, table resources and color-index behavior under
   [#16](https://github.com/OpenAEC-Foundation/ifccad/issues/16), including indices
   251–255. PDF output and batch publishing belong to a selected host/integration
   after fonts, line/fill rendering, Dimension display and plot styles are
   qualified; stored page settings alone do not prove scale-correct output.
   Xrefs build on resource identity but additionally require referenced-document
   ownership, origin, cycles, reload/bind and archive rules. Dynamic-block retention
   needs an approved dependency-preservation design and qualified codec support;
   the spline pilot does not provide it. These are separate later slices, not
   prerequisites for completing current annotation work.
5. **Establish large-drawing targets from representative evidence.** Under
   [#6](https://github.com/OpenAEC-Foundation/ifccad/issues/6), define licensed
   workloads, reference hardware, retained content and separate parsing,
   conversion, validation, display and peak/resident-memory costs. The proposed
   15-second and 20-times-file-size figures remain hypotheses. Measurements run
   only on explicit request, without automatic timing gates. Consider indexed
   access ([#3](https://github.com/OpenAEC-Foundation/ifccad/issues/3)) only when
   the workload justifies it; do not promote deferred placement/proof tuning
   solely from this issue.

Keep command/script migration, DWT selection, desktop packaging/parity, dynamic
authoring and MCP/dry-run UI in the selected application's backlog or a separately
approved integration. Public compatibility/release evidence can build on the
development matrix. Review calendar package versioning
([#19](https://github.com/OpenAEC-Foundation/ifccad/issues/19)) independently of
profile and physical-encoding revisions. No weekly release, general CAD editor,
universal recovery or byte-identical regenerated DWG-section contract is adopted.
Every future slice still needs its own bounded design and relevant verification.

## Later semantic coverage and exchange

Expand supported CAD entity families and authored drawing semantics with
language-neutral rules, shared validation, production readback and actual
DXF/DWG tests. Diagnose losses and target restrictions. Review conversion
coverage whenever the pinned CAD codec public model changes.

Consider CAD meaning and reference cases, then implement the IFCCAD mapping and coverage. Keep unsupported
families and numerical policies visible in the separate converter contracts.

The named simple line-pattern slice adds drawing-local definitions (including
Continuous), local references, global/entity scale and polyline generation.
Unused definitions survive. Complex text/shape definitions retain their named
identity and references with an explicitly diagnosed continuous fallback under
Allow; Reject refuses that loss. Native shapes, external assets, current
creation defaults and model-tab annotation scaling remain later work. This is
semantic expansion of the provisional contract, not its publication.

The earlier Text/MText slice implemented native styles, typed rich content, authored
inheritance, strict JSON readback and enclosing/estimated/partial bounds evidence.
Its separate CAD route qualifies a bounded profile with anchor accuracy and
unassessed font geometry. Native content and CAD editing-dependency normalization
are distinguished in diagnostics. The independent IFCCAD slice now provides
its own native model, schemas, validation and CAD exchange tests, retaining
the same bounds-quality meanings. Text annotations,
dynamic fields, attributes and font engines remain separate work. See
[text support](docs/text.md). This does not publish or freeze the initial contract.

## Hatch and pattern checkpoint

The Solid Hatch slice implements independent OCDraw/IFCCAD typed storage,
structural/join/source validation, contour bounds, CAD conversion and inspection.
Actual AC1032 DXF/DWG tests retain area rules, loops and associations, with
contour/occurrence numerical evidence and separately unassessed fill.
The Solid slice was verified and accepted on main before LinePattern implementation.
The pattern slice adds embedded families, authored phase and qualified CAD conversion
with contour/pattern numerical evidence and separately unassessed fill; native spline boundaries,
intersection auditing and runtime associative updates remain separate work.
The gradient slice adds nine predefined types, two stored colors, independent mode/tint and partial inactive settings in both native models. CAD exchange is qualified for exact retained local axes and AC1032; inactive DXF settings and unavailable named color metadata remain diagnosed codec limitations. Gradient fields/pixels are unassessed.
See [Hatch support](docs/hatch.md). No initial contract publication is implied.

## SOLID and TRACE checkpoint

Before Dimension graphics, the SOLID/TRACE slice adds native SOLID and TRACE as ordinary Model/Paper/block
entities sharing one planar four-corner descriptor with signed extrusion thickness.
All finite local corner combinations are supported, including crossing, retraced
and wholly collapsed states. Preserve their distinct CAD types in qualified exchange and reuse the
same payloads in later graphics parts. This replaces the proposed conversion to
Solid Hatch and avoids a new opaque preservation family. The concrete
[design](docs/geometry/solid-trace-design.md), revised after the
[OCS behaviour investigation](docs/geometry/solid-trace-cad-behaviour.md), is
implemented and verified. Required coordinate/envelope guarantees, optional filled-surface
evidence and unassessed viewer display remain separate. Missing surface evidence
does not refuse or omit an otherwise supported finite record.
The checkpoint includes native schemas, readers, CAD exchange and inspection. It does not expand
Hatch-source, viewport-clip or Dimension-selector eligibility or add polyline widths.

## Ordinary Dimension checkpoint

The ordinary Dimension slice adds independent native linear/aligned entities and
styles, typed source selectors, pure measurement/formatting and qualified CAD
exchange/inspection. Definition and explicit text anchors use the existing hard
tolerance; complete display and picture regeneration remain unassessed. Seven
additional kinds, extended selectors and joint annotative Text/MText/Dimension
remain subsequent slices. The repair round adds native fraction presentation,
qualified CAD override coverage and localized inspector properties within this
checkpoint. Following the native SOLID/TRACE precursor, the graphics
slice implements optional native worked-out Dimension graphics and qualified CAD
picture-block capture/reuse, initially for linear and
aligned. It precedes the remaining kinds and extended selectors; annotative
per-context graphics stay with the later joint annotation slice. The active experimental graphics contract and its bounded transfer profile are
documented in
[Dimension support](docs/dimension.md). This adds
content within the existing sequence; it does not reopen layout/workspace work
or publish the initial contract.

The original bounded graphics slice deferred widths and used zero-width paths
and native SOLID/TRACE arrows for positive cases. The separate width candidate
below now extends that primitive coverage and runs the original linked red case.
Whole-picture rollback, complete-resource transfer and independent Allow/Reject
versus entity-recovery policies retain their existing meaning.

<a id="planned-polyline-width-follow-up"></a>

## Polyline-width implementation candidate

The separate [native polyline width contract](docs/geometry/polyline-widths.md)
is implemented as a review candidate on top of bounded Dimension graphics.
Constant, explicit vertex pairs and classic defaults/optional overrides retain
explicit zeros and dormant records in IFCX/CBOR. Mandatory segment-band proofs and
located Partial display bounds cover ordinary Model/Paper/block and graphics use.
Six ordinary CAD versions have explicit qualification/loss rules; stored wide
Dimension pictures are AC1032-only. Final gate status is recorded with the candidate;
bounded planar comparison in the unchanged host passes for both originals and
all six AC1032 files.
Presentation lineweight remains independent; extrusion thickness stays deferred.

The original red linear/aligned Dimension `0x527` in sample_AC1032, its pictureblock
`*D8`, and both INSERTs of `_ArchTick` are the concrete follow-up acceptance case.
Retain marker LWPOLYLINE width 0.15 (source entity `4A6`), its layer/ByBlock context,
the placed graphics and the original MText wrap width zero/stacked-fraction text.
Qualify complete picture capture, native IFCX/CBOR readback and actual supported
DXF/DWG re-export; numerical/visual claims retain the graphics design's existing
limits. Test the relevant source content/resources without requiring unrelated
unsupported content elsewhere in the full sample to become lossless. Source
handles locate evidence, not stable output identities. No narrower or zero-width
replacement of the marker satisfies this acceptance case.

## Later physical encodings and performance

Provisional decision on 2026-10-21: focus on standalone CAD-profile documents
within IFCX; see the [scope and lifecycle boundary](docs/experiments/ifccad-document-lifecycle.md#provisional-standalone-cad-scope--2026-10-21).
Mixed IFCX editing/writeback, full-source-plus-CAD-cache storage and a general
IFCX fragment encoding wait for a concrete application. Existing source
retention, composition and conversion-loss behavior remain unchanged.

CAD semantic coverage and qualified exchange retain the sequence above. Keep
CBOR/Zstd correct as those semantics evolve. The next optional storage step is
a separately requested whole-document JSON/CBOR/native-Zstd comparison of
opening/writing time and peak/resident memory on agreed inputs and hardware;
see the [bounded proposal](docs/experiments/ifccad-cbor.md#next-investigation).
This records no measurement authorization. Indexed chunks and partial save wait
for a consumer that can exploit partial access/updates or profiling that shows
another sufficient benefit. Full conversion through `CadDocument` alone does
not establish such a need. Columnar encoding remains a later measured choice.

IFCCAD has an [experimental base-CBOR candidate](docs/experiments/ifccad-cbor.md)
with local fixed numeric field codes, a separate mapping revision, production
readback and shared document validation. It is a standalone CAD-document encoding,
not a binary serialization of arbitrary IFCX graphs. Its
[controlled comparison](docs/benchmarks/ifccad-cbor-results-summary-v1.md) records
the qualified baseline scope. Optional experimental whole-file Zstd compression
has configurable levels with default 3; raw CBOR remains the default. These
experiments select no columns, chunking or final storage recommendation, and
do not publish a supported version.

Logical semantics remain independent of streams' physical packing, compression
and chunking. Measure representative workloads and expand controlled recipes
before using results to justify physical encoding choices. A future physical
encoding may use shared buffers/ranges without changing ownership meaning.
Historical package benchmarks remain retained evidence for their old corpus;
the [standalone OCDraw report](docs/benchmarks/ocdraw-size-exchange-v1.md)
records its own applicable results.

Compact, directly openable storage is an IFCCAD adoption goal. Compare
size, full/partial opening time and peak/resident memory on representative
drawings before selecting binary storage, integrated compression or indexed
chunks. IFCCAD storage experiments
must identify compatibility boundaries and coordinate with evolving IFCX
technology. Current JSON/gzip probes do not establish native compressed formats
or future IFCX capabilities. Measurements run only on explicit user request,
for the full experiment or a requested selection; they are never automatic
completion gates, including after relevant code or dependency changes.

## Verification and freeze criteria

- All current entities, ownership, order and shared blocks survive strict
  production reader/writer readback and semantic conversion tests.
- Unknown core fields and invalid references are rejected.
- Candidate conformance/schema/mapping/model agree; numbered collections are
  untouched. All new entity schemas begin at v1.
- Full format, lint and workspace tests pass, as do applicable inspector checks.
- Explicitly requested measurements use the pinned dependency and fresh directories; they are not automatic gates.
- Only then record the first supported 0.1.0 contract. Release operations
  require a separate explicit request.

The implemented lifecycle API conventions and provisional caller migration are
documented in the [IFCCAD lifecycle and API conventions](docs/experiments/ifccad-document-lifecycle.md). This
organization refinement does not change milestone order or semantic coverage.

The complex line-pattern slice extends the existing drawing model and converter
coverage without adding external asset resolution to the core. The IFCCAD route keeps closed text/SHX markers, independent source identity/watermarks, and separate
producer/derived bounds evidence. Resource resolution and future source kinds
remain deferred to issue 15; upright requires adoption and qualification of the
upstream codec repair (issue 108), followed by its native semantic slice.
See [line patterns](docs/line-patterns.md). Milestone ordering is unchanged.

## Parked OCDraw checkpoint

OCDraw 0.1.0 remains provisional and unpublished. Its model/JSON separation,
ordered scope ownership, package retirement and bounded spline preservation
were implemented before parking. Those milestones are historical, not ongoing
parallel obligations. Frozen sources, tests and exact checkpoint/dependency
identity are indexed by [FREEZE.json](legacy/ocdraw/FREEZE.json). Restoring OCDraw
requires the complete checkpoint and a new explicit development decision.
