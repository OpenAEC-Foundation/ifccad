# Dimension editing investigation, 29 October 2026

Two distinct failures were reproduced against the actual local Studio WASM
bundle. The original investigation changed no production code; its follow-ups
record the Document endpoint fix and OCS issue #2325. The earlier storage/exchange gates remain evidence for those boundaries;
they did not cover these interactive editing failures.

## Reproduction identities

- IFCCAD catalogue: `d0a03ed42554ed1ce9917c5d6db2803e81a5e896`,
  `examples/ifccad/entities.ifcx`, SHA256
  `db661bf46194e5969c297319baf17155e8c61fb1cfb202e739bdd42d45895974`.
- Studio base: `4d1cd17dafd7d47cd76f8674835636776eb90fd6`, with the documented
  dependency overrides and its existing development-main/release-worker bundle.
- Document: `ad6fec3f99dae5f7f0e93aca8ef5e4c98ca680ea`; codec: `501b694`.
- Host `IFCCAD-BUILD.json` SHA256:
  `fe497ea0d2e750d1052244104b30bfe78364188f09fb5a30dc99868754a16f9e`.

A disposable local test page uses Studio's documented `ocs_control_submit/take`
API via the existing Explorer control wrapper. It opens a fresh catalogue,
captures full Dimension/entity/object/block records, performs editor operations,
and captures another snapshot after each operation. The radius test also uses
a real mouse grip drag in the canvas. User drawing tabs were not edited.

## 1. Source endpoint identity collapses the linear dimension

The source line starts at `(65,210,0)` and ends at `(83,222,0)`. The native
horizontal projection is 18. Its converted DIMASSOC references have distinct
stored snap points and snap types 13 (start) and 1 (end), but both have default
`main_subent_type=0` and `main_gs_marker=0`.

| State | Line start / end | Dimension first / second | Measurement | Picture link |
| --- | --- | --- | --- | --- |
| Open | (65,210) / (83,222) | (65,210) / (83,222) | 18 | `*D1` |
| Move line by (3,2) | (68,212) / (86,224) | (68,212) / (68,212) | 0 | empty |
| Undo | (65,210) / (83,222) | (65,210) / (65,210) | 0 | empty |

Document's `src/ifcx/dimension.rs::apply_native_associations` constructs the
references with default subentity metadata. Studio's
`src/scene/dimension_assoc.rs::resolve_reference_chain` selects its legacy
marker resolver for these start/end references. `resolve_local_by_marker`
uses `source_points(line)[marker]`: both zero markers select the start point.
Undo replays the restored line through association refresh, repeating the
collapse and clearing the restored picture link.

### Isolated diagnostic comparison

Only in the disposable in-memory drawing, set the first reference to
`main_subent_type=1, main_gs_marker=0` and the second to
`main_subent_type=1, main_gs_marker=1`, matching Studio's point-marker convention.
No geometry, saved picture or production mapping is changed for this comparison.

- Moving the line by (3,2) preserves measurement 18 and distinct endpoints.
- Undo restores the exact original Linear record and its `*D1` picture link;
  the original block remains present.
- Changing the line end's X from 83 to 86 changes measurement 18 to 21.
  Undo restores 18 and the exact original Linear record/picture link.

This proves the endpoint-identity mismatch and a working Studio-side metadata
combination. A production mapping fix belongs in Document. Before adopting
that combination, qualify both endpoints through typed import and actual DXF/
DWG readback; this browser experiment alone does not establish a general CAD
subentity encoding contract. Apply the same correction to supported endpoint
roles through the shared mapping, not just this example.

## 2. Direct grip undo loses the saved picture block

Open a fresh catalogue, select Radius (`C2` in this import), and drag its curve
point grip. Take a snapshot, undo once, and take another snapshot.

| State | Radius | Picture link | `*D3` record and owned line `BF` |
| --- | --- | --- | --- |
| Open | 15 | `*D3` | present |
| Grip moved | 20.539522848752352 in this gesture | empty | absent |
| Undo | 15 | `*D3` | **absent** |
| Redo | 20.539522848752352 | empty | absent |

After undo, the **entire serialized Radius record equals its original record**,
but the referenced block and its owned line were not restored. This separates
the history failure from measurement, endpoint resolution and visual rounding.
The arbitrary dragged value is not itself an expected numeric contract.

The relevant Studio paths are:

- `src/app/update/viewport.rs` captures grip originals as edited entities;
  `commit_active_grip_edit` commits `push_entity_group_history`.
- `src/app/command_driver/utilities.rs::capture_grip_history_originals` captures
  solid-history objects, not Dimension picture records/children.
- `src/scene/modify.rs::apply_grip` calls `invalidate_dim_block_recorded`.
  Its removed-child capture requires an active undo recorder. The interactive
  grip path instead builds its entity-group history after the gesture.
- `src/modules/draw/modify/explode.rs::invalidate_dim_block` removes the
  anonymous block record, block markers and owned entities.
- `src/app/history.rs::push_entity_group_history` retains the supplied entity
  and object images, without the removed picture block/children.

Thus direct-grip history restores the Dimension's old block name while omitting
the deleted block dependency. This warrants an OCS issue and a regression test
for an imported Dimension with nonempty stored graphics. Exact restoration
must include the block record, ordered members, block markers, owned entities
and original Dimension reference. Cover redo and cancelled grips too. Do not
claim those additional cases passed in this investigation.

## Upstream status and next steps

OCS main was checked at `7c5594ce9529695f4f11c8fc95a43d710ebb37d4`.
The five relevant resolver/grip-capture/commit/invalidation functions are
byte-for-byte equal to those in the pinned source. That is source evidence;
the newer main was not built or runtime-tested. Focused searches for open
issues using `dimension undo` and `DIMASSOC` returned no matches; this is not
an exhaustive duplicate-issue search.

Recommended order:

1. Add a Document endpoint-identity regression and correct the shared mapping,
   checking actual DWG/DXF and the two demonstrated browser operations.
2. File the direct-grip history issue in OCS with the reproduction above and
   before/after database evidence. Keep its fix in Studio's history boundary.
3. Repeat the focused browser edit/undo/redo checks against the final pinned
   pair before describing editing as qualified. Native format/schema identifiers
   need no change merely to repair these runtime/mapping failures.

Full local JSON snapshots and checked assertions are retained in the existing
ignored task evidence directory: `investigation-zero-original.json`,
`investigation-endpoint-markers.json`, `investigation-length-markers.json`,
`investigation-radius-grip-undo.json`, `investigation-upstream-check.json` and
`investigation-results.json`. The test harness and check scripts are retained
there for reproduction. No full build, performance measurement, commit, push,
merge or publication was performed for this investigation.

## Subsequent authorized endpoint fix

After the investigation, the user authorized correcting the zero-measurement
mapping in the still-open Document PR #7. Commit
`4723b0e6b4b38354853eb5ce1d0f9af23a8bbb2f` was pushed to that PR. Both conversion
directions now retain supported Line vertex identities. Legacy unspecified
coordinate references remain readable, while unsupported or contradictory
marker/type/snap combinations are diagnosed.

The final portable Document fmt and all-target Clippy/full-test matrix for
default, serde and serde+ifcx passed with unchanged inputs. The regression
checks typed conversion and production ASCII DXF/binary DXF/AC1032 DWG before
and after source edits, including stale snap caches. A newly generated catalogue
DXF was opened in the existing Studio bundle: moving the source retained 18;
changing its end X by 3 produced 21, without a diagnostic metadata patch.
These are endpoint checks, not a complete interactive undo qualification.

The additional physical-DXF automation run did not revert the moved source
when undo was requested. Its history path was not isolated in this fix; do not
attribute restoration of all physical-import undo behavior to the endpoint
correction. The independently proven direct-grip picture-block loss also remains
open OCS work.

Explorer and Studio continue to pin Document `ad6fec3`; they were not rebuilt or
repinned during the Document-only correction. Fresh native IFCX opening through
that older embedded mapping still needs the matching dependency update before
users receive the fix there. Native format identifiers and dependencies did not
change in this correction. No OCS issue was submitted.

## Matching local Explorer update

The next user instruction authorized rebuilding local Explorer and Studio with
Document `4723b0e`. Both are now updated. A fresh native IFCX import passes the
move/undo/lengthen/undo/redo sequence, including exact original Linear record
and picture restoration. A recovery dialog from an earlier test drawing was
observed blocking one automated undo attempt; closing it allowed the sequence
to pass. The earlier unisolated physical-import undo observation above is not
evidence of an additional independent history bug. The direct mouse-grip test
that deletes the picture block remains a separately reproduced failure.

## Source-line midpoint grip: separate history omission

The user's subsequent check used the source line's **middle grip**, rather
than the previously passing `entities_transform` move operation. This was
reproduced with the updated Document `4723b0e` Studio bundle and a fresh native
catalogue. Select only the blue source line, drag its midpoint, finish the
gesture, then use the normal Ctrl+Z shortcut. No recovery dialog was present.

| Stage | Line / Dimension points | Measurement | Picture link | `*D1` block |
| --- | --- | --- | --- | --- |
| Before | (65,210) to (83,222) | 18 | `*D1` | present |
| Midpoint drag | both translated by approximately (5.773,3.802) | 18 | empty | present |
| Ctrl+Z | original coordinates restored | 18 | **empty** | present |

This is not a recurrence of the endpoint collapse: both endpoints stay distinct
and measurement stays 18. The source line, both Dimension measurement points
and its dimension-line position return to their original values, but its
original picture reference does not. The renderer continues with generated
graphics instead of the saved picture.

Unlike the direct Radius grip case, the source-line grip leaves the picture
block itself intact. `refresh_associative_dimensions` clears the dependent
Dimension's block name, while the interactive grip group captures the source
line rather than the dependent Dimension's original state. Its capture of
Dimension before-images is conditional on an active undo recorder; the grip
path instead commits its entity group after the gesture. Consequently undo
re-resolves the source coordinates without restoring the original picture link.

The OCS issue should cover both missing dependent-Dimension before-images for
source grips and missing deleted picture dependencies for direct Dimension
grips. Their acceptance tests must exercise actual mouse grips, not substitute
only a transform command. The earlier successful command/API move/undo result
remains bounded to that operation path. Local evidence:
`source-midpoint-grip-undo.json` and `source-midpoint-grip-after-undo.png`.
This follow-up changes documentation/diagnostic instrumentation only.

At the user's explicit request, both grip/undo cases were reported together in
[OpenCADStudio issue #2325](https://github.com/HakanSeven12/OpenCADStudio/issues/2325).
The published report includes the public reproduction file, exact tested
dependency revisions, mouse-grip steps, expected/actual database states and
the relevant history code paths. No OCS fix has been applied yet.

## Direct DWG/DXF opening confirmed

At the user's request, both actual mouse-grip paths were also tested after
opening the exported catalogue directly as AC1032 DWG and ASCII DXF. All four
cases reproduce, with Studio `4d1cd17` and Document `4723b0e`. These runs use
the physical CAD readers, not IFCX import. The test files were previously
exported from the public native catalogue using the corrected Document mapping.

For both formats, a direct Radius grip followed by Ctrl+Z restores the entire
original Radius record but leaves its picture block deleted. A source-line
midpoint grip followed by Ctrl+Z restores the source line and all Dimension
data except the empty picture link, while retaining its original block and
measurement 18. Checks against complete before/after records passed for all
four observations. Startup/recovery dialogs were dismissed before gestures.

Evidence: `cad-{dxf,dwg}-{radius,source}-grip.json` and
`cad-grip-results.json`, with input hashes. The local reproduction package
`ocs-2325-dwg-dxf-repro.zip` contains both files, steps and provenance.
The issue's short body now records the four confirmed CAD cases. No production
code, build configuration or dependency revision was changed for these tests.
