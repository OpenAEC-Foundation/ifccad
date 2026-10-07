# CadDocument to OCDraw source coverage

The complete typed `OcdrawDocument` is the logical conversion boundary.
Logical export returns it with the existing loss/mapping/accuracy evidence;
encoded export delegates to the core encoder. Raw document import validates
before construction; validated reader import uses the same implementation.
These are fresh conversions: no identity-preserving CAD editing session,
watermark reconstruction or new source coverage is implied.

This inventory is pinned to opencadcodec revision
`063c10671fe7833d562f772159771318c7a0ebb9`. It defines what the
`CadDocument -> OCDraw` exporter must either represent or diagnose. Updating the
dependency requires reviewing every row. The pinned opencadcodec
`semantic_inventory_v1` is the export coverage traversal: its categories are
matched exhaustively, while existing typed exporters own field and geometry
checks. Default table, class and object content is compared by semantic role
and payload, rather than only collection length. The pinned fresh-document and
DWG-read bootstrap forms account for codec-created standard scaffolding.
Field-level classification remains a manual contract pending
[opencadcodec issue #50](https://github.com/HakanSeven12/opencadcodec/issues/50).
The Rust dependency remains named `opencadcodec` locally; its package/repository is
opencadcodec. The [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-05.md)
records added and changed public fields for both converters.

Statuses are `Exact`, `PartialLoss`, `SkippedLoss`, `NonSemantic`, and
`FatalIfInconsistent`. `Reject` rejects detectable semantic loss within this pinned public model,
except numerical rounding proved to stay within the configured geometric
tolerance. Accepted rounding remains loss evidence. Private/raw opencadcodec state is
outside that bounded guarantee.

## Document and header

### Local scopes and blocks

Local ordinary block definitions
now retain name, base point, description, anonymous flag, symbolic insertion
unit, explodability and signed-uniform scaling policy; supported primitive
contents remain in their definition scope without base-point pretranslation.
External definitions remain unsupported. Present begin markers are checked
against their records, and missing INSERT targets and cycles are fatal under
both loss policies. The pinned DWG reader preserves nonzero marker base points
and anonymous block names. Defects #52/#55 are resolved; the former anonymous
name repair is no longer needed.
All present marker conflicts are fatal; absent DXF markers are allowed.
Block descriptions survive both DXF and DWG (#49). Extra paper-block DWG
markers can still refer to the primary paper owner and are rejected.

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
| `header.insertion_units` | Exact/PartialLoss | All 25 CAD codes 0â€“24 map exactly; unknown codes become `unitless` plus `UnsupportedUnit`. Coordinates are never rescaled. |
| `header.plotstyle_mode`, `header.paper_space_linetype_scaling` | Exact | Drawing plot-style mode and each emitted layout's saved linetype-scaling intent; saved Paper values use layout flag bit 1; the current header supplies active Model state rather than replacing inactive layouts. |
| `header.point_display_mode`, `header.point_display_size` | Exact/SkippedLoss | Supported PDMODE glyph/enclosure bits and finite PDSIZE values map to grouped `Drawing.attributes.pointDisplay`; PDSIZE zero remains distinct from explicit negative five percent. Unsupported bits or nonfinite size skip the setting with `UnsupportedHeaderField { point_display }`. |
| `header.model_space_block_handle` and the related `Layout.block_record` | Exact/FatalIfInconsistent | The relationship selects the one model layout. A stale nonnull header handle with no block-record target is corrected only when the named `*Model_Space` block record and exactly one layout agree on another handle; this covers a pinned DXF-reader handle-repair defect without changing the caller's document. Null, missing, or ambiguous structure remains fatal. Numeric handle replacement itself is not loss. |
| `header.handle_seed`, table-control handles, dictionary handles, and standard-record handles | NonSemantic | Numeric serialization identity alone is ignored. Meaningful referenced content is covered at its table/object/entity source. |
| `header.project_name` | SkippedLoss | `UnsupportedHeaderField { project_name }`. |
| `header.model_space_extents_min/max`, `header.paper_space_extents_min/max` | NonSemantic | Cached geometry bounds may be unset, stale or recomputed by a codec. Output OCDraw bounds are calculated from emitted geometry; these caches are not independent drawing settings. Drawing limits are separate and remain diagnosed. |
| `header.current_layer_name`, `model_space_ucs_name/origin/x_axis/y_axis` | Exact/SkippedLoss | With a mapped active VPORT, the current Layer and named/unnamed/World UCS become drawing workspace state. Missing target Layer/UCS or an invalid frame is diagnosed. |
| `header.show_model_space`, `paper_space_block_handle` | Exact/SkippedLoss | Model space selects the model layout. For paper space, a matching exported layout with a recoverable paper canvas becomes active. An absent or unresolved block or canvas receives loss evidence and falls back to the model layout, keeping the drawing valid. A viewport-free paper layout remains without a canvas even when it has valid layout limits. Zeroed viewport IDs remain unresolved rather than guessed. |
| Every other `HeaderVariables` drawing setting (mode flags, precision, scales, other current defaults, dimension variables, limits, dates and textual metadata) | SkippedLoss | A conservative `header.other_semantics` diagnostic is emitted whenever the public header differs from the pinned fresh-document baseline after mapped/nonsemantic fields are normalized. |
| `summary_info` | SkippedLoss | One `DocumentSummaryInformation` diagnostic. |
| `source_path` | NonSemantic | Host filesystem provenance is not drawing drawing semantics. |
| `notifications` | NonSemantic | Parser/writer messages are operational state, not source drawing content. |
| `preview` | SkippedLoss | One `UnsupportedCollection { preview }` diagnostic when present. |

## Tables, objects, and public side views

| Source surface | Status | Export or diagnostic contract |
| --- | --- | --- |
| `layers` | Exact/PartialLoss/SkippedLoss | Source order and unused layers are retained. On/Off, global Freeze, Lock, plottability, freeze-in-new-viewports and description are distinct native fields. Exact color, opacity, linetype name, and numeric/default lineweight become a deduplicated appearance. Unsupported references retain their loss diagnostics; a required unrepresentable appearance skips the layer. |
| Named ordinary linetypes | Exact/PartialLoss/FatalIfInconsistent | All simple actual definitions and unused records become local pattern IDs. Empty definitions remain separately named. Text/shapes use one diagnosed named continuous fallback per definition under Allow; Reject refuses it. Modified ByLayer/ByBlock scaffolding and xref provenance are diagnosed separately. Invalid patterns/alignment and unresolved or contradictory references are errors. |
| Ordinary local `block_records` | Exact/PartialLoss/FatalIfInconsistent | Definitions allocated before ordered contents; public metadata retained as above. Attribute flags, preview and insertion-count bytes are partial losses. Record handles must be distinct/non-null; references, membership and cycles are checked. Unicode-fold name collisions cannot merge or retarget definitions. |
| Other `line_types`; `text_styles`; unsupported `block_records`; `dim_styles`; `app_ids`; `views`; unmapped `vports`/`ucss`; `vx_table` | SkippedLoss | One stable `UnsupportedTableRecords` summary per affected table. Mapped VPORT/UCS base fields are removed from the residual comparison; meaningful unmapped fields still produce a loss. Default opencadcodec bootstrap records are not reported. |
| Active model `*Active` VPORT entries and UCS table | Exact/PartialLoss | Ordered model windows keep normalized rectangles, view, grid, grid snap, stored UCS and activation flag. Named UCS records keep name, frame and elevation even when unused. Multiple active entries remain ordered, but opencadcodec exposes no certain active-window identity; this receives a loss diagnostic. Orthographic/external UCS data and residual VPORT fields receive loss diagnostics. |
| Default model/paper layout and bootstrap dictionaries/objects | NonSemantic until changed or referenced | The untouched empty `Layout1` scaffold is omitted. A primary reserved `*Paper_Space` record with no layout, content, insert reference or authored metadata is codec bookkeeping, not a local block definition; changed metadata/content is not discarded. Real paper layouts are emitted in tab order; model layout name is retained. Correctly named ACAD_LAYOUT dictionary entries are represented through those typed layouts, including absence of the bootstrap Layout1. Dictionary flags, aliases, unknown keys and dangling/non-layout targets remain independently classified. |
| `objects` variants other than mapped `Layout` | SkippedLoss | Added or semantically changed objects contribute to one `objects` collection summary. Pinned bootstrap roles are matched through dictionary references, so a numeric handle change alone is not loss. Named reusable `PlotSettings` objects/page setups remain outside the native effective layout setting. Relationships exposed on entities/layers receive their more precise source-item reasons. |
| `classes` | SkippedLoss | One `UnsupportedCollection` summary for added, changed or extra duplicate class definitions; fresh and DWG-read standard class metadata are baseline scaffolding. |
| `vx_control_entries`, `block_visibility_params`, `context_scales`, `block_representations`, `fields`, `dgn_ls_definitions`, `dgn_ls_components`, `section_view_style`, `view_rep_refs`, `section_view_reps` | NonSemantic duplicate view | The V1 inventory omits these decoded side views. Their backing table, entity or object content is classified once at its canonical inventory part; an unattached side-view map entry alone is not a drawing component. |
| opencadcodec caches/indexes, flat-storage bookkeeping, raw EED/ACDS payload bookkeeping, block membership caches, and next-handle allocation | NonSemantic/private boundary | Not accessible as independent public drawing semantics. Semantic typed content exposed elsewhere remains covered. |

## Layout, plot and viewport source-field inventory

This section applies to the pinned public `objects::Layout`, embedded plot
fields and `entities::Viewport`. A valid drawing produced by the converter is
strict-loaded in the converter tests; the exact profile below is narrower than
the native OCDraw contract. `UnsupportedSemantic` names the affected source
field or family; `Reject` returns no drawing when such a loss is present.

| Pinned source fields | Status | Mapping or diagnostic | Reverse test |
| --- | --- | --- | --- |
| `Layout.name`, `tab_order`, `block_record`, `viewport`, `viewports` | Exact/PartialLoss/FatalIfInconsistent | Drawing layout name, list order and scope binding; a missing paper block record is structural failure. The unique overall viewport ID 1 owned by the paper block supplies the canvas even if the `Layout.viewport` link is missing. A layout with no owned VIEWPORT and no viewport link exports no paper canvas. Authored paper VIEWPORT entities follow owner/order. | Paper canvas and viewport roundtrip |
| `Layout.flags` bits 1/2, `min_limits/max_limits`, header `paper_space_linetype_scaling` | Exact/PartialLoss | `limitsChecking`, optional authored `limits`, `paperSpaceLinetypeScaling`. Non-rectangular limits and unrelated layout flag bits diagnose loss. | Paper layout roundtrip |
| `paper_width/height`, `plot_paper_units`, `plot_rotation`, `plot_margin_*`, `plot_printer_name`, `paper_size` | Exact/SkippedLoss | Independent layout medium (mm), plot unit, page rectangle/rotation and device/media hints. The pinned reader parses padded ASCII DXF integers into typed fields. Those fields are authoritative after edits; retained raw codes never override them. Both dimensions zero mean absent media; media-only states retain dimensions without invented plots. Invalid media/page state omits the affected value with loss. Exact scalar conversion is required. | Millimetre A4 and padded DXF plot roundtrip |
| `plot_type`, `plot_window_*` | Exact/SkippedLoss | Extents, Limits, Window and paper Layout map by mode. Active Display or NamedView omits all `plotSettings` with a specific loss; an invalid window does likewise. | Layout and Display tests |
| `plot_scale_numerator/denominator`, `plot_scale_type`, `plot_origin_x/y`, `plot_flags.plot_centered` | Exact/PartialLoss/SkippedLoss | Fixed/Fit scale and centered/media-relative offset. Unsupported Layout+Fit or Layout+Centered omits the complete plot value. A standard-scale preset code is reported as lost UI metadata. | Fixed Layout scale roundtrip |
| `shade_plot_mode/resolution/dpi`, `plot_style_sheet`, `plot_flags.plot_plot_styles` | Exact/PartialLoss | Native shading and style application/name. An active external CTB/STB table name is retained, but absent table contents produce loss. | Plot table-name and mode test |
| `plot_flags.plot_viewport_borders/draw_viewports_first/plot_hidden/print_lineweights/scale_lineweights` | Exact | Native named `PlotOptions` fields; no single opaque flags word. | Writer and paper roundtrip |
| `plot_flags` remaining bits, `plot_page_name`, `plot_view_name/handle`, `visual_style_handle`, `paper_image_origin_*`, insertion base/elevation/UCS, reactors/dictionary | PartialLoss | Nondefault values produce field/family `UnsupportedSemantic` diagnostics. Extent caches and derived image/scale caches are not independent effective settings. | Source-loss tests where present |
| `Viewport.center/width/height`, `view_center/target/direction/height`, `twist_angle`, `lens_length`, `render_mode`, enabled/locked flags, front/back clip flags and distances | Exact/SkippedLoss | Native frame/view/render/clip fields, including a stored zero dormant lens in Orthographic. Perspective remains whole-viewport skipped pending CAD fixture calibration. Invalid geometry is not approximated. | Rectangular and zero-lens viewport readback |
| `Viewport.status` bit 0x10000 and `clip_boundary_handle` | Exact/SkippedLoss | Activation and stored reference are independent. Active Circle, full Ellipse and closed straight/bulged PlanarPolyline clips use shared converted-geometry validation in the same paper scope. Dormant stored references need ownership, uniqueness and convertible geometry but no active family/frame test. Both orders are retained; all conflicting claimants are skipped. An active missing boundary, a missing/skipped stored target, or wrong-scope target skips the dependent viewport with loss evidence; Reject refuses it. Hard geometry errors remain fatal. | Active/dormant native readback and locally patched DXF/DWG exchange |
| `Viewport.frozen_layers` | Exact/PartialLoss | Each resolved handle becomes a relational frozen-layer override; unknown handles receive `MissingTarget`. The pinned Viewport model has no viewport appearance-override fields. | Native writer and reverse test |
| Overall paper `Viewport` ID 1 view/grid/snap/UCS | Exact/PartialLoss | View center, target, direction, height, twist, clip, grid, snap and stored UCS become a `paperCanvas` workspace row. A nondefault screen-sized frame and nonpositive disabled snap spacing are diagnosed as loss; the latter is normalized to positive defaults required by OCDraw. The active viewport context remains unavailable. The pinned DWG reader retains viewport IDs/status. Unresolved or malformed overall identity still receives loss evidence. | Paper canvas roundtrip and active-tab recovery test |
| Authored paper `Viewport` snap/grid/UCS, visual style/background/lighting and viewport plot-style fields | PartialLoss | Nondefault source state is reported. Per-viewport workspace rows are not asserted natively by this converter. | Source-loss tests where present |
| `Viewport.off_screen` | PartialLoss | DXF off-screen/active-limit state has no native equivalent. True receives explicit loss evidence on both overall paper canvases and authored viewports; Reject prevents output. | DXF readback and both loss policies |

PageSetup objects, CTB/STB file contents, Display/NamedView state and perspective
calibration remain deferred. A source's raw DXF plot-settings code pairs are a
codec preservation mechanism, not evidence that all unclassified future codes
were natively represented.

## Entity types and geometry

| `EntityType` variant | Status | Contract |
| --- | --- | --- |
| `Point` | Exact/PartialLoss/SkippedLoss | Finite WCS location, normal and X-axis angle map to an oriented OCDraw Point placement. Normalization is diagnosed; nonzero thickness or an invalid frame skips the entity. Point size/form come from the drawing header. |
| `Circle`, `Arc` | Exact/PartialLoss/SkippedLoss | Finite OCS centre, positive radius and valid normal map through the pinned arbitrary-axis frame. Arc start/end angles map to start plus positive sweep; zero/full/multiple-turn sweeps, invalid frames and nonzero thickness skip the entity. Normalization is diagnosed. Sample residuals locate proven errors; a conservative centre-and-axis bound covers the entire parameter-matched curve and stored sweep rounding. |
| `Ellipse` | Exact/PartialLoss/SkippedLoss | Finite WCS centre and major-axis vector, valid normal and minor/major ratio map to Ellipse or EllipseArc according to exact full versus partial parameter span. Nearly full spans remain EllipseArc; invalid frame, ratio or span skips the entity. The entire parameter-matched curve is bounded in addition to representative samples. |
| `Line` | Exact/PartialLoss/SkippedLoss | Finite XYZ endpoints are copied exactly. Non-default finite normals are partial source-property loss; geometry is retained. Unsupported thickness still skips the entity. |
| `LwPolyline` | Exact/PartialLoss/SkippedLoss | At least two finite local XY vertices, finite elevation and a finite nonzero normal define a plane through the pinned arbitrary-axis interpretation. Vertices, signed bulges including a dormant final open bulge, order and closure map to PlanarPolyline. Width-only sources retain the zero-width centre path under Allow with `PolylineWidth` partial loss; Reject refuses the loss. PLINEGEN maps to continuous generation. Nonzero thickness and invalid curved segments skip the whole entity. Normal normalization and opaque vertex IDs are diagnosed. |
| `Polyline2D` | Exact/PartialLoss/SkippedLoss | Ordinary planar vertices and bulges map to PlanarPolyline using the same OCS preparation. Width-only sources follow the centre-path partial-loss policy. Fit/spline-fit, nonzero vertex Z and unsupported flags or thickness skip the whole entity. |
| `Polyline`, `Polyline3D` | Exact/SkippedLoss | Ordinary finite straight XYZ vertices, closure and continuous-generation flags map to SpatialPolyline. Codec-created vertex handles and an empty/default vertex layer are scaffolding; nondefault vertex layers and semantic vertex/source properties remain unsupported. Fit/spline-fit, mesh/polyface and unsupported vertex/source properties skip the whole entity. The export result preserves no source-specific 3D polyline variant identity. |
| `Insert` | Exact/PartialLoss/SkippedLoss/FatalIfInconsistent | Ordinary local references map to BlockInstance. OCS insertion coordinates map to owning-scope placement using the actual pinned CAD axes. Qualified trig intervals and exact residual propagation check each occurrence. Unsupported array/attribute/view/external variants skip as a whole; missing/cyclic targets are fatal. |
| `Viewport` | Exact/PartialLoss/SkippedLoss | Paper-owned orthographic viewports map frame, target/direction/height/twist, render mode, enabled/locked state, front/back clip and frozen layers. A supported same-paper circle, full ellipse or closed straight/bulged planar polyline may be an active clip, regardless of order. Missing/invalid/unsupported or conflicting active boundaries and perspective skip the whole viewport; deferred snap/grid/UCS and visual state are diagnosed as partial loss. Appearance overrides are not exposed by the pinned CAD Viewport model. |
| `Block`, `BlockEnd` | NonSemantic/FatalIfInconsistent/SkippedLoss | Matching structural markers supply record scaffolding, not drawable entities. Contradictory exposed begin-marker name/owner/base is fatal; unmatched markers are unsupported entities. |
| `Text`, `MText`, `Helix`, `Dimension`, `Hatch`, `Solid`, `Face3D`, `Ray`, `XLine`, `AttributeDefinition`, `AttributeEntity`, `Leader`, `MultiLeader`, `MLine`, `Mesh`, `RasterImage`, `Solid3D`, `Region`, `Body`, `Surface`, `Table`, `Tolerance`, `PolyfaceMesh`, `Wipeout`, `Shape`, `Underlay`, `Seqend`, `Ole2Frame`, `PolygonMesh`, `Light`, `SectionSymbol`, `ViewBorder`, `Extended`, `Unknown` | SkippedLoss | Whole entity receives `UnsupportedEntityType`; no geometry is approximated. |
| Model, paper, or supported definition ownership | Exact/FatalIfInconsistent | Per-owner explicit entity-handle order is retained; known-owner contents must occur exactly once with consistent membership. |
| `Spline` | CapturedTyped (opt-in) / SkippedLoss (disabled) / FatalIfInconsistent | SupportedTyped captures every interpreted parameter/common variant before native representation/restoration assessment; Model/Paper/local definitions retain live opaque owner/order, valid unsupported owners retain detached source/order provenance with placement loss. No native NURBS profile or guessed bounds. |
| Unsupported block owner | SkippedLoss | `BlockOwnedEntity`. |
| Null or unknown owner | FatalIfInconsistent | All safely detectable owner problems are aggregated before return. |
| Source handle number | NonSemantic | Replaced by a sequential OCDraw ID; `ExportEntityMapping` records the operational correspondence. |

## `EntityCommon`

| Field | Status | Contract |
| --- | --- | --- |
| `handle` | NonSemantic | See mapping rule above. |
| `owner_handle` | Exact/SkippedLoss/FatalIfInconsistent | See ownership rules above. |
| `layer` | Exact/SkippedLoss | Case-insensitive lookup to an emitted source layer; no replacement layer is invented. |
| `color`, `transparency`, `linetype`, `line_weight` | Exact/SkippedLoss | Per-property ByLayer/ByBlock/Explicit modes are retained. A required unrepresentable explicit value skips the entity. |
| `color_name` | Exact/SkippedLoss | `catalog$name` becomes named/color-book identity; malformed or inherited combinations are diagnosed. |
| `invisible` | Exact | Inverted into OCDraw visibility. |
| `extended_data`, `graphic_data`, `reactors`, `xdictionary_handle`, color-book/visual-style handles, material fields, `shadow_flags`, plot-style fields | PartialLoss | Geometry can still emit; all present meanings are bundled into the same entity diagnostic. |
| `entity_mode` | NonSemantic when consistent | Redundant physical ownership encoding; authoritative ownership is `owner_handle`. |
| `has_ds_data` | NonSemantic bookkeeping | The corresponding modeler entity/geometry is independently skipped as unsupported. |

Global header LTSCALE and supported EntityCommon linetype scales map exactly to
native positive scales. Invalid scales are rejected. Resolved pattern handles
are reference evidence, not independent semantic loss; name/handle contradictions
are structural errors. LwPolyline PLINEGEN and classic planar/spatial bit 128
map to perSegment/continuous generation. Current CELTYPE/CELTSCALE/PLINEGEN
defaults and unmodeled annotation scaling retain existing header loss reporting.

## Maintenance rule

Viewport clipping coverage is assessed from the interpreted CadDocument. The
[remaining local repair](../../../patches/opencadcodec-viewports/README.md) retains
the independent viewport-off bit. This upstream base retains activation and
group 340 references and maps VIEWPORT DXF degree angles to radians. Literal file tests and full selected
DXF/DWG chains cover the expanded families, dormant references and mixed order.
The previous pin stripped activation and DXF references; historical verification
does not retroactively qualify that unmodified base. DWG readback without an overall canvas still
reclassifies the first authored viewport; the passing profile includes a canvas.

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
and exhaustive Rust matches where opencadcodec exposes closed enums (`EntityType`,
color/transparency/lineweight modes in the converter). Relationships and
non-entity EED retain separate inventory summaries only when a typed source
diagnostic does not already own their meaning. Open maps and private/raw
internals remain bounded by the inventory contract and this manual field
matrix. New opencadcodec fields or variants must update this file, tests, and the
scanner before the pinned revision changes. The bounded spline snapshot now owns
separate capture/native-capability/restoration classification; no IFCPR resource
or broader raw/private/shared provider is introduced.

## Coordinate-frame accuracy

Provisional OCDraw 0.1.0 stores XYZ lines, spatial XYZ polylines and planar XY polylines
with an optional whole
plane placement. The converter alone interprets CAD arbitrary axes, using the
actual axes returned by the pinned helper after robust scaled normalization.
It never assumes repeated normalization preserves all source normal bits.

`geometry_tolerance` is a hard Euclidean limit under both `Allow` and `Reject`.
The default is exactly one micrometre for known drawing units and zero for
unitless drawings. Explicit physical tolerances require a known unit. Unit
conversion and squared residual comparison use exact rational values; distance
reports give outward bounds in drawing units. Every emitted line endpoint,
polyline vertex and active bulged-segment midpoint is covered. Failure to establish accuracy returns a typed error
and no partial drawing. A numerical diagnostic within the limit is exempt from
Reject, but `transfer_assessment()` still records `LossDetected`.

Circle, Arc, Ellipse and EllipseArc use a conservative full-curve bound from
centre and axis-vector residuals, with qualified phase intervals and a stored
sweep-rounding allowance. The same bound is propagated through each nested block
occurrence. If it exceeds tolerance while no sample proves an exceedance, the
converter refines equal angular subintervals through at most 32 pieces. Any
remaining uncertainty is reported as `NumericalProofIncomplete`.

The proof concerns the interpreted CadDocument geometry, not original CAD file
bytes, cached bounds, private codec state or recovery of unsupported source data.


Standalone mapping details and target limitations are in [COVERAGE.md](COVERAGE.md). Source scanning is independent of format encoding; its results are included in standalone export outcomes.

## Typed spline preservation field and restoration boundary

The complete field audit and payload/predicate format are in
[SPLINE-SNAPSHOT-V1.md](SPLINE-SNAPSHOT-V1.md). CapturedTyped is preservation
capability evidence, not native Exact geometry. Every Spline/SplineFlags parameter
and public EntityCommon field is captured, including linetype_handle,
graphic_data, all retained style/material/plot-style handles/flags, entity_mode,
has_ds_data and ExtendedData.raw_dwg_eed, except raw_record (separate
storageSupplementOmitted). Source enum identity, presence, ordered arrays and
binary64 bits survive production native readback, including nonfinite values.

Native layer and the complete appearance are exposed only where exactly
representable; otherwise those properties remain in the snapshot, never receive
invented defaults, and do not filter capture. In particular Default lineweight
and invalid native scale leave appearance absent. Current native properties are
later authoritative. Unsupported attached objects, application/raw context and
unresolved reference mappings can prevent restoration independently of capture.
Actual omitted content elsewhere retains ordinary source diagnostics and Reject.

A complete typed snapshot may satisfy capture Reject despite absent native
geometry or deferred writing. Geometry assessment separately lists unassessed
opaque definition/occurrence sources. The DXF/AC1032 DWG fixture matrix qualifies
actual comparisons in every supported owner kind; it is not an input whitelist
or a whole-file lossless/complete private-state preservation claim.
For the opaque spline path, unresolved common pattern references remain in the snapshot and make native appearance/restore unavailable. A provable name/handle contradiction remains fatal. Ordinary native entity/table pattern validation is unchanged.

## Layout-output revision

Layout media can exist without complete plot settings. PlotSettings uses plotUnit
and page; old embedded media is rejected. CAD mm dimensions never acquire an inch
label. Active standard/custom selectors are authoritative; contradictory standard
preset/factor state is diagnosed. Saved Paper PSLTSCALE values use layout bit 1,
independent of limits checking and plot model-type flags. The current header tracks
the active layout; inactive Model state is not overwritten by an active Paper value.

Physical Paper accuracy derives from each fixed mapping, including nested roots;
unknown/Fit/pixel defaults to exact coordinates. Explicit physical requests require
known mappings even for empty layouts. No combined cross-domain maximum is exposed.
Exact scalar conversion, raster qualification and medium-only default ambiguity
follow [layout output](../../../docs/layout-output.md). Evidence: ocdraw_plot_settings,
ocdraw_plot_exchange, paper_plot_accuracy and layout_plot_codec tests.

## Upstream pin update — 2026-10-07

Both converters use opencadcodec `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
Clipping activation/group 340 and VIEWPORT angle units now come from merged
upstream PRs #88/#89; only the independent viewport-off repair remains selected.
References above to clipping/angle defects of the previous unmodified pin are
historical evidence, not limitations of this new base. The [current dependency
audit](../../../docs/geometry/opencadcodec-update-2026-10-07.md) records new public
fields and compatibility decisions. Native spline semantics remain absent;
new OCDraw snapshots use payload v2 and retain v1 read/restore support. Fit-only
spline DXF parameterization remains a target-codec limitation pending PR #99.
Unresolved source layer handles must not silently become a native layer 0.
Known resolved handles are relationship identity and are rebuilt from native
layer references. Canonical one-byte-per-INSERT count framing is derived;
unfamiliar/mismatched count storage retains loss evidence. Additional table,
associative/count and solid-history data remain at the existing unsupported
family boundaries. No benchmark evidence is extended by this update.
