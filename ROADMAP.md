# OCDraw and IFCCAD development roadmap

This document is authoritative for development sequencing and milestone status
in this repository. Schemas and conformance define the format contract;
Rust code and tests define implementation behavior. README summarizes this
sequence and must be updated with it.

## Parallel development tracks

This repository actively develops standalone OCDraw and the independent
IFCCAD profile. The initial OCDraw sequence below retains its architectural
dependencies and completion criteria. IFCCAD has its own provisional contract
and coverage progression; either route can continue independently.

Consider CAD semantics together, with reference drawings and validation evidence
reused where applicable. Each model, schema, reader, writer and conversion route
keeps its own contract. New support is verified separately; feature parity is
not assumed or required before either route can progress. OCDraw opening remains
independent of IFCX.

## Initial OCDraw contract

The initial OCDraw 0.1.0 contract remains provisional. The sequence below
expresses architectural dependencies; it is not a release calendar.

1. **Typed logical model and JSON separation - implemented.** Reader, writer
   and converter exchange typed drawing records. JSON fields and packing stay
   in the codec. Production readback validates generated outputs. The complete
   `OcdrawDocument` lifecycle is implemented as a refinement of this boundary:
   builder construction, shared authored validation, explicit bounds preparation
   and one document encoder retain IDs and allocation watermarks. CAD conversion
   accepts/returns the logical document; identity-preserving CAD editor sessions
   and merging independent copies remain future designs. See
   [lifecycle contracts](docs/ocdraw-document-lifecycle.md).
2. **Standalone parity and package retirement - implemented.** Geometry, blocks,
   layers, layouts and drawing state have standalone readback. Active
   package IFCX/IFCDR/IFCPR Rust routes and schemas are retired. DXF/DWG tests retain source/target limitations and
   numerical accuracy checks. Tools, examples and standalone measurements use OCDraw.
3. **Ordered scope ownership - implemented.** A scope's direct ordered `entities`
   list is authoritative for ownership and draw order. Stored entity owner IDs and both order streams were removed in a separate
   contract change.
4. **JSON stream simplification - implemented.** Present stream names and the
   versioned mapping identify streams without a stream directory. Unused streams
   may remain absent.
5. **Initial supported contract - candidate verified.** Full Rust gates,
   candidate conformance and the standalone primitive exchange experiment pass.
   Recording OCDraw 0.1.0 and entity schema v1 as the first supported contract
   remains a separate step. No compatibility layer is required for unpublished
   intermediate contracts.

The first bounded OCDraw preservation slice builds on the implemented lifecycle
and ownership boundaries: complete typed spline snapshots, native common
properties, durable envelope/opaque transport and separately qualified CAD
restoration. It does not publish the initial contract or implement native spline
geometry. Broader providers, raw/private/shared storage and external semantic
links (including IFC) require separate concrete follow-up designs. No IFCPR
resource or generic foreign-semantic extension protocol is revived.

## IFCCAD development track

The CAD-native-in-IFCX implementation and browser inspection route coexist on
main with standalone OCDraw. Native IFCCAD opening, composed graph inspection
and direct DXF/DWG roundtrip use their own model, schemas, validation and
converter. OCDraw opening does not require an IFCX file or package.
The experiment includes points, lines, circles/signed arcs, full/partial ellipses,
straight/bulged planar paths, straight spatial paths and local block geometry,
native Model/Paper layouts, named simple line patterns, scales and polyline
pattern generation, with strict readback and pinned DXF/DWG tests.
Allow/Reject conversion policies expose or reject supported loss classifications.
Optional scope bounds have explicit atomic preparation. Geometry validation and
CAD accuracy are shared with OCDraw through neutral helpers; converters expose
adjustable hard tolerances and per-coordinate-domain numerical evidence.
Model and multiple Paper layouts now convert with explicit tab order, optional media and explicit plot mappings. The viewport slice adds camera, perspective/depth/display state and circle/full-ellipse/closed straight-or-bulged path clipping, with CAD exchange verified using explicit codec development repairs; upstream dependency adoption remains a separate follow-up; complex
text/shape patterns use a diagnosed whole-definition fallback under Allow.
This integration does not freeze IFCCAD compatibility or expand the standalone
OCDraw contract. Continued IFCCAD development expands drawing semantics and
conversion coverage in bounded slices, including later text, annotations and
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

The next IFCCAD slice addresses drawing/workspace state (UCS, model windows,
paper canvases, grid/snap and active choices) using this established output boundary.
Other layer/entity/block presentation differences and preservation remain separate.

## Later semantic coverage and exchange

Expand supported CAD entity families and authored drawing semantics with
language-neutral rules, shared validation, production readback and actual
DXF/DWG tests. Diagnose losses and target restrictions. Review conversion
coverage whenever the pinned CAD codec public model changes.

Consider the meaning and reference cases of new CAD features across both tracks,
then implement each format's explicit mapping and coverage. Keep unsupported
families and numerical policies visible in the separate converter contracts.

The named simple line-pattern slice adds drawing-local definitions (including
Continuous), local references, global/entity scale and polyline generation.
Unused definitions survive. Complex text/shape definitions retain their named
identity and references with an explicitly diagnosed continuous fallback under
Allow; Reject refuses that loss. Native text/shapes, external assets, current
creation defaults and model-tab annotation scaling remain later work. This is
semantic expansion of the provisional contract, not its publication.

## Later physical encodings and performance

Logical semantics remain independent of streams' physical packing, compression
and chunking. Measure representative workloads and expand controlled recipes
before using results to justify physical encoding choices. A future physical
encoding may use shared buffers/ranges without changing ownership meaning.
Historical package benchmarks remain retained evidence for their old corpus;
the [standalone OCDraw report](docs/benchmarks/ocdraw-size-exchange-v1.md)
records its own applicable results.

Compact, directly openable storage is an adoption goal for both tracks. Compare
size, full/partial opening time and peak/resident memory on representative
drawings before selecting binary storage, integrated compression or indexed
chunks. OCDraw can explore its own storage choices; IFCCAD storage experiments
must identify compatibility boundaries and coordinate with evolving IFCX
technology. Current JSON/gzip probes do not establish native compressed formats
or future IFCX capabilities. Measurements run only on explicit user request,
for the full experiment or a requested selection; they are never automatic
completion gates, including after relevant code or dependency changes.

## Verification and freeze criteria

- All current entities, ownership, order and shared blocks survive strict
  production reader/writer readback and semantic conversion tests.
- Unknown core fields, malformed columns and invalid references are rejected.
- Candidate conformance/schema/mapping/model agree; numbered collections are
  untouched. All new entity schemas begin at v1.
- Full format, lint and workspace tests pass, as do applicable inspector checks.
- Relevant measurements use the pinned dependency and fresh directories.
- Only then record the first supported 0.1.0 contract. Release operations
  require a separate explicit request.

The implemented lifecycle API conventions and provisional caller migration are
documented in [model IO conventions](docs/model-io-conventions.md). This
organization refinement does not change milestone order or semantic coverage.
