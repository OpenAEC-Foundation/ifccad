# Dimension support

IFCCAD is the active track. OCDraw descriptions below retain historical context;
its implementation and fixtures are parked. See [parking](ocdraw-parking.md).
Only IFCCAD code, schemas and current converter coverage define active behavior.

OCDraw and IFCCAD independently store ordinary linear and aligned dimensions,
local named DimensionStyles, authored overrides and optional same-owner source
relations. Their models, IDs, schemas, readers and converters remain separate.
The shared neutral values and formatter do not depend on a CAD runtime or font
engine. Active schemas and `conformance/next` define the format contract.

## Definition, measurement and text

A dimension owns its definition points and presentation state. Both reference
points are required local XYZ coordinates. The dimension-line point is XY in
the local dimension plane, placed in 3D by the common placement. Reference Z
uses the exact cross product of the stored X/Y axes; stored axes are not
renormalized. Linear measurement is the absolute projection of the local XY
reference difference onto its direction; aligned measurement is the XY distance
between the projected references. Independent reference Z values survive but
do not change either planar measurement. Zero measurements are valid. Numeric overflow or unavailable
evaluation is an error, never a replacement zero. ExtensionDirectionAngle is an
absolute local direction when explicitly present; absence requests ordinary
automatic extension placement.

The authoritative native data contains no cached measurement. Editors and viewers
evaluate the stored definition, then format that result. Formatting is pure and
does not look up linked geometry, resolve fonts or draw glyphs. Numeric factors
affect text only. Paper-only factors apply according to the owning scope; they
never change coordinates or the raw measurement.

Labels can be generated, suppressed or a template of literal TextRuns and
measurement tokens. Literal runs remain literal, including CAD-looking `<>` or
field syntax. CAD conversion rejects ambiguous or unsupported encodings rather
than reinterpreting native literals. Formatting failure can be reported separately
from a valid raw measurement. The inspector shows both authored label state and
the formatted runs.

Primary and enabled alternate numbers retain `fractionLayout` (inline,
horizontal, diagonal) and positive finite `fractionScale`, defaulting to inline
and 1.0 when absent. Layout and the stacked digit-height ratio are independent of
numeric precision, rounding and unit factors. Inactive choices survive storage.
Primary overrides preserve explicit inline/1.0 resets; alternate overrides keep
their existing complete-profile replacement behavior. The derived formatted
label exposes typed text/fraction fragments alongside the existing plain text
projection. Literal TextRuns never become fractions by parsing their contents.
The exact formatter reduces fractions and carries rounded inches into feet.

Text placement is style-driven or explicit with local position, rotation and
attachment. Rotation is an offset from the dimension's default text orientation,
including its effective orientation policy. It is not an absolute world text
angle or a font-layout certificate. Explicit zero, automatic placement and a
generated cache are distinct choices.

## Styles and authored presence

DimensionStyles refer to local TextStyles; fonts remain symbolic requests.
Effective settings resolve base style, kind-specific override and entity override
in that order. Each component appearance has independent sparse color, opacity,
line-pattern and weight leaves. InheritEntity, ByLayer, ByBlock and Explicit
remain separate modes.

Presence is meaningful: false, zero, empty and equal-to-parent values are retained.
Omitted leaves inherit; an explicit leaf resets that dependency. Null is rejected.
Unused styles and future-kind style entries survive native storage. References
must resolve in the same drawing. Style/marker render cycles are invalid even
in unused definitions. Construction updates allocation counters atomically.

## Associations

Relations identify a dimension role, source entity and typed point selector:
`position`, `start`, `end`, `center`, or `vertex` with an index. These selector
names are semantic API names; coincident entity field names are intentional.
Only eligible source/selector pairs in the same owner are accepted. Each role
may be associated once. Forward references are valid after final binding.

Loading, saving, formatting and conversion never update the definition from the
source. A stored dimension can therefore measure 4000 while its linked source
now measures 4500. An editor or a separate associative-update engine decides when
and how to refresh definition points. Detaching an unsupported CAD relation
retains the stored definition and produces a located diagnostic.

## Bounds, inspection and CAD exchange

Without a renderer, dimension display bounds are unavailable. Existing scopes
can retain partial/estimated evidence; definition anchors do not establish an
enclosing box for text, markers or extension lines. Conversion assessments
separate submitted definition/explicit text anchors from unassessed display,
including nested occurrences and the existing owner-specific Paper tolerance.

CAD import places the frame at the WCS dimension-line point and keeps both
full spatial reference anchors. Explicit OCS text is mapped to that plane;
out-of-plane explicit text remains a hard numerical failure. The existing
Euclidean tolerance and nested-occurrence checks include all XYZ components.
The active experimental schema requires three reference components; older XY
definitions are rejected, not silently completed. The mutable experimental CBOR version-1 mapping uses the same XYZ contract.

Both converters use `cad-dimension` for ID-free preparation and bind their own
identities afterward. A CAD measurement cache is ignored; disagreement with the
stored definition is diagnosed. Ordinary WCS definition points and OCS explicit
text are qualified with literal fixtures and actual DXF/AC1032 DWG tests.
DSTYLE uses bounded typed source entries, then resolves coupled fields against
the complete source style before native binding. Qualified fixed extension length,
component ACI colors, separate markers and fraction layout/scale are supported.
Inactive DIMTVP under DIMTAD=1 and inactive subunit settings cannot independently
reject a style. Active subunit conversion remains unsupported. DSTYLE preserves
qualified sparse overrides. Missing references, malformed
structures and numerical excess cannot become default style/layer/geometry.

CAD has several narrower representation boundaries. Coupled extension fields
must agree. Type-style dependencies can be materialized as complete `$0`/`$1`
CAD substyles with explicit factoring loss; reimport retains them as named
styles. Line spacing and arrow flips become per-entity state with dependency
evidence. A CAD fixed TextStyle height is materialized into nominal Dimension
height with dependency-loss evidence; presentationScale remains separate. CAD
export accepts a matching fixed height and diagnoses a conflicting height without
changing the shared TextStyle. Complex marker,
background, tolerance, annotation and dynamic-field profiles can be unsupported.
Simple single-object CAD associations qualify line start/end, circle center and
point position; other native selectors remain supported natively.

Output has no generated anonymous CAD picture. Visibility requires host
regeneration, and dictionary/reactor activation of automatic updates is
unassessed. The pinned reader loses clone group 12; DXF also omits certain flips
and true colors and suppresses some near-default values. These limitations are
explicit, not display fidelity claims. See the complete
[CAD field audit](dimension/cad-field-audit-2026-10-09.md) and both converters'
[OCDraw](../legacy/ocdraw/crates/ocdraw-convert/README.md) /
[IFCCAD](../crates/ifccad-convert/README.md) coverage contracts.

## Design and subsequent slices

The approved design includes nine kinds: linear, aligned, two-line angular,
three-point angular, radius, diameter, ordinate, arc length and jogged radius.
Only linear/aligned entity definitions are implemented in this slice. The other
seven kinds remain separate implementation slices, with their own measurement,
validation, bounds and CAD qualification.

Curve parameters, occurrence paths, intersections, trans-space references and
external sources require a later selector/update contract. Annotative Text/MText
and Dimension belong in one later shared annotation slice. Dynamic fields,
attributes and font resolution remain separate work. Generating or refreshing
Dimension graphics and executing associative updates belong to editors or external
engines. This development does not publish or freeze version 0.1.0.

The maintained writers `write_dimension` and `write_ifccad_dimension` demonstrate
ordinary styles, kind/local overrides, Model/Paper/block owners and source
references. They refuse to overwrite an existing output file. Their corresponding
inputs under `examples/ocdraw` and `examples/ifccad` can be opened independently.

Dimension follows the shared [drawing defaults](drawing-defaults.md) for common
entity appearance, visibility and line-pattern scale. OCDraw omits a default
column only when every row has that default; mixed columns remain complete.
Component colors support the symbolic `Foreground` choice, just like other
presentation colors. Authored style settings and sparse override leaves retain
their presence, including explicit false, zero and empty reset values.

## Explorer and codec qualification

The Explorer labels entities, styles, fields and enums in Dutch and English.
Stored definition, authored label/overrides and source relationships remain
separate from calculated measurement, formatted label and effective settings.
Horizontal/diagonal fractions have an escaped, accessible text preview. That
preview is not a CAD glyph-layout or graphical bounds certificate. Full-width
identities and the original source view remain available.

DWG text placement uses the explicitly selected
[text-position polarity repair](../patches/opencadcodec-dimension/README.md)
alongside the viewport-off repair. DWG flags bit 0 is the opposite of DXF 70 bit
128; independent raw-bit tests prevent matching codec reader/writer errors from
masking one another. This does not qualify the unmodified dependency or introduce
rotated automatic placement. The normal explicit native placement remains the
representation for user-positioned source text.

### CAD decimal-separator default

CAD `DIMDSEP=0` (NULL) means the decimal point `.`. Import resolves this sentinel
before native validation, for unused DIMSTYLE records as well as effective
DSTYLE group 278 overrides. An explicit local zero remains an authored native
`.` override, including when the inherited style already uses `.`. Other source
characters retain their meaning; native control characters remain invalid.
Export may write the equivalent explicit character code 46 rather than the CAD
sentinel. No native NUL character or locale-dependent fallback is introduced.
See [Autodesk DIMDSEP](https://help.autodesk.com/cloudhelp/2020/ENG/AutoCAD-Core/files/GUID-0CDD6AA6-CDD8-4C9D-A270-A3DA00DCE63E.htm).

## Planned worked-out graphics

The planned graphics slice targets IFCCAD only, after the stable migration/Hatch
boundary and the [native SOLID/TRACE precursor](geometry/solid-trace-design.md)
in [ROADMAP.md](../ROADMAP.md), before the remaining Dimension kinds.
OCDraw remains parked. Ordinary linear/aligned Dimensions gain an optional typed
ordered vector representation alongside the authoritative definition. The design
covers complete source picture capture, local resources and qualified CAD
picture-block reuse, without a built-in generator.

IFCX uses a Dimension-owned graphics child group whose ordered part nodes carry
registered geometry/text attributes. Parts stay outside ordinary Model/Paper/block
ownership and draw order. The same logical value is encoded directly in CBOR as
an optional record with ordered parts, without IFCX node paths. Experimental CBOR
version 1 is updated in place; optional Zstd transport remains separate. There is
no legacy parity or compatibility work.

The [concrete design mapping](dimension/graphics-design.md) specifies the proposed
`graphics` child role, group/part paths, role attributes, CBOR codes and CAD part
profiles. Explicit Point helpers stay non-displaying. Qualified SOLID and TRACE
use their native primitive payloads and retain their separate CAD types, including
when used by marker blocks. The proposed Hatch substitution has been superseded.
All finite local corner combinations, including crossing and collapsed records,
remain native graphics content. Required coordinate/envelope evidence is separate
from optional surface coverage and unassessed viewer display; an unknown fill is
not by itself a partial-cache or unsupported-content condition.
The CAD cache adapter distinguishes picture coordinates from an ordinary INSERT
transform.

The red sample's `_ArchTick` marker also requires polyline width 0.15. The current
zero-width native polyline profile cannot preserve it; the SOLID/TRACE precursor
does not add that support. Polyline widths are an agreed separate follow-up after
bounded graphics, with the complete red sample Dimension as their acceptance
case. They do not block the current graphics plan. Here the sample verifies
explicit unsupported-width reporting and absence of a silently truncated picture;
positive capture/reuse cases use complete supported graphics. See the
[follow-up scope](../ROADMAP.md#planned-polyline-width-follow-up).

Existing keywise IFCX composition still applies: omitting a child key does not
delete it. The current XYZ reference anchors and XY measure
plane remain unchanged; the Dimension placement applies to local graphics once.
This slice can write complete fresh documents. Editing existing source graphs,
fragment preservation, replacement/deletion protocols and transactions remain later work.

The core validates structure, geometry and hard references. Editors or external
engines assess correspondence with the definition and generate, update or remove
graphics explicitly. No required input-basis snapshots or automatic validity
analysis are planned. Unsupported visible parts cannot yield silently incomplete
graphics; existing loss and explicit partial-import policies remain distinct.
Native graphics and opaque preservation remain separate. Annotative per-context
graphics stay with the later joint annotation slice. This is design under review,
not current storage or CAD display fidelity support.

The SOLID/TRACE precursor is implemented and verified in its isolated candidate: native IFCX/CBOR storage, all finite corner descriptors, qualified CAD type retention and separate position/surface/display evidence. Dimension graphics and polyline widths remain subsequent slices; this does not add picture capture or an update/generation engine.
