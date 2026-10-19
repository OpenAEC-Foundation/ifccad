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

- Specify meaning in a language-neutral logical contract before optimizing
  physical encoding. Independent implementations can use the published model.
- Keep geometric entities central and typed. Modelspace, paperspace and shared
  block definitions follow the same ownership and order rules.
- Store layers, layouts, appearance choices and drawing-bound state together.
  Last-opened files and application-specific navigation belong to applications.
- Preserve source meaning natively where supported. Otherwise diagnose a loss
  or reject inconsistent data; numerical accuracy is an explicit hard limit.
- Verify exchange through production readers and real CAD codec roundtrips.
- Make compact, directly openable files an adoption goal. Evaluate opening time
  and memory alongside transfer size; storage compression alone does not imply
  a compact in-memory drawing or efficient partial access.

## Future integration and preservation

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
