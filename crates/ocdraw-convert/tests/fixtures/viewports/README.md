# Viewport clipping and angle fixture

`reference-clip-activation.dxf` was copied from the local IFCCAD viewport
development slice and extended with literal VIEWPORT group 50 = 30 degrees
and group 51 = 90 degrees. It is hand-authored, not produced by the writer
under test or by AutoCAD. Group 90 = 98304 enables non-rectangular clipping;
group 340 points forward to CIRCLE handle 1002.

The fixture isolates codec activation, reference and angle decoding. It is
not an independent application interoperability certificate.
