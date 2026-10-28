# CAD presentation conversion

ID-free scalar rules shared by the independent IFCCAD and OCDraw adapters.
This companion depends on opencadcodec, not either native drawing model.
Ownership, references, inherited modes and located loss policy belong to callers.

Concrete colors retain RGB and optional ACI/named identity. An unsupported or
RGB-inconsistent indexed identity falls back to the authored RGB with a typed
loss reason. Named identity is returned separately; its layer/entity/override
serialization and representability remain adapter-specific.

Opacity imported as `1 - byte/255` returns that exact transparency byte.
Other valid opacity values use CAD's upward transparency rounding. The mapping
returns the reconstructed opacity and whether its value changed.
Lineweights select the nearest of the 24 standard CAD table values. Interior
binary64-distance ties choose the lower value; outside the interval the nearest
endpoint is selected. Invalid scalars are errors, never silently clamped.

`presentation_scalars` tests all 256 opacity bytes, independent authored and
invalid values, all standard weights and boundary/tie cases, RGB/ACI consistency
and inherited/unsupported source colors. Scalar success alone does not establish
DXF/DWG transport support or independent application interoperability.

Viewport XRecord scalar helpers qualify AcCmColorBase RGB/ACI method bytes,
explicit alpha methods 2/3, and explicit hundredths-of-mm lineweights. Opacity
uses the ordinary CAD byte-derived convention before exact byte recognition.
Strict CAD-only section coordinates are shared; native ID binding remains in
each adapter. Unknown sections, conflicting owners and incomplete streams stay
outside the qualified grammar. Only completely consumed records and dictionaries
may be exempted from residual inventory loss; mixed future data remains visible.
Named/custom override metadata is not qualified and never silently asserted exact.
