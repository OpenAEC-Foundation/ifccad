# Open CAD Drawing 0.1.0 logical contract

This document defines the drawing meaning represented by the OCDraw 0.1.0
registry. `registry-0.1.0.json` lists types, fields, defaults, and references;
`json-mapping-0.1.0.json` maps them to the initial JSON encoding. The JSON
document schema describes its envelope and closed records. A valid drawing must
also satisfy the cross-record rules here. These rules apply to reader and writer
backings through the same semantic validation.

Schema IDs start at `v1` within OCDraw; they do not inherit IFCDR's schema
revision numbers. These IDs and the drawing version remain provisional
until OCDraw 0.1.0 is fixed as the first supported version.

## Document and identity

`header.format` is `open_cad_drawing` and `header.version` is `0.1.0`.
`drawingId` is a stable, nonempty drawing identity independent of file path. The
unit is explicit; `unitless` is valid. `nextEntityId`, `nextLayerId`, `nextLinePatternId`, and
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
Layer.

Every scope has a required ordered `entities` list containing only entity IDs.
The list is authoritative for both ownership and draw order. Every entity must
occur exactly once in exactly one scope list; every listed ID must resolve to a
central typed entity record. An empty scope has `entities: []`. The entity record
has no owner field. Implementations may derive an inverse ownership index.
Entity IDs remain drawing-wide and independent of ownership: moving an entity
between scopes does not allocate a new ID.

This applies equally to ModelSpace, PaperSpace, and block-definition scopes.
Each block definition selects exactly one block scope. A block instance belongs
to the scope listing its ID and separately references its shared definition.
Nested definition references must be acyclic. Viewport entities belong to paper
scopes and view the model scope. A block definition does not copy its entities
into instances. All records stay in central typed streams; there are no streams
per block. The JSON mapping places each list directly in its scope record,
without offsets or counts. A later physical encoding may pack the lists into
one shared ID buffer with ranges without changing this logical model.

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
table. Line pattern values are typed local definition IDs, not unresolved names.
`Continuous` is a named empty definition, without a fixed ID.

A viewport's per-Layer override record may independently override color,
opacity, line pattern, and line weight with explicit values. Missing override
fields retain the Layer's defaults. It refers directly to a Layer ID and has
no appearance-override ID. Its frozen flag remains independent.

## Named line patterns

`linePatterns` is a drawing-local definition table. Each record has a unique
unsigned 32-bit `id` (zero is valid), a nonempty unique `name`, optional
`description`, and required ordered `pattern` sequence. Names use the pinned
Unicode full case fold. Unused records and differently named empty records are
retained. Omission of the table means no definitions; readers synthesize none.
`header.nextLinePatternId` is greater than every allocated pattern ID and is not
reduced when deleting a record. No ID is implied by a table position or CAD handle.

Positive finite pattern numbers are strokes, negative numbers are gaps, and zero
is a dot, in drawing units. An empty pattern is continuous. A nonempty pattern
has at least two entries, a nonnegative first entry, and a finite positive sum
of absolute lengths. Preserve entry order and values without merging elements.
The name `Continuous`, under the same case fold, requires an empty sequence;
`ByLayer` and `ByBlock` are reserved selection names and cannot name definitions.

Simple patterns use A-type endpoint alignment: open lines/arcs begin and end
with a stroke, adjusting endpoint strokes as needed; a path too short for a
pattern may appear continuous. Closed curves have no open endpoint. This contract
does not introduce a selectable alternative alignment or phase.

Layers require `linePatternId`. Entities retain `linePatternMode` (omission means
`ByLayer`) and a nullable `linePatternId`: `Explicit` requires a resolving local
ID; inherited modes forbid a nonnull ID. Viewport layer overrides may have an
explicit pattern ID; absence retains the layer value. All references are local.

Drawing and entity `linePatternScale` are finite positive values, default 1.
Base lengths are multiplied by drawing and entity scale without rewriting shared
definitions. Entity scale is independent of inheritance mode. Layout paper-space
scaling retains its existing meaning. Model-tab annotation scaling and current
creation defaults are outside this native slice.

Planar and spatial polylines have `linePatternGeneration`: `perSegment` (default)
restarts the pattern on each edge; `continuous` traverses the path across vertices,
including a closed path's closing edge. This changes appearance, not geometry or
bounds. No text or shape payloads are supported in native patterns.

The JSON table is an array of records; pattern numbers are signed JSON numbers.
Entity properties retain column arrays with nulls for absent explicit IDs.
Missing scale/generation columns select logical defaults. Old string-valued
`linePattern` fields are unknown core fields and rejected in this provisional
contract. See the versioned registry and JSON mapping for all field shapes.

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

### Stored planar polyline bulges

Each planar vertex retains its signed bulge. The bulge describes the segment
from that vertex to the next vertex. For an open polyline the final bulge is
dormant: it is retained as drawing information and does not contribute a segment
or change the geometric enclosure. Closing the polyline activates that value.

## Initial JSON encoding

Polyline coordinate pools (`x`, `y`, and spatial `z`) use signed JSON numbers,
including fractional values. Pool offsets and vertex counts remain
non-negative integers.

The document's `header.version` selects the JSON schema, registry, and mapping.
The required `streams` object may be empty. Each present property (for example
`lineStream`) selects one versioned stream schema in that mapping. Streams not
needed by the drawing may be omitted; there is no stream directory in the file.
Each payload declares its row count and contains columns defined by the mapping.
Required columns must be present and have the declared length. Optional columns
use the registered logical default or absence. Pooled polyline vertex columns
and viewport-layer child ranges obey their separately defined packing rules.
Unknown streams and columns are invalid in this version. A future physical
index is an encoding detail and does not duplicate logical scope ownership.

## Native geometry

All coordinates and stored numeric values are finite binary64 values. Scope
bounds are finite ordered XYZ intervals. Line endpoints and spatial polyline
vertices are directly in owner-scope coordinates. Spatial polylines have at
least two vertices and no placement or bulges.

`CoordinateFrame3` stores an origin and X/Y axes. Each axis's exact squared
length differs from one by at most the binary64 value nearest `1e-12`; the
absolute exact X/Y dot product is at most that value. Stored axes are preserved,
not repaired. A local planar point (u,v) maps to origin + u X + v Y; the normal
is the stored X cross Y. A Point's origin defines its position; the rest of its
frame retains presentation orientation.

Circle and Arc use positive radius and a coordinate frame. Ellipse and
EllipseArc use positive semi-major and semi-minor radii, with minor no greater
than major. Their local parameterization is (a cos(t), b sin(t)), where a=b for
circles. Arc kinds store finite start and signed nonzero sweep in radians,
with absolute sweep strictly less than binary64 TAU. Full curves have their
own stream kinds. No implicit arc direction change or radius approximation is
part of native readback. Planar polylines retain at least two local XY vertices,
closure and signed bulges (tan of one quarter of the included segment angle).

A block instance subtracts the definition base point from child coordinates,
then applies signed XYZ scale, local rotation in radians and its placement.
The local Z direction is the placement X cross Y. Scale components are finite
and nonzero, including negative and very small values. Uniform scaling means
exact signed equality of all three components. A Uniform definition rejects
instances that do not satisfy it. Nested transforms retain their full affine
composition; nonuniform transforms are not decomposed into guessed rotations.
The owning scope bounds enclose the instance's transformed definition geometry.

## View and workspace validation

Views require finite center, target, direction, positive height and finite twist.
Direction has a positive finite norm. Orthographic lens length may be absent or
nonnegative; perspective requires a positive lens length. Stored clip distances
are finite. AtDistance requires a distance; active back clipping lies behind the
active front clip (AtCamera places the front at the direction norm). Paper
canvas views are orthographic.

A viewport has a positive finite paper frame with a finite exact enclosure.
An enabled paper clip requires an existing boundary entity in the same paper
scope. Any stored boundary reference, including a dormant one, is unique to one
viewport and resolves in that scope. An active boundary is a Circle, a full
Ellipse (not EllipseArc), or a closed PlanarPolyline. All-straight polylines
require at least three distinct XY vertices; a polyline with an active nonzero
bulge requires at least two. Signed zero does not distinguish vertices. Every
segment, including the closing segment, must be valid. The entire represented
curve must lie in paper Z=0 within the frame's finite outward enclosure of
center plus/minus half dimensions; touching that enclosure is allowed. For
circles, ellipses and curved polyline segments, placement origin Z and both
in-plane direction Z components must be exactly zero. Straight segments use
exact placed endpoint checks. In-plane rotation and either plane orientation
are allowed. Conservative geometry bounds crossing a frame edge are not proof
that the curve crosses it. Do not approximate curves or apply a tolerance
epsilon to clip validation. Dormant references need not satisfy active geometry
eligibility. Self-intersections and zero signed area do not themselves invalidate
a boundary; no new stored fill rule is introduced.
Per-layer overrides select distinct existing layers and carry at least one
effective change. Custom shading quality requires dpi 100..32767; other quality
modes do not carry dpi.

Model windows require drawing view state and positive normalized rectangles
within [0,1]. Grid spacing is finite and nonnegative; major frequency is positive.
Snap spacing is finite and positive, with finite base and angle. An active model
window using its stored UCS must agree with the current model UCS. Paper canvases
select unique paper scopes. An active canvas's stored/current UCS agree. An active
paper viewport belongs to its canvas and has a viewport-workspace row; if it uses
stored UCS that selection agrees with the canvas current UCS. Viewport-workspace
rows select distinct existing paper viewports whose scope has a canvas.
