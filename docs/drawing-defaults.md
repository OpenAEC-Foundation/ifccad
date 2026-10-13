# Drawing defaults and compact JSON

OCDraw and IFCCAD give corresponding CAD fields the same default meaning while
retaining separate logical models, schemas, codecs and conversion routes.
Readers expand an omitted default into the complete typed document before shared
semantic validation. JSON Schema annotations do not inject these values.

## Default matrix

| Native field | Meaning when omitted | JSON mapping |
| --- | --- | --- |
| Line endpoint Z | +0 | OCDraw may omit a whole `z1` or `z2` column; IFCCAD accepts `[x,y]` as `[x,y,0]` |
| Entity color/opacity/line pattern/line weight selection | ByLayer, independently | OCDraw may omit mode columns; IFCCAD may omit properties or the entire native appearance object |
| Native entity visibility | true | Missing column or field |
| Drawing/entity line pattern scale | 1 | Missing column or field |
| Block rotation | +0 radians | Missing field within the transform |
| Block scale | [1,1,1] | Missing complete scale field; partial scales are invalid |
| Layer visible/plottable | true | Missing fields |
| Layer frozen/locked/frozenInNewViewports | false | Missing fields |
| Layer opacity | 1 | Missing field; IFCCAD stores it inside concrete layer appearance |
| Planar/spatial polyline closed | false | Missing column or field |
| Polyline line pattern generation | perSegment | Missing column or field |
| Text/MText rotation, Text oblique angle/thickness | +0 | Missing column or field |
| Text/MText backward/upsideDown | false | Missing column or field |
| TextStyle widthFactor | 1 | Missing field |
| TextStyle obliqueAngle | +0 | Missing field |
| TextStyle vertical/creationBackward/creationUpsideDown | false | Missing fields |
| Line-pattern marker scale/offset/rotation | 1 / [+0,+0] / relative +0 | Missing fields; absolute zero rotation remains explicit |
| Hatch LinePattern fill origin/rotation/scale | [+0,+0] / +0 / 1 | Missing fields |

Layer color, line pattern and line weight remain explicit concrete values.
ByLayer and ByBlock are selection modes, not evaluated colors or appearance
snapshots. Explicit mode requires its value; inherited modes forbid a value.
Native entity layer references, IDs, allocation watermarks, units, radii and
mandatory geometric parameters are never inferred from omissions.

## Exact numerical omission

Writers omit only the exact default. For newly compacted binary64 values, the
comparison preserves the sign of zero: explicit -0 remains stored, as does every
nonzero value, however small. No geometric tolerance or quantization participates.
IFCCAD line endpoints accept exactly two or three finite numeric components;
missing X/Y, null and other lengths remain invalid.

Z omission applies to line endpoints only. Spatial-polyline XYZ vertices/pools,
placement origins and axes, bounds, camera vectors and individual scale
components retain their current contracts. Placement omission rules remain
format-specific: this work does not add IFCCAD placement or axis defaults.

## Columns, presence and composition

OCDraw omits a default column only when every row has that default. A mixed
column remains complete, in row order and with its declared count. Optional
appearance-value columns retain their existing null placeholders where a row
has an inherited mode; a wholly unused value column may be absent. Other nulls,
short columns, missing vertex pools and invalid ranges are still rejected.
Omission never turns missing required vertex data into zeros.

IFCCAD applies defaults after its existing fragment composition. Later attribute
objects replace earlier objects as before. A later `ifccad::entity` without
appearance therefore means all ByLayer, even if an earlier fragment specified
an explicit color. Source-conflict checks do not canonicalize fragments before
applying RejectConflicts. Original source bytes remain available independently
of encoding a fresh CAD-profile file.

These rules are a bounded whitelist. Authored inline/paragraph formatting,
fontrequest bold/italic options, viewport overrides, workspace choices and
inactive gradient settings retain their presence meaning. A stored false, zero
or empty array can be an authored reset. Missing opaque native appearance stays
unknown rather than becoming ByLayer; opaque envelope requirements are unchanged.
Polyline closure omission does not alter Hatch boundaries or active clip
eligibility. An active viewport clip must still use an eligible closed contour.

## Compatibility and evidence

Both active 0.1.0 contracts remain provisional. Updated readers accept existing
fully expanded data spellings, but older readers can reject newly compact output.
IFCCAD imports resolve the current bundled experimental module. Inline profile
definitions still must match that module exactly; older inline definitions can
require an explicit update even when their data values are fully expanded. This
does not publish a supported version, define a binary format or add a legacy
compatibility layer. Only active schemas and candidate `conformance/next`
fixtures change; numbered conformance collections remain immutable.

Writer output is loaded through production readers and strict drawing validation.
Tests compare complete native semantic content, signed-zero bits where relevant,
composition, malformed input and the existing CAD conversion boundaries.
Historical size reports retain their own revisions and coverage. No new
size/exchange measurement is authorized by, or part of, this defaults change.
