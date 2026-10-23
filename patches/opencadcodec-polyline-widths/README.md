# Explicit polyline-width codec patch

Base: `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32`, upstream package `acadrust`
0.6.6, dependency/import alias `opencadcodec`. This patch is part of the native
polyline-width slice. It does not apply the historical Dimension or Hatch-angle
repairs: those fixes are already upstream in this base.

Apply `0001-preserve-polyline-widths.patch` to a clean checkout of that base with
`git apply`. The CI codec action applies it before selecting the existing Cargo
patch template. Public `Vertex2D.start_width`/`end_width` are independent Options;
`Polyline2D::effective_vertex_widths` resolves only None against its matching
header default. Explicit zero and equal-to-default overrides remain explicit.
`DxfWriter::with_polyline2d_write_mode(PreserveClassic)` keeps classic children
and presence; default LightweightWhenEligible materializes inherited widths.
Its handle budget uses the same policy as the section writer. Constant lightweight
output emits 43 without competing 40/41 and rejects conflicting authored controls.
DXF signed-zero defaults/constant/overrides remain explicit. Native DWG BD numeric
fields have no invented DXF absence flags: inherited leaves materialize and need
separate IFCCAD target-loss reporting. This is not dependency-preserving DWG proof.

Focused tests: `polyline_width_defaults`, `polyline_width_exchange`,
`semantic_inventory`, `dxf_r12_writer` and `issue63`. They include literal ASCII
source, independent ASCII/binary tag inspection, block child/owner handles,
post-read default edits, signed zero, DWG materialization and actual-version
promotion. Binary AC1015/AC1018 remain promoted to AC1024; promoted files do not
qualify the requested version.

Standalone verification on Rust 1.98.0 finds upstream limitations: strict Clippy
reports pre-existing violations; the full suite fails in
`dynamic_action_codecs::properties_table_numeric_rows_roundtrip_in_dxf_and_dwg`.
That test also fails on pristine bd10efc with the same lock/toolchain. These are
not claimed as passing gates. The standalone formatter also reports pre-existing
formatting differences; unrelated format-only edits were excluded from this patch. Task-local logs retain the exact results; no broad
codec cleanup or lint suppression is included. A nested standalone codec test
checkout may need an empty local `[workspace]`; that scaffold is not in this patch.
