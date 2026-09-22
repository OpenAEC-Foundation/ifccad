# CadDocument export coverage contract

This inventory is pinned to cadcodec/acadrust revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. It defines what the
`CadDocument -> IFCCAD` exporter must either represent or diagnose. Updating the
dependency requires reviewing every row. [cadcodec issue #30](https://github.com/HakanSeven12/cadcodec/issues/30)
tracks a compiler-visible upstream semantic inventory that can replace parts of
this manual audit.

Statuses are `Exact`, `PartialLoss`, `SkippedLoss`, `NonSemantic`, and
`FatalIfInconsistent`. `Reject` rejects detectable semantic loss within this pinned public model,
except numerical rounding proved to stay within the configured geometric
tolerance. Accepted rounding remains loss evidence. Private/raw cadcodec state is
outside that bounded guarantee.

## Document and header

### Local scopes and blocks

Local ordinary block definitions
now retain name, base point, description, anonymous flag, symbolic insertion
unit, explodability and signed-uniform scaling policy; supported primitive
contents remain in their definition scope without base-point pretranslation.
External definitions remain unsupported. Present begin markers are checked
against their records, and missing INSERT targets and cycles are fatal under
both loss policies. The known DWG marker inconsistency is diagnosed with a
reference to cadcodec #52; absent DXF markers are allowed.

Ordinary instances retain references, placement, rotation, signed scale and
visibility without explosion. Nested occurrence-space assessment includes outer
scaling of inner conversion errors. Changed source normal values are reported
as `SourceNormalNormalized`, including instances of empty definitions.
Unsupported definition contents are diagnosed
and `BlockContentLoss` identifies affected instances. Arrays (including nondefault
1x1 spacing), attributes and view-representation inserts are omitted as whole
unsupported entities, not emitted as ordinary instances. Dynamic behavior and
external references remain unsupported; their exposed attachments/collections
retain the existing loss diagnostics. This does not resume the parked exhaustive
inventory migration or certify fields erased before the CadDocument boundary.

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `version`, `maintenance_version`, `dwg_source_version` | NonSemantic | Physical source-codec selection is not drawing semantics. |
| `header.insertion_units` | Exact/PartialLoss | All 25 CAD codes 0–24 map exactly; unknown codes become `unitless` plus `UnsupportedUnit`. Coordinates are never rescaled. |
| `header.model_space_block_handle` and the related `Layout.block_record` | Exact/FatalIfInconsistent | The relationship selects the one model layout; null, missing, or ambiguous structure is fatal. Numeric handle replacement itself is not loss. |
| `header.handle_seed`, table-control handles, dictionary handles, and standard-record handles | NonSemantic | Numeric serialization identity alone is ignored. Meaningful referenced content is covered at its table/object/entity source. |
| `header.project_name` | SkippedLoss | `UnsupportedHeaderField { project_name }`. |
| `header.model_space_extents_min/max`, `header.paper_space_extents_min/max` | NonSemantic | Cached geometry bounds may be unset, stale or recomputed by a codec. Output IFCDR bounds are calculated from emitted geometry; these caches are not independent drawing settings. Drawing limits are separate and remain diagnosed. |
| Every other `HeaderVariables` drawing setting (mode flags, precision, scales, current defaults, dimension variables, limits, UCS, dates and textual metadata) | SkippedLoss | A conservative `header.other_semantics` diagnostic is emitted whenever the public header differs from the pinned fresh-document baseline after exact/nonsemantic fields are normalized. |
| `summary_info` | SkippedLoss | One `DocumentSummaryInformation` diagnostic. |
| `source_path` | NonSemantic | Host filesystem provenance is not package drawing semantics. |
| `notifications` | NonSemantic | Parser/writer messages are operational state, not source drawing content. |
| `preview` | SkippedLoss | One `UnsupportedCollection { preview }` diagnostic when present. |

## Tables, objects, and public side views

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `layers` | Exact/PartialLoss/SkippedLoss | Source order and empty layers are retained. `off || frozen` maps to visibility. Exact color, opacity, linetype name, and numeric/default lineweight become a deduplicated appearance. Auxiliary flags/references are bundled as partial loss; a required unrepresentable appearance skips the layer. |
| Standard `Continuous`, `ByLayer`, `ByBlock`, and `Dashed` linetypes | Exact | Their names supply the initial supported appearance vocabulary. |
| Ordinary local `block_records` | Exact/PartialLoss/FatalIfInconsistent | Definitions allocated before ordered contents; public metadata retained as above. Attribute flags, preview and insertion-count bytes are partial losses. Record handles must be distinct/non-null; references, membership and cycles are checked. Unicode-fold name collisions cannot merge or retarget definitions. |
| Other `line_types`; `text_styles`; unsupported `block_records`; `dim_styles`; `app_ids`; `views`; `vports`; `ucss`; `vx_table` | SkippedLoss | One stable `UnsupportedTableRecords` summary per affected table. Default cadcodec bootstrap records are not reported. |
| Default model/paper layout and bootstrap dictionaries/objects | NonSemantic until changed or referenced | Required cadcodec database scaffolding is not diagnosed merely for existing. Model layout name is exported exactly. |
| `objects` variants `Dictionary`, `Layout`, `XRecord`, `Group`, `MLineStyle`, `ImageDefinition`, `UnderlayDefinition`, `PlotSettings`, `MultiLeaderStyle`, `TableStyle`, `TableContent`, `Scale`, `ObjectContextData`, `SortEntitiesTable`, `DictionaryVariable`, `VisualStyle`, `Material`, `ImageDefinitionReactor`, `GeoData`, `SpatialFilter`, `RasterVariables`, `BookColor`, `PlaceHolder`, `DictionaryWithDefault`, `WipeoutVariables`, `BlockVisibilityParameter`, `DynamicBlock`, `Associative`, `ClassObject`, `DataObject`, `Field`, `FieldList`, `RegisteredClass`, `DgnLineStyle`, `ProxyObject`, and `Unknown` | SkippedLoss | Added object content beyond the pinned bootstrap set produces an `objects` collection summary. Relationships exposed on entities/layers receive their more precise source-item reasons. |
| `classes`, `vx_control_entries`, `block_visibility_params`, `context_scales`, `block_representations`, `fields`, `dgn_ls_definitions`, `dgn_ls_components`, `section_view_style`, `view_rep_refs`, `section_view_reps` | SkippedLoss | One stable `UnsupportedCollection` summary when non-default content exists. |
| cadcodec caches/indexes, flat-storage bookkeeping, raw EED/ACDS payload bookkeeping, block membership caches, and next-handle allocation | NonSemantic/private boundary | Not accessible as independent public package semantics. Semantic typed content exposed elsewhere remains covered. |

## Entity types and geometry

| `EntityType` variant | Status | Contract |
| --- | --- | --- |
| `Line` | Exact/PartialLoss/SkippedLoss | Finite XYZ endpoints are copied exactly. Non-default finite normals are partial source-property loss; geometry is retained. Unsupported thickness still skips the entity. |
| `LwPolyline` | Exact/PartialLoss/SkippedLoss | At least two finite local XY vertices, finite elevation and a finite nonzero normal define a plane through the pinned arbitrary-axis interpretation. Local points, order and closure are retained. Normal normalization and nonzero opaque vertex IDs (including negative IDs) are partial losses. Nonzero thickness/width/bulge and PLINEGEN still skip the whole entity. Geometric rounding is assessed separately and target range/evaluation failure is fatal under both policies. |
| `Insert` | Exact/PartialLoss/SkippedLoss/FatalIfInconsistent | Ordinary local references map to BlockInstance. OCS insertion coordinates map to owning-scope placement using the actual pinned CAD axes. Qualified trig intervals and exact residual propagation check each occurrence. Unsupported array/attribute/view/external variants skip as a whole; missing/cyclic targets are fatal. |
| `Block`, `BlockEnd` | NonSemantic/FatalIfInconsistent/SkippedLoss | Matching structural markers supply record scaffolding, not drawable entities. Contradictory exposed begin-marker name/owner/base is fatal; unmatched markers are unsupported entities. |
| `Point`, `Circle`, `Arc`, `Ellipse`, `Polyline`, `Polyline2D`, `Polyline3D`, `Text`, `MText`, `Spline`, `Helix`, `Dimension`, `Hatch`, `Solid`, `Face3D`, `Ray`, `XLine`, `Viewport`, `AttributeDefinition`, `AttributeEntity`, `Leader`, `MultiLeader`, `MLine`, `Mesh`, `RasterImage`, `Solid3D`, `Region`, `Body`, `Surface`, `Table`, `Tolerance`, `PolyfaceMesh`, `Wipeout`, `Shape`, `Underlay`, `Seqend`, `Ole2Frame`, `PolygonMesh`, `Light`, `SectionSymbol`, `ViewBorder`, `Extended`, `Unknown` | SkippedLoss | Whole entity receives `UnsupportedEntityType`; no geometry is approximated. |
| Model-space or supported definition ownership | Exact/FatalIfInconsistent | Per-owner explicit entity-handle order is retained; known-owner contents must occur exactly once with consistent membership. |
| Paper-space or unsupported block owner | SkippedLoss | `PaperSpaceEntity` or `BlockOwnedEntity`. |
| Null or unknown owner | FatalIfInconsistent | All safely detectable owner problems are aggregated before return. |
| Source handle number | NonSemantic | Replaced by a sequential IFCDR ID; `ExportEntityMapping` records the operational correspondence. |

## `EntityCommon`

| Field | Status | Contract |
| --- | --- | --- |
| `handle` | NonSemantic | See mapping rule above. |
| `owner_handle` | Exact/SkippedLoss/FatalIfInconsistent | See ownership rules above. |
| `layer` | Exact/SkippedLoss | Case-insensitive lookup to an emitted source layer; no replacement layer is invented. |
| `color`, `transparency`, `linetype`, `line_weight` | Exact/SkippedLoss | Per-property ByLayer/ByBlock/Explicit modes are retained. A required unrepresentable explicit value skips the entity. |
| `color_name` | Exact/SkippedLoss | `catalog$name` becomes named/color-book identity; malformed or inherited combinations are diagnosed. |
| `invisible` | Exact | Inverted into IFCDR visibility. |
| `linetype_scale`, `linetype_handle`, `extended_data`, `graphic_data`, `reactors`, `xdictionary_handle`, color-book/visual-style handles, material fields, `shadow_flags`, plot-style fields | PartialLoss | Geometry can still emit; all present meanings are bundled into the same entity diagnostic. |
| `entity_mode` | NonSemantic when consistent | Redundant physical ownership encoding; authoritative ownership is `owner_handle`. |
| `has_ds_data` | NonSemantic bookkeeping | The corresponding modeler entity/geometry is independently skipped as unsupported. |

## Maintenance rule

The 2026-09-11 update reviewed the public document/header, entity/common,
table/object and appearance surfaces against the previous `a0f7d44` pin.
Line, LwPolyline, EntityCommon and document/header fields remain unchanged.
New spline/MTEXT fields and solid-history/hatch-context object fields fall
under the existing unsupported entity/object categories. Serialized drawing
variables (including current transparency and default hatch origin) use
Dictionary/DictionaryVariable objects and are diagnosed by the objects scan;
a regression verifies reporting and Reject for a non-default hatch origin.
Upstream's transparency quantization now rounds the transparency byte upward
to match the complementary packed opacity; import coverage remains bounded.

The scanner uses exhaustive Rust matches where cadcodec exposes closed enums
(`EntityType`, color/transparency/lineweight modes in the converter). Open maps
and private/raw internals are bounded by explicit collection checks and this
matrix. New cadcodec fields or variants must update this file, tests, and the
scanner before the pinned revision changes. Fully preserved future IFCPR content
will cease to be package loss even when it remains non-native in IFCDR.

## Coordinate-frame accuracy

IFCDR 0.8.0 stores XYZ lines and local XY polylines with a complete optional
plane placement. The converter alone interprets CAD arbitrary axes, using the
actual axes returned by the pinned helper after robust scaled normalization.
It never assumes repeated normalization preserves all source normal bits.

`geometry_tolerance` is a hard Euclidean limit under both `Allow` and `Reject`.
The default is exactly one micrometre for known drawing units and zero for
unitless drawings. Explicit physical tolerances require a known unit. Unit
conversion and squared residual comparison use exact rational values; distance
reports give outward bounds in drawing units. Every emitted line endpoint and
polyline vertex is covered. Failure to establish accuracy returns a typed error
and no partial package. A numerical diagnostic within the limit is exempt from
Reject, but `transfer_assessment()` still records `LossDetected`.

The proof concerns the interpreted CadDocument geometry, not original CAD file
bytes, cached bounds, private codec state or recovery of unsupported source data.
