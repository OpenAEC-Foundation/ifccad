# opencadcodec dependency audit — 2026-10-23

Target: upstream `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`, replacing
`325fa98d2b881f48e106a830d5cf161624f63131`. Upstream declares package `acadrust`
0.6.6. All seven active codec-dependent manifests preserve the dependency name
`opencadcodec` with `package = "acadrust"`. The core remains independent.
The archived OCDraw checkpoint is untouched.

## Repairs and selection

The exact [Hatch angle fix #132](https://github.com/HakanSeven12/opencadcodec/issues/132)
is included. The DXF writer searches neighboring representable degree values
for an exact inverse through its reader. Both reported stored angles are
regression-owned through all six offered output versions; no tolerance is
loosened and physical comparison remains enabled. This does not guarantee an
exact degree preimage for every binary64 radian value.

[Dimension text-position polarity #117](https://github.com/HakanSeven12/opencadcodec/issues/117)
is upstream in the reader, writer and default flag. No local source patch is
selected. Historical patch files remain provenance only; the shared CI action
and explicit local checkout configuration select the unmodified new base.

The upstream Model layout/root repair (#122) now completes the minimal
handleless-DXF scaffold before conversion. A production reader regression checks
that repaired graph without requiring our former recovery diagnostic. Separate
explicit public-model fixtures retain the narrow recovery and all six authored/
ambiguous rejection controls; source documents remain unchanged. The reader also
resolves the relocated named ACAD_LAYOUT cache (upstream 8dbca89). Its physical
DXF test now requires the corrected cache without an IFCCAD recovery; separate
public-model stale-cache and twelve conflict cases remain unchanged.

## Public model and source coverage

| Changed surface | Active treatment |
| --- | --- |
| Package/library name | Keep existing local import alias; no IFCCAD public rename. |
| `LineTypeComplexData.upright` | Maps to native exclusive Upright(angle), including typed true with raw source_flags zero. Conflicting absolute/upright selectors and unknown flags remain diagnosed. Old DXF loss is disclosed and verified through physical readback. |
| Removed `Viewport.stacking` | Independent literal group-68 checks use `dxf_status`, separate from group-69 `id`. Unsupported authored ranks remain located source losses. |
| Header `create_ellipse_as_polyline`, `dim_arc_symbol`, `dim_fixed_ext_line_on`, `dim_leader_arrow_block_handle` | Existing complete header residual comparison diagnoses nondefault state; focused regression covers each field and Reject. No new native header settings; upright is the user-authorized native addition in this adoption. |
| Common Serde references/flags, spline scenario, associative provenance | More fields now serialize rather than being skipped. Common/source inventories retain their explicit rules; unsupported objects remain diagnosed. No broad ignore-list additions. |
| Dimension angular stored measurement in radians and existing style transport repairs | Angular kinds remain unsupported by the current native Dimension subset; existing explicit losses remain. Linear/aligned raw-bit polarity and physical transport checks remain required. |
| ACI palette correction | Keep indexed identity; no conversion to RGB using the old palette or new rendering claim. |
| Dynamic-block properties table values/fields | Existing unsupported-object/block classifications remain; no new native dynamic-block semantics. |
| Entity modification module moves/new helpers and ACIS/table APIs | No converter editing, explosion or native entity coverage is enabled. Existing type/entity classifications remain. |

Relevant open upstream issue #91 concerns future retained-record transport. It
is outside this pin adoption and does not authorize a new preservation route.
Closed #132 and #117 supply the bounded dependency fixes above.

## Spline snapshot audit

The explicit IFCCAD-independent snapshot DTO enumerates every Spline/common
field. The new upstream Serde exposure does not change this DTO. New captures
use the new codec revision with unchanged payload version 3; audited 325fa98
version-3 snapshots retain their explicit r12Metadata presence rule. Earlier
version-1/version-2 literal fixtures and revision restrictions remain unchanged.
Unknown revisions, inconsistent identity and missing required optional-presence
carriers remain rejected. Raw replay records stay excluded.

## Qualification

The local task summary records actual final verification and exact source,
lock/configuration, dependency, toolchain and environment identities. Required
checks include focused source/preservation/Dimension controls, the production
Hatch angle tests, completed original foundation DXF/DWG cells, the full Rust gate, Explorer tests
and bounded packaged WASM readback. The user explicitly stopped the repeated
external matrix after 19 passing cells, before the native upright extension;
this is partial evidence, not a complete
24-cell qualification of this candidate. Focused upright cases cover the
AC1024/AC1027 DXF boundary and DWG AC1024.
Missing or failed evidence is not a passing qualification. No size/exchange
measurement or Open CAD Studio runtime update is part of this adoption.
