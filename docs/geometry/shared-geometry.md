# Shared geometry and conversion accuracy

The independent OCDraw and IFCCAD contracts use `ocdraw::geometry_kernel` for primitive
values, coordinate frames, length-unit tokens, intrinsic geometric validation,
conservative bounds and analytic Paper clip containment. A borrowed
`GeometryRef` covers points, lines, circles, signed arcs, full/partial ellipses,
straight or bulged planar paths, and straight spatial paths. It carries no IDs,
ownership, appearance, schema, codec packing or polyline pattern-generation state.
Each model adapts its own records and retains its own reference validation.
Existing public geometry-type imports under `ocdraw::ocdraw` remain reexports.

The core has no CAD-runtime dependency. The workspace `cad-geometry-convert` companion
owns common CAD-plane preparation, exact/rational and interval calculations,
paired primitive/curve evaluation, unit resolution, and nested block occurrence
assessment. Both converters depend on it directly; neither converter depends
on the other. Coverage inventories, losses, IDs and route errors remain separate.
Both converters use the shared CAD unit-token registry and code order. IFCCAD's
local unit-code helper adapts its string-valued units to that registry.
OCDraw's typed spline preservation remains separate from native primitive proof:
retained spline sources and their occurrences are registered as unassessed.
The report's `is_complete()` remains false for those sources; primitive status
and residuals never certify the preserved spline's geometry.

Both converter crates use `from_cad/` and `to_cad/` for direction-specific
orchestration and construction, `mapping/` for format adapters, and `source/`
for CAD input inspection. `mapping/geometry.rs` translates each format's native
records to and from shared geometry. Source inspection calls the shared CAD
geometry helpers directly rather than going through a local forwarding module.
Format-specific `geometry_context.rs` and `geometry_assessment.rs` remain at each
converter's root. Matching folder responsibilities do not imply matching format
coverage or a dependency between converters.

Both converters wrap the shared `ExchangeState` with format-specific owner/domain
contexts. Model and definition-local assessment use the drawing unit; Paper uses
its layout's fixed plot mapping, without an independently authored coordinate unit.
See [layout output](../layout-output.md) for medium, scale and migration rules.

## Accuracy policy

Both directions expose `geometry_tolerance`. Default is one micrometre in known
Model units and on known fixed physical Paper output; unknown domains require zero
coordinate residual. `drawing_units(x)` applies directly in each numerical domain.
Explicit metres/mm requests require a known drawing unit or fixed physical Paper
mapping, including empty retained layouts. Medium dimensions and shaded DPI never
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
underflowed ellipse ratios and coincident target endpoints of a nonzero bulged
segment. A generous tolerance does not authorize an invalid target curve.
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
collections remain unchanged. See the [geometry sample](../../examples/ifccad/hello-geometry.ifcx).

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
