# Dimension support

OCDraw and IFCCAD independently store ordinary linear and aligned dimensions,
local named DimensionStyles, authored overrides and optional same-owner source
relations. Their models, IDs, schemas, readers and converters remain separate.
The shared neutral values and formatter do not depend on a CAD runtime or font
engine. Active schemas and `conformance/next` define the format contract.

## Definition, measurement and text

A dimension owns its definition points and presentation state. Linear measurement
is the absolute projection onto its direction; aligned measurement is the distance
between its two points. Zero measurements are valid. Numeric overflow or unavailable
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

Both converters use `cad-dimension` for ID-free preparation and bind their own
identities afterward. A CAD measurement cache is ignored; disagreement with the
stored definition is diagnosed. Ordinary WCS definition points and OCS explicit
text are qualified with literal fixtures and actual DXF/AC1032 DWG tests.
DSTYLE preserves qualified sparse overrides. Missing references, malformed
structures and numerical excess cannot become default style/layer/geometry.

CAD has several narrower representation boundaries. Coupled extension fields
must agree. Type-style dependencies can be materialized as complete `$0`/`$1`
CAD substyles with explicit factoring loss; reimport retains them as named
styles. Line spacing and arrow flips become per-entity state with dependency
evidence. Fixed-height font dependencies are unqualified. Complex marker,
background, tolerance, annotation and dynamic-field profiles can be unsupported.
Simple single-object CAD associations qualify line start/end, circle center and
point position; other native selectors remain supported natively.

Output has no generated anonymous CAD picture. Visibility requires host
regeneration, and dictionary/reactor activation of automatic updates is
unassessed. The pinned reader loses clone group 12; DXF also omits certain flips
and true colors and suppresses some near-default values. These limitations are
explicit, not display fidelity claims. See the complete
[CAD field audit](dimension/cad-field-audit-2026-10-09.md) and both converters'
[OCDraw](../crates/ocdraw-convert/README.md) /
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
attributes, font resolution, dimension rendering and host update execution are
also separate work. This development does not publish or freeze version 0.1.0.

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
