# IFCCAD supplemental profile rules 0.2.0

This is a normative companion to the [experimental contract](experimental-contract-0.2.0.md),
the ordinary [IFCX declarations](ifccad-profile-0.2.0.ifcx), and the
[local-value supplement](supplemental-values-0.2.0.schema.json). All apply after
fragment composition. It is not an IFCX schema extension or an independent drawing format.
The existing `urn:example:ifccad:0.2.0` import remains the only drawing import needed
by the bundled offline reader. Full IFCCAD validation includes local values,
graph relationships and domain algorithms, not just ordinary IFCX attribute shapes.

Rule IDs below remain stable when their implementation or upstream representation
changes. A schema `$defs` entrypoint is only a local-value check; it does not
certify graph validity, numerical representability, or all conditions of its rule.
The typed reader and shared logical validators enforce the remaining conditions.
JSON Schema does not inject defaults or normalize input. Type/shape overlap retained
in text definitions protects existing decoder checks; it is bounded to these
local values, not a second whole-document schema.

## Validation phases and upstream migration

Native defaults are applied after composition, before typed domain validation;
see [drawing defaults](../../docs/drawing-defaults.md). Line endpoints MUST have
two or three numeric components; a missing third component means +0. Other XYZ
positions, spatial-polyline vertices, axes and scale components retain their
existing requirements. Native entity appearance MAY omit individual properties
or the whole object, defaulting each to ByLayer; null remains invalid. Opaque
appearance absence remains unknown and its envelope rules are unchanged.
Block rotation/scale default to +0/[1,1,1]; transform and placement remain required.
Layer opacity defaults to 1; planar/spatial polyline closure defaults to false.
Writers MUST preserve nondefaults and explicit -0 in newly compacted fields.

`C` is JSON parsing/composition, `V` is consumed composed attribute validation,
`G` is graph projection/reference validation, and `D` is typed domain validation.
Writer output must pass the production reader and strict domain validation.
Typed construction must also pass `D`; a wire schema is not a substitute.

The `gap` column refers to [the IFCX wish register](../../docs/experiments/ifcx-upstream-gaps.md).
Migration conditions used by each row are:

- **G1/G2/G3/G5:** pin an upstream identity/order/composition/import contract and
  prove the current semantics and exact identity/source preservation before adoption.
- **G4:** upstream must express and enforce the row's role, ownership, reference,
  conditional or node constraint with the same final-graph meaning.
- **G6:** upstream must express reusable local variants with equivalent field
  presence, exclusivity and whole-value composition semantics.
- **G7:** upstream must express the row's exact range, string, closure or presence
  condition and preserve explicit zero/false versus absence. Prototype omission of
  an already declared check is an implementation gap, not missing IFCX syntax.
- **P:** profile/domain semantics, including numerical predicates, need an explicitly
  equivalent upstream domain rule and qualified implementation; adopting a scalar
  constraint alone does not retire their algorithm.

G8 tracks discoverability of these mandatory obligations. Ordinary IFCX consumers
are not assumed to execute this supplement. Never delete a rule/supplement before
pinning the actual upstream revision and proving equivalence with its positive and
negative cases. No private `Union`, `oneOf` or capability keyword is added to IFCX.

## Attribute applicability

Every current CAD attribute appears here. Paths are canonical drawing-local paths,
not suffix-only references. Foreign namespaces remain outside this catalog.

| Attribute | Role and applicable rule groups | Local entrypoint |
|---|---|---|
| `ifccad::drawing` | drawing root; WIRE, ID, OWN, APPEARANCE, WORKSPACE | `drawingExtras` |
| `ifccad::layout` | Model/Paper; OWN, LAYOUT, BOUNDS | `layoutExtras` |
| `ifccad::layer` | drawing definition; OWN, APPEARANCE | `layerExtras` |
| `ifccad::entity` | native ordered drawable; OWN, APPEARANCE | `entityExtras` |
| `ifccad::linePattern` | drawing definition; ID, PATTERN | `patternExtras` |
| `ifccad::symbolSource` | independent drawing definition; ID, SYMBOL | `symbolSource` |
| `ifccad::blockDefinition` | drawing definition; OWN, GEOMETRY, BOUNDS | `blockDefinitionExtras` |
| `ifccad::blockInstance` | ordered drawable; OWN, GEOMETRY | `blockInstanceExtras` |
| `ifccad::geom::solid`, `ifccad::geom::trace` | placed ordered filled descriptors; FILLED | `geometryFilledExtras` |
| `ifccad::geom::placement` | required local frame on placed payloads; GEOMETRY | `placementExtras` |
| `ifccad::geom::point` | placed empty point payload; GEOMETRY | `geometryPointExtras` |
| `ifccad::geom::lineSegment` | direct XYZ; GEOMETRY | `geometryLineExtras` |
| `ifccad::geom::circle` | placed circle; GEOMETRY | `geometryCircleExtras` |
| `ifccad::geom::arc` | placed signed arc; GEOMETRY | `geometryArcExtras` |
| `ifccad::geom::ellipse` | placed full ellipse; GEOMETRY | `geometryEllipseExtras` |
| `ifccad::geom::ellipseArc` | placed partial ellipse; GEOMETRY | `geometryEllipseArcExtras` |
| `ifccad::geom::planarPolyline` | placed XY/outgoing bulges; GEOMETRY, PATTERN | `geometryPlanarExtras` |
| `ifccad::geom::spatialPolyline` | direct XYZ; GEOMETRY, PATTERN | `geometrySpatialExtras` |
| `ifccad::hatch` | placed Hatch drawable; HATCH, GEOMETRY, OWN, BOUNDS | `hatchExtras` |
| `ifccad::viewport` | Paper drawable; VIEWPORT | `viewportExtras` |
| `ifccad::textStyle` | drawing definition; ID, TEXT | `textStyle` |
| `ifccad::text` | placed drawable; TEXT | `text` |
| `ifccad::mText` | placed drawable; TEXT | `mText` |
| `ifccad::preservation` | drawing-owned collection; PRESERVATION, ID | `preservationExtras` |
| `ifccad::preservationRecord` | collection member; PRESERVATION | `preservationRecordExtras` |
| `ifccad::opaqueEntity` | ordered opaque drawable; PRESERVATION, BOUNDS | `opaqueEntityExtras` |
| `ifccad::drawingWorkspace` | drawing choices; WORKSPACE | `drawingWorkspaceExtras` |
| `ifccad::modelViewState` | Model choices; WORKSPACE | `modelViewStateExtras` |
| `ifccad::ucsDefinition` | drawing definition; WORKSPACE, ID | `ucsDefinitionExtras` |
| `ifccad::modelWindow` | ordered drawing window; WORKSPACE, ID | `modelWindowExtras` |
| `ifccad::paperCanvas` | Paper canvas; WORKSPACE | `paperCanvasExtras` |
| `ifccad::viewportWorkspace` | Paper viewport aids; WORKSPACE | `viewportWorkspaceExtras` |

## IFCCAD-WIRE

| ID | Requirement and algorithm | Phase / gap | Positive and negative evidence |
|---|---|---|---|
| IFCCAD-WIRE-001 | Header/envelope MUST use `ifcx_alpha`, the experimental profile `0.2.0`, nonempty header identity/dataVersion/author/timestamp, one drawing, and the bundled import or exactly matching inline declarations. Changed or incomplete historical inline definitions MUST be rejected; only the current module is accepted. | C/G/D; G5 | [native](../../tests/ifccad_native.rs), [preservation](../../tests/ifccad_preservation.rs) |
| IFCCAD-WIRE-002 | Duplicate keys within one JSON object MUST fail. Same-path fragments merge children/inherits/attributes by key in file order; repeated values replace whole values, not nested arrays/overrides. Null inheritance removes its key; child/attribute null remains a marker. RejectConflicts MUST reject differing repeated values, including foreign nodes. | C; G3 | [composer](../../src/ifccad/source/composition.rs), [text wire](../../tests/ifccad_text_wire.rs) |
| IFCCAD-WIRE-003 | CAD validation MUST run on the composed graph, retaining exact immutable original bytes and foreign context. Foreign children confer no CAD ownership. No supplement may validate all foreign objects or interpret provider bytes as CAD fields. | C/V/G; G3/G8 | [profile rules tests](../../tests/ifccad_profile_rules.rs), [native](../../tests/ifccad_native.rs) |
| IFCCAD-WIRE-004 | Closed native value objects MUST reject unknown members and forbidden nulls as specified in the contract and local variant definitions. Unsupported CAD namespaces/payloads MUST fail at their existing role boundaries. No legacy unknown-field or null acceptance exception is provided. | V/G; G7 | [geometry](../../tests/ifccad_geometry.rs), [text codec](../../tests/ifccad_text_codec.rs), [workspace](../../tests/ifccad_workspace_state.rs) |
| IFCCAD-WIRE-005 | Fresh encoded output MUST load through the production reader and retain typed CAD meaning; a semantic mismatch is an explicit integrity failure, retaining its cause. | G/D; P | [encoder tests](../../src/ifccad/encode.rs) |

## IFCCAD-ID

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-ID-001 | Every path segment ID MUST be canonical decimal uint64 (no signs/leading zeros); references MUST use complete canonical same-drawing paths. IDs MUST not pass through binary64. Ordinary domains permit zero; preservation record IDs are positive. | G; G1 | [ID tests](../../tests/ifccad_id_management.rs), [text wire](../../tests/ifccad_text_wire.rs) |
| IFCCAD-ID-002 | Each positive exact watermark MUST exceed every live ID in its independent domain. Entity IDs span all owners; layout IDs span Model/Paper. Text styles, patterns, blocks, layers, records, UCS and windows have separate domains. Allocation MUST use checked arithmetic without mutation on exhaustion; deletion/reservation MUST not lower history. Snapshot validation checks current maxima, not deleted history. | V/D; G4/G7 | [allocation](../../src/ifccad/logical/allocation.rs), [ID tests](../../tests/ifccad_id_management.rs) |
| IFCCAD-ID-003 | Five original counters are required. Text counter omission means 1 only without style records/references; collection requires record counter; without collection omission means untouched 1. Workspace identities/state require both explicit workspace counters; legacy input without workspace may use 1. Advanced empty-domain history MUST survive. | G/D; G4 | [text wire](../../tests/ifccad_text_wire.rs), [preservation](../../tests/ifccad_preservation.rs), [workspace](../../tests/ifccad_workspace_state.rs) |

## IFCCAD-OWN

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-OWN-001 | Drawing children MUST include one Model and every Paper/layer/pattern/block/style/UCS/window definition. Each ordered drawable MUST have exactly one Model/Paper/block owner. Numeric child keys MUST be canonical contiguous indices from 0 with distinct drawable targets. Named children do not confer order/ownership. | G; G2/G4 | [native](../../tests/ifccad_native.rs), [text wire](../../tests/ifccad_text_wire.rs) |
| IFCCAD-OWN-002 | A native drawable MUST carry entity plus exactly one supported primitive/block/viewport/Text/MText payload; role combinations MUST be compatible. An opaque drawable has its separate exclusive role. A standalone geometry attribute on foreign context does not become an owned CAD entity. | G; G4 | [geometry](../../tests/ifccad_geometry.rs), [preservation](../../tests/ifccad_preservation.rs) |
| IFCCAD-OWN-003 | Block-instance targets MUST resolve to same-drawing definitions; the complete definition-reference graph MUST be acyclic, including unused definitions. Reordering changes draw positions, never identities. | G/D; G4 | [geometry](../../tests/ifccad_geometry.rs), [bounds](../../tests/ifccad_bounds.rs) |

## IFCCAD-APPEARANCE and IFCCAD-PATTERN

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-APPEARANCE-001 | Colors MUST be either the exclusive closed `{kind:Foreground}` choice or closed RGB-byte objects with optional closed nonempty indexed/named metadata; concrete indexed ACI 7 (ASCII-case-insensitive system) MUST be rejected; indexed values MUST be exact uint64; opacity MUST be finite in [0,1]; lineWeight MUST be finite >=0 mm. Layer name MUST be nonempty. Entity layer/pattern references MUST resolve in the drawing. | V/G/D; G7/G4 | [domain checks](../../src/ifccad/logical/document_validation.rs), [profile tests](../../tests/ifccad_profile_rules.rs) |
| IFCCAD-APPEARANCE-002 | Each entity property mode object MUST contain only mode for ByLayer/ByBlock, and exactly mode/value for Explicit. Preserve modes, including top-level ByBlock with no universal fallback. Layer named `0` and ByBlock chains defer per occurrence to containing instances; this contract supplies no resolved-appearance evaluator. | V/D; G6/P | [line patterns](../../tests/ifccad_line_patterns.rs), [wire modes](../../src/ifccad/codec/json/validate.rs) |
| IFCCAD-PATTERN-001 | Drawing/entity scale MUST be finite >0; absence means 1. Polyline generation absence means perSegment, alternatively continuous. Signed lengths use owner coordinates and scale multiplicatively; no resolved renderer is implied. | V/D; G7/P | [pattern tests](../../tests/ifccad_line_patterns.rs) |
| IFCCAD-PATTERN-002 | Pattern names MUST be nonempty and unique under pinned Unicode 17 full case folding without trim/normalization; ByLayer/ByBlock names are forbidden. Continuous MUST have empty pattern. Nonempty pattern MUST contain >=2 finite values with first >=0 and finite positive sum of absolute lengths. Unused definitions MUST survive. | V/D; G7/P | [pattern algorithm](../../src/ifccad/logical/patterns.rs), [pattern tests](../../tests/ifccad_line_patterns.rs) |

## IFCCAD-GEOMETRY

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-GEOMETRY-001 | Native coordinates/scalars MUST be finite binary64 in fixed right-handed local XYZ. Placement MUST retain origin/axes and pass the shared orthonormal right-handed coordinate-frame predicate without normalization. Direct XYZ line/spatial polyline MUST forbid placement; placed payloads MUST require it. | V/D; G7/P | [geometry](../../tests/ifccad_geometry.rs), [geometry kernel](../../src/geometry_kernel) |
| IFCCAD-GEOMETRY-002 | Circle/arc radius and ellipse axes MUST be positive. Ellipse major MUST be >=minor; equal axes remain Ellipse. Arc starts retain finite unreduced radians; signed sweep MUST satisfy `0 < abs(sweep) < binary64 TAU`. Full circles/ellipses remain distinct kinds. | V/D; G7/P | [geometry](../../tests/ifccad_geometry.rs), [next cases](../../conformance/next/ifccad/cases.json) |
| IFCCAD-GEOMETRY-003 | Polyline MUST have >=2 vertices. Planar outgoing bulges MUST match vertex count; omission means zeros. Closed activates every segment, open retains dormant final bulge. Exact coincident endpoints MUST describe a point while retaining both vertices and their finite outgoing bulges. No epsilon, deduplication or inferred arc is permitted. Newly collapsed nonzero bulged chords during CAD conversion MUST fail. Such segments do not supply contour curvature. Spatial paths have neither bulges nor placement. | V/D; G4/P | [geometry](../../tests/ifccad_geometry.rs) |
| IFCCAD-FILLED-001 | Intrinsic corners and signed thickness MUST be finite. No simplicity, convexity, area or distinct-point test gates native validity. | D; P | [filled geometry](../../tests/filled_geometry.rs), [native storage](../../tests/ifccad_filled.rs) |
| IFCCAD-FILLED-002 | Filled placement MUST satisfy the existing frame predicate; all four base and four top positions MUST have a finite conservative enclosure using the exact stored-axis cross product. | D; P | [filled geometry](../../tests/filled_geometry.rs), [native storage](../../tests/ifccad_filled.rs) |
| IFCCAD-GEOMETRY-004 | Block definitions MUST have nonempty names, finite XYZ base points and supported insertionUnit tokens. Block transforms MUST have valid placement, finite rotation and finite nonzero signed XYZ scale. Evaluation MUST subtract definition base point, apply scale/rotation then placement. Model/definitions use drawing units; Paper root has its own numerical domain. Instance scale explicitly includes any required unit conversion; insertionUnit MUST NOT silently rescale content. | V/D; P | [geometry](../../tests/ifccad_geometry.rs), [bounds](../../tests/ifccad_bounds.rs) |

## IFCCAD-LAYOUT

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-LAYOUT-001 | Model MUST have tabIndex 0 and no name; Paper MUST have nonblank name unique with implicit Model under pinned full case fold without trim/normalization. Paper tabIndices MUST be contiguous 1..count and unique; reading/writing normalize by index, not IDs. Old Paper coordinate-unit fields MUST fail. | G/D; G4/P | [layout metadata](../../tests/ifccad_layout_metadata.rs) |
| IFCCAD-LAYOUT-002 | Optional media MUST have supported physical unit or px and finite positive width/height; unitless, incomplete, null or unknown media fields MUST fail. Media without plot is valid on both owners and MUST NOT imply coordinate scaling/bounds. Limits rectangle MUST be ordered; absent flags default limitsChecking=false and paperSpaceLinetypeScaling=true. | V/D; G6/G7/P | [layout output](../../tests/ifccad_layout_output.rs), [layout contract](../../docs/layout-output.md) |
| IFCCAD-LAYOUT-003 | Complete plotSettings MUST use its documented page/area/mapping/output/options variants. Plot requires media; positive printable area MUST lie within unrotated medium with compatible physical mm/in versus raster px output. Layout area requires Paper fixed/offset; Limits requires Model limits. Fit/absent/pixel mapping MUST NOT infer fixed physical coordinates. Conditional fields, exact mapping and numeric range are checked by the plot kernel and the [layout-output contract](../../docs/layout-output.md). | V/D; G6/G7/P | [plot kernel](../../src/plot_kernel), [layout output tests](../../tests/ifccad_layout_output.rs) |

## IFCCAD-VIEWPORT

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-VIEWPORT-001 | Viewport MUST belong to Paper and refer to its drawing's unique Model. Its finite Paper frame MUST have positive dimensions and finite outward enclosure of center Ãƒâ€š±half dimensions. View enabled/visible/locked are independent authored booleans; no media scaling/inferred values. | V/G/D; G4/G7/P | [viewport tests](../../tests/ifccad_viewports.rs) |
| IFCCAD-VIEWPORT-002 | Model view MUST have finite values, positive height, and nonzero direction whose exact norm is <=binary64 MAX; direction is preserved. Perspective requires positive finite lens mm; Orthographic permits absent/nonnegative dormant lens. AtDistance requires finite signed distance, back forbids AtCamera, active back MUST be strictly behind active front; AtCamera uses exact direction norm. Dormant distances survive. | V/D; G6/G7/P | [workspace predicate](../../src/workspace_kernel/validation.rs), [viewport tests](../../tests/ifccad_viewports.rs) |
| IFCCAD-VIEWPORT-003 | Enabled clip MUST have boundary; every stored boundary MUST resolve within same Paper owner and be exclusive to one viewport, including dormant references. Active boundary MUST be circle/full ellipse/closed planar polyline in exact Paper Z=0 inside outward frame enclosure. Straight contour needs >=3 distinct XY vertices; active bulged contour >=2. Tangency, concavity/reflection/self-intersections are permitted. Use shared analytic whole-curve containment; never sampling, projection or tolerance epsilon. | G/D; G4/P | [viewport algorithm](../../src/ifccad/logical/viewports.rs), [clip predicate](../../src/geometry_kernel), [viewport tests](../../tests/ifccad_viewports.rs) |
| IFCCAD-VIEWPORT-004 | Viewport override rows MUST reference unique complete same-drawing live layers and optional live patterns; color/opacity/weight obey ordinary concrete constraints; no-op rows MUST fail; frozen false MUST NOT thaw global freeze; input order is not semantic. Writer sorts numeric uint64 identities without binary64 conversion. | G; G4 | [viewport tests](../../tests/ifccad_viewports.rs) |

## IFCCAD-TEXT

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-TEXT-001 | textStyle MUST satisfy `$defs/textStyle`: closed font request/style fields, primitive types, absence versus null, and local variants. Name MUST be nonempty, NUL-free and case-fold-unique; `validate_text_style` supplies positive/range/domain conditions. Preserve unused styles and authored overrides. | V/D; G6/G7/P | [text codec](../../tests/ifccad_text_codec.rs), [text wire](../../tests/ifccad_text_wire.rs), [text semantics](../../docs/text.md) |
| IFCCAD-TEXT-002 | Text MUST satisfy `$defs/text`, including anchored/wholeTextMiddle/aligned/fit layouts and decorated literal runs. `validate_text_layout` and `validate_text_runs` enforce pure semantic conditions; obliqueAngle MUST be finite with abs < PI/2, thickness and rotation finite. Placement/style roles are separately required. Native IO MUST NOT interpret CAD control codes. | V/D; G6/G7/P | [text tests](../../tests/ifccad_text.rs), [text codec](../../tests/ifccad_text_codec.rs), [shared values](../../src/text) |
| IFCCAD-TEXT-003 | MText MUST satisfy `$defs/mText` and referenced character/paragraph/column/background/stack variants; explicit zero/false/empty-tab reset and Unicode/paragraph order MUST survive. Pure validators check nominal height, wrap/column compatibility, stack/tab/background limits and effective inherited formats using the requested style; dimensions use owner coordinates and relative factors use nominal height. No font resolution or truncation is implied. | V/D; G6/G7/P | [text codec](../../tests/ifccad_text_codec.rs), [text semantics](../../docs/text.md), [shared values](../../src/text) |
| IFCCAD-TEXT-004 | Style reference MUST resolve to a drawing-owned same-drawing textStyle role, with exact zero-valid uint64 identity and watermark history. Text MUST carry placement, MUST NOT coexist with another drawable payload and MUST NOT be an active Paper clip. Repeated text/style attributes replace entire values under composition, not nested formats/content. | G/D; G4/G3 | [text wire](../../tests/ifccad_text_wire.rs), [profile tests](../../tests/ifccad_profile_rules.rs) |

## IFCCAD-BOUNDS

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-BOUNDS-001 | Optional supplied bounds MUST be finite ordered XYZ boxes; boundsQuality requires a box and cannot be null or stored partial. Missing quality with box means producer enclosing. Empty scopes MUST omit bounds. Omission MUST NOT mean empty, valid enclosure or permission to skip arithmetic checks. | V/D; G7/P | [bounds](../../tests/ifccad_bounds.rs), [text bounds](../../tests/ifccad_text_bounds.rs) |
| IFCCAD-BOUNDS-002 | Evaluate all primitives, every nested/unused definition and occurrence using conservative finite arithmetic. Opaque/unavailable contributions make owner/ancestor partial and require absent complete bounds. Empty-definition origin fallback is permitted only for actually empty definitions, never opaque-only or nonempty empty-glyph content. Numeric overflow remains fatal even without a box or with unavailable geometry. | D; P | [bounds algorithm](../../src/ifccad/logical/bounds.rs), [opaque bounds](../../tests/ifccad_opaque_bounds.rs), [text bounds](../../tests/ifccad_text_bounds.rs) |
| IFCCAD-BOUNDS-003 | Derived all-known primitives are enclosing; complete estimates including nonempty text are estimated; missing contribution is partial. Supplied enclosing MUST contain proven primitive contributions, not claim proven font contours. Estimated and producer-enclosing boxes MUST contain the proven primitive contributions checked by `validate_supplied`; neither is validated as a font-contour enclosure. Estimates used by explicit preparation and fresh enclosure proof remain distinct. Recompute MUST be explicit and transactional, changing boxes/qualities only after all checks succeed. Frame stroke uses proven physical mapping; unknown units/ByBlock width cannot fabricate enclosure. | D; P | [bounds algorithm](../../src/ifccad/logical/bounds.rs), [text bounds](../../tests/ifccad_text_bounds.rs) |

## IFCCAD-PRESERVATION

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-PRESERVATION-001 | Envelope MUST be version 1 with required sources; source IDs/providers/revisions MUST be nonempty, IDs unique, optional version absent or nonempty. Record sourceId/sourceKey pair MUST be unique and source resolve. Payload schema/predicate/slot/sourceKey MUST be nonempty; payload/condition versions positive. Core wrappers are closed; bytes MUST use canonical padded standard Base64. | V/D; G7/P | [preservation tests](../../tests/ifccad_preservation.rs), [wire](../../src/ifccad/codec/json/preservation.rs) |
| IFCCAD-PRESERVATION-002 | Preservation targets MUST have canonical same-drawing path for stated role. Opaque entity MUST exclusively carry opaqueEntity among CAD roles and resolve complete entity record with exact matching live subject. Subjected records MUST match their opaque drawable. Detached archive MUST clear subject and have no draw-order/export obligation. Optional native appearance/layer obey ordinary constraints when present. | G/D; G4 | [preservation predicate](../../src/ifccad/logical/preservation_validation.rs), [preservation tests](../../tests/ifccad_preservation.rs) |
| IFCCAD-PRESERVATION-003 | Binding slots MUST be unique within record. Bindings/conditions are soft targets: missing target and record cycles remain storage-valid. Unknown schema/predicate/opaque bytes MUST remain transportable; qualified coverage MUST NOT grant automatic restoration. Adapters separately qualify capture/restoration eligibility and loss. | D; P | [preservation tests](../../tests/ifccad_preservation.rs), [preservation contract](../../docs/preservation.md) |

## IFCCAD-WORKSPACE

| ID | Requirement and algorithm | Phase / gap | Evidence |
|---|---|---|---|
| IFCCAD-WORKSPACE-001 | Workspace value wrappers MUST be closed; World/Named/Unnamed UCS and Canvas/Viewport choices have exclusive allowed fields. UCS name MUST be nonempty and case-fold-unique; frame/elevation finite and frame valid. References MUST resolve same drawing/domain. Model-window rectangle components MUST be in [0,1] with min<max and positive aspectRatio. | V/G/D; G6/G7/P | [workspace codec](../../src/ifccad/codec/json/workspace.rs), [workspace tests](../../tests/ifccad_workspace_state.rs) |
| IFCCAD-WORKSPACE-002 | Grid spacing MUST be finite >=0 and majorLineFrequency positive uint32. Snap spacing MUST be >0 when enabled, >=0 when disabled; base/angle finite. Preserve dormant values. Canvas frame MUST have finite XYZ center and positive dimensions but no geometry/bounds effect. Paper canvas view MUST be Orthographic; Model windows use ordinary view/clip rules. | V/D; G6/G7/P | [workspace kernel](../../src/workspace_kernel/validation.rs), [workspace tests](../../tests/ifccad_workspace_state.rs) |
| IFCCAD-WORKSPACE-003 | Drawing choices remain unspecified when absent. Stored UCS and activation MUST remain distinct; omission of useStoredUcs means true. Current Paper UCS requires known active context; active viewport MUST have same-owner workspace. If active context uses stored UCS, explicit current UCS MUST equal that stored choice. Model active-window choices obey the same activation consistency. Model/windows/viewport aids use drawing coordinates; Paper canvas uses its Paper domain. No medium scaling. | G/D; G4/P | [workspace algorithm](../../src/ifccad/logical/workspace.rs), [workspace tests](../../tests/ifccad_workspace_state.rs) |
| IFCCAD-WORKSPACE-004 | All UCS/window definitions MUST be drawing-owned. Drawing modelWindows array is the sole ordered list of unique live window references; graph order/child names are incidental. Workspace state MUST NOT enter draw order or geometry bounds. | G; G2/G4 | [workspace codec](../../src/ifccad/codec/json/workspace.rs), [workspace tests](../../tests/ifccad_workspace_state.rs) |

## Strict native values and diagnostics

All consumed native CAD records and nested authored value objects MUST reject
unknown members and explicit null values. Optional authored values MUST be
omitted when absent; explicit false, zero and empty arrays retain their meanings.
Layer/entity/block/pattern wrappers and block transforms obey the same closure
rules as text. Workspace snapshots and authored viewports obey the same view-null
rules. There is no legacy acceptance exception for these cases, nor for older
inline module definitions. These strict rules are exercised in
[profile-rule tests](../../tests/ifccad_profile_rules.rs).

Core-generated wire, graph and typed validation messages use
`RULE-ID location: detail`. Source locations use drawing/node/attribute/field
paths when available; identity-free scalar or allocation APIs report their
component/operation location. Context adds the actual owner without replacing
the underlying rule, original detail, or additional errors. Typed error wrappers
continue retaining their original phase and source. User-constructed reports
are not rewritten merely by displaying or wrapping them.

Closing native values MUST NOT close arbitrary foreign IFCX content, general
IFCX header context, or interpret opaque preservation payload/baseline bytes as
JSON. Those existing source/transport boundaries remain intact.

Other supplements are deliberately partial: graph-dependent counter maxima,
canonical reference roles, finite binary64 arithmetic, geometric relationships,
Unicode folding and provider restoration eligibility remain the named algorithms
above. JSON Schema mathematical `integer` is not a promise to round integer
tokens through binary64 or to accept them in place of typed uint64 parsing.

## IFCCAD-HATCH

| Rule | Normative obligation | Phase; gap | Evidence |
|---|---|---|---|
| IFCCAD-HATCH-001 | Hatch MUST have one placement and a qualified Solid, LinePattern or Gradient fill. All nested loop/boundary/edge objects MUST be closed typed variants. Full circles/ellipses and partial signed arcs MUST obey shared primitive parameter rules. Loops MUST be nonempty; implicit closed polylines MUST have matching outgoing bulges and sufficient distinct vertices. | V/D; G6/G7/P | [Hatch](../../docs/hatch.md), [native wire](../../tests/ifccad_hatch_wire.rs) |
| IFCCAD-HATCH-002 | Each entry of ordered loop sources MUST reference a complete boundary-contributing native object in the same direct owner, by canonical complete drawing-local entity identity. Eligible kinds are Line, Circle, Arc, Ellipse/EllipseArc, open/closed PlanarPolyline/SpatialPolyline, Text, MText, BlockInstance and Paper Viewport. Missing, wrong-kind and wrong-owner targets MUST fail even when associationEnabled is false. Stored contours remain authoritative; IO MUST NOT compare source geometry or regenerate the Hatch. | G/D; G4/P | [source lifecycle](../../tests/ifccad_hatch_associations.rs) |
| IFCCAD-HATCH-003 | Authored joinTolerance MUST be finite/nonnegative in local units, default nearest binary64 decimal 1e-9. Exact zero requires proven equality. Excessive and numerically unprovable joins MUST fail separately. IO MUST NOT snap, bridge or increase the limit. Validation MUST NOT claim intersection/nesting/fill evaluation. | V/D; G7/P | [kernel](../../tests/hatch_geometry.rs), [creation units](../../tests/hatch_tolerance.rs) |
| IFCCAD-HATCH-005 | Gradient fill MUST have complete typed gradient settings and exactly two concrete colors. Inactive settings MAY lack type/colors; nonfinite values, shift/tint outside [0,1] and all-default empty state MUST fail. One/two-color mode and tint MUST NOT regenerate stored colors. | V/D; G6/G7/P | [native gradient wire](../../tests/hatch_gradient_wire.rs) |

Existing WIRE/ID/OWN/GEOMETRY/BOUNDS rules continue to apply. A local supplement
validates tags/fields, not topology, join proofs or source agreement. The reader
validates only final composed values; whole Hatch attribute replacement remains
the ordinary IFCX composition rule. No IFCX union dialect is introduced.

## Presentation additions

Drawing PointDisplay is a closed glyph/enclosure/tagged-size value with positive
finite absolute/percentage size or defaultFivePercent, independent of Point
anchor/bounds. Common native entity visibility defaults true; viewport-local
visibility and frozenLayers are rejected. Layer description/on-off/freeze/lock/
plottability/new-viewport freeze remain independent. Block description/anonymous/
explodable/uniformScaling persist; uniform instance scale is exactly equal on
all three signed axes. Viewport plotShadingOverride is a mode-only enum; layout
quality/DPI remains separate. See [presentation](../../docs/presentation.md) and
[native regression tests](../../tests/ifccad_viewport_presentation.rs). Local
supplements enforce closure/types; graph validation owns refs and uniform policy.

`IFCCAD-PRESENTATION-001` identifies invalid logical PointDisplay size at its
drawing field. Local drawing supplements also enforce the closed form/size shape.
`IFCCAD-GEOMETRY-004` additionally rejects nonuniform signed scales on an
instance whose definition requires uniformScaling. Existing registered attribute
rule families continue to own closed metadata and viewport-row checks.

## Embedded line families (IFCCAD-HATCH-001)

LinePattern MUST contain nonempty families and finite origin/rotation/positive
scale, defaulting to (0,0)/0/1 when absent. Literal optional name/description
may be empty but are not resource/library references. Every family MUST retain
angle, basePoint, signed [along,perpendicular] offset and ordered dash/gap/dot
sequence. Perpendicular MUST be nonzero; no density minimum is imposed.
Dash/gap length MUST be positive/finite; Dot MUST forbid length. Empty sequence
means continuous, all-gap is valid, all-dot/zero period invalid. Exact sum of
nonempty sequence lengths MUST be positive and within finite binary64 range.
No normalization or origin movement during contour edits. Unknown/null/inactive
fields MUST fail. Ordinary inline IFCX declarations expose a tagged superset;
the local supplement and typed domain checks enforce the actual variants.
Whole Hatch attribute replacement and all contour/source/ID rules still apply.
Evidence: tests/hatch_pattern.rs and tests/hatch_pattern_wire.rs.

## Ordinary Dimension rules

| Rule | Requirement | Evidence |
| --- | --- | --- |
| IFCCAD-DIMENSION-001 | All nine native definitions, text placement and effective settings MUST pass shared value validation. Measurement MUST use stored definition; refs MUST resolve to the required kind and same owner; render/block cycles MUST fail. Directed angles/spans and signed ordinate MUST retain their explicit meanings; presentation points MUST NOT change measurement. | `tests/ifccad_dimension.rs`, `tests/ifccad_dimension_bounds.rs`, `tests/dimension_remaining_kinds.rs`, `tests/ifccad_dimension_remaining_relations.rs` |
| IFCCAD-DIMENSION-002 | Dimension and style values MUST obey their closed supplemental/IFCX declarations. Style/TextStyle/marker/pattern/source refs MUST be canonical complete same-drawing paths. Authored override presence and atomic LaterWins replacement MUST survive strict readback. | `tests/ifccad_dimension_wire.rs` |
| IFCCAD-ID-002 | `nextDimensionStyleId` MUST be positive and exceed all live zero-valid uint64 style IDs; deletion MUST NOT lower it and allocation exhaustion MUST be atomic. | `tests/ifccad_dimension.rs` |

Ordinary Dimension storage/readback does not establish CAD exchange, font layout
or runtime associative updates. See the independent conversion coverage as it is
qualified. Annotative Text/MText and Dimension remain a joint future slice.
## Complex line patterns and symbol sources

`ifccad::linePattern.pattern` is an ordered array of plain signed numbers or
closed `{length,marker}` objects. Positive lengths retain strokes, negative
lengths retain gaps; zero with a marker contributes no dot. Simple nonempty
periods require two elements; complex periods permit one. Both require a
nonnegative first length and a finite positive absolute-length sum.

Text markers carry `kind:text`, literal `text` and a complete `textStyle` path.
Symbol markers carry `kind:symbol`, `symbolSource` path and uint32 `shapeNumber`.
Both support finite positive scale, finite two-component offset, and relative,
absolute or upright rotation in radians. Upright follows the tangent and permits
a half-turn to keep the marker readable; it is mutually exclusive with absolute. Omission defaults are 1, [0,0], and relative zero;
null, unknown fields and fields of another variant are rejected. References are
canonical complete paths within the same drawing and must resolve to their
respective roles after composition. Later fragments replace whole attribute
values, including the array; RejectConflicts rejects conflicting fragments.

`ifccad::symbolSource` is a closed `{kind:shx,fileName}` payload on
`/cad/dN/symbolSource/N`, referenced by its drawing children. Source IDs are
independent exact uint64 values, including zero. Duplicate filenames retain
separate identity. The required `nextSymbolSourceId` is positive, greater than
all existing source IDs, and is never lowered by deletion. Allocation overflow
is atomic. Filenames are nonempty basenames except . and .., without slash,
backslash or NUL. Internal reference errors are fatal; external asset absence
is nonfatal and the core performs no asset lookup.

The IFCX declaration supplies the Array wrapper. The profile-owned supplemental
schema supplies its closed number/marked-element alternatives, and semantic
validation supplies period, transform and graph rules. This is an independent
experimental contract, not an adopted buildingSMART symbol resource standard.

Gradient endpoints require concrete fixed RGB colors; Foreground is not a
gradient endpoint. Concrete ACI 7 identity remains invalid under the common
color contract. No background-dependent endpoint is resolved during IO.

## Dimension fraction presentation

Primary and enabled alternate number settings store `fractionLayout` (`inline`,
`horizontal`, `diagonal`) and finite positive `fractionScale`. Omission defaults
to `inline` and `1.0`, including old files. These settings affect only fractional
and architectural number presentation; inactive values remain stored and
validated. Precision, rounding, units and raw measurement are unchanged.
Primary sparse overrides retain explicit `inline` and `1.0` resets. Alternate
overrides retain the existing complete enabled-profile replacement rule.
Literal label text is never parsed as a fraction. Formatted fragments are derived
inspection results, not an authoritative stored label or measurement cache.

## Ordered filled primitives: SOLID and TRACE

`ifccad::geom::solid` and `ifccad::geom::trace` are distinct drawable payloads,
sharing `geometryFilledExtras`. Each requires sibling `ifccad::geom::placement`
and exactly four numeric local XY `corners`; `thickness` is optional with default
**+0.0**. Null/unknown members and additional geometry payloads are rejected.
Composition replaces each complete attribute with LaterWins, without merging
corner members. Writers omit +0 thickness, retain explicit -0 and preserve all
four slots. CAD boundary order is **1,2,4,3**; no sorting or deduplication occurs.

Every finite corner combination is allowed, including crossings, overlap,
retracing, repeated corners and line/point collapse. No minimum area, simplicity,
convexity, distinct-point count or surface cover is a validity condition.
`IFCCAD-FILLED-001` rejects nonfinite intrinsic values. `IFCCAD-FILLED-002`
rejects invalid placement or unavailable finite conservative enclosure.
For stored axes U,V, N=U×V is evaluated exactly without normalization;
B_i=O+U*x_i+V*y_i, T_i=B_i+N*thickness. The parameter enclosure covers all four
base and all four top points, even when they coincide. It is not a universal
rendered footprint or a host filling rule. Appearance/layer fields remain
stored; line pattern/lineweight do not expand this parameter enclosure.

Neither kind becomes a Hatch source, active viewport clip or Dimension-selector
target. Native records contain no topology/proof/render flags. Conversion
requires its ordinary hard position evidence independently of optional simple
surface correspondence; absent surface evidence does not invalidate, omit or
lose a finite descriptor. Viewer rendering remains unassessed.

## Owned Dimension graphics

IFCCAD-DIMENSION-GRAPHICS-001: optional Dimension graphics is an independent ordered value with required SourceSnapshot/Generated origin. ReferencePoint purpose requires Point and does not display. Payloads are the existing primitive, Text/MText, BlockInstance, Solid/Trace bodies and Solid, unassociated, source-free Hatch; Viewport, Dimension and opaque content are excluded, including reachable marker contents. Validation does not check freshness against definitions/styles.

IFCCAD-DIMENSION-GRAPHICS-002: part layers/patterns/styles and local markers must resolve within the drawing, with no graphics render cycles. Parts have no ordinary entity ID or allocation counter. Omitted layer inherits the owner; explicit layer zero is a reference. Every part, including hidden/helper content, is validated.

IFCCAD-DIMENSION-GRAPHICS-003: a Dimension's optional children.graphics must target its canonical /cad/dD/eE/graphics group. The group requires dimensionGraphics and children, with contiguous numeric roles I targeting /cad/dD/eE/graphics/pI. Each part requires dimensionGraphicsPart and exactly one permitted payload. Groups have no placement/appearance/payload, parts have no CAD children or ordinary entity roles. Direct XYZ lines/spatial paths and BlockInstance forbid separate placement; all other permitted kinds require their existing placement. Known graphics nodes require exactly one proper owner; orphan, shared, cross-Dimension, noncanonical, null and unknown core values fail. Existing IFCX key composition runs before these checks; absence of a child key in a later fragment does not delete earlier links.

See [Dimension mapping](../../docs/dimension/graphics-design.md) for exact fields, defaults and frame conventions. These rules add neither implicit child inheritance nor a generator/update protocol.

## Hatch editing state

IFCCAD-HATCH-006: optional classicSettings and boundaryConstruction must each be
nonempty. Classic optional preferredFill/creationType/name/description/origin/
definitionLines distinguish authored presence, including empty strings, zero XY and
empty lists. Rotation is finite/unwrapped (default 0), scale finite positive
(1), double false by default. Each direct local XY definition line requires finite
angle/basePoint/offsetVector and optional signed dashLengths (default []). Its XY
vector MUST be nonzero; a nonempty dash list MUST have positive finite absolute
period. Signed zeros, ordering and duplicate lines MUST be retained. Definition
values MUST NOT be interpreted as normalized active along/perpendicular families;
active transverse spacing and CAD compatibility require separate qualification. Neither completeness nor agreement with active fill is
required. Construction seedPoints are ordered finite XY (default []), including
duplicates/outside points; pixelSize is finite nonnegative (0). A fully default
record is invalid. These values never regenerate geometry or set provenance.
Closed IFCX members reject null/unknown keys; LaterWins replaces the whole Hatch.
Shared typed validation applies equally to IFCX, CBOR and Zstd readback.

## Planar polyline widths

IFCCAD-POLYLINE-WIDTH-001: optional planarPolyline.widths is a closed Constant,
PerVertex or Defaulted value. Omission means Constant positive zero. Every stored
full width is finite and nonnegative; signed zero is valid. PerVertex has exactly
N complete [start,end] pairs. Defaulted requires both defaults and exactly N
closed override objects with independent optional start/end leaves. Null is not
absence. Absent leaves inherit only the matching header default; explicit zero
and explicit equal-to-default values remain explicit. Validate unused defaults
and dormant final records. The final outgoing record is inactive while open.

IFCCAD-POLYLINE-WIDTH-002: ordinary and Dimension graphics use the same neutral
value, validation and derived view in packed and separate geometry backings.
Native IFCX/CBOR retain modes, defaults, presence and all binary64 bits. Only
Constant exact positive zero may be omitted; no equality/zero normalization of
PerVertex or Defaulted is allowed. Widths do not alter Hatch/viewport centerline
contours or add extrusion-thickness semantics. Whole attribute replacement
applies; override arrays are never merged by index.
