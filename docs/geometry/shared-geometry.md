# Shared geometry and conversion accuracy

The active IFCCAD core uses `ifccad::geometry_kernel` for primitive values,
coordinate frames, length-unit tokens, intrinsic validation, conservative bounds
and analytic Paper clip containment. `GeometryRef` is borrowed and carries no
identity, ownership, appearance, schema or physical packing. The core has no CAD
runtime dependency. `ifccad-convert` adapts native records and owns references.

`cad-geometry-convert` owns CAD-plane preparation, exact/rational and interval
calculations, unit resolution and paired primitive/nested occurrence evidence.
IFCCAD conversion directions share these kernels and retain separate directional
coverage, errors and model ownership. Typed spline preservation remains outside
native primitive proof: preserved sources and occurrences are unassessed and keep
`is_complete()` false. Primitive residuals do not certify opaque geometry.

The converter keeps `from_cad`, `to_cad`, `mapping` and `source` responsibilities.
Model and definition-local assessment use the drawing unit; Paper uses its fixed
plot mapping. See [layout output](../layout-output.md).
OCDraw and its former reexports are parked; the frozen checkpoint retains the
old kernels. Active code must not depend on that reference archive.

## Accuracy policy

Both directions expose `geometry_tolerance`. Default is exactly 1e-9 in each
numerical domain's own coordinates, independent of Model units and Paper plot
scale. Exact remains zero. `drawing_units(x)` applies directly in each numerical domain.
Explicit metres/mm requests require a known drawing unit or fixed physical Paper
mapping, including empty retained layouts, unless an explicit independent
`with_coordinate_fallback(x)` supplies a coordinate limit for unknown domains.
Known physical domains retain the physical request. Medium dimensions and shaded DPI never
supply missing Paper meaning. Exact rational scale factors resolve physical limits
without rounding the accepted bound. An unrepresentable target medium/plot mapping
cannot certify physical output; source/target limits are combined conservatively.

Domain reports include identity, coordinate meaning, limit, counts, status and worst
source/occurrence. A Paper meaning contains its output factor or Unknown, not a
physical coordinate-unit declaration. There is no cross-domain maximum. Nested
occurrences are checked in every retained root domain, including signed/nonuniform
scaling; definition-local acceptance alone is insufficient.

The hard limit is independent of semantic Allow/Reject. Proven within-limit
rounding has `RoundedWithinTolerance` evidence and is accepted with either
policy. Proven exceedance or incomplete proof returns a typed Geometry error
without output. Source-aware raw-number projection checks and CAD setter scale
clamping remain hard failures; tolerance does not excuse omitted semantics,
inexact IDs, widths, fitted curves or unsupported metadata.

Generic and dedicated spatial CAD polyline backings recognize vertex flag 0
or 32 as ordinary straight-path infrastructure. Fitted/mesh vertex flags remain
unsupported. Classic 2D widths retain separate header defaults and optional vertex leaves
through the [native width contract](polyline-widths.md); unsupported vertex
metadata is never silently folded into a bulged path. Direct, DXF and DWG source tests exercise these
backings independently of native-to-CAD emission.

Full conic interiors are included in the proof. Bulged segments use enclosing
parameter-matched complete segment circles. This deliberately conservative
enclosure may reject a tight limit even where the directed arc would fit; the
converter never relaxes the request or silently samples its way to acceptance.
CAD construction also refuses partial spans rounded to zero/full turns,
underflowed ellipse ratios and newly coincident target endpoints of a nonzero
source bulged chord. Already coincident source vertices remain point-shaped
segments with both vertices and original bulges retained. No epsilon or
deduplication is used; these segments do not supply curvature for contour
eligibility. See [coincident polyline segments](coincident-polyline.md).
A generous tolerance does not authorize an invalid target curve.
Evidence applies to the CadDocument conversion boundary. Independent file-codec
readback remains necessary and does not establish application rendering fidelity.

## Native bounds and clips

IFCCAD optional Model/Paper/definition bounds are supplied drawing data. Native
loading and encoding validate present boxes and preserve them; they do not
repair them. `recompute_ifccad_document_bounds` explicitly computes all boxes
and assigns them atomically after successful preparation. Missing boxes do not
force eager occurrence-bound evaluation during native loading. Empty scopes
have no box; viewport Paper frames contribute bounds independently of visibility,
while camera targets and view extents do not enlarge Paper geometry.

Active Paper clips use circles, full ellipses, or closed planar straight/bulged
paths, exactly in Paper Z=0. Shared analytic containment checks the entire curve
against the finite outward viewport frame, allowing tangency without a tolerance
or sampling substitute. Dormant references retain owner/exclusivity requirements
but need not use an active-eligible family. No new winding, area or
self-intersection restriction is introduced.

## CAD block boundary

Local definitions and ordinary INSERTs retain separate ownership, base-point
subtraction and occurrence transforms. Present BLOCK name/owner/base data must
agree with its block record; contradictory structure is an error under both
loss policies. A missing marker is distinct from a present zero base point.
The converter does not repair contradictory markers by renaming anonymous
definitions. Native-to-CAD construction creates consistent records and markers.

CAD insertion parameters must be interpreted in the codec's OCS convention.
Copying an oblique `insert_point` directly into an owning-scope origin loses
that meaning. Rotation/scale and block-base subtraction are separate operations.
Target setter changes are checked even for empty definitions; a small signed
scale clamped by the codec is not an exact native value. Non-neutral native
frames can require an explicitly diagnosed parameterization change. The
occurrence proof carries correlated source-minus-target intervals through each
nested transform, with instance path and leaf evidence. Local rounding can
therefore fail after a large outer scale; it is never assumed acceptable from
an extracted angle or a definition-local result alone.

The current qualification is recorded by the
[dependency audit](opencadcodec-update-2026-10-16.md),
[block tests](../../crates/ifccad-convert/tests/blocks.rs) and the converter's
[directional coverage](../../crates/ifccad-convert/README.md#supported-slice).
These checks concern retained codec semantics, not every downstream CAD host
or source information already lost before `CadDocument` construction.

### Historical codec qualification (2026-10-02)

The earlier `block_codec_boundary` probe used unmodified opencadcodec
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`. Its `Door` definition had base
`(2,0,0)`, line `(2,0,0)..(3,0,0)`, all insertion-unit codes 0..24 and an
anonymous flag independent of name. INSERTs used position `(3,4,5)`, normal
`(1,2,3)`, angle `0.7`, and scales `(2,3,-4)` or `(-2,-2,-2)`.

That checkpoint resolved nonzero DWG BLOCK bases (#52), anonymous BLOCK_CONTROL
ordinals including unresolved slots (#55), and DXF description group 4 (#49).
The synthetic AutoCAD 2025 anonymous-name fixture qualified marker agreement and
INSERT naming, with attributed INSERTs still diagnosed; its original
[provenance](../../legacy/ocdraw/crates/ocdraw-convert/tests/fixtures/README.md)
remains with the frozen checkpoint. At that pin an extra Paper DWG marker could
still carry the primary Paper owner (#78), and scale setters replaced magnitudes
below `1e-12` with positive `1e-12`, including decoded values. Those are dated
dependency observations, not a list of current unresolved codec defects.

Both writers serialized the insertion point directly; DXF wrote degrees and
DWG radians, while the public transform helper evaluated OCS * translation *
rotation * scale. Independent sampled assertions used
`u=(-2,1,0)/sqrt(5)`, `v=(-3,-6,5)/sqrt(70)`, `w=(1,2,3)/sqrt(14)` and
reference sin/cos literals. Empty-definition tests exposed already-clamped
scales. Marker, explodable and uniform-scale checks, actual DXF/DWG exchange
and production native readback bounded this evidence. The old IFCCAD slice
diagnosed block descriptions; later metadata support must be assessed from the
current tests/coverage rather than this historical restriction. The later
correlated occurrence regression includes locally acceptable rounding rejected
under `1e9` outer scaling. Sampled codec checks and certified whole-domain
conversion proof remain separate claims.

## Provisional version and qualification

This revises experimental IFCCAD 0.1.0 under the existing schema URI. It does not
promise that old inline definitions or earlier readers accept the new families.
Active schemas and `conformance/next` define the updated contract; numbered
collections remain unchanged. See the [geometry sample](../../tests/fixtures/ifccad/hello-geometry.ifcx).

The expanded AC1032 family corpus, including signed arcs, partial ellipses,
dormant outgoing bulges and spatial paths, passes production DXF/DWG readback.
Viewport ellipse/bulged clip exchange uses the explicitly documented three
[development codec patches](../../patches/opencadcodec-viewports/README.md)
against `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`. Those repaired viewport
results do not qualify the unmodified pin. No controlled size experiment was rerun.
The unmodified pin was separately exercised: the expanded non-viewport family
exchange passes, while all four `viewport_codec` regressions fail on activation,
boundary or independent off-state retention. Local patch configuration and lock
state are restored before the integrated checks.

The inspector exposes IFCCAD conversion and export evidence, including readback
evidence and structured numerical failures. Identity fields in this evidence
are decimal strings to preserve uint64 values in JavaScript. A user tolerance
selector and live-viewer integration remain subsequent application work; native
drawings do not store conversion preferences.

## Workspace scalar boundary

The core workspace_kernel shares ID-free view/grid/snap/canvas values and intrinsic exact predicates. cad-workspace-convert adapts CAD scalar fields. Format-owned bindings retain identifiers and source/target losses. Workspace fields do not enter primitive geometric certification or alter per-layout Paper tolerance; canvas frames do not contribute bounds.

## Hatch contours

The ID-free `geometry_kernel::hatch` module supplies typed planar boundaries,
join proofs, conservative bounds and creation-limit resolution. Each format
retains its own identity, source integrity and encoding. Shared CAD preparation
registers complete contour curves and occurrence evidence; fill remains
unassessed. Source edits do not regenerate stored contours during IO. Explicit
detachment keeps geometry; invalid source deletion fails validation.
See [Hatch support](../hatch.md).

## SOLID/TRACE descriptors and proof scope

The [filled descriptor](solid-trace-design.md) stores four ordered local XY slots
and signed thickness in one existing frame; two native kinds preserve CAD identity.
All finite topologies remain supported. Exact rational evaluation and outward
bounds include B0..B3 and T0..T3 with the cross product of the stored axes.
Line/point collapse is not empty geometry and does not remove these eight pairs.
Native save preserves authored bounds and does not recompute them; explicit bounds
recomputation remains atomic. Filled parameters have no pattern-marker or stroke
adapter and are not eligible clip boundaries.

CAD preparation checks actual codec OCS positions against all eight native
positions in the existing Model/Paper/definition/occurrence domains. Common
simple-polygon cells optionally prove mathematical surface/sweep correspondence;
concave shapes require an interior diagonal. No common cover gives Unassessed,
never a numeric failure or omission. Required position proof failures remain hard.
`filled_surfaces()` exposes retained native/occurrence identities and Proven or
Unassessed coverage. Unknown surface coverage keeps geometry `is_complete()` false,
even with Exact positions. Neither Exact nor Proven certifies a host picture.
These records are conversion evidence, absent from native IFCX/CBOR payloads.


Dimension graphics uses owned primitive payloads and the same shared validators
and bounds/preparation kernels as ordinary entities; parts allocate no native
entity IDs. Conversion uses tagged group/part/occurrence keys and pairs the
actual source and target picture frames. The parent Dimension placement applies
once to every part, including helpers for numeric proof. ReferencePoint helpers
are excluded only from display/bounds. Nested marker transforms retain the
ordinary occurrence arithmetic. Eight SOLID/TRACE positions, optional surface
coverage and unassessed display remain independent evidence.

Planar width bands use local full widths, shared exact operands and the actual
stored plane/occurrence maps. Full nondegenerate strips are required independently
of Allow/Reject; joins and host display remain separate. Definition-local proof
does not replace nested Model/Paper occurrence proof. The core's explicit
`geometry_centerline_bounds` supports Hatch/clip contour semantics, while
`polyline_width_display_bounds` supplies segment contributions and located Partial
reasons for drawing bounds. Derived effective widths do not replace authored state.
