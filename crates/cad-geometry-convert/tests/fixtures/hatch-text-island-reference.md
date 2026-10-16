# Text island reference tags

This hand-authored ASCII DXF uses AC1032 and millimetres. Hatches AB and AC
each contain two concentric outer circle contours of radius 4 and 3, followed
by an explicit four-edge rectangular text contour from (-1,-0.5) to (1,0.5).
The circles carry flags 17 and 1. The third path carries 128 in AB and 136
in AC: TextIsland alone and TextIsland plus Textbox. Both Hatches are Solid,
Normal, non-associative, with no source handles or seed points.

The independent expectation is exact preservation of those loop flags and
stored contours. The rectangle is surrounded by two outer contours. The
fixture does not claim external-app generation, pixel evaluation or text-font
geometry. Tests read its tags through the production DXF reader. Separate
production-writer/readback tests qualify flag transport for all native fills
and area rules in AC1032 DXF and DWG.
