# CAD layout reference correction

The IFCCAD converter resolves `ACAD_LAYOUT` from the root named dictionary,
then validates its identity/ownership and every layout's name/owner membership.
The pinned DXF reader can leave `header.acad_layout_dict_handle` at its bootstrap
value even when source dictionaries use different handles. A null, missing or
wrong-type derived cache gets a located Recovery diagnostic only when the named
relationship is unique and structurally consistent. Conflicting existing
Dictionary targets and actual source contradictions remain errors. The caller's
CadDocument is never changed; the resolved role also governs residual comparison.

The converter separately checks Model VPORT table references and Paper VIEWPORT
entity references. Model selection is not supported native state, so Allow keeps
the supported drawing with a located loss diagnostic and Reject refuses that
loss. Missing/wrong-kind references and Paper targets from another owner remain
fatal under both policies. Individual VPORT parameters retain their existing
loss classification. This distinction matches the independently decoded Model
`*Active` reference in sample_AC1032; see also Autodesk's explanation of
[Model space viewports](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-Core/files/GUID-3E43911D-0A0F-4900-BE32-5EF846AF36D8.htm).

## Practice reproduction, 2026-10-05

The five original files below now convert under Allow and pass the production
strict native reader. This is supported-subset conversion, not complete drawing
or rendering fidelity. No size/performance measurement was run. Reference tests
also cover relocated-dictionary DXF and Model-VPORT DWG exchange, logical/encoded
agreement, source immutability and invalid references under both policies.

| Original | SHA-256 | Model entities | Local definitions | Paper layouts |
| --- | --- | ---: | ---: | ---: |
| 3bm funderingsherstel DXF | `054317ae053be20d9c0f9c8650b5070f76826a7eede9690cd9c572700ea6149c` | 1070 | 127 | 2 |
| 3bm funderingsherstel DWG | `24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5` | 1070 | 127 | 2 |
| sample_AC1032.dwg | `0e8faaca949c9429c92240082d9ea4d3524aca5baf7c7815ec439245c321b528` | 54 | 7 | 3 |
| sample_AC1032_ascii.dxf | `c2934f616ecd7097e773b85e5d6e0548d21ccc635abcc8e7276f7d4a585a7ae2` | 54 | 7 | 3 |
| sample_AC1032_binary.dxf | `e2c59abecc7bc0fd0ace7b9b6f083d2a16d36077b7a7d7ceccd2aa7edfb348e1` | 54 | 7 | 3 |

The codec base is `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`, using the existing
explicit three-patch development configuration in
[patch provenance](../../patches/opencadcodec-viewports/README.md). No codec pin
or patch was changed for this correction. Patched viewport results do not
qualify the unmodified upstream dependency. Exact original filename for the
3bm pair: `2705_model Funderingsherstel - Constructie - Sheet - CP-21 - Constructietekening`.
Unsupported geometry, metadata, Model view selection and plot state retain
located loss evidence. Paper media, overall-viewport identity and plot/layout
state expansion remain independent follow-ups.

Verification on main basis `c26ce87`, after adapting to the IFCCAD crate/API names:
`cargo fmt --all -- --check`, strict workspace Clippy and `cargo test --workspace`
pass offline with the explicit codec development configuration. The workspace
reports 496 passed, zero failed and one existing ignored test, including the
seven new layout-reference regression groups. Four positive regression tests
were observed failing with the original structural errors before the repair.
Negative tests cover both policies and logical/encoded routes. Review was
performed inline under the repository's no-delegation convention.

All five files were additionally checked through the updated production viewer
adapter used by the browser route: no conversion failure and strict validation
available. This is host-side adapter verification; a new browser WASM build or
site deployment was not performed for this correction.
