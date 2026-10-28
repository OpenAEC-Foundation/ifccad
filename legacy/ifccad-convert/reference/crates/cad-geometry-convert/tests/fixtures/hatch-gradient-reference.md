# Hand-authored Hatch gradient reference

Authored 2026-10-13 for correctness qualification, not a benchmark input.
The DXF is literal AC1032 data, independent of the gradient writer.
Pinned codec ab2eecdbffc31120b5ad6d899f6fc67cf21ede39, viewport-off patch 0003 only.

Hatch AB has one closed asymmetric rectangle (-2,-1)–(5,3) in +Z OCS,
normal area rule, and active INVSPHERICAL gradient. Authored angle=.75 radians,
shift=.5, one-color flag=true, tint=.8. Ordered colors are RGB(220,80,30)
and RGB(30,60,150), at values 0 and 1. Color 2 is independently specified;
no tint regeneration is part of the expectation. ACI fallback tags precede
authoritative RGB tags; swapping those tags characterizes the pinned parser gap.

The fixture establishes stored parameters and contour meaning. It has no
external CAD application/rendering certification or gradient-field guarantee.
