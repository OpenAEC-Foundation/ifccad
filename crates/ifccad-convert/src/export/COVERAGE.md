# CadDocument export coverage contract

This inventory is pinned to cadcodec/acadrust revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. It defines what the
`CadDocument -> IFCCAD` exporter must either represent or diagnose. Updating the
dependency requires reviewing every row. The pinned cadcodec
`semantic_inventory_v1` is the export coverage traversal: its categories are
matched exhaustively, while existing typed exporters own field and geometry
checks. Default table, class and object content is compared by semantic role
and payload, rather than only collection length. The pinned fresh-document and
DWG-read bootstrap forms account for codec-created standard scaffolding.
Field-level classification remains a manual contract pending
[cadcodec issue #50](https://github.com/HakanSeven12/cadcodec/issues/50).

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
retain the existing loss diagnostics through their canonical inventory objects.
This does not certify fields erased before the CadDocument boundary.

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `version`, `maintenance_version`, `dwg_source_version` | NonSemantic | Physical source-codec selection is not drawing semantics. |
| `header.insertion_units` | Exact/PartialLoss | All 25 CAD codes 0–24 map exactly; unknown codes become `unitless` plus `UnsupportedUnit`. Coordinates are never rescaled. |
| `header.plotstyle_mode`, `header.paper_space_linetype_scaling` | Exact | Drawing plot-style mode and each emitted layout's saved linetype-scaling intent; the latter is one CAD header value copied to every layout. |
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
| `layers` | Exact/PartialLoss/SkippedLoss | Source order and unused layers are retained. On/Off, global Freeze, Lock, plottability, freeze-in-new-viewports and description are distinct native fields. Exact color, opacity, linetype name, and numeric/default lineweight become a deduplicated appearance. Unsupported references retain their loss diagnostics; a required unrepresentable appearance skips the layer. |
| Standard `Continuous`, `ByLayer`, `ByBlock`, and `Dashed` linetypes | Exact | Their standard definitions supply the initial supported appearance vocabulary. A modified definition, including an altered on-demand `Dashed`, is diagnosed rather than accepted by name alone. |
| Ordinary local `block_records` | Exact/PartialLoss/FatalIfInconsistent | Definitions allocated before ordered contents; public metadata retained as above. Attribute flags, preview and insertion-count bytes are partial losses. Record handles must be distinct/non-null; references, membership and cycles are checked. Unicode-fold name collisions cannot merge or retarget definitions. |
| Other `line_types`; `text_styles`; unsupported `block_records`; `dim_styles`; `app_ids`; `views`; `vports`; `ucss`; `vx_table` | SkippedLoss | One stable `UnsupportedTableRecords` summary per affected table. Default cadcodec bootstrap records are not reported; a changed or extra duplicate bootstrap record is reported even when the usual name is present. |
| Default model/paper layout and bootstrap dictionaries/objects | NonSemantic until changed or referenced | The untouched empty `Layout1` scaffold is omitted. Real paper layouts are emitted in tab order; model layout name is retained. |
| `objects` variants other than mapped `Layout` | SkippedLoss | Added or semantically changed objects contribute to one `objects` collection summary. Pinned bootstrap roles are matched through dictionary references, so a numeric handle change alone is not loss. Named reusable `PlotSettings` objects/page setups remain outside the native effective layout setting. Relationships exposed on entities/layers receive their more precise source-item reasons. |
| `classes` | SkippedLoss | One `UnsupportedCollection` summary for added, changed or extra duplicate class definitions; fresh and DWG-read standard class metadata are baseline scaffolding. |
| `vx_control_entries`, `block_visibility_params`, `context_scales`, `block_representations`, `fields`, `dgn_ls_definitions`, `dgn_ls_components`, `section_view_style`, `view_rep_refs`, `section_view_reps` | NonSemantic duplicate view | The V1 inventory omits these decoded side views. Their backing table, entity or object content is classified once at its canonical inventory part; an unattached side-view map entry alone is not a drawing component. |
| cadcodec caches/indexes, flat-storage bookkeeping, raw EED/ACDS payload bookkeeping, block membership caches, and next-handle allocation | NonSemantic/private boundary | Not accessible as independent public package semantics. Semantic typed content exposed elsewhere remains covered. |

## Layout, plot and viewport source-field inventory

This section applies to the pinned public `objects::Layout`, embedded plot
fields and `entities::Viewport`. A valid package produced by the converter is
strict-loaded in the converter tests; the exact profile below is narrower than
the native IFCCAD contract. `UnsupportedSemantic` names the affected source
field or family; `Reject` returns no package when such a loss is present.

| Pinned source fields | Status | Mapping or diagnostic | Reverse test |
| --- | --- | --- | --- |
| `Layout.name`, `tab_order`, `block_record`, `viewport`, `viewports` | Exact/FatalIfInconsistent | Drawing layout name, list order and scope binding; a missing paper block record is structural failure. The conventional overall viewport ID 1 is scaffold, while authored paper VIEWPORT entities follow owner/order. | Paper layout and viewport roundtrip |
| `Layout.flags` bit 2, `min_limits/max_limits`, header `paper_space_linetype_scaling` | Exact/PartialLoss | `limitsChecking`, optional authored `limits`, `paperSpaceLinetypeScaling`. Non-rectangular limits and unrelated layout flag bits diagnose loss. | Paper layout roundtrip |
| `paper_width/height`, `plot_paper_units`, `plot_rotation`, `plot_margin_*`, `plot_printer_name`, `paper_size` | Exact/SkippedLoss | Inline `media` geometry, printable area, rotation, device/media hints. Both dimensions zero mean absent `plotSettings`; invalid positive media or margins omit the complete plot value and diagnose loss. | Millimetre A4 roundtrip |
| `plot_type`, `plot_window_*` | Exact/SkippedLoss | Extents, Limits, Window and paper Layout map by mode. Active Display or NamedView omits all `plotSettings` with a specific loss; an invalid window does likewise. | Layout and Display tests |
| `plot_scale_numerator/denominator`, `plot_scale_type`, `plot_origin_x/y`, `plot_flags.plot_centered` | Exact/PartialLoss/SkippedLoss | Fixed/Fit scale and centered/media-relative offset. Unsupported Layout+Fit or Layout+Centered omits the complete plot value. A standard-scale preset code is reported as lost UI metadata. | Fixed Layout scale roundtrip |
| `shade_plot_mode/resolution/dpi`, `plot_style_sheet`, `plot_flags.plot_plot_styles` | Exact/PartialLoss | Native shading and style application/name. An active external CTB/STB table name is retained, but absent table contents produce loss. | Plot table-name and mode test |
| `plot_flags.plot_viewport_borders/draw_viewports_first/plot_hidden/print_lineweights/scale_lineweights` | Exact | Native named `PlotOptions` fields; no single opaque flags word. | Writer and paper roundtrip |
| `plot_flags` remaining bits, `plot_page_name`, `plot_view_name/handle`, `visual_style_handle`, `paper_image_origin_*`, insertion base/elevation/UCS, reactors/dictionary | PartialLoss | Nondefault values produce field/family `UnsupportedSemantic` diagnostics. Extent caches and derived image/scale caches are not independent effective settings. | Source-loss tests where present |
| `Viewport.center/width/height`, `view_center/target/direction/height`, `twist_angle`, `lens_length`, `render_mode`, enabled/locked flags, front/back clip flags and distances | Exact/SkippedLoss | Native frame/view/render/clip fields. Perspective remains whole-viewport skipped pending CAD fixture calibration. Invalid geometry is not approximated. | Rectangular viewport roundtrip |
| `Viewport.clip_boundary_handle` | Exact/SkippedLoss | A previously mapped same-paper closed straight `LwPolyline` becomes an active native `paperClip` reference. Missing, unsupported or forward references skip the entire viewport. | Closed and missing clip tests |
| `Viewport.frozen_layers` | Exact/PartialLoss | Each resolved handle becomes a relational frozen-layer override; unknown handles receive `MissingTarget`. The pinned Viewport model has no viewport appearance-override fields. | Native writer and reverse test |
| `Viewport` snap/grid/UCS, visual style/background/lighting and viewport plot-style fields | PartialLoss | Nondefault source state is reported, not silently replaced by a native default. | Source-loss tests where present |

PageSetup objects, CTB/STB file contents, Display/NamedView state and perspective
calibration remain deferred. A source's raw DXF plot-settings code pairs are a
codec preservation mechanism, not evidence that all unclassified future codes
were natively represented.

## Entity types and geometry

| `EntityType` variant | Status | Contract |
| --- | --- | --- |
| `Line` | Exact/PartialLoss/SkippedLoss | Finite XYZ endpoints are copied exactly. Non-default finite normals are partial source-property loss; geometry is retained. Unsupported thickness still skips the entity. |
| `LwPolyline` | Exact/PartialLoss/SkippedLoss | At least two finite local XY vertices, finite elevation and a finite nonzero normal define a plane through the pinned arbitrary-axis interpretation. Local points, order and closure are retained. Normal normalization and nonzero opaque vertex IDs (including negative IDs) are partial losses. Nonzero thickness/width/bulge and PLINEGEN still skip the whole entity. Geometric rounding is assessed separately and target range/evaluation failure is fatal under both policies. |
| `Insert` | Exact/PartialLoss/SkippedLoss/FatalIfInconsistent | Ordinary local references map to BlockInstance. OCS insertion coordinates map to owning-scope placement using the actual pinned CAD axes. Qualified trig intervals and exact residual propagation check each occurrence. Unsupported array/attribute/view/external variants skip as a whole; missing/cyclic targets are fatal. |
| `Viewport` | Exact/PartialLoss/SkippedLoss | Paper-owned orthographic rectangular viewports map frame, target/direction/height/twist, render mode, enabled/locked state, front/back clip and frozen layers. A mapped earlier same-scope closed straight `LwPolyline` may be an active clip. Missing or unsupported active boundaries and perspective skip the entire viewport; deferred snap/grid/UCS and visual state are diagnosed as partial loss. Appearance overrides are not exposed by the pinned CAD Viewport model. |
| `Block`, `BlockEnd` | NonSemantic/FatalIfInconsistent/SkippedLoss | Matching structural markers supply record scaffolding, not drawable entities. Contradictory exposed begin-marker name/owner/base is fatal; unmatched markers are unsupported entities. |
| `Point`, `Circle`, `Arc`, `Ellipse`, `Polyline`, `Polyline2D`, `Polyline3D`, `Text`, `MText`, `Spline`, `Helix`, `Dimension`, `Hatch`, `Solid`, `Face3D`, `Ray`, `XLine`, `AttributeDefinition`, `AttributeEntity`, `Leader`, `MultiLeader`, `MLine`, `Mesh`, `RasterImage`, `Solid3D`, `Region`, `Body`, `Surface`, `Table`, `Tolerance`, `PolyfaceMesh`, `Wipeout`, `Shape`, `Underlay`, `Seqend`, `Ole2Frame`, `PolygonMesh`, `Light`, `SectionSymbol`, `ViewBorder`, `Extended`, `Unknown` | SkippedLoss | Whole entity receives `UnsupportedEntityType`; no geometry is approximated. |
| Model, paper, or supported definition ownership | Exact/FatalIfInconsistent | Per-owner explicit entity-handle order is retained; known-owner contents must occur exactly once with consistent membership. |
| Unsupported block owner | SkippedLoss | `BlockOwnedEntity`. |
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

The scanner uses an exhaustive `SemanticPartV1` match for source categories,
and exhaustive Rust matches where cadcodec exposes closed enums (`EntityType`,
color/transparency/lineweight modes in the converter). Relationships and
non-entity EED retain separate inventory summaries only when a typed source
diagnostic does not already own their meaning. Open maps and private/raw
internals remain bounded by the inventory contract and this manual field
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
