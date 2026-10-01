# OCDraw development roadmap

This document is authoritative for development sequencing and milestone status
in this repository. Schemas and conformance define the format contract;
Rust code and tests define implementation behavior. README summarizes this
sequence and must be updated with it.

## Initial OCDraw contract

The initial OCDraw 0.1.0 contract remains provisional. The sequence below
expresses architectural dependencies; it is not a release calendar.

1. **Typed logical model and JSON separation - implemented.** Reader, writer
   and converter exchange typed drawing records. JSON fields and packing stay
   in the codec. Production readback validates generated outputs.
2. **Standalone parity and package retirement - implemented.** Geometry, blocks,
   layers, layouts and drawing state have standalone readback. Active
   IFCX/IFCDR/IFCPR Rust routes and schemas are retired. DXF/DWG tests retain source/target limitations and
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

Future preservation and external semantic links (including IFC) require their
own concrete designs. They may later use extension points within OCDraw;
there is no current preservation resource or generic extension protocol to
standardize ahead of a use case. IFCX integration can remain an independent
experimental development in this repository. The planned next integration brings
the existing CAD-native-in-IFCX experimental branch onto main and adds its browser
inspection route. Both developments may coexist on main for the duration of the
experiment, with separate contracts and no IFCX dependency for OCDraw. This
follow-up is not implemented by the standalone OCDraw integration.

## Later semantic coverage and exchange

Expand supported CAD entity families and authored drawing semantics with
language-neutral rules, shared validation, production readback and actual
DXF/DWG tests. Diagnose losses and target restrictions. Review conversion
coverage whenever the pinned CAD codec public model changes.

## Later physical encodings and performance

Logical semantics remain independent of streams' physical packing, compression
and chunking. Measure representative workloads and expand controlled recipes
before using results to justify physical encoding choices. A future physical
encoding may use shared buffers/ranges without changing ownership meaning.
Historical package benchmarks remain retained evidence for their old corpus;
the [standalone OCDraw report](docs/benchmarks/ocdraw-size-exchange-v1.md)
records its own applicable results.

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
