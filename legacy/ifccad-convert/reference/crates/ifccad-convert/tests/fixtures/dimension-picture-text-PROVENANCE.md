# Literal Dimension picture text fixture

`dimension-picture-text-reference.dxf` is hand-authored ASCII DXF, not output
from the converter or CAD writer under test. It names AC1032, an anonymous `*D1`
block with one MTEXT at owner coordinate (120,12,0), height 2.5, width zero and
literal `4'-0\S1/4;"`, linked by an aligned Dimension whose line point is
(100,10,0) and references are (100,0,0)/(140,0,0). Explicit high handles avoid
collision with the reader's standard table defaults. The test asserts local
(20,2,0), no wrapping, stack syntax and typed text through independently written
ASCII/binary DXF and DWG reader reopen. It does not certify font glyph layout.
