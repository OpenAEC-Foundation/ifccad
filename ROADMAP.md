# IFCCAD development roadmap

This document is authoritative for sequencing and status. Schemas/conformance
and current Rust code define contracts; dated local plans retain their slice scope.

## Current sequence and worktrees

IFCCAD is the active track. OCDraw is parked as a reference archive, without
feature parity, routine builds or tests; see [parking](docs/ocdraw-parking.md).
The accepted CBOR revision-2/native-Zstd checkpoint precedes this migration.

1. Complete and integrate the IFCCAD-main-track migration against current main.
   It changes workspace/API ownership, archive and application routing, not format semantics.
2. Keep Hatch B paused until that integration. Rebase once afterward, replace the
   old codec pin and dual-model scope, and decide its IFCCAD/CBOR contract boundary
   before implementation. Do not couple later work to its unresolved boundary.
3. Resume worked-out Dimension graphics on the resulting stable basis, then the
   remaining kinds/selectors and later joint annotation slice. Graphics and Hatch
   contract changes require explicit dependency assessment when they overlap.
4. Expand semantics and investigate storage/performance as described below.

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

## Ordinary Dimension checkpoint

The ordinary Dimension slice adds independent native linear/aligned entities and
styles, typed source selectors, pure measurement/formatting and qualified CAD
exchange/inspection. Definition and explicit text anchors use the existing hard
tolerance; complete display and picture regeneration remain unassessed. Seven
additional kinds, extended selectors and joint annotative Text/MText/Dimension
remain subsequent slices. The repair round adds native fraction presentation,
qualified CAD override coverage and localized inspector properties within this
checkpoint. The next planned slice adds optional native worked-out Dimension
graphics and qualified CAD picture-block capture/reuse, initially for linear and
aligned. It precedes the remaining kinds and extended selectors; annotative
per-context graphics stay with the later joint annotation slice. The graphics
design is under review and does not yet extend the active contract. See
[Dimension support](docs/dimension.md). This adds
content within the existing sequence; it does not reopen layout/workspace work
or publish the initial contract.

## Later physical encodings and performance

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
documented in [model IO conventions](docs/model-io-conventions.md). This
organization refinement does not change milestone order or semantic coverage.

The complex line-pattern slice extends the existing drawing model and converter
coverage without adding external asset resolution to the core. The IFCCAD route keeps closed text/SHX markers, independent source identity/watermarks, and separate
producer/derived bounds evidence. Resource resolution and future source kinds
remain deferred to issue 15; upright requires upstream codec support (issue 108).
See [line patterns](docs/line-patterns.md). Milestone ordering is unchanged.

## Parked OCDraw checkpoint

OCDraw 0.1.0 remains provisional and unpublished. Its model/JSON separation,
ordered scope ownership, package retirement and bounded spline preservation
were implemented before parking. Those milestones are historical, not ongoing
parallel obligations. Frozen sources, tests and exact checkpoint/dependency
identity are indexed by [FREEZE.json](legacy/ocdraw/FREEZE.json). Restoring OCDraw
requires the complete checkpoint and a new explicit development decision.
