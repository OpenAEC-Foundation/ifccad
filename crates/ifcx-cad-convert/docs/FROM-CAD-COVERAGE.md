# CadDocument → IFCX-CAD coverage

The logical `cad_document_to_ifcx_cad_document` routes return an independently
validated typed document with diagnostics and mappings, without an IFCX byte
bridge. Existing encoded routes call them, then core profile encoding and
production loading. Coverage, loss acceptance, recoveries and fresh ID
allocation are shared. Encoding creates a new CAD-profile file; no source graph
writeback is performed.

Pinned semantic inventory V1, opencadcodec
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`. Exhaustive matches classify every
inventory category. Serde residual checks cover fields plus typed comparisons
for skipped-serde common/marker/object state. Data not exposed by the codec has
no asserted coverage. Default Allow returns the supported subset with located
loss diagnostics; explicit Reject prevents output for any semantic loss.
The [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-02.md)
records the added public fields and their treatment in both converters.

## Inventory categories

Fresh imports allocate entity, layer, layout, block-definition and pattern IDs
from 1 using the core's independent checked uint64 domains. The drawing stores
all five resulting next-ID watermarks, including empty domains. Allocation
exhaustion is fatal under Allow and Reject. Numeric IDs are not CAD handles or
table positions; mappings preserve source associations for this conversion.
Reimporting CAD does not restore a previous IFCX-CAD allocation history.

| Category | Treatment |
| --- | --- |
| Header | Map insertion units and linetype scale; compare other settings/names with pinned default. Derived extents, seed and explicitly listed control/dictionary/name-cache handles are excluded in `source.rs` |
| Layers | Map identity/name/concrete appearance. Diagnose changed flags, descriptions, named colors, plotting/material/XREF metadata or other residual fields |
| Local BlockRecords | Map name/base/unit/ordered contents. Begin/end identity and reverse insert handles are structural/derived. Diagnose flags, description, preview, insert-count bytes, scaling/explodability restrictions and layout association |
| Model/Paper BlockRecords | Validate role/ownership; metadata including unit/base must remain default |
| LineType | Actual signed-length definitions, names/descriptions and unused records; canonical ByLayer/ByBlock are mode scaffolding. Whole text/shape pattern fallback to named empty under Allow, one Modified diagnostic per definition; invalid values/alignment/name-handle targets fail both policies |
| TextStyle/DimStyle/AppId/View/VPort/Ucs/Vx | Only default records, record handles normalized; additional/changed records diagnosed |
| Classes | Default definitions only; normalize derived numbering/instance counts and version metadata |
| Entities | Supported fields below; every other family diagnosed. Full inventory includes structural markers hidden by `entities()` |
| Layout objects | Unique consistent Model link; pinned Model/Layout1 fields only. Validate viewport references; exclude derived extents and structural handles. Modified plot/UCS/layout fields or additional layouts diagnosed |
| Other typed/unsupported objects | Compare pinned scaffold by named dictionary roles and typed values; additional/modified/unsupported objects diagnosed |
| Summary/Preview | Changed summary or any preview diagnosed |
| Relationships | Entity/marker/Layout ownership and typed drawing references remain structural checks. Unresolved ownership of omitted non-Layout objects is located loss, including object kind and owner handle. Reactors/extension dictionaries, including unresolved endpoints, are diagnosed losses rather than global structural failures |
| Non-entity extended data | Diagnosed, including undecodable payloads; no preservation in this slice |

Defaults come from fresh CadDocument for in-memory/DXF, and one cached
unmodified fresh-DWG write/read for DWG. Roles follow named dictionaries,
never handle coincidence/count alone. Standard style handle/name caches and
duplicate raw DXF style records are normalized beside their typed comparisons;
authored style fields remain compared. Source identity/reference collisions fail.
The Model layout's `plot_flags.model_type` is also derived role bookkeeping:
the DXF writer sets it from the Model name. Other plot flags remain diagnosed.

An overall viewport is excluded only for the Layout1-linked ID-1 viewport with
all constructor-default values apart from verified owner/identity/storage fields.
Authored view settings or dimensions fail that classification. Other Paper
entities and extra layouts are omitted with diagnostics under Allow.
This comparison includes the newly exposed `Viewport.off_screen`; true prevents
the viewport from being silently classified as default scaffold.

## Entity fields

| Fields | Treatment |
| --- | --- |
| Common identity/owner/layer | Non-null unique identity; one ordered owner; existing layer; return mappings |
| Common appearance | Independent ByLayer/ByBlock/Explicit modes. True RGB, resolved named patterns, entity pattern scale and supported weights. Indexed/named color and Default weight diagnosed |
| Common linetype handle/entity mode/raw record/data-store flag | Typed-name/owner or encoding caches, not persisted separately; name/linetype handle agreement is validated |
| Other common fields | Typed residual check includes XDATA/raw EED, visibility, named colors, graphics, material/plot/shadow/visual styles, references, reactors and dictionaries |
| Line start/end | Direct XYZ; diagnose thickness/nondefault normal |
| Circle center/radius | +Z normal only, canonical XY placement; diagnose thickness/nondefault normal |
| LwPolyline vertices/closure/elevation | Straight XY points at stored Z. Map plinegen to perSegment/continuous; diagnose bulges, widths, vertex IDs, constant width, thickness and nondefault normal |
| Insert target/point/rotation/scales | Existing local target, +Z normal, canonical XY placement; diagnose attributes, arrays/MINSERT, spacing, view/sequence handles and other residual fields |
| BLOCK/ENDBLK | If present, record handle/type/name/owner/base must agree; diagnose delimiter/common changes. Missing DXF/programmatic markers are normal |

Unused definitions participate in cycle/target validation. Case-insensitive
duplicate layer/block names, duplicate/null handles and multiple owners fail.
A stale missing model cache is recovered only through unique consistent
BlockRecord/Layout agreement, with diagnostic; conflicting existing cache fails.

## Partial conversion policy

Unsupported families and incompatible geometry residuals omit the whole entity,
including bulges, widths, nondefault normals/thickness and Insert arrays or
attributes. Unsupported common metadata (for example XDATA or invisibility)
is omitted with a diagnostic while representable geometry is retained. No
flattening, block explosion or preservation is performed.

Normal local definitions retain their supported children in source order.
Reserved/anonymous names, XREF/external/unloaded flags or paths, and dynamic
objects directly owned by a definition cause definition omission along with
referring inserts. Each instance whose target loses content receives
`block-content-loss`, propagated through nested definitions. Metadata omissions
on a definition also participate in this propagation. Mappings cover emitted
objects only.

Indexed color is projected through opencadcodec's canonical ACI palette to RGB,
with index/context loss reported. Unavailable layer color becomes white;
inherited layer opacity becomes opaque; unavailable/default weight becomes
explicit 0.25 mm. Unsupported numeric weights use the closest standard CAD
weight. Complex text/shape definitions become empty under their original name and ID, including unused definitions; each has one `line-pattern-complex` modification. Finite source period disagreements are diagnosed and derived from elements. Entity ByLayer/ByBlock modes remain
stored. Unknown drawing or block unit codes become unitless without scaling,
with modification evidence. Unsupported table/header/object/source metadata is
diagnosed and omitted. No full-source preservation claim follows from Allow.

Both policies reject essential drawing reference/owner/marker conflicts, cycles,
invalid scalar values in known geometry and numeric precision failures. A
unique model-cache repair has action Recovery and does not count as a loss.
Unresolved ownership of unsupported objects and optional reactor/extension
relationships is omitted with loss diagnostics under Allow; Reject refuses that
loss. This does not repair the source or preserve the unsupported metadata.
Existing dictionary-entry and Layout/block/viewport consistency checks remain
unchanged. `tests/metadata_relationships.rs` covers the metadata/structure boundary
through logical and encoded conversion and production-reader readback.

Metadata is caller supplied. New IDs follow source table/owner enumeration and
remain u64; layer/definition vectors follow current core lexical path ordering.
Entity vectors retain owner order. Layout mappings use actual Layout handles;
definitions use BlockRecord handles. No handle or cross-file ID persistence claim.

Output goes through the production writer and strict reader. Tests cover large
IDs, different handles, unused definitions, cycles, missing references, tiny
scales, common-field loss and semantic DXF/DWG exchange, including nested/shared
nonzero-base DWG blocks after upstream #52 was fixed. `tests/loss_policy.rs` covers
partial owner order, missing mappings, nested loss propagation, explicit
appearance adaptations, foreign graph loss and the Allow/Reject boundary.

Line-pattern and layer/definition vectors follow lexical path ordering. Signed
lengths and drawing/entity scale values are included in exact projection checks.
Tests in `tests/line_patterns.rs` include real pinned DXF/DWG exchanges, optional
defaults, Unicode/target lookup collisions and whole complex-pattern fallback.
