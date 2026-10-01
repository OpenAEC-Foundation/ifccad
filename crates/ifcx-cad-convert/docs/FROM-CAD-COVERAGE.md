# CadDocument → IFCX-CAD coverage

Pinned semantic inventory V1, cadcodec
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. Exhaustive matches classify every
inventory category. Serde residual checks cover fields plus typed comparisons
for skipped-serde common/marker/object state. Data not exposed by the codec has
no asserted coverage. Unsupported semantics prevent successful output.

## Inventory categories

| Category | Treatment |
| --- | --- |
| Header | Map insertion units; compare other settings/names with pinned default. Derived extents, seed and explicitly listed control/dictionary/name-cache handles are excluded in `source.rs` |
| Layers | Map identity/name/concrete appearance. Diagnose changed flags, descriptions, named colors, plotting/material/XREF metadata or other residual fields |
| Local BlockRecords | Map name/base/unit/ordered contents. Begin/end identity and reverse insert handles are structural/derived. Diagnose flags, description, preview, insert-count bytes, scaling/explodability restrictions and layout association |
| Model/Paper BlockRecords | Validate role/ownership; metadata including unit/base must remain default |
| LineType/TextStyle/DimStyle/AppId/View/VPort/Ucs/Vx | Only default records, record handles normalized; additional/changed records diagnosed |
| Classes | Default definitions only; normalize derived numbering/instance counts and version metadata |
| Entities | Supported fields below; every other family diagnosed. Full inventory includes structural markers hidden by `entities()` |
| Layout objects | Unique consistent Model link; pinned Model/Layout1 fields only. Validate viewport references; exclude derived extents and structural handles. Modified plot/UCS/layout fields or additional layouts diagnosed |
| Other typed/unsupported objects | Compare pinned scaffold by named dictionary roles and typed values; additional/modified/unsupported objects diagnosed |
| Summary/Preview | Changed summary or any preview diagnosed |
| Relationships | Owner membership checked; dangling inventory endpoints fatal. Reactors/extension dictionaries diagnosed |
| Non-entity extended data | Diagnosed, including undecodable payloads; no preservation in this slice |

Defaults come from fresh CadDocument for in-memory/DXF, and one cached
unmodified fresh-DWG write/read for DWG. Roles follow named dictionaries,
never handle coincidence/count alone. Standard style handle/name caches and
duplicate raw DXF style records are normalized beside their typed comparisons;
authored style fields remain compared. Source identity/reference collisions fail.

An overall viewport is excluded only for the Layout1-linked ID-1 viewport with
all constructor-default values apart from verified owner/identity/storage fields.
Authored view settings or dimensions fail that classification. Other Paper
entities and extra layouts are rejected.

## Entity fields

| Fields | Treatment |
| --- | --- |
| Common identity/owner/layer | Non-null unique identity; one ordered owner; existing layer; return mappings |
| Common appearance | Independent ByLayer/ByBlock/Explicit modes. True RGB, Continuous, supported weights only. Indexed/named color and Default weight diagnosed |
| Common linetype handle/entity mode/raw record/data-store flag | Typed-name/owner or encoding caches, not persisted separately |
| Other common fields | Typed residual check includes XDATA/raw EED, display scale, visibility, named colors, graphics, material/plot/shadow/visual styles, references, reactors and dictionaries |
| Line start/end | Direct XYZ; diagnose thickness/nondefault normal |
| Circle center/radius | +Z normal only, canonical XY placement; diagnose thickness/nondefault normal |
| LwPolyline vertices/closure/elevation | Straight XY points at stored Z. Diagnose bulges, widths, vertex IDs, constant width, thickness, plinegen and nondefault normal |
| Insert target/point/rotation/scales | Existing local target, +Z normal, canonical XY placement; diagnose attributes, arrays/MINSERT, spacing, view/sequence handles and other residual fields |
| BLOCK/ENDBLK | If present, record handle/type/name/owner/base must agree; diagnose delimiter/common changes. Missing DXF/programmatic markers are normal |

Unused definitions participate in cycle/target validation. Case-insensitive
duplicate layer/block names, duplicate/null handles and multiple owners fail.
A stale missing model cache is recovered only through unique consistent
BlockRecord/Layout agreement, with diagnostic; conflicting existing cache fails.

Metadata is caller supplied. New IDs follow source table/owner enumeration and
remain u64; layer/definition vectors follow current core lexical path ordering.
Entity vectors retain owner order. Layout mappings use actual Layout handles;
definitions use BlockRecord handles. No handle or cross-file ID persistence claim.

Output goes through the production writer and strict reader. Tests cover large
IDs, different handles, unused definitions, cycles, missing references, tiny
scales, common-field loss and semantic DXF/DWG exchange. The known nonzero DWG
base/marker conflict is rejected without repair.
