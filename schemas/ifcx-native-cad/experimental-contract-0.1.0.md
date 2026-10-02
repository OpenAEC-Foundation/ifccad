# Experimental IFCX-native CAD profile 0.1.0

This is an opt-in, standalone IFCX alpha experiment, independent of the OCDraw drawing contract. Released historical IFCCAD conformance collections remain unchanged. Generic IFCX content may coexist with these CAD nodes. A reader claiming this CAD profile must enforce the rules below after composing fragments for equal paths.

## File and identity

The file has IFCX `header`, `imports`, `schemas`, and `data`. `header.ifcxVersion` is `ifcx_alpha`. `header.dataVersion` belongs to the dataset. The drawing value's `profileVersion` is `0.1.0` and identifies this provisional CAD proof. This alpha contract can be revised without increasing that number; it does not promise compatibility across experimental reader revisions. The recommended drawing imports `urn:example:ifccad:0.1.0`, whose schema-only IFCX document is `experimental-profile-0.1.0.ifcx`, and leaves drawing-local `schemas` empty. The `example` URN namespace is registered for documentation and experimentation; this identifier is intentionally temporary and must be replaced by an organization-controlled published URI before production exchange. During this experiment, the reader revision determines the schema definition for this URI; the identifier does not yet promise an immutable published schema. The strict reader resolves this one experimental URI from its bundled profile, without network access. Alternatively, a drawing may inline exactly matching definitions from that module's `schemas`; a missing import and missing local definitions fail. This is a bounded offline resolver, not general IFCX import resolution. Conditional requirements and cross-node constraints in this document exceed IFCX's per-attribute descriptions.

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
through CAD serialization. See the [allocation design](../../docs/experiments/ifcx-cad-id-management.md).

By default, multiple `data` fragments with the same path compose in file order. Their `children`, `inherits`, and `attributes` objects merge by key; for a repeated key, the later value replaces the entire earlier value, including a geometry or appearance attribute object. Other repeated node fields are replaced as whole values. A `null` inheritance value removes that key during composition, while a `null` child or attribute value remains a marker in the composed node. This matches the observed upstream flattening behavior; deletion during expanded graph loading and attribute deletion are not defined by this CAD profile. The reader also exposes a diagnostic `RejectConflicts` policy: it accepts disjoint and identical fragments but rejects differing repeated values, including on foreign IFCX nodes. That stricter option is a development check, not a claim that IFCX forbids overwrites. Duplicate JSON object keys within one object are errors under either policy. The reader validates CAD roles, references, geometry, and draw order on the final composed nodes under either policy. Unknown IFCX nodes and unknown non-CAD attributes are retained under the same selected composition rule. The CAD profile does not use `inherits` for blocks or appearance. A foreign IFCX node may refer to a CAD entity without acquiring ownership.

## Scope and order

The drawing has named `children` referring to exactly one Model layout, zero or more Paper layouts, all layers, all line-pattern definitions (including unused ones), and all block definitions. Pattern membership is unordered; canonical typed collection order follows lexical paths, independently of element order. Layouts share the drawing's layers and block definitions. Layout child names identify membership, while required `tabIndex` values encode presentation order. Model has index 0; Paper indices are unique and contiguous from 1 through the number of Paper layouts. Reader and writer normalize Paper collections by tab index, independently of IDs. Reordering changes indices without changing identity or allocation watermarks. Paper names are nonblank and unique under Unicode 17 full case folding, including the implicit name `Model`, without trimming or normalization. Each layout and block definition uses `children` keys `"0"`, `"1"`, … as draw positions; keys must be canonical and contiguous from zero. Empty scopes are allowed. Values are paths to distinct drawable entities. Numeric order is authoritative independent of JSON member order. An entity is a numeric child of exactly one layout or block definition, and every `ifccad::entity` node has such an owner. Reordering changes child keys but not entity paths. Other named IFCX children may coexist but confer no CAD ownership. There is no separate draw-order or `scope` attribute. A block instance is one ordered entity in its owner's children; it references one definition by its `definition` field. All definition targets must resolve and the definition-reference graph must be acyclic, including unused definitions.

## Attributes

| Attribute | Required value / role |
| --- | --- |
| `ifccad::drawing` | `profileVersion`, `lengthUnit`, the five required allocation watermarks above; optional positive finite `linePatternScale` defaults to 1; one drawing node |
| `ifccad::layout` | `kind: "Model", tabIndex: 0`, or `kind: "Paper"` with required `name`, `tabIndex`, coordinate `lengthUnit` and optional `paper: { width, height, lengthUnit }`; Model forbids name, lengthUnit and paper fields, including null values |
| `ifccad::linePattern` | nonempty `name`, optional `description`, ordered signed-real `pattern` array; drawing-owned definition |
| `ifccad::layer` | `name`, direct concrete `appearance` values |
| `ifccad::blockDefinition` | `name`, `basePoint` XYZ, `insertionUnit` |
| `ifccad::entity` | resolvable drawing-local `layer`, four property modes in `appearance`; optional positive finite `linePatternScale` defaults to 1 |
| `ifccad::geom::lineSegment` | `start` and `end` XYZ; finite segment, unlike unbounded IFC4 `IfcLine` |
| `ifccad::geom::planarPolyline` | at least two XY `vertices`, `closed`, plus `ifccad::geom::placement`; optional `linePatternGeneration` defaults to `perSegment`, alternatively `continuous` |
| `ifccad::geom::circle` | positive finite `radius`, plus `ifccad::geom::placement` |
| `ifccad::geom::placement` | finite XYZ `origin`, `xAxis`, `yAxis`; valid orthonormal right-handed frame under the shared OCDraw geometric predicate |
| `ifccad::blockInstance` | definition path and transform with placement, finite rotation in radians, nonzero finite XYZ scale |

Every owned drawable has `ifccad::entity` and exactly one of the four drawable payload attributes. Unsupported `ifccad::geom::*` payloads fail explicitly. An independently used `ifccad::geom::circle` on a non-CAD IFCX node is not thereby a CAD entity. Coordinates use a fixed right-handed local XYZ convention; there is no implicit world alignment. The 25 length-unit tokens match the current OCDraw registry. Model and block-definition coordinates, including definition base points, use the drawing's length unit. Each Paper layout declares its own coordinate `lengthUnit`, including `unitless`. Direct Paper geometry sizes and placement origins use this coordinate unit. Optional `paper` describes a physical medium: positive finite width and height, and a physical registry unit (`unitless` is disallowed). Omit `paper` for an unsized sheet; null, partial or unknown medium fields fail. Medium dimensions are independent of coordinate units and geometry bounds, and imply neither a coordinate rescale nor clipping. No requirement places entities inside the physical medium. Existing example files have been migrated; missing new metadata fails without legacy synthesis. The experimental profile version and import remain unchanged.

A block definition's insertion unit records intent but does not silently scale coordinates. Transform evaluation subtracts the definition base point, applies stored scale and rotation, then applies placement. For a paper-owned instance, its scale maps drawing-coordinate numbers into paper-coordinate numbers; any unit conversion must be included explicitly. For example, the [paper-layout fixture](../../examples/ifcx-native-cad/hello-paper-layouts.ifcx) has centimetre model/block coordinates and an A3 sheet in millimetres, with a paper instance scale of `[10, 10, 10]`. A nested instance inside a definition stays in drawing units. Reading and writing never evaluate these transforms or normalize their values.

The local circle shape aims at the analytic meaning of IFC4 `IfcCircle`; the namespace remains local until an official IFCX geometry attribute has an exact roundtrip contract. A line segment is not equivalent to unbounded `IfcLine`; planar polyline bulges, widths, and complex segments are outside this proof. The separate `placement` attribute can later be reused with a published geometry vocabulary if equivalent.

## CAD appearance

Layer appearance directly contains `color` as `#RRGGBB`, `opacity` in `[0,1]`, `linePattern` as a same-drawing pattern-definition path, and nonnegative `lineWeight` in millimetres. Each entity has these four properties independently as `{ "mode": "ByLayer" }`, `{ "mode": "ByBlock" }`, or `{ "mode": "Explicit", "value": ... }`. Modes survive readback; no appearance definition or binding node exists. For an explicit entity pattern, `value` is that same complete path; inherited modes must have no value. The typed API uses `IfcxCadLinePatternId(u64)`. Literal names such as `Continuous` are not references and are rejected.

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

The strict reader and writer are in `src/ifcx_cad/`, exported as `ocdraw::ifcx_cad`. The writer strict-reads its own output and compares the typed CAD meaning. It emits JSON only. The standalone OCDraw reader validates a different contract. A separate `ifcx-cad-convert` companion provides a bounded direct mapping to cadcodec `CadDocument`; its direction-specific coverage documents define conversion limits. Viewports, plot settings, paper-to-model viewing transforms, annotation, indexed colors, complex text/shape line patterns and effective appearance evaluation are outside this profile version.
