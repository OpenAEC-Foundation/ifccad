# Open CAD Drawing vision

OCDraw describes CAD drawings as an open, application-independent information
model and exchange format. Drawings are useful deliverables in their own right,
including CAD-authored work and geometry generated from building models. A
standalone drawing must not depend on an IFC project graph to be understood.

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

## Future integration and preservation

Preservation retains source information that cannot yet be modeled natively;
external associations refer to semantic information such as IFC objects. These
have different purposes and lifecycles. Both might eventually fit extensible
OCDraw records, but neither is implemented or standardized by a speculative
common extension protocol today. Design them from concrete use cases with
identity, ownership, persistence, unknown-content handling and conformance
requirements. The former IFCPR experiment is available in Git history as
inspiration; it is not an unchanged future OCDraw payload.

External CAD references such as images or xrefs can be added later. Standalone
means opening without IFCX; it does not require embedding all future external
sources in one file. IFCX-native CAD can remain an independent experimental
route in this repository.

## Success criteria

Drawings can be authored, read, inspected and exchanged without one required
application, without encoding assumptions in conversion, and with inspectable
loss/accuracy evidence. Optimization follows representative measurement and
must not weaken ownership, order, block sharing or semantic fidelity.
