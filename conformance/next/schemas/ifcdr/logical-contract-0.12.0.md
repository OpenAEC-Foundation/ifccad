# IFCDR 0.12.0 logical contract

This candidate extends the [0.11.0 logical contract](logical-contract-0.11.0.md).
The [registry](registry-0.12.0.json) names the logical values and collections;
the [JSON mapping](json-mapping-0.12.0.json) defines one reference encoding.
All unchanged 0.11.0 invariants continue to apply.

## Coordinate frames

`coordinateFrame3` replaces the logical name `planePlacement`. It has an
origin and orthonormal X and Y unit axes, with directed Z = X × Y. The three
components are finite, and a writer does not normalize supplied axes. Entity
fields still named `placement` retain their existing geometry semantics.

## Workspace values

The optional resource-level `drawingViewState` contains the current model
UCS selection and the ID of the active model window. If present, it requires
at least one model-window row. Absence asserts no portable view state.

`ucsDefinitionTable` contains named definitions. Each has a resource-local
`ucsId`, nonempty name, frame and finite elevation. IDs and Unicode 17.0
full-case-folded names are unique within the resource. Definitions may be
unused. World is implicit, with no definition row.

A `ucsSelection` has exactly one form: `World` has neither ID nor frame;
`Named` has an ID resolving to a definition in the same resource and no
inline frame; `Unnamed` has an inline frame and no ID. An unnamed selection
is saved state and survives read/write.

`modelWindowTable` contains ordered model-editor windows. Each row has a
unique stable `modelWindowId`, a normalized rectangle, view, positive
aspect ratio, render mode, grid and grid snap values, a stored UCS selection
and an activation flag. Rectangles have finite coordinates in [0,1], positive
width and height, and may touch or overlap. The active ID in
`drawingViewState` must resolve. These rows show one ModelSpace and are not
drawable entities or geometry scopes.

`paperCanvasTable` contains at most one editor canvas for each PaperSpace
scope. Its orthographic view is in that paper coordinate domain. The row
stores grid, snap, stored and current UCS selections, and an active context:
the canvas or a paper viewport entity in the same scope.
`viewportWorkspaceTable` may contain one row per authored paper viewport.
Its `viewportEntityId` resolves to an actual Viewport in a PaperSpace scope
that has a canvas row. The row stores grid, snap, a stored UCS and whether
activation applies it. An inactive viewport without a row asserts no
portable editor-aid state.

## Workspace references

All IDs resolve in the resource where they occur. No missing ID is replaced
by World, another window or another viewport. Deleting a referenced row or
entity requires an atomic update to the workspace state. Viewport workspace
rows do not duplicate the scope ID supplied by their Viewport entity.

## Workspace values and activation

Grid spacing is nonnegative in each axis. Zero means follow the corresponding
positive snap spacing. The major-line frequency is positive. Snap spacing is
strictly positive in both axes; its base and angle are finite. Disabled grid
and snap retain their settings as dormant authored values.

Views retain the existing ViewDefinition rules: finite members, nonzero
direction, positive height and valid clip/projection combinations. Paper
canvas views are orthographic. The model-window aspect ratio is finite and
positive.

The current model UCS is a saved snapshot. If the active model window applies
its stored UCS, that selection must equal the current snapshot. Otherwise the
window retains its stored selection without applying it. A paper canvas
selected as active must have stored UCS equal to current UCS. An active paper
viewport that applies its stored UCS must equal the canvas row's current UCS;
one configured to keep current may differ. Inactive contexts retain their
own UCS values.

Named UCS values are drawing-local definitions, interpreted in the selected
model or paper scope's coordinate domain. No implicit model-to-paper transform
is applied. This use-context rule also applies to unnamed frames.

These logical rules apply equally to reader and writer backings. The JSON
mapping may reject malformed physical shapes and ranges but does not replace
semantic validation.
