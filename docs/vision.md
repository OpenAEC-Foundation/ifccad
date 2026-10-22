# IFCCAD vision

IFCCAD describes CAD drawings as typed, application-independent information
within IFCX. Drawings remain understandable deliverables in their own right,
including authored CAD and geometry generated from building models. Identity,
relationships and composition offer opportunities alongside other building
information, without making an IFC project graph a requirement for every drawing.

IFCCAD is the active track. OCDraw is parked as a recoverable source reference;
future work follows demonstrated CAD usability, semantic fidelity and measured
performance rather than maintaining two models at feature parity. Neither the
IFCCAD profile nor its experimental CBOR/Zstd transport is a published standard.

## Principles

- Keep ordinary CAD-only exchange useful alongside BIM-derived drawings.
  A drawing and a building model are distinct deliverables; optional semantic
  links should help coordination without becoming a prerequisite for CAD use.
- Specify meaning in a language-neutral logical contract before optimizing
  physical encoding. Independent implementations can use the published model.
- Keep geometric entities central and typed. Modelspace, paperspace and shared
  block definitions follow the same ownership and order rules.
- Store layers, layouts, appearance choices and drawing-bound state together.
  Last-opened files and application-specific navigation belong to applications.
- Preserve source meaning natively where supported. Otherwise diagnose a loss
  or reject inconsistent data; numerical accuracy is an explicit hard limit.
- Verify exchange through production readers and real CAD codec roundtrips.
- State native validity, application support, semantic transfer, numerical
  accuracy and visual evaluation as separate claims. Qualified operations and
  conformance cases must make their limits inspectable.
- Make compact, directly openable files an adoption goal. Evaluate opening time
  and memory alongside transfer size; storage compression alone does not imply
  a compact in-memory drawing or efficient partial access.

## Future integration and preservation

The [provisional scope decision](experiments/ifccad-document-lifecycle.md#provisional-standalone-cad-scope--2026-10-21)
focuses current development on standalone CAD-profile documents within IFCX.
Broader mixed-document editing and source writeback await a concrete application;
they are not prerequisites for extending CAD semantics or evaluating the existing
whole-document CBOR/Zstd route. This leaves the long-term integration direction open.

Future BIM-aware consumers could use drawing/object links for selection,
coordination, provenance or regeneration. Generated content needs an explicit
relationship to its origin and to subsequent CAD edits before regeneration can
safely replace it. These are concrete future integration questions, not an
implemented mixed-document editing or cache-container contract.

Preservation retains source information that cannot yet be modeled natively;
external associations refer to semantic information such as IFC objects. These
have different purposes and lifecycles. Each requires a concrete IFCCAD design, but neither is implemented or standardized by a speculative
common extension protocol today. Design them from concrete use cases with
identity, ownership, persistence, unknown-content handling and conformance
requirements. The former IFCPR experiment is available in Git history as
inspiration; it is not an unchanged future IFCCAD payload.

External CAD references such as images or xrefs can be added later. A drawing does not need to embed every future external source.
Future shared identity, incremental contributions and progressive delivery are
opportunities to investigate with IFCX, not promises of a complete CAD workflow
in its current alpha technology. The
[IFCX Core project](https://www.buildingsmart.org/standards/calls-for-participation/ifcx-core-modularisation-call-for-project-sponsorship/)
is still evaluating underlying technology and serialisation choices.

## Success criteria

Drawings can be authored, read, inspected and exchanged without one required
application, without encoding assumptions in conversion, and with inspectable
loss/accuracy evidence. Optimization follows representative measurement and
must not weaken ownership, order, block sharing or semantic fidelity.
Independent implementations and focused validators should be able to use the
language-neutral schemas and conformance material without relying on this Rust
implementation or a specific CAD runtime. Development order and exit criteria
remain in [ROADMAP.md](../ROADMAP.md).
