# Dimension fixture provenance

`tilted-aligned.dxf` is hand-authored ASCII DXF, not writer-generated output.
It specifies an aligned dimension with normal `(1,0,0)`, WCS groups 10/13/14
and literal OCS group 11 `(2,7,3)`. The independent arbitrary-axis expectation
is U=(0,1,0), V=(0,0,1), so text WCS=(3,2,7) and native local=(2,7).
No font rendering or external CAD application produced this fixture.

`cad-dimension/tests/codec_boundary.rs` also makes explicit literal mutations:
group 12 `(19,23,29)` for a clone and group 16 `(2,7,3)` for a two-line angular
dimension. Clone group 12 exposes a pinned reader defect: the point is collected
but never assigned. This is recorded as a limitation, not qualified as supported.
Group 16 is retained by the CAD model; angular dimensions remain a later slice.

The same test target independently writes and reads actual DXF and AC1032 DWG
with tilted geometry, explicit text, a stale measurement cache and a DIMASSOC
start reference. It compares semantic values and retains the stale definition.
These are codec correctness tests, not size measurements or CAD-host display
certification. Dependency base and local patch are specified in the field audit.

## repair-profiles.dxf

Hand-authored minimal AC1032 ASCII DXF for the 2026-10-16 Dimension repair.
Coordinates are synthetic (aligned distance 48.25), not project geometry.
The explicit ACAD DSTYLE combines architectural diagonal fraction layout and
scale, ACI red, fixed extension length/enabled, and separate null/closed-filled
marker resets. The marker switch follows the first marker deliberately to test
order independence. It reproduces setting combinations observed in sample_AC1032
and the foundation investigation without embedding either source drawing.
