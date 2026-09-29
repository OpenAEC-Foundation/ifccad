# Experimental IFCX-native CAD profile 0.1.0

This is an opt-in, standalone IFCX alpha experiment. It does not change any released IFCCAD package, IFCDR resource, or conformance collection. Generic IFCX content may coexist with these CAD nodes. A reader claiming this CAD profile must enforce the rules below after composing fragments for equal paths.

## File and identity

The file has IFCX `header`, `imports`, `schemas`, and `data`. `header.ifcxVersion` is `ifcx_alpha`. `header.dataVersion` belongs to the dataset. The drawing value's `profileVersion` is `0.1.0` and selects these CAD rules. The recommended drawing imports `urn:example:ifccad:experimental-cad:0.1.0`, whose schema-only IFCX document is `experimental-profile-0.1.0.ifcx`, and leaves drawing-local `schemas` empty. The `example` URN namespace is registered for documentation and experimentation; this identifier is intentionally temporary and must be replaced by an organization-controlled published URI before production exchange. The strict reader resolves this one experimental URI from its bundled profile, without network access. Alternatively, a drawing may inline exactly matching definitions from that module's `schemas`; a missing import and missing local definitions fail. This is a bounded offline resolver, not general IFCX import resolution. Conditional requirements and cross-node constraints in this document exceed IFCX's per-attribute descriptions.

The complete path is a node identity. The profile uses `</cad/dN>` for one drawing, `</cad/dN/layout/N>` for its one Model layout, `</cad/dN/layer/N>` for layers, `</cad/dN/block/N>` for block definitions, and `</cad/dN/eN>` for drawable entities. `N` is a canonical base-10 `uint64` with no sign or leading zero; entity IDs are unique within the drawing and are not repeated as attributes. Path spelling is an experimental CAD convention, not a claim that IFCX requires UUIDs or these paths. References are complete path strings.

The JSON writer displays `path` first in each node, followed by `children` where present and then `attributes`. JSON member order does not affect node meaning; numeric child *keys* carry CAD draw order.

Multiple `data` fragments with the same path compose by union of disjoint `children`, `inherits`, and `attributes` keys. Identical repeated values are harmless; conflicting values are errors. Duplicate JSON object keys are errors before composition. Unknown IFCX nodes and unknown non-CAD attributes are retained. The CAD profile does not use `inherits` for blocks or appearance. A foreign IFCX node may refer to a CAD entity without acquiring ownership.

## Scope and order

The drawing has named `children` referring to its Model layout, all layers, and all block definitions. The layout and each block definition use `children` keys `"0"`, `"1"`, … as draw positions; keys must be canonical and contiguous from zero. Values are paths to distinct drawable entities. Numeric order is authoritative independent of JSON member order. An entity is a numeric child of exactly one layout or block definition, and every `ifccad::entity` node has such an owner. Reordering changes child keys but not entity paths. Other named IFCX children may coexist but confer no CAD ownership. There is no separate draw-order or `scope` attribute. A block instance is one ordered entity in its owner's children; it references one definition by its `definition` field. All definition targets must resolve and the definition-reference graph must be acyclic, including unused definitions.

## Attributes

| Attribute | Required value / role |
| --- | --- |
| `ifccad::drawing` | `profileVersion`, `lengthUnit`; one drawing node |
| `ifccad::layout` | `kind: "Model"` on the Model scope node |
| `ifccad::layer` | `name`, direct concrete `appearance` values |
| `ifccad::blockDefinition` | `name`, `basePoint` XYZ, `insertionUnit` |
| `ifccad::entity` | resolvable drawing-local `layer`, four property modes in `appearance` |
| `ifccad::geom::lineSegment` | `start` and `end` XYZ; finite segment, unlike unbounded IFC4 `IfcLine` |
| `ifccad::geom::planarPolyline` | at least two XY `vertices`, `closed`, plus `ifccad::geom::placement` |
| `ifccad::geom::circle` | positive finite `radius`, plus `ifccad::geom::placement` |
| `ifccad::geom::placement` | finite XYZ `origin`, `xAxis`, `yAxis`; valid orthonormal right-handed frame under the shared IFCDR geometric predicate |
| `ifccad::blockInstance` | definition path and transform with placement, finite rotation in radians, nonzero finite XYZ scale |

Every owned drawable has `ifccad::entity` and exactly one of the four drawable payload attributes. Unsupported `ifccad::geom::*` payloads fail explicitly. An independently used `ifccad::geom::circle` on a non-CAD IFCX node is not thereby a CAD entity. Coordinates use the drawing's length unit and a fixed right-handed local XYZ convention; there is no implicit world alignment. The 25 length-unit tokens match the current IFCDR registry. A block definition's insertion unit records intent but does not silently scale coordinates. Transform evaluation subtracts the definition base point, applies stored scale and rotation, then applies placement. No normalization is written back.

The local circle shape aims at the analytic meaning of IFC4 `IfcCircle`; the namespace remains local until an official IFCX geometry attribute has an exact roundtrip contract. A line segment is not equivalent to unbounded `IfcLine`; planar polyline bulges, widths, and complex segments are outside this proof. The separate `placement` attribute can later be reused with a published geometry vocabulary if equivalent.

## CAD appearance

Layer appearance directly contains `color` as `#RRGGBB`, `opacity` in `[0,1]`, `linePattern` currently `Continuous`, and nonnegative `lineWeight` in millimetres. Each entity has these four properties independently as `{ "mode": "ByLayer" }`, `{ "mode": "ByBlock" }`, or `{ "mode": "Explicit", "value": ... }`. Modes survive readback; no appearance definition or binding node exists. This first proof supports true color and Continuous only.

Per occurrence, Explicit uses its own value, ByLayer uses the effective layer, and ByBlock uses the containing instance's corresponding property. A definition entity on the layer *named* `0` uses its containing instance's layer. For a one-level instance, an instance property that is ByLayer resolves from the instance's own layer. A top-level ByBlock mode remains stored but has no universal effective fallback in this proof. Nested appearance evaluation is specified for future work: follow layer-0 and ByBlock outward through instances until a concrete layer or value is found. The current resolver rejects nested chains rather than guessing.

## Prototype boundary

The strict reader and writer are in `src/experimental_ifcx/`. The writer strict-reads its own output and compares the typed CAD meaning. It emits JSON only. The existing IFCCAD package reader does not validate this standalone experiment. Paper layouts, viewports, annotation, indexed colors, custom line patterns, nested visual resolution, and DWG/DXF conversion are outside this version.
