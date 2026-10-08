# Experimental IFCCAD profile 0.1.0

IFCCAD is an opt-in, standalone IFCX alpha experiment, independent of the OCDraw drawing contract. Generic IFCX content may coexist with these CAD nodes. A reader claiming this CAD profile must enforce the rules below after composing fragments for equal paths. Project-owned API and schema file names use IFCCAD. The underlying IFCX syntax, `.ifcx` serialization, `ifccad::` schema namespace and profile URI retain their meaning. Released historical conformance collections remain unchanged.

## File and identity

The file has IFCX `header`, `imports`, `schemas`, and `data`. `header.ifcxVersion` is `ifcx_alpha`. `header.dataVersion` belongs to the dataset. The drawing value's `profileVersion` is `0.1.0` and identifies this provisional CAD proof. This alpha contract can be revised without increasing that number; it does not promise compatibility across experimental reader revisions. The recommended drawing imports `urn:example:ifccad:0.1.0`, whose schema-only IFCX document is `ifccad-profile-0.1.0.ifcx`, and leaves drawing-local `schemas` empty. The `example` URN namespace is registered for documentation and experimentation; this identifier is intentionally temporary and must be replaced by an organization-controlled published URI before production exchange. During this experiment, the reader revision determines the schema definition for this URI; the identifier does not yet promise an immutable published schema. The strict reader resolves this one experimental URI from its bundled profile, without network access. Alternatively, a drawing may inline exactly matching definitions from that module's `schemas`; a missing import and missing local definitions fail. This is a bounded offline resolver, not general IFCX import resolution. Conditional requirements and cross-node constraints in this document exceed IFCX's per-attribute descriptions.

The complete path is a node identity. The profile uses `/cad/dN` for one drawing, `/cad/dN/layout/N` for Model and Paper layouts, `/cad/dN/layer/N` for layers, `/cad/dN/block/N` for block definitions, `/cad/dN/linePattern/N` for named pattern definitions, and `/cad/dN/eN` for drawable entities. Model and Paper layouts share one layout-ID domain. `N` is a canonical base-10 `uint64` with no sign or leading zero; entity IDs are unique within the drawing and are not repeated as attributes. USD-style angle brackets are not part of these paths. Path spelling is an experimental CAD convention, not a claim that IFCX requires UUIDs or these paths. References use the same complete path strings.

The JSON writer displays `path` first in each node, followed by `children` where present and then `attributes`. JSON member order does not affect node meaning; numeric child *keys* carry CAD draw order.

### Persistent allocation state

The drawing attribute requires `nextEntityId`, `nextLayerId`, `nextLayoutId`,
`nextBlockId` and `nextLinePatternId`. They are IFCX `Integer` values checked
as exact uint64 watermarks by the profile. Each must exceed every currently
present ID in its domain. Entities across Model, Paper and block contents
share the entity counter; Model and Paper layouts share the layout counter.
Layers, block definitions and patterns have independent domains. New empty
domains start at 1; existing canonical ID 0 remains valid. Drawing identity
is caller supplied; there is no drawing-ID counter.

Native reading/writing preserves supplied IDs and watermarks. Editing geometry,
renaming, reordering or changing ownership within the drawing preserves paths.
A copy is a new object and receives a new ID. Allocation advances with checked
arithmetic; deletion or an unused reservation never lowers the watermark.
The counter may equal `uint64::MAX`, but allocation then fails without mutation;
an object with that ID cannot satisfy a greater uint64 watermark. IDs are never
wrapped, compacted or silently reassigned. Exact integer persistence must not
pass through floating point, including above JavaScript's safe integer range.

These are CAD-profile rules, not an IFCX concurrent allocator. Validation is
performed on the final composed drawing. A later `ifccad::drawing` replaces
the whole attribute, so updates must supply the complete object; counters are
not merged by maximum. A snapshot can check current IDs but cannot prove that
a historically deleted ID was never reused or that a still-valid counter was
never lowered. Maintaining that history is an editor responsibility.

Missing counters fail the current strict experimental reader; they are not
inferred on load. Initializing an older file from current maxima cannot recover
deleted-ID history and needs an explicit allocation baseline. The experimental
URI/version remains 0.1.0 without an immutable compatibility promise.
DWG/DXF import initializes fresh allocation state and retains source associations
only through conversion mappings, without promising persistent native identity
through CAD serialization. See the [allocation design](../../docs/experiments/ifccad-id-management.md).

By default, multiple `data` fragments with the same path compose in file order. Their `children`, `inherits`, and `attributes` objects merge by key; for a repeated key, the later value replaces the entire earlier value, including a geometry or appearance attribute object. Other repeated node fields are replaced as whole values. A `null` inheritance value removes that key during composition, while a `null` child or attribute value remains a marker in the composed node. This matches the observed upstream flattening behavior; deletion during expanded graph loading and attribute deletion are not defined by this CAD profile. The reader also exposes a diagnostic `RejectConflicts` policy: it accepts disjoint and identical fragments but rejects differing repeated values, including on foreign IFCX nodes. That stricter option is a development check, not a claim that IFCX forbids overwrites. Duplicate JSON object keys within one object are errors under either policy. The reader validates CAD roles, references, geometry, and draw order on the final composed nodes under either policy. Unknown IFCX nodes and unknown non-CAD attributes are retained under the same selected composition rule. The CAD profile does not use `inherits` for blocks or appearance. A foreign IFCX node may refer to a CAD entity without acquiring ownership.

## Scope and order

The drawing has named `children` referring to exactly one Model layout, zero or more Paper layouts, all layers, all line-pattern definitions (including unused ones), and all block definitions. Pattern membership is unordered; canonical typed collection order follows lexical paths, independently of element order. Layouts share the drawing's layers and block definitions. Layout child names identify membership, while required `tabIndex` values encode presentation order. Model has index 0; Paper indices are unique and contiguous from 1 through the number of Paper layouts. Reader and writer normalize Paper collections by tab index, independently of IDs. Reordering changes indices without changing identity or allocation watermarks. Paper names are nonblank and unique under Unicode 17 full case folding, including the implicit name `Model`, without trimming or normalization. Each layout and block definition uses `children` keys `"0"`, `"1"`, … as draw positions; keys must be canonical and contiguous from zero. Empty scopes are allowed. Values are paths to distinct drawable entities. Numeric order is authoritative independent of JSON member order. An entity is a numeric child of exactly one layout or block definition, and every `ifccad::entity` node has such an owner. Reordering changes child keys but not entity paths. Other named IFCX children may coexist but confer no CAD ownership. There is no separate draw-order or `scope` attribute. A block instance is one ordered entity in its owner's children; it references one definition by its `definition` field. All definition targets must resolve and the definition-reference graph must be acyclic, including unused definitions.

## Attributes

| Attribute | Required value / role |
| --- | --- |
| `ifccad::drawing` | `profileVersion`, `lengthUnit`, allocation watermarks; optional positive finite `linePatternScale` defaults to 1; optional `plotStyleMode` defaults colorDependent, alternatively named; one drawing node |
| `ifccad::layout` | Model `kind`, tabIndex 0 or Paper kind/name/contiguous tabIndex; optional media, limits, limitsChecking (false), paperSpaceLinetypeScaling (true), complete plotSettings and bounds. Model forbids name; old paper/coordinate lengthUnit fields are rejected. |
| `ifccad::linePattern` | nonempty `name`, optional `description`, ordered signed-real `pattern` array; drawing-owned definition |
| `ifccad::layer` | `name`, direct concrete `appearance` values |
| `ifccad::blockDefinition` | `name`, `basePoint` XYZ, `insertionUnit` |
| `ifccad::entity` | resolvable drawing-local `layer`, four property modes in `appearance`; optional positive finite `linePatternScale` defaults to 1 |
| `ifccad::geom::lineSegment` | `start` and `end` XYZ; finite segment, unlike unbounded IFC4 `IfcLine` |
| `ifccad::geom::point` | empty object plus required placement; its origin is the position and its axes retain presentation orientation |
| `ifccad::geom::planarPolyline` | at least two XY `vertices`, `closed`, optional outgoing `bulges` array plus placement; optional `linePatternGeneration` defaults to `perSegment`, alternatively `continuous` |
| `ifccad::geom::spatialPolyline` | at least two direct owning-scope XYZ vertices, closure and optional pattern generation; placement and bulges are forbidden |
| `ifccad::geom::circle` | positive finite `radius`, plus `ifccad::geom::placement` |
| `ifccad::geom::arc` | positive radius, finite radian `startParameter` and signed `sweepParameter`, plus placement |
| `ifccad::geom::ellipse` | positive `semiMajorRadius >= semiMinorRadius`, plus placement |
| `ifccad::geom::ellipseArc` | ellipse fields plus finite radian start and signed sweep, plus placement |
| `ifccad::geom::placement` | finite XYZ `origin`, `xAxis`, `yAxis`; valid orthonormal right-handed frame under the shared OCDraw geometric predicate |
| `ifccad::blockInstance` | definition path and transform with placement, finite rotation in radians, nonzero finite XYZ scale |
| `ifccad::viewport` | same-drawing Model reference, Paper frame, camera/view, render/display state, Paper clip and frozen-layer references; Paper ownership only |

Every owned drawable has `ifccad::entity` and exactly one of the ten drawable payload attributes. A viewport has no separate geometry or placement payload. Unsupported `ifccad::geom::*` payloads fail explicitly. An independently used `ifccad::geom::circle` on a non-CAD IFCX node is not thereby a CAD entity. Coordinates use a fixed right-handed local XYZ convention; there is no implicit world alignment. The 25 length-unit tokens match the current OCDraw registry. Model and block-definition coordinates, including definition base points, use the drawing's length unit. Paper layouts have independent numerical coordinate domains, without a physical
coordinate-unit declaration. Optional `media: {unit,width,height}` retains physical
registry dimensions or raster px; unitless media, partial/null/unknown fields and
nonpositive/nonfinite dimensions are invalid. Media are valid on Model and Paper,
including without plot settings, and never establish coordinate meaning or bounds.

Complete optional plotSettings has plotUnit (mm/in/px), page (printableArea,
rotation and name hints), area, mapping, output and options. A medium is required.
Printable rectangles have positive area inside the unrotated medium; physical
media pair with mm/in output and raster media with px. Fixed scale relates
coordinate length to output length without rewriting geometry or transforms.
Fit/absent/pixel output provides no fixed physical interpretation. Layout area
requires Paper with fixed/offset mapping; Limits requires Model and authored limits.
Paper-space linetype scaling is layout-local and independent of plot presence.
Unknown fields and removed spellings are rejected; no old-file synthesis is added.
See [layout output](../../docs/layout-output.md) for output choices and exchange limits.


A block definition's insertion unit records intent but does not silently scale coordinates. Transform evaluation subtracts the definition base point, applies stored scale and rotation, then applies placement. For a paper-owned instance, its scale maps drawing-coordinate numbers into paper-coordinate numbers; any unit conversion must be included explicitly. For example, the [paper-layout fixture](../../examples/ifccad/hello-paper-layouts.ifcx) has centimetre model/block coordinates and an A3 sheet in millimetres, with a paper instance scale of `[10, 10, 10]`. A nested instance inside a definition stays in drawing units. Reading and writing never normalize or rewrite these transforms; present scope bounds require conservative transform evaluation for validation.

The local circle shape aims at the analytic meaning of IFC4 `IfcCircle`; the namespace remains local until an official IFCX geometry attribute has an exact roundtrip contract. A line segment is not equivalent to unbounded `IfcLine`; planar polyline bulges are supported; widths and fitted/spline segments remain outside this profile. The separate `placement` attribute can later be reused with a published geometry vocabulary if equivalent.

## Paper viewports

`ifccad::viewport` is an ordinary drawable with the drawing-wide entity identity,
layer and appearance. It belongs to one Paper layout; Model and block ownership
are invalid. Its required `model` path refers to the drawing's unique Model
layout, not a block definition or another Paper layout. Required fields are
`frame`, `view`, `renderMode`, `viewEnabled`, `viewLocked`, `visible`, `paperClip`
and `frozenLayers`. Unknown fields at every nested level are rejected. Optional
fields are omitted when absent; explicit null is invalid. Composition replaces
the whole viewport attribute, including omitted optional fields, before validation.

`frame` has XY `center` and positive `width`/`height` in owning Paper coordinate
units. Center and dimensions are finite, and the exact rectangle enclosure must
remain inside the finite binary64 range. Physical media neither rescale nor bound
the frame. Unsized Paper layouts without a fixed physical output mapping remain valid.

`view` has XY DCS `center`, XYZ Model `target`, a target-to-camera XYZ `direction`,
positive `height`, finite `twist` in radians, `projection`, `frontClip`, and
`backClip`; `lensLengthMm` is optional. Center, target, direction and height use
Model units. Direction is preserved without normalization and its exact Euclidean
norm must be positive and within the finite binary64 range. Projection is
`Orthographic` or `Perspective`. Perspective requires positive finite lens length
in millimetres regardless of Model or Paper units; Orthographic permits an absent
or finite nonnegative dormant lens. No camera values are inferred from media.

Each depth clip has `mode` and optional finite signed `distance`. Modes are
`Disabled`, `AtCamera`, and `AtDistance`; back clipping disallows `AtCamera`.
`AtDistance` requires a distance measured from the target along the view direction,
in Model units. Dormant distances remain stored when disabled or when the front
plane is at the camera. If both planes are active, the back distance must be
strictly less than the front distance; `AtCamera` uses the direction norm.
No near-zero tolerance or implicit clamping is applied to these rules.

Render modes are `TwoDimensional`, `Wireframe`, `HiddenLine`,
`FlatShadedWithoutEdges`, `FlatShadedWithEdges`, `SmoothShadedWithoutEdges`, and
`SmoothShadedWithEdges`. Visibility of the Paper viewport entity, enabled Model
view and zoom locking are independent booleans. Storing these values does not
implement a viewport renderer or certify application-specific print appearance.

`paperClip` has required `enabled` and optional complete `boundary` entity path.
Enabled clipping requires a boundary. A stored boundary, including when disabled,
must exist in the same Paper owner, must not itself be a viewport, and may be
claimed by only one viewport. Dormant boundary geometry is otherwise unrestricted.
An active boundary is a Circle, full Ellipse or closed planar polyline with
straight/bulged segments. All-straight contours need three distinct XY vertices;
a contour with active nonzero bulges needs two, permitting two-semicircle contours.
Signed zero does not distinguish vertices. Standalone arcs, elliptic arcs,
spatial polylines, points and block instances are not active clip families.
The whole represented curve must lie in Paper Z=0 inside the frame's finite
outward binary64 enclosure of center plus/minus half dimensions. Tangency is
allowed. Curved placements require origin and both axes' Z components exactly
zero; straight segments use exact placed endpoint checks. Valid rotated and
reflected placements, concave paths, self-intersections and zero signed area
are permitted without adding a new stored fill rule. Analytic curve extrema
are checked; an oversized conservative bounding box is not proof of escape.
No sampling, projection or conversion-tolerance epsilon substitutes for these
rules. Dormant references retain their independent ownership/exclusivity rules
without active shape/frame eligibility.

`frozenLayers` is a set of complete same-drawing layer paths. Every target must
exist and occur once. Native writing sorts by numeric uint64 ID, without passing
through floating point; input array order is not semantic. These are viewport
layer overrides, not global layer visibility changes.

The [viewport example](../../examples/ifccad/hello-viewports.ifcx) and
[candidate conformance cases](../../conformance/next/ifccad/README.md)
exercise strict reader/writer behavior. CAD conversion has separate coverage and
dependency boundaries documented by the companion converter.

## CAD appearance

Layer appearance directly contains `color` as `#RRGGBB`, `opacity` in `[0,1]`, `linePattern` as a same-drawing pattern-definition path, and nonnegative `lineWeight` in millimetres. Each entity has these four properties independently as `{ "mode": "ByLayer" }`, `{ "mode": "ByBlock" }`, or `{ "mode": "Explicit", "value": ... }`. Modes survive readback; no appearance definition or binding node exists. For an explicit entity pattern, `value` is that same complete path; inherited modes must have no value. The typed API uses `IfccadLinePatternId(u64)`. Literal names such as `Continuous` are not references and are rejected.

The stored modes retain ordinary CAD intent per occurrence: Explicit supplies its own value, ByLayer uses the effective layer, and ByBlock defers to the containing block instance's corresponding property. An entity on the layer *named* `0` inside a block definition inherits the containing instance's effective layer; nested layer-0 and ByBlock chains continue outward through instances. A top-level ByBlock mode remains stored but has no universal effective fallback in this proof. The document has no resolved-appearance field. The experimental reader validates references, layer names, modes and explicit values but does not compute effective appearance; rendering or conversion may do that per occurrence.

## Named line patterns

A definition's path supplies its identity; the payload has no repeated ID. Names
are unique under Unicode 17 full case folding, without trimming or Unicode
normalization. `ByLayer` and `ByBlock` are modes, never definition names.
`Continuous` is reserved for an empty array, but has no reserved numeric ID.
Any other named empty definition is also valid. The reader never synthesizes a
Continuous definition. Layer and explicit entity targets must exist with the
`ifccad::linePattern` role in this drawing and be included in drawing children.
A later fragment replaces the whole `ifccad::linePattern` value, including its
array, before these checks; it does not concatenate elements.

Positive values are strokes, negative values are gaps and zero is a dot. Empty
means continuous. A nonempty array needs at least two elements, a nonnegative
first element, finite values and a finite positive sum of absolute lengths. No
strict alternation is required. Lengths use the owning geometry's coordinate
unit (drawing unit for Model/blocks, sheet unit for direct Paper geometry).
Drawing and entity scales multiply the base lengths. The stored `perSegment`
polyline mode restarts per segment; `continuous` advances along the path. CAD
alignment A semantics apply, without adding a renderer or resolved appearance.
Block occurrence scaling and inherited properties are evaluated downstream.

The schema uses current IFCX `Object`, `Array`, `Real`, `String` and `Enum`
descriptions. Path target roles, Unicode uniqueness, finite sums, scale defaults
and mode-dependent requirements are profile rules enforced by the reader, not
new IFCX core syntax. Text/shape segments, PSLTSCALE, annotation scaling and new
entity creation settings remain outside the native profile.

## Prototype boundary

The strict reader and writer are in `src/ifccad/`, exported as `ocdraw::ifccad`. The writer strict-reads its own output and compares the typed CAD meaning. It emits JSON only. The standalone OCDraw reader validates a different contract. A separate `ifccad-convert` companion provides a bounded direct mapping to cadcodec `CadDocument`; its direction-specific coverage documents define conversion limits. A viewport projection/rendering API, annotation, indexed colors, complex text/shape line patterns and effective appearance evaluation are outside this profile version.

## Primitive geometry rules

All represented values are finite binary64. Readers and writers retain supplied
placement axes and radian parameters without normalization. A circular point is
`radius * (cos(t), sin(t))`; an elliptic point is
`(semiMajorRadius * cos(t), semiMinorRadius * sin(t))` in the placed local plane.
Arc sweeps satisfy `0 < abs(sweepParameter) < binary64 TAU`. Start parameters are
not reduced modulo a turn. Full circles/ellipses retain distinct kinds from arcs,
and equal-radius Ellipse remains Ellipse. Unknown primitive/placement members,
missing fields and explicit null values are invalid. Direct XYZ geometry has no
supporting placement attribute.

A typed planar polyline has one finite outgoing bulge per vertex. Wire omission
resolves to all zero; a present array must exactly match the vertex count. A
nonzero bulge represents signed included angle `4 * atan(bulge)`. Closed paths
activate every outgoing segment; the last bulge of an open path is retained but
not evaluated. An active curved segment with coincident endpoints is invalid;
straight repeats remain valid. Spatial polylines remain a separate straight XYZ
family even when coplanar. Neither native family has width or fit semantics.
These intrinsic rules use the same geometry predicates as OCDraw, through
independent model adapters; no OCDraw file or CAD runtime is needed.
## Optional bounds and explicit preparation

Model, Paper and block-definition values may include `bounds` with exactly
`min` and `max` finite ordered XYZ arrays. Empty owners require absence;
nonempty owners may omit bounds. Explicit null, partial or unknown fields fail.
Present bounds must contain the independently proven conservative geometry subset,
including complete curves, nested signed block transforms and Paper viewport
frames at Z=0. They are geometric metadata in the owner's coordinate unit,
independent of physical media. They do not define cropping or display visibility.
An actually empty-definition instance contributes its placement origin; a
nonempty definition with empty glyph extents does not acquire fabricated points.
`boundsQuality` may accompany a box as `enclosing` or `estimated`. Omission with
a present box declares enclosing. Quality without a box, explicit null quality
and stored partial boxes fail. A supplied enclosing glyph claim is a producer
declaration, not font-engine verification. Estimates are navigation aids; readers
must not use them to prove exclusion. Missing opaque/unavailable contributions
require absent bounds and derive `partial`, even when a known subset has extents.

`recompute_ifccad_document_bounds` is explicit and atomic. It validates authored
content while ignoring stale bounds, prepares all scopes, and only then replaces
all bounds and qualities. Failure changes no field. Native loading/encoding preserves valid
supplied bounds without recomputation. Transform evaluation for supplied-bounds
validation does not normalize or rewrite transforms. Owner identity domains
remain distinct: a layout and a block may have the same numeric ID.

`assess_ifccad_document_bounds` exposes independent enclosing/estimated/partial
quality, coverage and enclosure evidence. Availability may include estimated
glyph boxes; it does not certify contours. Glyph estimates and known primitive
subsets propagate through signed/nonuniform block transforms. Arithmetic checks
for known contributions remain hard with absent bounds and unavailable neighbours.

## Text and MText

The independent closed nested mapping is
[`text-values-0.1.0.schema.json`](text-values-0.1.0.schema.json). Its required
fields, enums, variant exclusivity, primitive types and unknown/null rejection
supplement the bundled IFCX module and semantic constraints described in
[`docs/text.md`](../../docs/text.md). No CAD formatting codes are interpreted by
native IO. Text runs and MText paragraph/inline boundaries, authored overrides,
literal Unicode, explicit false/zero and empty tab-stop resets remain unchanged.

Text styles are drawing children at `/cad/dN/textStyle/N`, with a nonempty,
NUL-free name unique under the pinned Unicode 17 full case fold, without trim or
normalization. The `ifccad::textStyle` value contains name, required symbolic font
request and style properties; ID is represented by the path, never repeated.
Unused styles survive native IO. Style IDs are independent zero-valid uint64
values. `nextTextStyleId` is positive and exceeds every live style ID; allocation
is checked and deletion does not lower it. Omission means 1 only for an empty
style domain without text references; present style records require the counter.
Counter-only empty domains retain advanced allocation history. No Standard style
or current-style workspace selection is synthesized by the core.

A native text entity has the existing `ifccad::entity` common attributes,
`ifccad::geom::placement`, and exactly one `ifccad::text` or `ifccad::mText`
drawable payload. Text cannot coexist with another drawable role. `style` is a
complete canonical same-drawing path to a text-style role, including IDs above
2^53. Text/MText are not eligible active Paper clip curves. Dormant references
retain the existing same-owner/exclusive constraints.

Text has anchored/wholeTextMiddle/aligned/fit layouts, literal decorated runs,
separate rotation/mirrors/shear and signed thickness. MText has ordered typed
paragraphs/inlines, separate characterFormat/paragraphFormat, all attachments,
flow, optional wrapping, static/dynamicAutoHeight/dynamicManualHeight columns and
background fill/padding/opacity/frame. Inline/background explicit colours use
this profile's RGB strings, independently of OCDraw colour metadata. Distances
use owning coordinates; inline relative heights and indentation/tab factors use
nominal MText height. Font overrides replace the whole request. Layout overflow
does not truncate content, and source-independent validation shares only pure
text values and geometry helpers.

Composition replaces an entire repeated text/mText/textStyle attribute value.
LaterWins does not concatenate content arrays or merge nested overrides.
RejectConflicts rejects differing payloads. Final composition validates style
membership, role, references, identities, counters, layout and scope bounds.
Foreign IFCX context and original fragments remain in the immutable source graph.

## Opaque source preservation

A drawing may have a child `preservation` pointing to `/cad/dN/preservation`, with
an `ifccad::preservation` object containing envelope `version: 1` and a required
`sources` array. Source context IDs are unique nonempty strings; provider/revision
are nonempty, origin is cadDocument/dwg/dxf and optional sourceVersion is omitted
when unavailable. Collection children `rN` point to `/cad/dN/preservation/rN`.
Record IDs form an independent positive uint64 domain, derived solely from paths.
The optional drawing `nextPreservationRecordId` is required whenever a collection
exists; without a collection it is omitted only at the untouched default 1.
Present watermarks must be positive and exceed every record ID; reservations and
deletions persist. Native-only input without this domain remains readable,
including the prior inline drawing schema lacking only this optional field.

A record has sourceId/sourceKey, category (entity/object/table/drawing/layout/shared),
role (complete/supplement/shared), representation (codecTyped/codecOpaque), optional
subject, dependencyCoverage (qualified/conservative/unknown), required bindings
and conditions arrays, and payload {schema,version,kind,bytes}. Bindings contain
slot/sourceKey/target; conditions contain target/predicate/version/baseline.
Targets contain only role/path. Roles drawing/entity/layer/linePattern/layout/
blockDefinition/record must match a canonical same-drawing complete path in that
identity domain. Layout/block IDs may coincide but their roles/paths cannot alias.
Payload/baseline bytes use canonical padded standard Base64. Positive payload and
predicate versions and nonempty schemas/predicates are required. Optional fields
are omitted; explicit null does not represent absence. Unknown core fields fail.
Unknown provider schemas and predicates remain byte-transportable.

An ordered drawable `/cad/dN/eN` is either native or has only
`ifccad::opaqueEntity` among CAD attributes. That opaque attribute has required
preservationRecord (full record path) and visible (Boolean), with optional
nativeLayer (full layer path) and nativeAppearance {appearance,linePatternScale}.
Present common values obey native constraints; absent values remain unavailable.
No native entity/kind/placement attribute may coexist. Its record must be a
complete entity record with the exact matching live entity subject. Detached
archives explicitly clear subject and have no draw-order position/export obligation.
Bindings and conditions are soft dependencies, so missing targets and record cycles
remain storage-valid; labeling coverage qualified does not grant restoration.

Composition merges node attribute keys. LaterWins replaces an entire repeated
opaqueEntity/preservationRecord value; RejectConflicts refuses differing values.
No recursive payload/baseline merge is allowed. Validate the final composed
projection and all hard opaque/subject/common links. The immutable original source
graph remains distinct from fresh logical encoding.

Opaque geometry makes complete bounds unavailable in its direct owner and every
reachable instance owner, including unused definitions. Those bounds must be absent;
Empty and Unavailable differ, so an opaque-only block cannot use an empty-origin
fallback. Native validation and transactional bounds preparation remain active.
Opaque clip references require dormant state, same Paper ownership and exclusive
use; active boundaries need supported native geometry.

The initial IFCCAD adapter qualifies only typed SPLINE snapshots. Capture, restoration
and physical CAD exchange are separately reported; no native spline evaluator or
exact original-storage replay is claimed. See `docs/preservation.md` for predicates,
coordinate meaning, source namespace rebinding and current pinned target restrictions.

## Drawing workspace

Optional workspace snapshots use `ifccad::drawingWorkspace` (current Layer and
active layout), `ifccad::modelViewState` (optional current Model UCS and active
window), drawing-owned `ifccad::ucsDefinition` and `ifccad::modelWindow` nodes,
`ifccad::paperCanvas` on a Paper layout, and `ifccad::viewportWorkspace` on an
authored Paper viewport. These records do not enter entity order or bounds.
The drawing's `modelWindows` reference array is the sole window order; graph
node and child-key order are incidental. References use complete same-drawing
paths. World, Named and Unnamed selections have distinct allowed fields.

UCS and model windows use independent persistent `nextUcsId` and
`nextModelWindowId` allocation watermarks. They exceed all live IDs and never
reduce after deletion. Actual workspace families require both explicit counters;
legacy CAD-only input without workspace identities may use initial values of 1.
Writers emit both counters. Native IDs never pass through floating point.

Model window and authored model-viewport workspace coordinates use drawing
units; Paper canvas values use that layout's Paper coordinates. No scale is
inferred from media. Stored UCS and its activation flag remain independent from
optional current state. Missing current choices stay unspecified. A current UCS
requires a known active Paper context; an active viewport has a workspace in the
same layout. A viewport workspace can exist without a canvas snapshot.

Canvas frames preserve finite XYZ center and positive dimensions without
contributing geometry. Grid spacing is finite and nonnegative; snap spacing is
positive while enabled and nonnegative while disabled. Preserve dormant values.
Intrinsic view, clip, grid and snap rules are shared with OCDraw through the
runtime-independent workspace kernel. Unknown fields/variants, foreign references,
nonfinite scalars, invalid activation consistency and counter exhaustion remain
errors. IFCX composition retains its established whole-attribute semantics.
