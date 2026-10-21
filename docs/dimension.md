# Dimension support

IFCCAD stores ordinary linear and aligned dimensions, local named
DimensionStyles, authored overrides, optional same-owner source relations and
optional worked-out graphics. The neutral values and formatter do not depend
on a CAD runtime or font engine. Active schemas and `conformance/next` define
the format contract. OCDraw remains [parked](ocdraw-parking.md).

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

Without stored graphics, Dimension display bounds are unavailable: definition
anchors do not establish an enclosing box for text, markers or extension lines.
With graphics, the existing primitive and text enclosure/estimate logic evaluates
the stored Display parts, including hidden parts, in the Dimension placement.
ReferencePoint helpers are validated but contribute no display enclosure.
Present empty and helper-only graphics are known empty; text retains its
Estimated/Partial limits. Conversion assessments
separate submitted definition/explicit text anchors from unassessed display,
including nested occurrences and the existing owner-specific Paper tolerance.

CAD import places the frame at the WCS dimension-line point and keeps both
full spatial reference anchors. Explicit OCS text is mapped to that plane;
out-of-plane explicit text remains a hard numerical failure. The existing
Euclidean tolerance and nested-occurrence checks include all XYZ components.
The active experimental schema requires three reference components; older XY
definitions are rejected, not silently completed. The mutable experimental CBOR version-1 mapping uses the same XYZ contract.

The converter uses `cad-dimension` for ID-free preparation and binds native
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

When graphics is present and eligible, export writes a private anonymous CAD
picture block and links it to the Dimension; it adds no scope INSERT. The
complete picture is staged atomically with its local marker resources. Without
stored graphics, visibility still requires host regeneration. There is no
built-in generator or freshness analysis. Automatic association updates remain
external. See [graphics mapping](dimension/graphics-design.md) for the exact
child roles, physical mapping, frame qualification and loss boundaries.

## Optional stored graphics

`graphics: None`, present empty graphics and nonempty graphics are distinct.
Required origin is `SourceSnapshot` or `Generated`; it describes provenance,
not validity relative to the independent definition/style. Parts are owned
values without ordinary entity IDs. IFCX uses `/cad/dD/eE/graphics` with ordered
`pI` children; CBOR revision 1 stores the same logical value directly.

The closed family includes lines, points, circular/elliptic curves, zero-width
polylines, local block instances, Text/MText, restricted solid Hatch, SOLID and
TRACE. Nested Dimensions, Viewports, opaque values and associated/patterned Hatch
are excluded from the picture and its marker closure. SOLID/TRACE retain all
finite ordered corners and signed thickness. Definition measurements, numerical
position evidence, optional filled-surface evidence and display equivalence
remain separate; storing graphics never certifies pixels.

CAD capture uses the actual block link and captures the entire eligible picture.
Unsupported content, including any reachable polyline width, loses the whole
picture under Allow and is rejected under Reject. Malformed/numerically invalid
parts remain errors, including hidden parts; partial import may omit the affected
Dimension as a whole. The definition is independently retained on a supported
picture-only loss. Shared ordinary INSERT uses keep their ordinary block too.
Local helpers retain an explicit Defpoints role. CAD cannot retain Generated
provenance: Allow transfers the picture and diagnoses `/graphics/origin`; Reject
refuses that metadata loss. Existing unrelated style/continuation metadata losses
still apply, so a complete CAD picture alone does not make import pass Reject.

The original red sample_AC1032 marker has width 0.15. Its complete picture is
therefore deferred to the polyline-width slice; no width is zeroed or substituted.
The current supported foundation sample retains all 87 stored pictures through
IFCCAD and actual DXF/DWG readback. This is transfer evidence, not a universal
rendering guarantee.

## Design and subsequent slices

The approved design includes nine kinds: linear, aligned, two-line angular,
three-point angular, radius, diameter, ordinate, arc length and jogged radius.
Only linear/aligned entity definitions are implemented in this slice. The other
seven kinds remain separate implementation slices, with their own measurement,
validation, bounds and CAD qualification.

Curve parameters, occurrence paths, intersections, trans-space references and
external sources require a later selector/update contract. Annotative Text/MText
and Dimension belong in one later shared annotation slice. Dynamic fields,
attributes and font resolution remain separate work. Generating Dimension
graphics and executing associative updates belong to editors or external engines. This development does not publish or freeze version 0.1.0.

The maintained writer `write_ifccad_dimension` demonstrates
ordinary styles, kind/local overrides, Model/Paper/block owners and source
references. It refuses to overwrite an existing output file. Its input under
`examples/ifccad` can be opened independently.

Dimension follows the shared [drawing defaults](drawing-defaults.md) for common
entity appearance, visibility and line-pattern scale.
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

## Graphics qualification boundary

The [complete graphics contract](dimension/graphics-design.md) records canonical
child ownership, physical codes and the bounded CAD cache profile. IFCX keywise
composition still applies: omitting a child key does not delete an existing key.
Fresh complete-document IO is supported; editing source fragments, transactions
and replacement/deletion protocols remain later work. Editors decide whether a
stored snapshot still corresponds to the definition. Annotative per-context
graphics remains part of the later shared annotation slice.
