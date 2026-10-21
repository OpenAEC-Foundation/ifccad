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
unsupported. Classic 2D polyline widths and vertex metadata are not silently
folded into a bulged path. Direct, DXF and DWG source tests exercise these
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
