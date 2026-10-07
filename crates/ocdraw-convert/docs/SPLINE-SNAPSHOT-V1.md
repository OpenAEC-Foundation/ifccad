# OCDraw typed spline snapshots

This is the first bounded preservation adapter. It stores every interpreted
`EntityType::Spline` offered by the pinned CadDocument model. Capture is not a
native NURBS profile, an evaluated-geometry proof or an input fixture whitelist.
OCDraw and IFCCAD remain separate; this adapter implements only OCDraw.

## Carrier and compatibility

Provider: `opencadcodec`; payload schema: `openaec.opencadcodec.spline`; version:
`2` for new capture; `1` remains supported for the previous revision. Kind: `adapterSnapshot`; category/role/representation:
`entity` / `complete` / `codecTyped`.

Current audited codec revision: `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
The previous `fe69506cb99dea6f4c4a73b690a27fdf04403ea0` revision remains accepted
for version-1 snapshots. Version 2 adds explicit nullable `dwgScenario` and
`common.layerHandle`; version 1 lacks those fields and restores them as None,
without inventing missing source state. The provider revision, payload version
and body revision must agree. Current capture always emits version 2.
See the [dependency audit](../../../docs/geometry/opencadcodec-update-2026-10-07.md).
Only the documented viewport-off repair remains selected; clipping and angle
repairs have merged upstream. PR #99's fit-spline DXF repair is not in this pin.

`SplineSnapshot` and `SplineCommonSnapshot` are private adapter types, independent
of core storage. Version belongs to the persisted format, not the Rust names.
Adding an audited compatible codec revision is an explicit audit. Incompatible
carriers require a new payload version. Adopting a future upstream snapshot API
must retain v1 read support or supply a verified migration. Upstream #91 does not
block this pilot; other providers and raw/private/shared storage remain follow-up.

The payload is a closed UTF8 JSON record with codecRevision, sourceHandle,
sourceOwnerHandle, sourceOrderIndex, flags, all spline parameter fields and common.
SourceHandle/sourceOwnerHandle must agree with the common fields and record
sourceKey. SourceOrderIndex is the original zero-based position in the owner's
authoritative list. Original handles and this index are provenance; current
logical ownership/order and newly constructed CAD handles are authoritative.

Every binary64 value is encoded as exactly 16 lowercase hexadecimal `to_bits()`
digits and reconstructed by `from_bits()`. XYZ vectors are ordered arrays of
exactly three such values. Integers retain source ranges; handles are canonical
lowercase hex without a prefix; byte vectors are canonical padded standard
Base64. None is explicit null; Some(empty) is a present empty value. All common
source fields are required even when their value is null. Source enum variant
names and unit/tuple/named variant shapes are retained; transparency uses its
public Rust variant names rather than upstream's differently renamed Serde tags.

Negative/zero/unusual degree, fit/control/mixed data, dormant flags, inconsistent
arrays, nonfinite values and NaN payloads are transported without repair. Decode
checks carrier shape, integer ranges, revision and identity; it performs no knot,
weight, planarity, normal, NURBS or finite-geometry admission test.

## Exhaustive field audit

Source projections destructure the pinned Spline/SplineFlags/EntityCommon types
without `..`; public enum matches are exhaustive. Private ExtendedData records
are read through `records()` and rebuilt with `add_record()` to retain order,
including duplicate application records. Ordinary upstream Serde is insufficient
because it omits retained common fields and raw EED.

| Source fields | Snapshot | Native / restore boundary |
| --- | --- | --- |
| degree | i32, exact | No native spline/evaluator; restored without clamping. |
| flags.closed/periodic/rational/planar/linear | Separate booleans | No geometric admission based on flags. |
| knots, weights | Ordered binary64 bits | No regeneration, sorting or normalization. |
| control_points, fit_points | Ordered XYZ bit vectors | Includes fit-only, mixed, empty and inconsistent arrays. |
| normal, begin_tangent, end_tangent | XYZ bit vectors | No normal/tangent normalization. |
| knot_tolerance, control_tolerance, fit_tolerance | Binary64 bits | No new tolerance policy. |
| knot_parameterization, cv_frame_visible, dwg_flags1, dxf_flags | Exact source scalar ranges | Dormant/full flag words retained; physical codec may normalize them as in direct writing. |
| dwg_scenario | Explicit nullable i32 in v2; unavailable in v1 | Retains which DWG storage scenario supplied the parameters; no invented closure/periodicity knowledge. |
| handle, owner_handle | Hex provenance | Fresh handle and mapped owner; target entity_mode follows actual owning scope. |
| layer | Complete source name | Optional native layer ID. Current native reference wins; otherwise source binding must resolve. |
| layer_handle | Explicit nullable source handle in v2; unavailable in v1 | Qualify the source name/handle association. An unresolved handle does not become native layer 0; current native layers reconstruct fresh handles. |
| color, color_name | Exact enum/optional metadata | Complete native appearance only when exactly expressible; no invented color/mode. |
| line_weight | Exact enum/i16 | `Default` has no exact native mode and leaves the whole appearance opaque. |
| linetype, linetype_handle | Source name/optional handle | Native pattern selection wins. Source fallback uses a qualified pattern binding; numeric source handles are not copied. |
| linetype_scale | Binary64 bits | Native appearance requires positive finite scale; otherwise preserve source value. |
| transparency | Exact source mode/u8 | All 256 byte-derived native opacity values restore exactly; arbitrary native edits use existing CAD quantization. |
| invisible | Bool | Current native visible bool is authoritative. |
| extended_data.records | Application names, ordered typed variants | Known typed reference/numeric/vector profile only; app registration must exist in constructed CAD. |
| extended_data.raw_dwg_eed | Ordered app handle/byte tuples | Bytes retained; no safe raw reference replay in fresh namespace. |
| graphic_data | Optional bytes | Stored; unqualified graphic/proxy context prevents restoration. |
| reactors, xdictionary_handle | Complete handle carriers | Stored; attached object context has no new restoring provider. |
| color_book_handle, full_visual_style_handle, face_visual_style_handle, edge_visual_style_handle | Optional handles | Stored; unqualified object context prevents restoration. |
| material_flags, material_handle, shadow_flags, plotstyle_flags, plotstyle_handle | Source flags/optional handles | Stored; nondefault unqualified context prevents restoration. |
| entity_mode | Optional source u8 | Retained as provenance; derive Model=2, Paper=1, Block=0 on restoration. |
| has_ds_data | Bool | Stored; shared data-store restoration is outside this adapter. |
| raw_record | Not included | `storageSupplementOmitted`; never reattach to fresh handles/owners. |

This does not retain private original-object maps/baselines or shared AcDs
storage, and is not a complete codec-state snapshot. An omitted storage
supplement is separate evidence within the bounded interpreted-data guarantee.

## Native representation, scopes and policies

`CadToOcdrawOptions::preservation_capture` defaults to Disabled. SupportedTyped
captures every spline before native representability/restore checks. ModelSpace,
PaperSpace and supported local definitions receive live opaque entities with
mixed original order. Nested/repeated occurrences and unused definitions are
retained. Unsupported but structurally valid owners receive a detached snapshot
and a located placement loss; corrupt owners/membership/references remain fatal.

Opaque geometry makes complete bounds unavailable in its scope and in scopes
reaching it through block instances. Bounds are null; native numerical checks
continue. No native spline bbox, preview geometry or control-point enclosure is
certified. Native-only geometry assessment remains distinct from unassessed
opaque definition/occurrence sources and `is_complete()`. This completeness
concerns the transferred drawing, not source content previously omitted under
ordinary loss handling.

Native layer/appearance may be absent, meaning source-payload state, not defaults.
Current native layer/appearance/visibility, layer names/defaults, pattern edits,
global/entity scale and current order are authoritative. They do not invalidate
spline parameters merely because they differ from capture. Baselines are not
mutated after native edits. Moving the opaque entity to another scope or changing
drawing coordinate unit remains conservatively refused; reverting those changes
can restore eligibility. Layout media units do not rescale drawing coordinates.

Capture Reject accepts fully stored interpreted spline content; lack of native
rendering or unqualified future writing is not itself lost captured data. Actual
omitted table/object/header/placement content continues blocking Reject. Export
RestoreSupported is the default; Skip explicitly drops live opaque content and
is allowed only under Allow. Allow may skip an ineligible live record with loss
and preservation evidence; Reject refuses the drop. Detached entities, absent
supplement subjects and unreferenced shared helpers have no drawable obligation.
Relevant unsupported complete non-entity/supplement roots are reported, not
silently ignored. Required record groups/cycles have no restoring engine here.

## Predicates and references

Every live spline requires v1 `openaec.ocdraw.splineEntityBinding` on its entity,
with closed `{preservationRecordId, ownerScopeId}` baseline, and v1
`openaec.ocdraw.splineCoordinateContext` on its owning scope with closed
`{kind, coordinateUnit}` baseline. Kind is Model/Paper/Block; coordinateUnit is
the drawing unit for every scope. Bounds, membership/order, layout name, block
base point, instance transforms and global pattern scale are not geometric guards.

Named bindings retain original source keys and native targets. Source-reference
conditions use v1 `openaec.ocdraw.sourceReferenceBinding` with closed
`{slot, sourceKey, targetRole}` baseline. Source common layer/pattern bindings
serve as provenance/fallback; a current native property supersedes them.
Required XDATA Handle/LayerName slots are checked from actual payload data,
including mandatory condition presence/coherence, typed domains, source keys
where offered, and actual target construction. Unknown predicates, missing
requirements and deliberately incomplete claims cannot be made safe by setting
dependencyCoverage to qualified. Extra unsupported Record dependencies are
refused rather than guessed or recursively restored.

Known XDATA handles and layer names are rebound after combined native/opaque
construction, including forward references. Missing/skipped targets propagate
to dependent restored entities. This profile requires a constructed application
registration; it does not synthesize missing APPID/table/object semantics.
Arbitrary String/ControlString/BinaryData application context and raw EED/graphic
bytes may embed unknown references and remain unqualified. Attached dictionary,
reactor/style/material/plot-style/shared context is retained but not reconstructed
by a new provider. Active clipping cannot use opaque geometry; dormant same-paper
unique references may. A skipped required clip target produces a construction
failure, never a dangling handle or rectangular placeholder.

Reports separate capture, native-property capability, restoration and physical
CAD exchange. Captured/restored typed data does not establish rendered,
byte-identical, whole-file lossless or all-variant CAD fidelity. Logical/encoded
wrappers, LossRejected errors and inspection summaries retain this evidence;
physical writer/readback failures remain failures and add targetCodecRejected
evidence for the affected output obligation.

## Verification scope

The independently hand-authored open cubic DXF checks literal source values.
Storage tests cover unusual/nonfinite bits, all source/common carriers including
Serde-skipped fields, exact IDs, canonical bytes, unknown providers/predicates,
soft references and shape/link failures. All 256 transparency bytes and opaque
Default lineweight receive dedicated in-memory restoration checks.

Actual production DXF and AC1032 DWG comparisons exercise an ordinary writable
cubic in ModelSpace, two Paper layouts, local/unused definitions, mixed
native/spline order and repeated/nested occurrences. Additional quadratic,
rational, closed/periodic, spatial, fit-only and mixed-control/fit fixtures each
passed direct-vs-preservation reader/writer comparisons in both formats. That
matrix qualifies those tests under the recorded dependency configuration; it
does not define admission rules or certify all later variants/revisions.

Both chains use the same explicit target version and physical configuration:
direct CadDocument -> writer -> production reader, versus CadDocument -> OCDraw
encode -> production read -> fresh CadDocument -> same writer/reader. Compare
parameter/common semantics and owner-relative order, resolving Paper owners by
layout identity rather than codec-reserved block names. Original numeric handles,
redundant entity_mode scaffolding, source spelling rebound to current names and
whole-file bytes are not equality oracles. Native edits and safe typed reference
rewrites have independent tests. Additional direct codec refusals are reported
as codec restrictions without invalidating stored snapshots.

The browser WASM smoke fixture separately tests explicit capture, native save/open,
DXF/DWG export and production readback. Required Rust gates and inspector tests
are correctness checks; no size/performance measurements or compression probes
are part of this slice.

The native-edit exchange target additionally covers the full capture/read -> native
edit -> encode/read -> fresh CAD -> actual DXF/AC1032 DWG chain. Its twelve positive
scenarios cover layer reassignment/defaults, pattern definition/rename/global scale,
color/opacity/weight/visibility/entity scale, native appearance supplied after
absence, unrelated line geometry, block base point/instance transform, Paper name
and order, and deletion with layer reassignment. Every scenario includes Model,
Paper, local/unused definitions and nested instances, unchanged source payloads and
baselines, and independent direct-codec parameter comparison. Delete/clear refusal,
fourteen condition mutations after save/reopen and unit change/reversion are also
covered. A real filesystem save/close/reopen test completes DWG readback.
