# Native planar polyline widths

Native model/validation/storage sections remain active. Direct `ifccad-convert`
exchange and codec qualification below describe the [archived reference](../converter-parking.md);
the [shared Document route](../architecture/explorer-document-route.md) has its own coverage.

IFCCAD stores full local-coordinate widths independently of presentation lineweight.
Both ordinary planar polylines and owned Dimension graphics use the same three modes:

- `constant`: one `width`; only the intrinsic positive-zero constant may be omitted.
- `perVertex`: exactly N explicit `[start,end]` pairs, including the dormant final pair of an open path.
- `defaulted`: independent `defaultStartWidth`/`defaultEndWidth` and exactly N `overrides` records. Missing `start`/`end` inherits the matching default; an explicit zero or a value equal to the default stays explicit. Null is rejected.

All scalars must be finite and nonnegative, including unused defaults and dormant leaves.
Signed zero and binary64 values are retained on native wire. Composition replaces the
whole width attribute; override arrays are never merged by vertex index. Effective
pairs are derived views, not a second editable source of truth. Editing a default
changes inherited leaves only. Recompute bounds explicitly after edits.

The active IFCX schemas and mutable CBOR revision 1 mapping define the wire contract.
Planar body field 5 carries widths in ordinary and graphics geometry; enum codes
0/1/2 select the three modes. CBOR width scalars use binary64. Released numbered
conformance collections remain unchanged; new cases live in `conformance/next`.

## Geometry and bounds

Every nondegenerate transferred segment requires a full-band positional certificate,
including taper and directed bulged arcs. Its local unit normal is transformed by
the actual stored plane and occurrence linear maps without renormalization. Block
translation does not affect normal offsets. A newly collapsed target chord with
positive width fails. Numeric exceedance/indeterminacy is a hard failure under both
loss policies; it never selects a Hatch editing fallback.

Source-coincident wide segments retain their authored state but have no defined
band tangent. Band evidence is then Unassessed. General joins, reversal, coincident
wide paths and unresolved pattern interaction cannot establish complete display
bounds. Proven segment contributions still check producer enclosures, but a Partial
scope cannot authorize negative-query culling or complete Zoom Extents. Isolated
segments and exactly tangent-continuous equal-cross-section joins have enclosing
bounds. Host pixels remain Unassessed even with proven band positions. Hatch source
and viewport clip contours keep their existing centerline semantics.

## Physical exchange qualification

The converter uses the explicit named-base patch in
[`patches/opencadcodec-polyline-widths`](../../legacy/ifccad-convert/reference/patches/opencadcodec-polyline-widths/README.md),
on acadrust/opencadcodec `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`.
The core crate has no codec runtime dependency. `Vertex2D` exposes optional width
leaves; classic DXF output must choose `Polyline2DWriteMode::PreserveClassic`.
Constant lightweight output uses group 43 without competing groups 40/41.

| Native state | ASCII DXF, six offered versions | Binary DXF | DWG, six offered versions |
| --- | --- | --- | --- |
| Constant / nonzero PerVertex | Numeric controls retained | Retained at actual version | Numeric controls retained |
| Defaulted, every leaf explicit | Defaults and explicit leaves retained | Same | Defaults and numeric leaves retained |
| Defaulted with inheritance | Defaults, absence and explicit zero/equal retained | Same | Inherited leaves materialize: located dependency loss |

The six ordinary versions are AC1015, AC1018, AC1021, AC1024, AC1027 and AC1032.
Binary requests AC1015/AC1018 promote to AC1024; these are not requested-version
passes. Defaulted DWG inheritance fails Reject before a download. Allow reports
each affected leaf even when effective geometry agrees. All-positive-zero PerVertex
mode recapture and DWG signed-zero normalization are separately disclosed. Stored
Dimension graphics with widths are qualified for AC1032 only. Width preflight is
separate from Hatch and upright line-pattern restrictions and performs no second
geometry conversion at a default tolerance.

`polyline_width_exchange` records 72 ordinary mode/format/version cells. Independent
ASCII/binary tags test presence/ownership; the raw DWG BD oracle tests numeric
POLYLINE/VERTEX values without invoking entity width decoders. Its reference is the
[ODA Open Design Specification](https://static.opendesign.com/files/guestdownloads/OpenDesign_Specification_for_.dwg_files.pdf),
version 5.4.1, POLYLINE (2D) and VERTEX (2D) layouts. Numeric DWG equality is not
classic dependency preservation.

## Original linked Dimension

The opt-in `polyline_width_acceptance` example selects actual Dimension `0x527`,
its `*D8` picture and marker/resource closure from an original supplied at runtime.
It refuses an existing output directory and verifies the approved source SHA-256.
No private source or local absolute path belongs in version control. Source tables
are conservatively retained with their marker dependencies; unrelated entity instances
and Paper layouts are explicitly excluded and inventoried. This is correctness
acceptance, not a size/exchange measurement or whole-sample lossless-import claim.

Both original DWG and linked ASCII DXF pass strict IFCX, CBOR, opt-in Zstd and
AC1032 ASCII/binary DXF/DWG readback. All nine parts, INSERTs `660`/`661`, marker
`4A6` width 0.15 bits, layer/ByBlock context and MText `662` wrap zero/stacked content
are retained. Actual stored binary64 placements feed band proof; nominal decimal
0.375 is not an exact placement oracle. Detailed task-local reports retain hashes,
source records, diagnostics, returned records and independent Dimension assessments.

The unchanged pinned Open CAD Studio host (2026.40.1, pin and asset hashes retained
locally) opened both originals and all six actual AC1032 outputs. Bounded planar
observations showed both red wide ticks, matching placement, no added wrapping and
the same stacked label. Initial sandbox loopback failures were resolved by the
approved local-only test server. Capture/state reports retain source/output hashes
and view operands. This qualifies the stated views only; it does not establish
universal host pixels, joins or fonts. No host/renderer patch is included.
Extrusion thickness remains a separate follow-up slice.
