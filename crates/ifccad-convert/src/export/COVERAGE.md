# CadDocument export coverage contract

This inventory is pinned to cadcodec/acadrust revision
`a0f7d444f1607bc4b2c881060cbe7ea1014253cb`. It defines what the initial
`CadDocument -> IFCCAD` exporter must either represent or diagnose. Updating the
dependency requires reviewing every row. [cadcodec issue #30](https://github.com/HakanSeven12/cadcodec/issues/30)
tracks a compiler-visible upstream semantic inventory that can replace parts of
this manual audit.

Statuses are `Exact`, `PartialLoss`, `SkippedLoss`, `NonSemantic`, and
`FatalIfInconsistent`. `Reject` guarantees that every loss detectable through
this pinned public model rejects the export. Private/raw cadcodec state is
outside that bounded guarantee.

## Document and header

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `version`, `maintenance_version`, `dwg_source_version` | NonSemantic | Physical source-codec selection is not drawing semantics. |
| `header.insertion_units` | Exact/PartialLoss | Seven IFCDR units map exactly; every other code becomes `unitless` plus `UnsupportedUnit`. Coordinates are never rescaled. |
| `header.model_space_block_handle` and the related `Layout.block_record` | Exact/FatalIfInconsistent | The relationship selects the one model layout; null, missing, or ambiguous structure is fatal. Numeric handle replacement itself is not loss. |
| `header.handle_seed`, table-control handles, dictionary handles, and standard-record handles | NonSemantic | Numeric serialization identity alone is ignored. Meaningful referenced content is covered at its table/object/entity source. |
| `header.project_name` | SkippedLoss | `UnsupportedHeaderField { project_name }`. |
| Every other `HeaderVariables` drawing setting (mode flags, precision, scales, current defaults, dimension variables, limits/extents, UCS, dates and textual metadata) | SkippedLoss | A conservative `header.other_semantics` diagnostic is emitted whenever the public header differs from the pinned fresh-document baseline after exact/nonsemantic fields are normalized. |
| `summary_info` | SkippedLoss | One `DocumentSummaryInformation` diagnostic. |
| `source_path` | NonSemantic | Host filesystem provenance is not package drawing semantics. |
| `notifications` | NonSemantic | Parser/writer messages are operational state, not source drawing content. |
| `preview` | SkippedLoss | One `UnsupportedCollection { preview }` diagnostic when present. |

## Tables, objects, and public side views

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `layers` | Exact/PartialLoss/SkippedLoss | Source order and empty layers are retained. `off || frozen` maps to visibility. Exact color, opacity, linetype name, and numeric/default lineweight become a deduplicated appearance. Auxiliary flags/references are bundled as partial loss; a required unrepresentable appearance skips the layer. |
| Standard `Continuous`, `ByLayer`, `ByBlock`, and `Dashed` linetypes | Exact | Their names supply the initial supported appearance vocabulary. |
| Other `line_types`; `text_styles`; non-layout `block_records`; `dim_styles`; `app_ids`; `views`; `vports`; `ucss`; `vx_table` | SkippedLoss | One stable `UnsupportedTableRecords` summary per affected table. Default cadcodec bootstrap records are not reported. |
| Default model/paper layout and bootstrap dictionaries/objects | NonSemantic until changed or referenced | Required cadcodec database scaffolding is not diagnosed merely for existing. Model layout name is exported exactly. |
| `objects` variants `Dictionary`, `Layout`, `XRecord`, `Group`, `MLineStyle`, `ImageDefinition`, `UnderlayDefinition`, `PlotSettings`, `MultiLeaderStyle`, `TableStyle`, `TableContent`, `Scale`, `ObjectContextData`, `SortEntitiesTable`, `DictionaryVariable`, `VisualStyle`, `Material`, `ImageDefinitionReactor`, `GeoData`, `SpatialFilter`, `RasterVariables`, `BookColor`, `PlaceHolder`, `DictionaryWithDefault`, `WipeoutVariables`, `BlockVisibilityParameter`, `DynamicBlock`, `Associative`, `ClassObject`, `DataObject`, `Field`, `FieldList`, `RegisteredClass`, `DgnLineStyle`, `ProxyObject`, and `Unknown` | SkippedLoss | Added object content beyond the pinned bootstrap set produces an `objects` collection summary. Relationships exposed on entities/layers receive their more precise source-item reasons. |
| `classes`, `vx_control_entries`, `block_visibility_params`, `context_scales`, `block_representations`, `fields`, `dgn_ls_definitions`, `dgn_ls_components`, `section_view_style`, `view_rep_refs`, `section_view_reps` | SkippedLoss | One stable `UnsupportedCollection` summary when non-default content exists. |
| cadcodec caches/indexes, flat-storage bookkeeping, raw EED/ACDS payload bookkeeping, block membership caches, and next-handle allocation | NonSemantic/private boundary | Not accessible as independent public package semantics. Semantic typed content exposed elsewhere remains covered. |

## Entity types and geometry

| `EntityType` variant | Status | Contract |
| --- | --- | --- |
| `Line` | Exact/SkippedLoss | Exact only for finite XY endpoints, zero endpoint Z, zero thickness, and normal `(0,0,1)`. All failed predicates are bundled. |
| `LwPolyline` | Exact/SkippedLoss | Exact only for at least two finite XY vertices, zero elevation/thickness/width/bulge, positive-unit-Z normal, and no PLINEGEN. Open/closed state and vertex order are exact. |
| `Point`, `Circle`, `Arc`, `Ellipse`, `Polyline`, `Polyline2D`, `Polyline3D`, `Text`, `MText`, `Spline`, `Helix`, `Dimension`, `Hatch`, `Solid`, `Face3D`, `Insert`, `Block`, `BlockEnd`, `Ray`, `XLine`, `Viewport`, `AttributeDefinition`, `AttributeEntity`, `Leader`, `MultiLeader`, `MLine`, `Mesh`, `RasterImage`, `Solid3D`, `Region`, `Body`, `Surface`, `Table`, `Tolerance`, `PolyfaceMesh`, `Wipeout`, `Shape`, `Underlay`, `Seqend`, `Ole2Frame`, `PolygonMesh`, `Light`, `SectionSymbol`, `ViewBorder`, `Extended`, `Unknown` | SkippedLoss | Whole entity receives `UnsupportedEntityType`; blocks are not exploded and no geometry is approximated. |
| Model-space ownership | Exact | Entity is emitted in cadcodec storage order. |
| Paper-space or another valid block owner | SkippedLoss | `PaperSpaceEntity` or `BlockOwnedEntity`. |
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

The scanner uses exhaustive Rust matches where cadcodec exposes closed enums
(`EntityType`, color/transparency/lineweight modes in the converter). Open maps
and private/raw internals are bounded by explicit collection checks and this
matrix. New cadcodec fields or variants must update this file, tests, and the
scanner before the pinned revision changes. Fully preserved future IFCPR content
will cease to be package loss even when it remains non-native in IFCDR.
