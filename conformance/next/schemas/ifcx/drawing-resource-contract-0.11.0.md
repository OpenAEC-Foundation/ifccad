# Drawing resource contract 0.11.0

This candidate is normative alongside ifccad-overlay-0.11.0.json, drawing core
0.2.0 and IFCDR 0.9.0. Its presence does not imply reader/writer support before
the implementation and candidate compatibility matrix are updated together.

## Representation and resource

An openaec:DrawingRepresentation node contains attributes.resource identifying
an IFCDR resource with format openaec.ifcdr, version 0.9.0 and role drawing.
Resource identity is independent of its external URI or inline storage.
Apply the existing resource-source-contract-0.9.0 rules for mutually exclusive
external (URI plus checksum) and inline (content) storage. IDs and node paths
remain opaque; their spelling does not infer scope kind or layout identity.

## Drawing and layout binding

Each Drawing selects its DrawingRepresentation through children.Representation.
Every listed DrawingLayout selects that same representation node, not merely
another node with the same resource ID. Its attributes.scopeId resolves inside
the selected IFCDR resource. A model layout MUST select its ModelSpace; a paper
layout MUST select a PaperSpace. A layout MUST NOT select a BlockDefinition.
Model space is identified by kind, not by an assumed ID of zero.

Missing references and wrong node types are independently invalid. Avoid
cascading equality/kind errors when their reference prerequisites are missing.
The representation, scope and kind checks require package validation beyond
the local JSON Schema shape checks. This slice adds no exclusivity requirement
on layout selection and no layout-presentation cardinality requirements beyond
the existing drawing core. In particular, it does not introduce sheet, viewport,
camera, clipping or plot-setting fields.

There is no stored IFCX backlink in scopeTable and no IFCX BlockDefinition node.
Reverse layout navigation may be indexed from the existing forward bindings.
One instance can reference only definitions in its own IFCDR resource. Scope
renumbering must update IFCDR and IFCX references together; entity IDs persist.

## Numeric and presentation boundaries

Paper coordinates form a separate numeric domain. The future paper-to-physical
mapping must not add a hidden scale to BlockTransform. A layout without plot
settings still has valid geometry; this contract does not infer physical output
size. Model bounds and paper bounds are independent; future viewport bounds are
their paper footprint, not a union with visible model geometry.

Symbolic appearance modes remain symbolic, including through block occurrences.
Drawing-level layer/appearance collection design and effective occurrence-style
resolution remain separate work. No such collections are introduced here.

## Extensions and retired vocabulary

DrawingGeometryRepresentation remains recognized retired vocabulary and is
reported as unsupported, including if unreferenced. attributes.geometry does
not substitute for attributes.resource. Unrelated unknown IFCX node types and
fields remain permitted at existing open extension points. This contract does
not promise to convert or rewrite those extension semantics losslessly.
