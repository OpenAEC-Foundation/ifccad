# opencadcodec dependency review, 2026-10-05

Both converter manifests pin unmodified opencadcodec 0.5.5 at
`fe69506cb99dea6f4c4a73b690a27fdf04403ea0`, replacing
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`. This base and its public-model audit were adopted from the local
IFCX-CAD worktree on 2026-10-05. Local Cargo.lock reflects the selected
upstream or patched configuration and remains ignored.
The [previous audit](opencadcodec-update-2026-10-02.md) and its measurements
retain their original dependency provenance.

The [upstream comparison](https://github.com/HakanSeven12/opencadcodec/compare/d96e3fa2fe5acbeac966f1db4c01142618bf9c79...fe69506cb99dea6f4c4a73b690a27fdf04403ea0)
contains ten commits and 21 changed files. Both directional coverage contracts
were reviewed against changed public declarations and the existing source scans.
There are no changes to HeaderVariables, EntityCommon, Layout, ordinary
supported geometry, Viewport or the semantic-inventory categories.

| Changed surface | Treatment in both converters |
| --- | --- |
| EmbeddedEntity::Body, carrying a polyline profile's modeler data | Embedded construction history remains inside unsupported solid/surface entity and history object families. It is not an ordinary supported polyline. |
| AssocPersistentSubentId class_code, values and leading_flag; AssocEdgeActionParam curve and AssocCurveValue | Authored associative objects remain diagnosed by the canonical non-scaffold object scan. No associative relationship or edge curve is flattened into native geometry. |
| Surface and solid-history transform matrix interpretation | Column-major public matrices belong to unsupported surface/history families. Supported instance placement is unchanged. |
| FIELD evaluation, table formulas, attribute-field attachment, plot stamping and format constants | Both converters diagnose canonical FIELD objects and unsupported attributed INSERTs/tables. They do not call the new evaluation or stamping helpers. |
| ACIS wire and spline encoding; associative surface/modeler serialization | Unsupported entity/object boundaries remain unchanged; these codecs add no native support. |
| Pre-R2013 DWG writing from DXF input and R2018 auxiliary-header maintenance fields | Physical codec corrections; existing conversion semantics are unchanged. AC1032 exchange tests exercise the R2018 route. |

## Authorized OCDraw development adoption

The user authorized taking both local codec repairs from `ifcx-cad-paperspace`
on 2026-10-05. [Patch provenance and setup](../../patches/opencadcodec-viewports/README.md)
record the exact base, clipping production diff from PR #88 and separate DXF
angle-unit diff. The repairs remain byte-identical to the selected local patches.
Local checkouts opt in through an ignored `.cargo/config.toml`; default
converter manifests still identify the unmodified upstream base. The authorized
main integration also prepares this exact base and both patches for automatic
Rust checks and the browser build, with the patch recipe included in cache keys.
Neither the Cargo cache nor the original IFCX-CAD worktree was changed.

The additional public `ViewportStatusFlags.non_rectangular_clipping` field
represents activation independently of the existing boundary handle. OCDraw
maps status bit 0x10000 in both directions, preserves dormant references and
normalizes the mapped bit out of deferred workspace-loss comparison. The IFCX-CAD
route in this checkout retains its existing coverage; authored viewports remain
outside that route's supported set. Its canonical unsupported-entity/serde
residual checks remain applicable to the new status field.

The DXF angle repair only changes physical groups 50/51 between degrees and
public-model radians. Neither native logical contract changes. Camera twist is
verified through both CAD file chains; literal degree values independently
qualify the reader and writer. Per-authored-viewport workspace snap state still
has its existing source/target loss treatment.

Unmodified base still loses clipping activation and DXF references and treats
VIEWPORT DXF angles as radians. The focused codec tests demonstrate those
failures; patched tests must never be attributed to the unmodified dependency.
Modern DWG without an overall paper canvas still reclassifies the first authored
viewport as overall; the clipping roundtrip profile includes that canvas.

## Verification

Focused red/green tests qualify each repair separately and then the OCDraw
mapping: bit/reference loss on the base, degree-angle failures with only the
clipping patch, codec success with both, and mapping failures until independent
activation/dormant references are carried through ordered construction.
Final local-patch verification: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` all passed with cached dependencies (offline);
436 tests passed, zero failed, including documentation tests. These results
qualify the exact base plus both local patches, not the unmodified base.
The release browser WASM build and its production smoke checks passed, including
active/dormant DXF clips, forward boundary references and nonzero camera twist.
The website suite passed 31 tests; its four Linux deployment-recovery tests were
skipped on Windows. The workflow's patch-preparation script was also exercised
against a clean checkout of the pinned base and produced the same three-file diff.
No size/exchange measurements were run.
