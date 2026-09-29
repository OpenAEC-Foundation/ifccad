# Open CAD Drawing 0.1.0 logical contract

This document defines the drawing meaning represented by the OCDraw 0.1.0
registry. `registry-0.1.0.json` lists types, fields, defaults, and references;
`json-mapping-0.1.0.json` maps them to the initial JSON encoding. The JSON
document schema describes its envelope and closed records. A valid drawing must
also satisfy the cross-record rules here. These rules apply to reader and writer
backings through the same semantic validation.

## Document and identity

`header.format` is `open_cad_drawing` and `header.version` is `0.1.0`.
`drawingId` is a stable, nonempty drawing identity independent of file path. The
unit is explicit; `unitless` is valid. `nextEntityId`, `nextLayerId`, and
`nextLayoutId` are greater than every currently allocated ID of their kind.
Allocation advances the watermark, including when the last allocated item is
later deleted. Entity IDs are nonzero 64-bit integers; Layer and Layout IDs are
separate unsigned 32-bit domains in which zero is an ordinary ID. A typed
reference selects its domain. IDs are not inferred from a name, array position,
CAD handle, or IFCX path.

The optional `plotStyleMode` field has the logical default `colorDependent`;
`named` must be explicit. Optional `pointDisplay` and drawing workspace fields
are absent when not authored. A writer may omit fields with registered logical
defaults but must not replace an authored value with a guessed one.

## Definitions, scopes, and entities

`layers` and `layouts` are drawing-level definition collections, not drawable
entity streams. A Layer has its own stable ID, nonempty unique name, flags, and
explicit color, opacity, line pattern, and line weight defaults. A Layout has a
stable ID, nonempty unique name, scope reference, model/paper kind, and unique
contiguous zero-based tab index. Names are compared with the pinned Unicode
17.0 full case fold. Unused definitions remain valid and survive read/write.
Layout limits, plot settings, and linetype scaling keep their existing typed
semantics; an incomplete plot-settings record is invalid.
The plot medium and any plot-window rectangle have positive width and height;
reversed or zero-area printable and window rectangles are invalid even when
their JSON fields individually satisfy the physical schema.

There is exactly one ModelSpace scope and one model Layout selecting it. The
model Layout has tab index zero. Every PaperSpace scope has exactly one paper
Layout. A Layout never selects a block-definition scope. There may be no paper
scopes, no entities, and no Layers. An empty scope has null bounds; a nonempty
scope's finite bounds enclose its exact geometry. Entity IDs are unique across
all scopes and object streams. Every entity's Layer ID resolves to a local
Layer. Draw order and block/viewport references obey the existing IFCDR
geometric and ownership rules, now wholly within one drawing.

For a minimal valid document, the header, one empty ModelSpace scope, one model
Layout and empty streams suffice. Its unit may be `unitless`, the Layers
collection may be omitted, and no workspace selection is implied. A new-file
template may explicitly create Layer 0; a reader does not synthesize it.

## Appearance

Each drawable entity stores four independent mode/value pairs: color, opacity,
line pattern, and line weight. The modes are `ByLayer`, `ByBlock`, or `Explicit`;
an omitted mode means `ByLayer`. `Explicit` requires its matching value, and
other modes forbid that value. The pairs may use different modes on one entity.
The symbolic modes remain stored even when the current effective rendering
matches another entity. A Layer's defaults are explicit values, not another
identified appearance object. There is no Appearance ID or appearance-binding
table. The initial native built-in line pattern is `Continuous`; other pattern
definitions require an additional native contract rather than an unresolved
name.

A viewport's per-Layer override record may independently override color,
opacity, line pattern, and line weight with explicit values. Missing override
fields retain the Layer's defaults. It refers directly to a Layer ID and has
no appearance-override ID. Its frozen flag remains independent.

## Workspace and external meaning

Optional drawing workspace state may select a current local Layer and/or an
active local Layout. A present selection contains at least one of these fields.
Every selected ID resolves within the document, including an unused Layer.
Model windows, paper canvases, viewport workspace records, and named UCS
definitions remain drawing-local and obey their existing typed state rules.
Model-window IDs and named UCS IDs are unique within their collections. The
active model-window ID and each Named UCS selection must resolve locally.
`World` carries no target, `Named` carries only its UCS ID, and `Unnamed`
carries only a valid coordinate frame.
Every UCS definition has a finite elevation and a valid `CoordinateFrame3`
whose stored X and Y axes meet its unit-length and perpendicularity rules.
The application-wide last-opened drawing is outside this contract.

No IFCX node, package descriptor, or IFCPR resource is needed to interpret a
native value. This version has no generic extension field. Unknown core fields
in this version are invalid; an unknown version is unsupported. Source
preservation and IFC relationships may later be optional, separately versioned
extensions, designed from concrete use cases. External CAD assets that affect
native drawing meaning require their own native reference rules.
