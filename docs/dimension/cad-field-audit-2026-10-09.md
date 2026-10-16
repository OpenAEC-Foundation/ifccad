# Ordinary dimension CAD field audit

Scope: linear/aligned native slice, opencadcodec base
`ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`. Tests use the repository's explicitly
maintained viewport-off patch, read-only. No dimension dependency patch is applied.
Code paths reviewed: `entities/dimension.rs`, `tables/dimstyle.rs`, DXF dimension
reader/writer, DWG dimension reader/writer and both associative object routes.
This audit qualifies the neutral helper; converter integration is tracked
separately in each converter's coverage contract.

R = represented, D = derived/recomputed, L = located loss, U = unsupported,
I = invalid input. Unknown active semantics never silently become defaults.

## DimensionBase and variants

| Public fields | Classification and rule |
| --- | --- |
| common | Adapter-owned layer/appearance/owner/handle plus existing common-field audit; consumed DSTYLE is removed only from the residual diagnostic view. |
| definition_point | D redundant base copy; variant point authoritative, discrepancy gets L. |
| text_middle_point | R OCS explicit text anchor; L automatic display cache. |
| insertion_point, insertion_scale, insertion_rotation | U nondefault cloned-picture transform. |
| dimension_type | R linear/aligned, I conflicting base tag, U all other kinds. |
| attachment_point | R explicit anchor, L automatic cache attachment; not conflated with DIMTAD. |
| text, user_text | R effective generated/suppressed/plain template; U markup/dynamic fields; alias storage factoring is not a native field. |
| normal | R plane; L normalization, I invalid/nonfinite. |
| text_rotation | R explicit CAD rotation offset; U nonzero automatic text rotation. No absolute glyph orientation proof. |
| horizontal_direction | U nonzero source UCS direction. Target basis reparameterization has a portability L. |
| style_name | R bound named style; I unresolved name; no Standard fallback. |
| actual_measurement | D cache ignored; fresh neutral measurement. |
| version, dwg_unknown_bit, dwg_flags_byte | L CAD bookkeeping/unqualified bits; positioned bit represented independently. |
| block_name | L picture cache link; ordinary block definitions remain in source inventory. |
| line_spacing_factor, line_spacing_style | R per-entity values, I invalid factor, U unknown mode; inherited native dependency becomes L on output. |
| flip_arrow1, flip_arrow2 | R per-entity flags; DXF transport limitation L for active flips. |
| text_user_positioned | R explicit/style-driven distinction. |
| Linear.first_point, second_point, definition_point | R WCS anchors projected to native plane, exact rational proof pairs. |
| Linear.rotation | R axis angle in CAD OCS. |
| Linear.ext_line_rotation | R nonzero CAD offset plus rotation becomes absolute native extension direction; zero denotes automatic, no separate presence in CAD model. U near-zero explicit native offset erased by DXF writer. |
| Aligned.ext_line_rotation | U nonzero, aligned oblique rule unqualified. |
| Aligned.first_point, second_point, definition_point | R WCS anchors and proof pairs. |
| Other dimension variant fields | U kind before preparation; no partial geometry masquerading as ordinary dimension. |

DXF groups 10/13/14 are WCS. Group 11 is OCS despite the pinned model comment.
Group 12 is OCS clone insertion; the current reader collects it but does not
assign it, so cloned DXF cannot be qualified through this model boundary.
Group 16 is OCS angular arc location and belongs to a later native kind.
Literal fixtures and actual DXF/AC1032 DWG tests pin these observations independently.
The group-52 offset rule follows [Autodesk linear dimensions](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-F0004556-493C-48D5-8619-61D6ADF05C04.htm).

## DimStyle: every public field

| Public fields | Classification and rule |
| --- | --- |
| handle, name | R adapter identity/name, no shared native ID. |
| xref_reference, xref_dependent, xref_handle | U external dependency; xref_resolved L unused bookkeeping. |
| dimclrd, dimclre, dimclrt, dimclrd_true_color, dimclre_true_color, dimclrt_true_color | R component colors; source 0 inherits entity, 256 layer; U unknown indices. DWG-only true colors get target DXF L. |
| dimdle, dimgap, dimlwd, dimsd1, dimsd2 | R line/text gap/weight/suppression; U drawing-default weight, I invalid range. |
| dimdli | L continuation creation spacing. |
| dimexe, dimexo, dimlwe, dimse1, dimse2, dimfxlon | R extension settings. |
| dimfxl | R when active, L inactive value. |
| dimasz, dimcen, dimtsz | R marker size/center/tick settings. |
| dimblk, dimblk1, dimblk2, dimblk_name, dimblk1_name, dimblk2_name, dimsah | R selected marker references; L inactive shared/separate choices; U unresolved legacy names. |
| dimldrblk | L leader-only style state. |
| dimtxt, dimtih, dimtoh, dimtix, dimtmove, dimtad | R qualified height/orientations/fit/movement/vertical anchor; U DIMTAD outside 0/1 or unknown movement. |
| dimjust, dimtvp, dimtxtdirection | U nondefault justification, active offset (DIMTAD=0) or reversed direction; L inactive DIMTVP under DIMTAD=1. |
| dimsoxd | U active outside-arrow suppression. |
| dimtfill, dimtfillclr, dimtfillclr_true_color | R none/canvas/opaque explicit color; L inactive color, U unknown mode/inherited color. True color gets target DXF L. |
| dimmzf, dimmzs | U active decimal subunit conversion with leading-zero suppression; L inactive settings, including ordinary DWG factor 100 defaults. |
| dimaltmzf, dimaltmzs | U active alternate decimal subunits; L inactive alternate state. |
| dimtxsty_handle, dimtxsty | R resolved textstyle; I unresolved reference. Adapter must reject ambiguous name/handle relationships. |
| dimscale, dimlfac | R positive presentation scale and numeric factor; negative DIMLFAC is Paper-only, never coordinate scale. U DIMSCALE=0 automatic viewport/Paper context. |
| dimlunit, dimdec, dimrnd, dimdsep, dimzin | R plain numeric format, precision, rounding, separator (DIMDSEP zero/NULL means decimal point) and zero flags; U unknown modes/bits, I invalid precision/separator. |
| dimaunit | R angle-format style metadata; U survey format. |
| dimadec, dimazin | L angular-only precision/zero state, no angular entities in this profile. |
| dimalt, dimaltf, dimaltd, dimaltu, dimaltrnd, dimapost, dimaltz | R active alternate number group; L inactive data. |
| dimalttd, dimalttz | U active alternate tolerance combination; L inactive state. |
| dimtol, dimlim, dimtp, dimtm, dimtdec | R basic deviation/limits values; I contradictory modes; L inactive values. |
| dimtfac, dimtolj, dimtzin | R positive fraction scale; U nondefault active tolerance layout/suppression; L inactive tolerance state. |
| dimatfit, dimtofl | R qualified fit/line policy; U unknown fit mode. |
| dimupt, dimfit, dimunit | L cursor/obsolete editing fields. |
| dimpost | R single measurement token plus plain prefix/suffix; U ambiguous token/markup/dynamic field. |
| dimfrac | R native inline/horizontal/diagonal fraction choice, retained even when inactive. |
| dimarcsym | R style metadata before/above/none, U unknown choice. |
| dimjogang | L radius-only state. |
| dimltex_handle, dimltex1_handle, dimltex2_handle | R component pattern refs; adapter must bind actual definitions. |
| annotative | U annotation contexts, deferred joint text/dimension slice. |

Native output rejects unequal settings for fields CAD couples across both
extension lines, independently inherited ByBlock component state, unsupported
built-in markers, independent component opacity, independent text stroke styling,
unqualified background opacity/padding and alternate tolerance combinations.
Marker blocks/textstyles/patterns are bound by each independent adapter.
Style arrow flips and line spacing need per-entity materialization with L.
Type-style factoring is adapter-owned: sparse native type settings can materialize
as complete CAD `$0`/`$1` substyles with located inheritance/presence loss; reimport
keeps complete named styles. The helper does not invent a style hierarchy.
Null explicit textstyle/marker/pattern handles are invalid. Zero-sized native
Mark/Lines choices are unsupported rather than converted to CAD None.
Plain labels/post strings exclude legacy `%%` controls, brace groups and caret
escapes as well as backslash markup/dynamic fields. Their interpretation requires
a later qualification, not literal copying.
The automatic scaling restriction follows [Autodesk DIMSCALE](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-LT/files/GUID-AEA309F0-B831-4432-8085-FB6CD49CDC78.htm).

## Overrides and associations

ACAD DSTYLE grammar: one `DSTYLE { code value ... }` subsection across records.
Code is Integer16, each value has its specified XDATA type. Duplicate subsection,
duplicate code, bad type, delimiter or nonfinite value is I. An unfamiliar dimvar
in otherwise supported structure is U. Other records/ACAD values remain residual.
Qualified codes: 3,40,41,42,44,45,46,49,73,74,75,76,77,78,140,141,142,144,146,147,
172,173,174,176,177,178,271,276,277,278,279,281,282,289,290,340,342,343,344.
Source entries are typed before switches/values are resolved against the style.
Output reparses and compares authored-leaf coverage;
otherwise it rejects differing factoring instead of silently filling absent leaves.

DIMASSOC public association fields: `dimension` R handle, `associativity` R first
two slots, `references` R one record per active slot; `trans_space` U true,
`rotated_type` U nonzero. Third/fourth slots U. Multiple objects claiming a
dimension I. Dangling source I; unsupported source/owner/selector is detached with L.

Reference fields: `class_name` must be AcDbOsnapPointRef; `xrefs` exactly one
source. `osnap_type` R line start (13), unambiguous endpoint (1), circle center
(3), point node (4). `osnap_point` is finite WCS: used only to identify an
unambiguous endpoint, then L cache. `main_subent_type`, `main_gs_marker`,
`xref_paths`, `osnap_distance`, `intersection_objects`, `intersection_subent_type`,
`intersection_gs_marker`, `intersection_xref_paths`, `has_last_point_reference`
must be empty/default; otherwise U. CAD object envelope/owner/reactors/dictionary
and editing state get L when consumed. Native-to-CAD relation construction keeps
the dimension definition unchanged; CAD-host automatic update is unassessed.
See [Autodesk DIMASSOC](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C0B96256-A911-4B4D-85E6-EB4AF2C91E27.htm).

## Codec and display limits

Actual tests qualify tilted ordinary anchors, explicit positioning, plain label
templates, fresh measurement and a simple DIMASSOC start relation in DXF and
AC1032 DWG. Literal clone group 12 exposes a reader defect; no fix is applied to
the dependency. Pinned DXF omits arrow flips and DIMSTYLE true colors, suppresses
nearly-default line-spacing factors, near-Z normals and tiny UCS-direction values
with epsilon comparisons. These are transport limitations, not numerical proof
of unchanged display. Arbitrary native parameterization and extreme-angle codec
roundtrip still require adapter evidence.

No anonymous picture generator was found. Native output therefore carries an
explicit picture portability loss: a CAD host must regenerate display. Existing
source picture definitions are independently audited; they are never discarded
merely because their names look anonymous. Fonts, glyphs, line/marker display,
host regeneration and enclosing display bounds remain unassessed.

## Repair qualification added 2026-10-16

A fixed positive CAD TextStyle height overrides DIMTXT and is materialized in
native nominal text height, with editing-dependency loss. DIMSCALE remains
separate. A target fixed height must match the effective native height; it never
silently replaces it. Local text-style changes can materialize a height override.

Primary DIMFRAC 0/1/2 maps to horizontal/diagonal/inline; DIMTFAC supplies the
fraction scale. Alternate DIMALTU has a separate mapping: 4/5 stacked
architectural/fractional, 6/7 inline. Native alternative diagonal stacks and
independent primary/alternate fraction scales have an explicit narrower CAD
output boundary. Active tolerance scaling remains unqualified. DSTYLE color
leaves use ACI; true-color overrides require an unsupported transport profile.
Coupled native leaves must retain qualified factoring on output.

DWG text-position polarity requires the explicitly selected
[codec repair](../../patches/opencadcodec-dimension/README.md). The raw-bit test
checks both positions independently of matching reader/writer behavior. Native
models, CAD definition anchors and strict-validation rules are unchanged.
