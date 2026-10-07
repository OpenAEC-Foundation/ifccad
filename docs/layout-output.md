# Layout media and effective plot settings

OCDraw and IFCCAD keep independent documents, schemas and codecs. Both use
ID-free values and predicates from `ocdraw::plot_kernel` for layout output.
The provisional 0.1.0 revision introduces a layout medium separate from complete
plot settings; it rejects the intermediate spellings described below.

## Medium and output configuration

Each Model or Paper layout may have `media: { unit, width, height }`. Dimensions
are positive finite values. Physical units use the shared registry without
`unitless`; `px` describes raster dimensions. A medium can exist without a printer,
scale, margins or complete plot settings. No setting is inferred from its size.

Complete optional `plotSettings` contains `plotUnit`, `page`, `area`, `mapping`,
`output` and `options`. `page` contains `printableArea`, `rotation` and optional
`deviceName`/`mediaName` hints. Dimensions are stored only on the layout medium.
A complete plot value requires that medium. Unknown fields, partial records and
explicit null optional values are rejected.

Printable-area coordinates use the unrotated medium's unit and lie within it
with positive area. Physical media pair with mm/in plot units; raster media pair
with px. Shaded-plot DPI is a quality choice, not a physical raster calibration.
Medium geometry imposes no containment, clipping or rescaling of drawing entities.

Areas are Layout (Paper only), Extents, Limits (Model with authored limits only)
and Window. Fixed scale stores positive `outputLength` and `scopeLength`; FitToArea
does not establish a constant physical mapping. Layout area requires fixed scale
and offset placement. Offset numbers use plot units and keep their Media or
PrintableArea reference. Centering and negative finite offsets retain their
existing meanings. Display/NamedView, shared page setups and rendering are deferred.

Drawing plot-style mode is colorDependent by default or explicitly named.
Shading/quality, plot-style application/name and the existing viewport-border,
Paper-last, hidden-Paper, line-weight, scaled-weight and transparency options are
effective values. An external CTB/STB name does not preserve its contents.

## Paper coordinates and linetype policy

Paper has independent numerical coordinates. It neither inherits the drawing
unit nor declares a separate physical coordinate unit. A fixed plot ratio relates
Paper-coordinate length to output length: `2 mm = 1 coordinate` prints 100
coordinates as 200 mm. Mapping edits never rewrite geometry, transforms or bounds.
Model and block-definition coordinates continue using the drawing unit; viewport
camera parameters remain Model-valued and lens lengths remain millimetres.

`paperSpaceLinetypeScaling` is a layout boolean, default true, independent of media
and plot settings. True sizes viewport dash patterns in Paper coordinates across
different view magnifications; false retains source-space pattern sizing. Global
and entity pattern scales remain independent. MSLTSCALE and a renderer are deferred.

## CAD exchange and numerical boundaries

CAD adapters interpret typed layout fields independently. DXF physical dimensions,
margins and offsets are millimetres regardless of the plot-unit selector. Imports
retain the medium in mm and preserve inch output separately; they do not relabel
unchanged millimetre numbers as inches. Both codecs qualify saved layout flag bit
1 independently of limits-checking bit 2 and the plot model-type flag. OCDraw's
existing active-layout selection also determines the current header PSLTSCALE;
IFCCAD does not gain workspace selection in this slice.

Active standard/custom scale selectors are respected. Contradictory active
standard preset/factor values omit the complete plot with located loss, retaining
valid media. Fixed output writes a canonical custom ratio. A completely default
effective plot state is canonicalized to absence: CAD cannot distinguish an
authored value identical to its constructor defaults from medium-only intent.
Export diagnoses an authored plot identical to CAD defaults, since its presence
cannot survive canonical native reimport. No print-equivalence claim follows.

Physical scalar unit conversions require an exact finite binary64 result under
both policies. Thus 5 inches converts to 127 mm, while an exact 1-inch-to-mm result
cannot be stored in binary64. Unsupported exact registry factors omit medium and
dependent plot state under Allow; Reject refuses the semantic loss. Inexact
conversion is a typed PlotNumeric failure. Geometry tolerance does not excuse it. Source printable-area corners that cannot
be represented exactly omit the complete plot with located loss while retaining
media; output scalar differences/conversions also refuse silent rounding.
Raster data are native, but the pinned CAD surface supplies no qualified raster
calibration; converters diagnose loss rather than assigning physical pixel size.
Printable-relative offsets, transparency and external style contents retain
explicit CAD restrictions.

Both routes report drawing and individual Paper domains, without a cross-domain
maximum. Default physical Paper accuracy is one micrometre on the output, divided
by the fixed metres-per-coordinate factor. Unknown/Fit/pixel mappings default to
zero coordinate residual. Explicit coordinate limits apply directly; explicit
physical requests require a fixed physical mapping, including empty layouts,
with typed PaperTolerance errors carrying the layout ID.
Output mapping loss cannot acquire a physical certificate: source/target limits
are combined conservatively and missing target mapping remains unknown. Nested
signed/nonuniform occurrences are checked in their root domain as well as locally.
Semantic Allow/Reject never waives hard numerical exceedance or incomplete proof.
Preserved splines remain unassessed.

## API and provisional migration

OCDraw reexports neutral `LayoutOutputSettings` as `LayoutSettings` and `PlotRect`
as `LayoutRect`. IFCCAD exposes `IfccadLayoutSettings`, `IfccadLayoutMedia`,
`IfccadMediaUnit`, `IfccadPlotSettings` and `IfccadPlotStyleMode`. Its layouts now
have `settings`; Paper `length_unit`/`paper` and `IfccadPaperSize` are removed.
OCDraw `PlotMedia`/`plotSettings.media` are replaced by layout media plus PlotPage.
Both cores still validate their own identities, membership and references.

IFCCAD wire changes remove layout-level `lengthUnit` and `paper`; drawing
`lengthUnit` stays. Existing maintained examples explicitly migrate authored
output intent to mappings without rescaling coordinates or block transforms.
Old spellings have no compatibility fallback. Released conformance stays immutable;
active candidate cases exercise media-only, inch, raster, linetype and invalid states.

Assessment consumers use per-domain `coordinate_meaning()` and resolved limits.
Paper evidence stores a mapping rather than a coordinate-unit label. Browser
`coordinateMeaning` reports exact physical factors as rational decimal strings;
uint64 identities remain decimal strings. Native opening remains independent of CAD.

See the separate [OCDraw](../crates/ocdraw-convert/docs/TO-CAD-COVERAGE.md) and
[IFCCAD](../crates/ifccad-convert/docs/TO-CAD-COVERAGE.md) coverage contracts and
[codec qualification fixture](../crates/cad-geometry-convert/tests/fixtures/layout-plot-PROVENANCE.md).
