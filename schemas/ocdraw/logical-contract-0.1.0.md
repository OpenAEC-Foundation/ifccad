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
Layout limits, limits checking and Paper-space linetype scaling are independent
of optional output configuration. A layout may retain `media: {unit,width,height}`
without plot settings, on Model or Paper. Physical medium units exclude unitless;
px identifies raster dimensions. Width/height are positive finite numbers.

Complete `plotSettings` requires media and has `plotUnit`, `page`, `area`, `mapping`,
`output`, `options`; old embedded `plotSettings.media` is rejected. Page rectangles
use unrotated medium units, have positive area and lie within the medium. Physical
media pair with mm/in output; raster media pair with px, without assumed DPI.
Windows and limits use layout-coordinate numbers. Layout area is Paper-only with
fixed scale/offset placement; Limits is Model-only with authored limits.

Paper-coordinate numbers do not inherit the drawing unit. Fixed outputLength /
scopeLength relates them to plot units; absent/Fit/pixel settings establish no fixed
physical meaning. Medium dimensions never rescale, clip or constrain geometry.
`paperSpaceLinetypeScaling` defaults true and sizes viewport patterns in Paper
coordinates; global/entity pattern scales remain independent. See
[layout output](../../docs/layout-output.md) for the complete meanings and CAD limits.

There is exactly one ModelSpace scope and one model Layout selecting it. The
model Layout has tab index zero. Every PaperSpace scope has exactly one paper
Layout. A Layout never selects a block-definition scope. There may be no paper
scopes, no entities, and no Layers. An empty scope has null bounds; a nonempty
scope with complete native geometry has finite enclosing bounds. Opaque content
makes complete bounds unavailable as defined below. Entity IDs are unique across
all scopes and object streams, including opaque entities. Present entity Layer
IDs resolve to a local Layer; an opaque entity may leave its source layer opaque.

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

Each natively interpreted drawable entity stores four independent mode/value pairs: color, opacity,
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

Model windows have positive normalized rectangles within [0,1]. They remain
valid without a current-window selection or drawing view state. Current Model UCS
and active model-window identity are independently optional; omission does not
select World or the first window. An empty selection-only record encodes as absent.
Grid spacing is finite and nonnegative; major frequency is positive. Snap spacing
is finite and positive while enabled and nonnegative while disabled; base and angle
remain finite. An active model window using stored UCS agrees with a supplied
current Model UCS. Paper canvases select unique paper scopes. Their optional saved
frame has finite XYZ center and positive width/height, without normalized bounds;
it does not contribute drawable geometry or bounds. Canvas `useStoredUcs` defaults
to true when absent. Optional current UCS requires a known active context. With
activation enabled, supplied stored/current UCS selections agree in the selected
context. An active Paper viewport belongs to that canvas and has a workspace row.
Viewport-workspace rows select distinct existing Paper-owned viewports and do not
require a canvas snapshot. Dormant records and unknown choices are not filled with
constructor defaults by native reading or writing.

## Preservation

The optional `preservation` collection is encoding-independent source information,
not native geometry or a CAD-runtime object. Its envelope has `version` (initial
value 1), nonzero uint64 `nextRecordId`, `sources` and `records`. Sources/records
are required ordered arrays, possibly empty. Absence means no preservation state;
a present empty collection may retain source contexts and allocation history.
Record IDs are nonzero uint64, distinct from entity IDs, unique, and strictly
below nextRecordId. Allocate with checked arithmetic; never lower the watermark
when deleting. Exact IDs must not be decoded through floating point.

Sources contain unique nonempty `id`, nonempty `provider` and `providerRevision`,
`origin` (`cadDocument`, `dwg`, `dxf`), and optionally nonempty `sourceVersion`.
No file origin is inferred when provenance is unknown. Source identity includes
the drawing identity, source ID and provider-defined source key; equal handles
in separate source contexts do not identify the same content.

Each record has id, a resolving sourceId, nonempty sourceKey, category
(entity/object/table/drawing/layout/shared), role (complete/supplement/shared),
representation (codecTyped/codecOpaque), optional subject, dependencyCoverage
(qualified/conservative/unknown), required bindings/conditions arrays and payload.
Payload contains nonempty schema, positive uint32 version, kind
(adapterSnapshot/rawDwgRecord/rawDxfGroups/rawSection), and owned bytes.
CodecTyped means the source codec understands the type, not that OCDraw does.
No schema-specific interpretation of payload or baseline bytes occurs in core.

Targets are tagged as drawing, entity, layer, linePattern, layout, scope,
blockDefinition or record. Drawing has no ID; other targets carry their domain's
ID. Entity/record IDs are nonzero uint64; layer/pattern/layout/scope/blockDefinition
IDs are uint32 (zero is valid). BlockDefinition uses its scope ID. Target roles
are distinct even when their numeric IDs are equal. Each binding has unique
nonempty slot, nonempty sourceKey and target. Each condition has target, nonempty
predicate, positive uint32 version and baseline bytes. Binding/condition targets
are soft: missing targets and record cycles do not invalidate storage. Eligibility
is evaluated by the adapter from current content, never stored as a valid flag.
A detached record's subject may be missing, including supplements after deletion.

An opaque entity has ordinary drawing-wide id, preservationRecordId, required
visible bool and nullable native layerId/appearance. It has no native geometry,
stored owner or certified bounds. Each opaque ID occurs exactly once in one
scope's entities list alongside native geometry and viewports. Its hard record
link must resolve to a complete entity-category record whose subject is this same
entity ID; a complete record has at most one live opaque entity. Native ID
uniqueness and nextEntityId include opaque entities. No phantom entity is created
for a detached record. Deleting an entity does not delete its preservation record;
deleting a record still linked by a live opaque entity is invalid.

Present native layer/appearance properties use the existing native reference and
scalar rules. The whole native appearance has color, opacity, linePattern and
lineWeight selections plus positive finite linePatternScale. Each selection is
ByLayer, ByBlock or Explicit; only Explicit has a value. Absence retains that
source property in the payload, not a request for inherited/default values. No
native layer or common appearance is synthesized for unsupported source values.
Present properties and visibility are authoritative at restoration; native edits
must not be overwritten by a stale source snapshot. Unknown provider schemas and
predicate versions transport unchanged; unknown core fields or envelope versions
remain unsupported. Generic storage does not certify restoration, rendering,
whole-file fidelity or the mathematical validity of provider geometry.

Initial JSON maps preservation to a closed optional root object; subject and
sourceVersion are omitted when absent and cannot be null. Payload bytes and
condition baselines use canonical padded standard Base64, including the empty
string for empty bytes. Reject whitespace, alternate alphabets, missing/extra
padding and noncanonical pad bits. Validate actual encoded/checked decoded
lengths and stream columns/counts before allocation based on rows; never trust
advertised counts. This selects no configurable size limit or external sidecar.

The registered opaqueEntityStream schema starts at v1. Count and all five columns
id, preservationRecordId, visible, layerId and appearance are required and equal
length. A null layerId/appearance retains the opaque source property. Each nonnull
appearance row is a closed native object with all five properties. The stream
has no owner or geometry column. An empty stream may be absent. Nonempty opaque
content requires its preservation records. Core snapshots and file transport
remain independent of opencadcodec and any provider payload serialization.

## Geometry completeness

Completeness is derived from membership and the acyclic native block-definition
graph, not an authored status field or a source control-point box. Geometrically
empty scopes have null bounds, including scopes containing only empty Text.
A fully available scope has finite bounds under the enclosure or text estimate
rules below. A direct opaque entity, or any
block occurrence reaching one, makes complete geometry bounds unavailable;
that scope must have null bounds. This applies to unused definitions and repeated
or nested occurrences. Unavailable is not empty: an instance of an unavailable
definition cannot use the empty-definition insertion-origin fallback.

Native scalar and geometric/numerical checks still apply in unavailable scopes,
including transformed native leaves inside nested opaque-affected definitions.
Only the comparison against unavailable complete enclosures is omitted. Active
viewport clipping cannot use an opaque boundary. A dormant reference may select
an opaque entity under ordinary same-paper-scope and unique-claim rules. Exporters
must not emit a dangling clip handle or substitute a rectangle if restoration
skips the target.

Explicit recomputation evaluates all native subsets and stages results before
changing any supplied bounds. Complete scopes receive finite enclosures, empty
or unavailable scopes receive null. Structure or native numerical failure leaves
all supplied bounds intact. Removing the last opaque requirement restores ordinary
complete bounds rules; callers recompute before encoding. Encoding validates
and retains supplied bounds instead of silently repairing them.

## Text

Text styles form a separate drawing-local table. IDs are unsigned 32-bit values;
zero is valid. Names are nonempty, NUL-free and unique under the pinned Unicode
full case fold, without trimming or normalization. Unused styles are retained.
`nextTextStyleId` must be positive and exceed every live style ID; deleting a
style does not reduce it. When both table and counter are absent the domain is
empty with watermark 1. A present table requires the counter, even when empty;
a counter without a table retains allocation history. No Standard style is
implicitly created.

Style font requests contain a family or CAD font name, optional big-font name,
and optional face/charset/pitch requests. Big-font requests require a CAD font
name. No font is loaded or substituted by the format. Width factor defaults to
1, oblique angle to 0, vertical and creation mirrors to false. Optional creation
height is positive; last-used height is nonnegative, including explicit zero.
These are creation metadata, not a second application of entity geometry.

Text has its own registered stream, with entityId, layerId, styleId, layout and
content required. Its common identity, appearance, visibility and ordered scope
ownership follow the existing entity rules. Placement uses the same identity
default and physical null-row convention as points. Rotation, oblique angle and
signed thickness default to 0; backward and upsideDown default to false. Native
read axes remain authored axes; the reader does not silently normalize them.

Layout is a closed tagged choice: anchored requires horizontal left/center/right,
vertical baseline/bottom/middle/top, positive height and widthFactor;
wholeTextMiddle requires positive height and widthFactor; aligned requires
positive length and widthFactor; fit requires positive length and height. These
distinct meanings are not collapsed. Content is an ordered sequence of literal
Unicode runs, each with text and underline/overline/strikeThrough flags defaulting
to false. Runs are nonempty; the sequence may be empty. No run merging occurs.
Control codes, raw tabs and line/paragraph separators are invalid in run text;
ordinary Unicode, combining characters and literal CAD-like spellings remain
literal. Dynamic fields and CAD markup are not interpreted by native readers.

`boundsQuality` is an optional producer declaration next to a nonnull scope box.
Missing quality means enclosing; explicit estimated means all extent contributions
are available but at least one is estimated. Neither declaration certifies font
glyph enclosure. Unknown contributions derive partial coverage and require null
bounds; partial cannot be stored as a box quality, and quality without a box is
invalid. Estimation and incomplete coverage propagate through block instances,
including unused definitions. Primitive enclosure and numerical checks remain
mandatory even alongside text estimates or unavailable content. Estimated or
partial results cannot prove a negative spatial query or safe glyph culling.
Explicit recomputation stages both boxes and quality before changing the document.
Empty Text contributes no box. The fontless estimator does not turn an enclosing
producer declaration into independently verified glyph evidence.

MText is a separate registered object stream. Count, entityId, layerId, styleId,
positive height and content columns are required. Its placement/common properties
follow Text; rotation and mirrors have the same defaults. Attachment defaults to
topLeft (nine horizontal/vertical combinations), flow to horizontal (also vertical
and byStyle). Optional characterFormat and paragraphFormat default to no overrides.
Wrap width, columns and background are optional; null rows mean absence only in
these five registered optional columns. Nested fields cannot be null. Content
has at least one paragraph; empty and trailing empty paragraphs are valid.

Character inheritance is style, entity characterFormat, paragraph characterFormat,
then inline characterFormat. Each optional property retains presence; explicit
false is not absence. A font override replaces the whole symbolic request. Colors
select entity, byLayer, byBlock or explicit native color. Height selects relative
positive factor or absolute positive distance; widthFactor and tracking are
positive, obliqueAngle lies strictly between minus/plus pi/2, position selects
bottom/center/top, and the three decorations are independent optional booleans.
Resolved formats are derived, never stored back over authored overrides.

Paragraph inheritance is entity paragraphFormat followed by paragraph overrides.
Alignment selects left/center/right/justified/distributed. Signed finite indent
factors use nominal entity height. Before/after spacing is nonnegative. Line
spacing is exact/atLeast positive distance or multiple positive factor. Tab stops
have positive strictly increasing position factors and left/center/right/decimal
alignment; only decimal has a required single Unicode-scalar separator, excluding
controls and bidi direction controls. Absence inherits; an explicit empty list
resets to the standard 4n grid. That grid does not restart after custom stops.

Inlines are closed run/tab/lineBreak/columnBreak/stack choices. Literal run text
has the Text Unicode/control rules. A stack has fraction/diagonalFraction/tolerance/
decimalTolerance kind, upper/lower literal text not both empty, top/center/bottom
alignment (center default), positive textScale (0.7 default), optional separator
only for decimalTolerance and optional characterFormat. Column breaks require
columns; breaks and tabs are structural inlines, never raw run controls.

Wrapping is optional positive wrapWidth, or columns, never both. Static columns
require count, columnWidth, gutter and columnHeight. Dynamic-auto-height requires
currentColumnCount and shared positive columnHeight. Dynamic-manual-height uses a
nonempty columnHeights list, positive fixed distances and at most one auto entry
at its end; it has no separate count. Width/heights are positive, gutter is
nonnegative, counts are unsigned 32-bit positive values and flowReversed defaults
false. Overflow does not change authored content, capacity or count.

Background fill selects none/color/canvas (none default); padding is nonnegative
absolute distance or relative factor (relative 0.5 default), opacity is within
0..1 (1 default) and frame defaults false. Frame-only backgrounds are valid.
Physical frame stroke must resolve in the owning coordinate domain; medium size
alone is not a Paper scale, and unresolved stroke produces partial coverage.
All derived numeric operations must stay finite; unknown extents do not suppress
numeric checks on neighboring primitives, text or block occurrences.

## Solid Hatch

A Hatch is an ordinary entity with its own stable ID, appearance, ownership and
draw order. One placement defines its local XY plane. It stores nonempty loops,
areaRule (normal/outer/ignore), joinTolerance and a typed fill. This active slice
supports only fill kind solid.

Boundary kinds are closed polyline, full circle, full ellipse and ordered
line/circularArc/ellipticArc edges. In hatchStream nested values, XY coordinates
are two-element numeric arrays. Outgoing bulges include the closing segment;
missing bulges mean zero, while supplied bulges match the vertex count.
Ellipses retain positive major/minor radii and a checked local X-axis; Y is its
CCW perpendicular. Stored curve parameters are not normalized or tessellated.

joinTolerance defaults to the binary64 value of 1e-9 in local coordinate units.
Every ordered edge join, including last-to-first, must be proven within the
stored limit. Zero requires exact agreement. Proven exceedance and incomplete
numerical evidence are distinct failures. Native IO does not snap geometry.

Normal alternates filled/excluded nested regions; Outer keeps the outer band;
Ignore does not subtract inner loops. Base validation does not prove
simplicity, absence of intersections, nesting or filled-area evaluability.
Evaluation remains a viewer/editor responsibility; no generated fill cache
or topology-valid flag is stored.

A loop may reference one exact sourceEntityId: a full Circle/Ellipse or closed
planar Polyline in the same owner. Missing, wrong-kind and wrong-owner sources
fail. IO does not update stored boundaries from sources. Explicit detach keeps
the boundary. Block-occurrence selectors and update reactors are separate work.

One hatchStream has ordinary common and placement columns, optional whole
areaRule/joinTolerance columns with defaults, and typed nested loops/fill.
All supplied columns match count. Unknown nested fields, null source references,
floating-point IDs and unavailable fill variants are invalid.

Stored contours contribute certified conservative bounds without nesting
classification. Preparation remains explicit and atomic. Pattern fills,
gradients, splines and CAD-route qualification are separate development work.
