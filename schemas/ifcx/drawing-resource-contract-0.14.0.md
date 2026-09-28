# IFCX drawing and workspace contract 0.14.0

The [IFCX overlay](ifccad-overlay-0.14.0.json) selects this contract with
`header.ifccadSchemaVersion: "0.14.0"`. The [drawing core](ifccad-drawing-core-0.5.0.json)
adds the optional Drawing `attributes.workspaceState`. The overlay requires
IFCDR 0.12.0 drawing resources. Older unmarked packages retain their prior
version-selection behavior. `dataVersion` is a caller-owned content version.

At most one `openaec:PackageWorkspaceState` node may occur. When present,
its `activeDrawing` and `activeLayout` attributes are IFCX node paths. They
must resolve to a Drawing and a DrawingLayout listed in that Drawing's
`children.Layouts`. The selected layout must resolve to the selected
Drawing's IFCDR representation and its correct model or paper scope.

A Drawing's optional `attributes.workspaceState` contains `currentLayer`,
an IFCX node path to a Layer listed in that Drawing's `children.Layers`.
The selected Layer need not have an IFCDR binding until a drawable entity
uses it. A stale or foreign Layer path is invalid.

If a package resume node is present, its selected Drawing must have a
workspaceState and its drawing resource must have IFCDR DrawingViewState
covering the selected layout. An unselected Drawing may carry either value
independently. With no package resume node, no active drawing or layout is
asserted. Incomplete present state is invalid and readers do not choose a
replacement.

The schema checks local node shape. Cross-node references, uniqueness and
IFCDR scope ownership are strict package-validation rules.
