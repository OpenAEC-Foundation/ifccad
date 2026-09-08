# IFCCAD candidate compatibility

This collection is the unpublished `1.1.0` candidate. Its active drawing
contract is IFCDR `0.7.0`, selected by IFCX overlay `0.7.0`, with drawing core
`0.2.0`. The logical registry uses meta-schema v2; the separate JSON mapping uses
meta-schema v1 and stream-directory v1. IFCPR remains
`0.2.0`. Historical schemas and `conformance/1.0.0` are reference artifacts,
not promises that the current reader supports their files.

## Supported drawing content

The IFCDR registry contains four streams: `line`, `polyline`, `entityOrder`,
and `entityOrderEntry`. It retains `scope`, `layerBinding`, `appearanceBinding`,
and `appearanceOverride` tables. Lines and polylines use the existing XY
geometry, resource units, scope membership, appearance modes, and draw order.
Omitted visibility means `true`. Polyline schema v3 requires at least two
vertices. Repeated vertices and coincident line endpoints are valid; the closed
flag is preserved independently of whether the first vertex is repeated.
Empty resources have no bounds (JSON null); nonempty resources require finite
bounds enclosing all geometry, including invisible entities. Conservative
bounds are valid; lineweight and scope bases do not expand these bounds.
Colors retain RGB and optional indexed (u64) and named metadata together. All
stored non-null overrides are validated, including unused values.

Prototype entity families, their child streams, and their exclusive support
tables are absent from this contract. Even empty discarded support tables are
not accepted by its closed-field rules. They can return in future versioned
contracts after their semantics are designed and tested.

## Operations and limits

| Content or operation | Primary implementation behavior |
| --- | --- |
| IFCDR 0.7.0 JSON with the registered content | Physical field/range checks in the JSON codec, shared logical geometry/reference/identity/order/bounds/appearance validation, and package binding checks; typed lines and polylines. |
| Directory writer | One drawing, one model layout, one external IFCDR resource; deterministic new-version output. |
| Resource access | External package-relative JSON resources. Existing scope and model/paper layout references remain readable; this change does not add inline resources or paperspace export. |
| IFCDR 0.6.0, 0.5.0 or another unsupported version | `IFCCAD_IFCDR_VERSION_UNSUPPORTED`; no strict typed package and no migration. |
| Unknown stream name or schema ID | `IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED` with stream/schema context when available; no strict typed package or unmodeled-entity view. |
| Malformed supported fields or known broken references | Structural or semantic error diagnostics; no strict typed package. |
| Unknown IFCX node types and open extension fields | Existing permitted read behavior remains; no new guarantee of conversion, editing, or lossless rewriting. |
| IFCCAD to CadDocument | Existing drawing conversion for typed lines and polylines, layers, units, order, and supported appearance data. Existing pattern fallback and line-weight rounding diagnostics remain. |
| CadDocument to IFCCAD | Existing exact finite 2D line and straight lightweight-polyline subset. Unsupported properties/entities are diagnosed under `Allow` or reject the export under `Reject`. No approximation or expanded native coverage. |
| IFCPR 0.2.0 | The limited checks described below; no converter preservation transfer. |

`IfcdrEntityRef::Unmodeled`, `UnmodeledEntityRef`, and
`ImportDiagnostic::UnmodeledEntitiesSkipped` have been removed. Unsupported
IFCDR content is blocked before the typed conversion API. This does not remove
the opposite conversion direction's unsupported CAD source diagnostics.

## Validity, support, and transfer

These are distinct claims:

- **Validity:** whether the applicable published contract is satisfied.
- **Support:** whether this implementation can perform the requested operation.
- **Lossless transfer:** whether that operation retains the relevant meaning.

The current report API and manifest schema do not yet encode all three as
independent results. `PackageValidationReport::is_valid()` means that the
implemented checks emitted no error diagnostics. Unsupported content is an
error-level blocker for `validated_package()`; this does not establish
invalidity under an unsupported contract.

The manifest's `invalid` category includes the explicitly named `unsupported.*`
cases because they cannot obtain a strict result from this reader. The old
version case also reports that its descriptor violates the selected 0.7.0
overlay; it is not a validation attempt against the old overlay. An unknown
stream does not produce dependent missing-entity/order or orphan-payload
diagnostics based on an unknown payload mapping. Independently established
physical errors remain reportable. A declared but unvalidated drawing resource
is not reported as an absent preservation target.

Strict typed access is all-or-nothing at package level. Diagnostics remain
inspectable on failure. There is no partial drawing API, automatic migration,
or pass-through preservation of unknown drawing content. New writer output
is tested for repeated byte determinism and production-reader semantic
roundtrips, not byte compatibility with earlier writers.

## IFCPR limitations

The loader checks IFCPR JSON against schema 0.2.0, descriptor/header resource
identity, exact resource-file checksums, and links to drawing resource IDs.
Existing fixtures exercise these checks independently of base drawing tests.

It does not fully validate blob contents, payload ranges, record dependencies,
or projection semantics. The following package cases remain explicitly deferred
in the Rust conformance runner:

- `invalid.blob-digest-mismatch`;
- `invalid.payload-range-invalid`;
- `invalid.record-reference-missing`;
- `invalid.dependency-cycle`;
- `invalid.projection-resource-missing`.

The last case also requires interpretation of the collection's `package.json`
resource index. The production directory reader loads `package.ifcx.json`.
Passing the implemented IFCPR checks is not a full preservation-validity proof
or a guarantee of lossless source roundtrip. The converter does not transfer
IFCPR records or source payloads.

## Remaining milestone 2 work

Reader and writer now use separate backings behind shared typed collection
access and logical validation. The JSON codec maps the published logical
contract to physical columns and pools. The encoder accepts validated resources
and preserves supplied IDs, order, conservative bounds and appearance metadata.
Builder completion validates its prepared resource, then validates the assembled
package in memory through the production reader before returning it.

Drawing-resource naming, inline/external normalization, complete
operation-specific reporting, and initial size measurements remain outstanding.
Broader entity semantics and preservation remain milestone 3.
