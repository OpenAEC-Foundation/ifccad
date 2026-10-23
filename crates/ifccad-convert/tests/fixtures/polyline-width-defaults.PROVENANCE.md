# Classic width dependency fixture

Hand-authored ASCII DXF, not produced by an IFCCAD or codec writer. Header defaults
are start 0.2/end 0.4. Four ordered vertices deliberately have these tags:

| Vertex | Group 40 | Group 41 |
| --- | --- | --- |
| 0 | absent | absent |
| 1 | explicit 0 | absent |
| 2 | absent | explicit 0 |
| 3 (dormant in open path) | explicit 0.2 | explicit 0.4 |

The final equal-valued overrides must not be inferred absent. Independent tag
oracles verify parent/child owners, distinct child handles and presence. Raw DWG
oracles use generic object framing and primitive bit values, independently of the
codec's entity width decoders. Reference: ODA Open Design Specification 5.4.1,
POLYLINE (2D), VERTEX (2D), LWPOLYLINE and BD/DD primitive layouts:
https://static.opendesign.com/files/guestdownloads/OpenDesign_Specification_for_.dwg_files.pdf

DWG stores numeric vertex widths; no absent/explicit selector is invented here.
The tests distinguish numeric agreement from classic default dependency retention.
