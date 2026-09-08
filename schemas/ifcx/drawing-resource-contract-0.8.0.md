# Drawing resource contract 0.8.0

This document is normative alongside `ifccad-overlay-0.8.0.json` and its
referenced drawing core 0.2.0 schema. The overlay selects IFCDR 0.7.0;
this vocabulary revision does not change IFCDR logical data or JSON encoding.

## Representation and resource

An `openaec:DrawingRepresentation` node describes a Drawing's CAD content.
Its `attributes.resource` descriptor identifies an IFCDR resource with
`format: "openaec.ifcdr"`, `version: "0.7.0"`, and `role: "drawing"`.
The descriptor's resource identity, external URI and checksum follow the
overlay schema and package resource validation rules.

Resource IDs and node paths are opaque identifiers. Their spelling need not
describe a drawing, scope or storage location. Resource identity is distinct
from the URI at which its bytes are stored.

## Drawing and layout relationships

Each `openaec:Drawing` references one DrawingRepresentation through
`children.Representation`. Every DrawingLayout listed in that Drawing's
`children.Layouts` MUST reference that same representation node through its
own `children.Representation`. Equality means equality of the referenced node
path, not merely equality of resource IDs or URIs. A second representation
node pointing to the same resource does not satisfy this rule.

A layout's `attributes.scopeId` MUST resolve to a scope in that resource.
Model and paper layouts can therefore select different scopes of the same
resource, with entities and draw order defined per scope by IFCDR.
This revision imposes no exclusive ownership across Drawings, no uniqueness
requirement on layout scope selections and no additional correspondence
between layout kinds and numeric scope kinds.

Missing references and references to the wrong node type are invalid in
their own right. Implementations should avoid adding representation-equality
errors when those reference prerequisites cannot be established. The
cross-node rules require package validation beyond local JSON Schema checks.

## Retired vocabulary and extensions

`openaec:DrawingGeometryRepresentation` is recognized retired vocabulary and
MUST be reported as unsupported by a reader of this contract, including when
the node is unreferenced. It must not be silently accepted as an unknown
extension. `attributes.geometry` does not replace the required
`attributes.resource` on a DrawingRepresentation.

Unrelated unknown IFCX node types and extension fields remain permitted where
the schemas leave those extension points open. Historical overlays remain
available for historical contracts; they do not define this reader profile.
