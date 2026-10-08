# opencadcodec update — 2026-10-08

The user requested the merged codec fixes during the workspace slice. Both
converters and the shared CAD companions now pin opencadcodec 0.6.0 at
`ab2eecdbffc31120b5ad6d899f6fc67cf21ede39`, replacing the 2026-10-07 base
`063c10671fe7833d562f772159771318c7a0ebb9`.

## Public-model audit

| Change | OCDraw treatment | IFCCAD treatment |
| --- | --- | --- |
| SPLINE knot parameterization, CV-frame visibility and DXF creation-method flags | Same typed fields and numeric parameterization meanings; merged PR #99 corrects DXF parsing/writing. Opaque typed snapshots retain the existing payload-v2 shape. | Spline remains an unsupported entity family; no native spline semantics added. |
| AssocVariable `reserved` replaced by owned `dependencies` with handle/flags | Already unsupported associative objects remain diagnosed through the semantic inventory. | Same unsupported-object boundary; dependency data is not silently accepted as native coverage. |
| Summary information DXF IO | Existing public fields remain diagnosed as summary information. | Existing summary/preview boundary remains diagnosed. |
| DWG secondary Paper ownership and overall-viewport identification | Updated ownership regressions require correct marker/drawable owners and strict readback. Overall role follows qualified owner/layout associations. | Same qualified role/ownership checks, independently adapted to IFCCAD. |
| VPORT/VIEWPORT workspace fields | Public scalar fields unchanged; shared adaptation and explicit residual classification added in the workspace slice. | Independently allocated uint64 UCS/window IDs and owned workspace attributes use the same scalar helpers. |

The only selected local repair is the independent explicit viewport-off bit
and its effect on group 68 and visibility helpers, submitted as
[PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). It is tied to
this base and remains explicit in the patch configuration and CI action.
Historical clipping/angle patches are not applied. The Cargo source cache is
not modified; patched tests do not establish unmodified-dependency fidelity.

New capture uses the new revision. Restoration recognizes audited payload-v2
snapshots from both ab2eecd and 063c106, plus legacy fe69506 payload-v1. The
provider/envelope/body revision agreement and mandatory nullable field presence
remain strict; unknown revisions remain rejected. There is no payload-version
bump or re-interpretation of typed knot parameterization values.

## Evidence and remaining limits

Secondary Paper BLOCK ownership and overall-role regressions failed on the
previous pin and pass on this base. Direct, DXF and AC1032 DWG workspace tests
compare fields and owner associations through both production native readers.
The separate upstream off-status PR has five new regressions, all 13 viewport
regressions and 1,354 library tests passing; broad upstream formatting has
pre-existing unrelated differences.

The public surface does not retain the complete positive VIEWPORT stacking
rank/current Paper selection. `paper_space_block_handle` identifies reserved
infrastructure, not a current tab selector: Paper mode alone cannot identify one
of several layouts. Multiple-window active ordering remains unqualified without
an established association. VIEWPORT GridFlags survive in CadDocument but become
default after the pinned DXF/DWG routes; VPORT GridFlags survive both. Each
nondefault VIEWPORT behavior receives a target portability diagnostic, while
runtime values remain copied. These limits are covered in
[workspace state](../workspace-state.md).

Primitive accuracy, Paper plot tolerance, source graph checks and spline
bounds/accuracy incompleteness remain independent. No controlled measurement was
run; older benchmark reports retain their recorded dependency provenance.
