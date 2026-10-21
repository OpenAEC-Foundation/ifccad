# SOLID/TRACE: concave and degenerate OCS cases

Research completed on 21 October 2026 at the user's request. This report revises
the [native design](solid-trace-design.md) and its implementation plan; it does
not implement the IFCCAD kinds, change active schemas or qualify the IFCCAD codec.
The user subsequently chose support for all finite local corner combinations.
That decision is reflected below; the recorded CAD observations are unchanged.

## Result and profile decision

The convex-only restriction was too narrow. The tested OCS build fills simple
concave SOLID and TRACE quads in both winding directions. It also fills triangles
spelled by adjacent duplicate corners outside the conventional repeated slots
3/4, and a positive-area boundary with a straight-through collinear corner.
These cases are included, preserving all four slots.

OCS also accepts, displays and saves finite crossing, retraced and wholly
collapsed records. File acceptance does not establish one portable filled region
or extrusion meaning. These states are now also valid native descriptors: they
are retained without inventing a universal fill or lower-dimensional render rule.
Unknown surface meaning is recorded as Unassessed, not unsupported/loss/invalid.
Conversion must not repair a walk, choose a convex hull, drop an isolated spur
or turn it into another entity kind to produce a simpler picture.

The alternatives were: retain convex-only support; add the evidenced simple
concave/redundant-corner cases; or admit every finite four-slot walk with a new
fill/lower-dimensional contract. The middle option was the first recommendation
after the experiment and is now superseded by the user's decision. All finite
descriptors are stored natively, while optional full-surface interpretation and
viewer guarantees stay separate. No universal render rule, generic preservation
wrapper, or native classification field is required by that decision.

## Tested application and method

- Installed desktop OCS **2026.42**, release build **b6fbac39ffc7**.
- Tool-schema digest: `d15005e14fc00154501db5d261cb9febae1bcc61f6d9b850e91b7fd1879c3b8e`.
- Main IFCCAD baseline: `56644eaacddeec9cacf9963124ac40a8c2671949`, with existing
  documentation and independent Explorer changes preserved. No product patches.
- Public OCS MCP API: session discovery/capabilities, separate document open,
  full `records`, actual viewport capture, `save_verified`, and DWG reopen.
- 16 configurations × SOLID/TRACE × three input variants = **96 filled records**:
  thickness 0, thickness +3, and reversed winding at thickness 0. Each drawing
  additionally has 32 labels, unique explicit handles and normal `(0,0,1)`.
- Views: top/2D wireframe for zero thickness; top and isometric for thickness +3,
  including the explicit `SHADEMODE FLATSHADEDWITHEDGES` mode. A 2D wireframe image
  alone does not show whether an extruded object has a valid filled mesh.

The authored, CC0 inputs are retained under
[fixtures/solid-trace-profile](fixtures/solid-trace-profile/cases.json).
They are reference CAD source material, not released conformance fixtures.
The initial draft with automatically assigned label handles collided with explicit
filled handles. It was discarded from all findings; only the corrected inputs
with 64 unique handles, zero malformed pairs and clean save audits qualify here.

## Observed cases

Case numbers match both `S00..S15` and `T00..T15` in the drawings. Boundary order
is stored slots **1,2,4,3**. “Filled” describes the observed area, independently
of the boundary lines OCS also draws. All 16 cases were accepted on import.

| Cases | Zero-thickness observation, both kinds | Thickness +3 observation | Revised native profile |
| --- | --- | --- | --- |
| 00 convex | Filled rectangle | Extruded filled body | Included |
| 01 triangle34 | Filled triangle, conventional slots 3/4 repetition | Extruded filled body | Included |
| 02–05 concave1..4 | Filled polygon; indentation preserved at each of the four corners | Extruded filled body for each indentation | Included |
| 06 crossing | Bow-tie boundary, no filled lobes | Base boundary only | Included; surface Unassessed |
| 07 repeat12 | Filled triangle | Base triangle outline only; no extruded body | Included triangle spelling; host display limitation below |
| 08 repeat13 | Filled triangle; slots 1/3 are adjacent across cyclic closure | Base triangle outline only | Included triangle spelling; host display limitation |
| 09 repeat24 | Filled triangle | Base triangle outline only | Included triangle spelling; host display limitation |
| 10 repeat14 | Retraced L-shaped boundary, no filled area | Base L-shaped boundary only | Included; surface Unassessed |
| 11 collinear | Line, no filled area | Base line only | Included; surface Unassessed |
| 12 point | Coincident point, no filled area | Base point only | Included; surface Unassessed |
| 13 flat_corner | Filled triangle-shaped region; extra corner is straight-through collinear | Extruded filled body | Included nonzero-area redundant corner |
| 14 backtrack | Filled triangle with a retraced spur | Extruded triangle-shaped body; does not sweep the complete spur | Included; surface Unassessed |
| 15 two_points | Retraced line, no filled area | Base line only | Included; surface Unassessed |

The reversed-winding input retains the same observed fills/outline-only states.
As a supplementary image check, the TRACE cases were sampled on the same 24×24
interior grid in the reopened-DWG and reversed-input captures. All 16 sample
counts matched. This is an image regression observation, not an area calculation
or a numerical geometry proof: colored boundary pixels also count, notably in
the crossing case. Viewport overlays/cursor/font output are not certified.

## Storage and readback evidence

At import, complete OCS entity records matched every expected raw corner slot,
type flag and thickness for all three input drawings. The inspector reports
both kinds internally as `Solid`; checking that label alone would miss TRACE.
The `is_trace` flag was checked independently for all 16 TRACE records per file.

The h=0 and h=3 drawings were each saved with OCS `save_verified` to ASCII DXF
and DWG, target AC1032. All four operations reported verified readback, zero
dropped entities, and audits with zero errors/warnings. The h=0 DWG was then
opened as a separate document: all 32 filled type flags, four corners and
thickness values compared exactly against the original source manifest. Its
actual viewport retained the concave fills and degenerate display states.

An independent literal DXF tag check of both written DXFs found **16 SOLID and
16 TRACE** records, with all four corner values and thickness equal to input.
This supplements OCS's common `Solid` manifest and avoids confusing that label
with object-kind loss. OCS's verified-save report is not independent DWG type-code
decoding; the later IFCCAD implementation still requires that separate gate.

| Artifact | SHA-256 |
| --- | --- |
| Original zero-thickness input | `00781d3d84d0846679444f12796331b20b61f015c22fe2c820c9abb6410ca35a` |
| Original thickness +3 input | `b3771a9d8006f5f0d8046d482e832e97db9a4edd76db52b63f21dbfbd3577b4f` |
| OCS h=0 DXF | `1a7253c4debfbeb8c1e7096693bdbcc8548bcdf12ba5fde2c2c35c484198d791` |
| OCS h=0 DWG | `21842f3be8b1fb81b859489bd8fb07f78c04be09e12e65bc73b35e998451a448` |
| OCS h=3 DXF | `4f53327d008f35d152a9c8459305b0ff131a9d4163fef1af7e6e0a3f17f6bc97` |
| OCS h=3 DWG | `337c99c5ca5f1e1d566a13e3cb118926efdaf1e8c85ba638686ee0542b64ae79` |

The exact input hashes, raw slots and display decisions are in the fixture
manifest. Detailed records, written CAD files, untouched viewport PNGs, API
schemas/build identity, source inspection and verification scripts/results are
retained locally in the ignored task directory
`.superpowers/research/solid-trace-profile-2026-10-21/`.

## Consequences for validation, evidence and OCS

Validate finite values, structure, frame and finite parameter enclosure for every
corner combination. Simple-area/triangle analysis is optional derived information,
not a validity or conversion gate. Do not reorder, remove or canonicalize any slot,
including zero-area/crossing/retraced states. All 16 source configurations are now
positive native/conversion retention cases. The binary/IFCX shape, placement,
signed-thickness field and CBOR codes stay unchanged.

Eight paired base/top positions alone do not prove a concave fill. Require a
common valid corner-index triangulation of source and target, then apply the
existing affine triangle/prism deviation argument to those covering cells.
A fixed diagonal or convex hull can fill a concave indentation incorrectly.
No common cover after rounding gives surface Unassessed without refusal, loss,
omission or partial-open recovery. The required eight-position accuracy proof
still passes or hard-fails independently at the unchanged tolerance. Neither
Exact positions nor a Proven surface certifies a host's display, and parameter
envelopes do not define a fill or predict a tight rendered footprint. No display
generator or stored triangle mesh is added.

For alternate triangle spellings with h=3, the tested OCS source triangulates a
temporarily reduced boundary but constructs extrusion from the original boundary
including zero-length edges. The observed base-only fallback is consistent with
that path. This is a **host rendering limitation to report**, not evidence that
the source thickness was lost or that native triangles must be rewritten. The
[exact tested source](https://github.com/HakanSeven12/OpenCADStudio/blob/b6fbac39ffc7/src/entities/solid.rs)
also has renderer tolerances/triangle classification that must not define the
native exact-equality contract. No OCS patch or external issue was created.

## Limits and reproduction

This qualifies the stated OCS build and cases. It does not establish AutoCAD,
BricsCAD or all CAD-host rendering, binary-DXF output, other DWG versions, tilted
normals, negative thickness or near-epsilon topology behaviour. Those remain
implementation qualification cases where the design requires them. This is no
benchmark, production-code test suite or numerical full-fill certification.

To reproduce, open the three retained DXFs as separate documents in the recorded
OCS build, inspect all four raw corner fields and `is_trace`/thickness, use top
and shaded-isometric views, then verified-save h=0/3 to separate AC1032 DXF/DWG
paths. Reopen the DWG and compare raw values plus the concave/degenerate pictures.
Use the fixture manifest's geometry and hashes, not an approximate redraw.
Preserve unrelated open drawings. The public
[OCS automation guide](https://github.com/HakanSeven12/OpenCADStudio/blob/b6fbac39ffc7/docs/automation/README.md)
describes the transport; live tool-schema discovery is authoritative for the build.

Open IFCCAD issue titles were rechecked: #2 covers bounded typed entity families;
#11 keeps placement investigations separate; #13 keeps runtime updates separate;
#17 is broader user-research/robustness context; #19 does not change the selected
CBOR revision policy. None requires an opaque wrapper for these native descriptors, a new kernel
runtime dependency or an OCS repair in this precursor.

## Implementation roundtrip qualification (21 October 2026)

The SOLID/TRACE candidate additionally converted each of the three retained source
DXFs through IFCCAD and back to actual ASCII DXF and AC1032 DWG bytes, with
production CAD and strict IFCCAD readback. The unchanged OCS build recorded above
opened all nine files (three originals and six roundtrips). Its complete entity
record API confirmed all 32 filled records/file (16 SOLID,16 TRACE), with exact
four raw slots, normal, thickness and type matching each original. Captured top
views showed the same filling/outline behaviour for the compared shapes. The
wireframe captures do not newly qualify shaded/isometric extrusion or establish
pixel equality; transient command overlays also differ. No OCS patch was applied.
Independent automated tests additionally cover binary DXF, negative thickness,
elevation and tilted normals; that is separate from this manual OCS observation.
