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

`ifccad-convert` uses an internal `GeometryContext` to track the active owner,
resolve and collect each coordinate-domain assessment, and attach IFCCAD domain
identities to shared numerical failures. It wraps the shared `ExchangeState`;
it does not define primitive geometry or persist conversion settings in a drawing.
OCDraw currently uses `ExchangeState` directly with one drawing-unit assessment.
OCDraw does retain per-layout plot-media units and scale mappings. This is distinct
from declaring a Paper coordinate length unit and selecting its own numerical
assessment domain. AutoCAD's per-layout page setup determines Paper unit meaning;
DXF represents that through plot units and scale. See the official
[model/Paper explanation](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-Core/files/GUID-990538B6-DDA1-4190-BCC0-BB5BA94C9879.htm)
and [PLOTSETTINGS fields](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-1113675E-AB07-4567-801A-310CDE0D56E9.htm).
IFCCAD's explicit Paper unit is a native contract choice; its current CAD export
supports unitless/inch/millimetre coordinate mappings and diagnoses other units.

## Accuracy policy

Both conversion directions offer a `geometry_tolerance`. The default is exactly
one micrometre in known coordinate units and zero in unitless coordinates.
`exact()` requires zero residual. `drawing_units(x)` applies the same finite,
nonnegative numerical limit in each domain. `metres(x)` and `millimetres(x)`
resolve physical limits independently in each domain; explicitly physical
requests refuse unitless domains, including empty retained Paper layouts.
Physical sheet media never supplies a missing coordinate unit.

IFCCAD Model and definition-local checks use the drawing unit. Each Paper layout
has a separate coordinate domain. Outcomes expose per-domain limits, counts,
status and worst source or nested occurrence path. There is no unqualified
global maximum across unlike units. Local definition acceptance is insufficient:
all retained root occurrences are also checked after signed/nested transforms.

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
