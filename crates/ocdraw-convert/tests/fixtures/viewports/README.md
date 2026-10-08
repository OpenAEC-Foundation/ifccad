# Viewport clipping and angle fixture

`reference-clip-activation.dxf` was copied from the local IFCCAD viewport
development slice and extended with literal VIEWPORT group 50 = 30 degrees
and group 51 = 90 degrees. It is hand-authored, not produced by the writer
under test or by AutoCAD. Group 90 = 98304 enables non-rectangular clipping;
group 340 points forward to CIRCLE handle 1002.

The fixture isolates codec activation, reference and angle decoding. It is
not an independent application interoperability certificate.

## Perspective reference

`reference-perspective.dxf` and `reference-perspective.json` reuse the independently
specified IFCCAD camera reference. They are hand-authored constants, not output
from the codec writer or an external CAD application. The test constructs a
separate orthographic overall Paper canvas and imports the two authored cameras
without inferring current Paper selection. Expected landmarks use test-only
Autodesk WCS/DCS camera calculations; they do not prove renderer accuracy or
projected curve tolerance. Direction magnitude and lens length in millimeters
are retained for Model insertion unit codes 0, 1 and 4.

`presentation-xrecord-scalars.json` contains literal method-tagged ACI 1,
explicit alpha and a 25 hundredths-of-mm lineweight. Both memory and physical
DXF/DWG tests compare native meaning against separately written expectations.
