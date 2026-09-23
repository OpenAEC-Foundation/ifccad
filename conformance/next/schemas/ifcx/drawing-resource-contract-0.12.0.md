# Drawing resource contract 0.12.0

This candidate is normative alongside ifccad-overlay-0.12.0.json, drawing core
0.3.0 and IFCDR 0.10.0. Its presence does not imply reader/writer support before
the implementation and candidate compatibility matrix are updated together.

## Representation and resource

An openaec:DrawingRepresentation node contains attributes.resource identifying
an IFCDR resource with format openaec.ifcdr, version 0.10.0 and role drawing.
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
the local JSON Schema shape checks. Exactly one Model layout is required and
MUST be first in Drawing.children.Layouts. Each PaperSpace scope in the
resource MUST be selected by exactly one Paper layout; no two Paper layouts
may select the same scope. Layout names are unique within the Drawing after
Unicode 17.0 full case folding (without Unicode normalization).

There is no stored IFCX backlink in scopeTable and no IFCX BlockDefinition node.
Reverse layout navigation may be indexed from the existing forward bindings.
One instance can reference only definitions in its own IFCDR resource. Scope
renumbering must update IFCDR and IFCX references together; entity IDs persist.

## Numeric and presentation boundaries

Paper coordinates form a separate numeric domain. Plot mapping MUST NOT add a
hidden scale to BlockTransform or rewrite paper geometry. A layout without
plotSettings still has valid geometry; no physical output size is inferred.
Model and paper scope bounds are independent. A viewport contributes its paper
frame footprint, not projected model geometry, to paper bounds.

Symbolic appearance modes remain symbolic, including through block occurrences.
Viewport-effective style inheritance is specified in the IFCDR 0.10.0 logical
contract; the package does not materialize occurrence styles.

## Drawing lists and shared definitions

Each Drawing has required children.Layers and children.Appearances arrays,
which may be empty and may contain unused nodes. Nodes may be listed by
multiple Drawings; membership is not exclusive ownership. Layer names are
unique per Drawing under Unicode 17.0 full case folding, without normalization;
Appearance names may repeat. All IFCDR layerBinding IFCX targets for a Drawing
MUST occur in its Layers list. All non-null appearanceBinding IFCX targets and
all listed Layers' default appearance targets MUST occur in its Appearances
list. These are one-way closure rules. Every listed reference resolves to the
right node type. Shared nodes have one value; drawing-specific changes need
distinct nodes.

Layer visible is On/Off, frozen is global Freeze, locked restricts editing,
plottable gates print output and frozenInNewViewports is only a new-viewport
authoring default. All five are explicit saved booleans. Thaw is frozen=false,
not another field. IFCX Appearance remains a complete explicit value; a
viewport's partial patch is an IFCDR appearanceOverride table row, not an IFCX
Appearance node.

## Layout limits and effective plotting

Layout limits are an optional authored Rect2 in the selected scope domain,
not scope bounds. limitsChecking is an explicit editor setting; it never
invalidates existing geometry. paperSpaceLinetypeScaling retains per-layout
linetype scaling intent. A present plotSettings is one complete inline
effective value for that layout, never a reference to reusable PageSetup.
Media dimensions and printable area are in the declared mm/in/px physical
unit, before rotation; device/media names are hints. An unconfigured zero-size
CAD medium maps to absent plotSettings, not an invalid present value.

PlotArea.Layout is valid only for paper and requires Fixed scale and Offset
placement. PlotArea.Limits is valid only for model and requires authored
limits. Extents use the selected scope's geometric XY bounds; a paper viewport
contributes only its frame. Window coordinates are in that scope domain.
Display and NamedView are not present in this version. FitToArea uses the
printable rectangle; Fixed stores positive outputLength in media units and
positive scopeLength in scope coordinates. Plot styles store Drawing mode,
per-layout apply switch and optional table name; CTB/STB table contents are
not native. ShadedPlot and plot options have the closed values and conditional
fields in drawing core 0.3.0. Unknown values or partial present settings are
invalid. A missing active external style table is a conversion/plot fidelity
gap, not proof that the package itself is structurally invalid.

Viewport entities belong to the paper scope selected by a layout; layouts do
not store a second viewport list. The resource's unique ModelSpace is selected
through each viewport's viewScopeId. PlotViewportBorders controls border
printing only; it never disables the view or its clipping boundary.

## Extensions and retired vocabulary

DrawingGeometryRepresentation remains recognized retired vocabulary and is
reported as unsupported, including if unreferenced. attributes.geometry does
not substitute for attributes.resource. Unrelated unknown IFCX node types and
fields remain permitted at existing open extension points. This contract does
not promise to convert or rewrite those extension semantics losslessly.
