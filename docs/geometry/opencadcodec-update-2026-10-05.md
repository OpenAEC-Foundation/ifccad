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
normalizes the mapped bit out of deferred workspace-loss comparison. At the initial OCDraw integration, the IFCX-CAD
route retained its earlier coverage and authored viewports were not yet supported. Its canonical unsupported-entity/serde
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

## IFCX-CAD viewport follow-up and rebase

The independent IFCX-CAD slice was rebased onto OCDraw clip main `d94d28b`.
Its native scope now covers Paper viewport camera, perspective, depth/display
state, frozen layers and circle/closed straight-polyline clipping. It still has
no ellipse/bulged-polyline native geometry, so the broader OCDraw clip contract
does not imply those IFCX-CAD capabilities. Import/export coverage contracts
record the direction-specific whole/partial loss and reference-resolution rules.

The angle repair has since been published separately as
[PR #89](https://github.com/HakanSeven12/opencadcodec/pull/89), commit
`c5ac46a33ed8b38e0be22a4853807fc8ed3d70aa`, directly based on `fe69506`.
Its independent literal DXF import and direct wire-value export tests both fail
on the base and pass after the fix. Upstream library/integration tests report
1,623 passed; doctests report 12 passed and 35 ignored. Existing upstream global
formatting and strict Clippy diagnostics remain disclosed in the PR.

A third explicit local repair preserves viewport-off bit 0x20000 and reflects
its effective value in DXF group 68. It adds serde-defaulted
`ViewportStatusFlags.is_off`; it is not included in PR #88 or #89. The shared
CI preparation now applies all three patches. Their ordered `git apply --check`
and actual application were verified against a clean checkout of `fe69506`,
without changing the Cargo cache. The manifests continue identifying that base;
local opt-in and CI select the repaired checkout. New field treatment remains
explicit: IFCX-CAD maps effective enabled state, OCDraw diagnoses its unmapped
viewport workspace state through typed residual comparison.

The camera reference uses hand-authored DXF and independently specified landmarks,
including off-axis/twisted perspective and signed clip planes. This is calculation
reference evidence, not an AutoCAD-produced drawing/rendering comparison. Native
examples and candidate conformance files pass the production strict reader.
Source-aware conversion also rejects viewport numeric projection loss under
both policies and normalizes frozen-layer set order without false loss reports.
Historical measurements retain their original coverage and dependency provenance;
no size or performance measurement was run for this slice.

Final verification after rebase, with the documented three-patch local checkout:
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo test --workspace` pass. The workspace reports 489 passed, zero failed
and one existing ignored test, including 13 doctests. A separate
`cargo test --doc --workspace` reports those 13 passing doctests. All Rust checks
used cached dependencies offline. The release browser WASM build and production
smoke pass both OCDraw and IFCX-CAD routes, including the new IFCX viewport cases.
The browser packaging tools needed normal tool-cache access outside the restricted
sandbox; no deployment or publication was performed. Review was performed inline
under the user's no-delegation convention. The implementation is ready for integration; upstream adoption of the three repairs remains a separate follow-up.
