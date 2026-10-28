# Remaining Dimension kinds: design decisions

Recorded on 26 October; updated on **28 October 2026**. **Design only; not implemented.**
The implementation plan requires review before execution. This document records the design discussion, not
newly supported schema, Rust API or CAD exchange coverage.

## Intent and scope

Extend the existing ordinary Dimension family with all seven remaining kinds:
radius, diameter, two-line angular, three-point angular, ordinate, arc length
and jogged radius. Keep measurement definitions, optional source associations,
presentation and optional stored graphics distinct.

The seven kinds form **one future implementation task**, with shared model,
validation, storage, inspection and test infrastructure. Radius/diameter,
angular kinds and the remaining kinds can be internal implementation steps.
Each step receives focused checks; the final combined candidate receives the
applicable full verification. This is an execution direction, not a schedule
or authorization to start implementation.

The active conversion route belongs to **opencaddocument**, through viewer/browser.
The direct converter and six CAD companions are archived and remain outside this
work. Implement native semantics in the independent core and both conversion
directions in Document against explicit development revisions. The published
alpha, deployed Explorer and embedded Studio pins are separate baselines.

## Approaches considered

| Approach | Trade-off and decision |
| --- | --- |
| Seven separate implementation tasks | Earlier individual handoffs, but repeats discovery, shared changes and final verification. Not selected. |
| One task with internal stages | Reuses shared preparation and verification while retaining focused checks per kind. Selected. |
| First build a general entity generator or association framework | Adds an independent project before coverage improves. Not selected; reuse concrete helpers where justified. |

For source relationships, extend typed measurement roles and bounded selectors.
A mandatory whole-curve association is not the common basis. Automatic updates
and a general dependency engine are separate work.

## Existing contracts carried forward

- Ordinary Dimension identity, Model/Paper/block ownership, draw order, local
  placement, styles, kind-specific settings, sparse overrides, labels and text
  placement continue to use the existing architecture.
- Preserve authored absence, explicit zero/false and equal-to-parent resets.
  Loading does not invent styles, references or semantic defaults from CAD
  runtime behavior.
- Measure the stored definition. Reading, writing, formatting, inspecting or
  converting does not refresh it from associated geometry.
- Follow the current spatial reference-anchor contract: preserve local XYZ
  reference data and distinguish planar measurement from spatial coordinates.
  Do not revive the older blanket assumption that all points are XY. The exact
  fields carrying independent reference heights must be specified in the native
  contract before implementation; source values cannot be silently flattened.
- Measurement is independent of presentation scale and text placement. Existing
  numeric formatting and distance-factor versus angular-format semantics remain
  shared. Display, font evaluation and numerical evidence are separate claims.
- Optional graphics retain the distinction between absent, present empty and
  present nonempty. Reuse the current bounded graphics contract; add no graphics
  generator, freshness engine or automatic association update.

Current schemas/conformance and current Rust code/tests remain authoritative for
implemented behavior. Historical designs that assume only XY anchors, no stored
graphics, seven independent tasks, or mandatory whole-curve relationships do not
override this record or the current implementation.

## Definitions and measurement

The names below describe logical meaning, not final Rust names or physical
field codes. Positions are expressed in the Dimension's local placement;
measurements use its defined planar geometry. Final field shapes must retain
the spatial reference-anchor distinction above.

| Kind | Stored meaning | Measurement |
| --- | --- | --- |
| Radius | Center, curve point and leader length | Planar center-to-curve-point distance. |
| Diameter | Two opposite diameter points and leader length | Planar distance between the points; center is their midpoint. |
| Two-line angular | Two lines defined by endpoint pairs, selected ray on each line, traversal and dimension-arc position | Selected directed sector between the supporting lines. |
| Three-point angular | Vertex, one point on each leg, traversal and dimension-arc position | Selected directed sector between the legs. |
| Ordinate | Origin, feature point, measurement-axis direction, absolute/signed mode and leader endpoint | Projection of feature minus origin onto the stored axis, with the selected value mode. |
| Arc length | Center, radius, start parameter, directed partial sweep or explicit full turn, dimension-arc position and optional leader | Radius times absolute angular span in radians. |
| Jogged radius | True center, curve point, displayed replacement center, jog point and jog angle | Planar true-center-to-curve-point distance; the replacement center does not change the radius. |

Horizontal/vertical dimensions remain linear dimensions with an appropriate
direction, not new kinds. Ordinate direction and origin are stored explicitly;
an ambient or current UCS does not change an already stored measurement.

Radius, diameter and jogged radius require a positive measurement. Ordinate
may measure zero. Scalars must be finite, radii positive and leader lengths
nonnegative. Overflow or unavailable evaluation is an explicit failure, not zero.

For two-line angular dimensions, the projected supporting lines must have a
unique finite intersection. Parallel/coincident lines are outside that kind.
Three-point angular dimensions require nonzero projected legs and can represent
a straight angle. Select the rays and traversal explicitly: moving the drawn
dimension arc must not silently choose another sector. Angular sectors are
nonempty and less than a full turn; a native major sector is not a claim that
every CAD target supports it.

## Associations

Associations are **optional per measurement role**, including independently
absent roles. A standalone dimension remains valid without a source entity.
Each relation identifies its role, a source entity and a typed eligible
selection. References must resolve within the same drawing and owner, using
native identities rather than CAD handles. A role has at most one relation.

| Kind | Measurement roles available for association |
| --- | --- |
| Radius | Center and curve point. |
| Diameter | First and second diameter point. |
| Two-line angular | Start/end of each of the two defining lines. |
| Three-point angular | Vertex and the two leg points. |
| Ordinate | Origin and feature point. |
| Arc length | Center and the begin/end selections of the explicitly stored measured interval. |
| Jogged radius | True center and curve point. |

Presentation controls such as text position, dimension-arc position, leader
endpoint, displayed replacement center and jog point do not acquire source
associations in this slice. Additional explicit CAD relationships to such points
must be accounted for by the converter, not silently ignored.

Use existing selectors where their meaning is sufficient: point position, line
start/end, circle/arc center and eligible polyline vertices. A specific straight
or circular polyline segment must be identified explicitly if supported; a
vertex index, geometric coincidence or a CAD graphics marker alone does not
establish the intended segment or stable identity through edits.

New bounded selections needed for circular curve points require a written
language-neutral parameter/domain contract before they are implemented. This
record does not select generic nearest-point, intersection, tangent or
perpendicular evaluation. Exact selector names and encoding are follow-up
contract details, not permission to infer missing relationships.

Association integrity is separate from geometric freshness. An eligible,
resolvable association can remain valid while its stored definition is stale.
Ordinary IO does not check equality with the current source, update points,
change the selected sector or silently detach native references.

On import, preserve explicit qualified source relations. If none exist, create
none. Matching coordinates are not evidence of association. On native creation,
the authoring application chooses which explicit relations to supply. Unsupported
relations require located loss reporting or refusal under the selected strict
policy; preserve an independently valid definition where the route permits it.

## Arc-length interval decision

The measured interval belongs to the dimension, even when there is an associated
source arc. Distinguish:

- the entire source arc, which might span only 90 degrees;
- a selected portion of that source arc;
- a full 360-degree circle.

Store a start parameter and a directed partial sweep, or an explicit full turn
with direction. A partial sweep is nonzero and strictly less than one turn in
magnitude, using the active circular-primitive boundary. Do not normalize a
major sweep to the shortest route, collapse a near-full sweep into a full turn,
or interpret equal endpoint coordinates as proof of a full circle.

For a source arc spanning 0 to 180 degrees, the dimension can independently
measure 30 to 90 degrees. Its optional begin/end source selections must describe
their actual source meaning; they do not instruct the loader to replace the
stored interval. Elliptical and spline arc lengths are outside this design.

DWG `ARC_DIMENSION` contains its own center, extension points, start/end
parameters, dimension-arc position and optional leader. Its optional source
associations remain separate. The `isPartial` field is present in the ODA storage
description, but Autodesk documents the method as reserved for future use.
It is not sufficient evidence for a whole-source versus partial-source decision.

The inspected acadrust 0.6.6 implementation on base `bd10efc` reads and writes
those fields. Its `DimensionArc::measurement()` additionally normalizes the
parameter difference, uses near-zero/full-turn thresholds and falls back to XY
endpoint angles. Those are implementation observations, not adopted IFCCAD
semantics or evidence that every stored combination is qualified. Reconcile
geometry, parameters and the actual frame with representative source/readback
cases before claiming a CAD mapping. Ambiguous or contradictory input receives
an explicit diagnosis or error; no guessed minor/full-circle fallback is agreed.

## Abstract conversion contract

The future opencaddocument conversion boundary must distinguish:

1. retained native definitions and independent source relationships;
2. retained styles, overrides, labels, placement and optional stored graphics;
3. diagnosed modification/omission, unsupported semantics and invalid structure;
4. native validity, numerical conversion accuracy, physical file readback and
   host display/update behavior as separate evidence.

Do not silently approximate measurements, invent source links, update definitions
or substitute generated graphics. A whole-curve link is not an automatic
replacement for source point references; use it only if equivalent meaning is
explicitly established. A codec helper or field name is not the format contract.

Keep semantic loss policy separate from hard numerical/structural failures.
A missing source association can be a diagnosed relation loss while a valid
definition is retained. Invalid required references, nonfinite values and
unqualified mandatory numerical results cannot be repaired by supplying zeros,
choosing another sector or relaxing a tolerance. Preserve applicable coordinate
domains, frame/occurrence effects and explicit limits without selecting a new
converter API here.

Stored graphics require independent treatment: supported definition conversion
does not imply graphics transfer, display equivalence or activated CAD updates.
A route unable to retain present graphics must report that loss explicitly.
Physical target/version restrictions and any unavoidable additional CAD state
losses belong to the eventual opencaddocument coverage contract.

No particular intermediate document type, diagnostic enum, module layout,
codec patch, DIMASSOC slot allocation or target-version matrix is frozen here.
Reinspect the selected repositories and dependencies when implementation starts.

## Deferred work and implementation preparation

Excluded: automatic updates/regeneration, graphics generation, cross-owner or
block-occurrence paths, Paper-to-Model/view-dependent links, external sources,
joint annotative contexts, general geometric constraints and a generic
preservation/association framework. Recorded relations do not promise these
operations. Broader source selectors require their own bounded contract.

Before implementation, resolve the technical contract details against then-current
schemas and code: exact point/height fields, role/selector shapes and eligible
sources, angular/numerical rules, IFCX declarations, shared validation rules and
CBOR field/variant allocations. Since alpha 2026.44.0 published profile 0.1.0
and CBOR mapping 1, advance the next logical profile to **0.2.0** and CBOR mapping
to **2**, unless those next identifiers have already been consumed by an intervening
publication. Update the single active mapping and `conformance/next` together.
Zstd framing is unchanged: transport revision **1**, with inner CBOR mapping 2.
Do not modify tagged contracts, numbered collections or relabel old bytes as new.
See [release policy](../releases.md). The software release number is selected
only at separately authorized publication, not reserved by this plan.

Prepare independent native readback, invalid-definition/reference cases,
signed/major/full interval cases, same-owner and stale-reference cases, mixed
Model/Paper/block ownership, styles and graphics regressions, and inspection
coverage. Qualify actual CAD mappings separately through opencaddocument and
production readers. Coordinate the final combined candidate's required checks;
documentation recording alone runs no Rust gate or measurement.

The 28 October scope and remaining contract decisions below supersede the earlier
assumption that converter details could all wait until implementation. A concrete
paired-repository plan is now recorded locally at
`docs/superpowers/plans/2026-10-28-remaining-dimension-kinds.md` (not a Git-shipped
link). Implementation is not authorized by preparing that plan.

## Development baseline, reviewed 28 October 2026

| Component | Exact inspected baseline | Role |
| --- | --- | --- |
| IFCCAD main | `671446dc69417f04d43b958697bf9b7cba785825` | Planning source baseline; working tree has existing local documentation/patch material, not a verified clean candidate. |
| Published native alpha | `v2026.44.0`: profile 0.1.0 / CBOR 1 / Zstd 1 | Fixed historical contract; not the development reader for new kinds. |
| opencaddocument | `8a31478370c1b877c7bcb42c5298d033354ad4e4` | Proposed paired development base. |
| acadrust selected by that Document | `501b694eee3433b157e56a32da4268dc8b3b5bd3` | Transitive codec pin; third repository changes only if a demonstrated codec blocker needs them. |
| Explorer Document pin | `47d7888ca6ae1a13b80c8cf5076f5efdbc5a5f2b` | Existing checkout pin, before PR #5/#6. |
| Embedded Studio Document pin | `8e2d60952880066365caba6597762f89999400dd` | Separate host pin; not upgraded by changing Explorer. |

[Document PR #5](https://github.com/HakanSeven12/opencaddocument/pull/5) merged
as `0a3c5bebc5157bc150369519867c70a470fa0dcf`; [PR #6](https://github.com/HakanSeven12/opencaddocument/pull/6)
as `d1a8dd64ae45022a6201df6e8daf01fca20d0346`. The only intervening change to the
selected main is the explicit codec dependency update to 501b694. PR #5 removes
placement-frame snapping and checks INSERT scale setters; PR #6 removes remaining
selected quantization and aligns picture qualification. Neither establishes a
universal numerical error bound or host-rendering guarantee. Prior bd10efc
observations above are historical, not qualification of this codec.

Use one resolved core instance for Document and the native test harness. Start
with explicit local path overrides and record source content hashes, lockfile,
patch configuration, features and toolchain. Once separately authorized commits
exist, record both immutable development SHAs. Do not invent future commits,
silently follow main, edit Cargo cache sources or run published-core and candidate-core
types together. Audit the codec update's public fields/mappings before accepting
the new base. Recheck main advancement when implementation resumes.

Two repositories suffice for the intended semantic work, provided the pinned
codec can read/write all selected CAD records. They do not suffice for direct
browser preview: that also needs the viewer/browser WASM and a Studio host with
a matching development core/Document reader. A codec defect is a separate explicit
dependency task, not an excuse to approximate or revive archived adapters.

## Remaining contract choices and proposed resolutions

These proposals make the plan reviewable; they are not implemented schema rules.
Confirm the table and freeze a field/code ledger before production changes.

| Choice | Proposed resolution / acceptance boundary |
| --- | --- |
| Spatial anchors | All independently authored measurement anchors use local XYZ; evaluation uses XY projections. Presentation-only positions use the current local-plane convention. Arc center is XYZ; radius/start/sweep define its planar endpoints. Additional independent CAD point/elevation state must be classified or diagnosed, never silently flattened. |
| Roles | Keep FirstPoint/SecondPoint meaning for linear/aligned. Add distinct center/curve point, diameter points, four line endpoints, angular vertex/legs, ordinate origin/feature, and arc center/start/end roles. No generic assumption of two anchors; no required graphical helper association. |
| Bounded selectors | Keep existing point selectors. Add an explicit logical segment index and bounded circular-point selection where required: a circle angle is in the source's stored local circular frame; an arc/active segment fraction is in [0,1] along its stored directed span. Validate source family/index/domain and same-owner references. No nearest-point or update engine. Freeze exact closed variants and source eligibility in the ledger. |
| Angular sectors | Native selected rays and traversal preserve major sectors. Two-line parallel/coincident projections are invalid; three-point straight angles are valid. CAD sector selection must be independently checked; narrower target cases receive diagnosed limitations, not a changed angle. |
| Signed ordinate | Native absolute and signed modes; ordinary CAD-compatible absolute cases must roundtrip. Qualify signed transport separately; do not export a negative native value as an unreported absolute value. |
| Arc interval | Retain partial directed sweep and explicit full turn. Endpoint coincidence and isPartial alone are insufficient. Native full/negative/major spans are mandatory tests; physical target eligibility must be explicit. |
| Numeric acceptance | Preserve stored binary64 inputs in native IO, use independent known-value/high-precision references for measurements, and record source-to-first-output residuals and later-cycle residuals separately. Agree per-fixture expected bounds before conversion acceptance; no implicit global 1e-9, epsilon, or archived proof guarantee. Exceeding declared case bounds blocks that case. |
| Graphics | Keep existing absent/empty/nonempty and provenance contract. Transfer supported source pictures atomically, preserving ordered parts/resources and hidden helpers; no regeneration requirement. Report unsupported picture/provenance separately from definition/association loss. A missing source picture does not invalidate a valid definition. |
| Reader transition | Test viewer supports the candidate profile/import and CBOR revision, explicitly rejects unsupported IDs, and never feeds 0.2.0 to a 0.1.0-only host. Default proposal is one active candidate reader, with the old alpha tested in its own pinned executable; retaining a historical reader in the new viewer is optional, not required. |

## Staged acceptance and user review

1. **Native and headless mapping:** implement all seven kinds and both Document
   directions with focused tests, strict production IFCX/CBOR readback and current
   linear/aligned regressions. Include Model/Paper/block ownership, independent
   XYZ, styles/resets/formatting, stale/absent/partial associations and graphics.
2. **Portable CAD review package:** provide original sources and returned ASCII
   DXF, binary DXF and AC1032 DWG, alongside candidate IFCX/CBOR and machine-readable
   losses/semantic comparisons. Include one labelled case per kind and mixed
   sheets; use mm/m/in and nontrivial placements. Record input/output hashes,
   revisions, target/actual version, reader recovery, expected values and unsupported
   cells. First qualify those physical routes; older targets are explicit matrix
   entries, not assumed passes. Missing required type support blocks the contribution.
3. **User CAD inspection hold:** user opens the files in the chosen CAD application,
   checking types/values/text/arrows/placement, repair warnings and re-save behavior.
   Record application/build and per-file outcome. Native validity and codec self-readback
   do not prove this. Do not proceed with Explorer integration until the user reports
   the selected review successful or explicitly changes this sequencing.
4. **Development Explorer/Studio:** build against the reviewed pair and matching
   host reader. Open the exact new IFCX/CBOR snapshots, inspect definitions,
   formatted values, associations and graphics separately, and exercise export.
   Test direct IFCX host import, not just a successful regenerated DWG. Keep the
   published Explorer deployment unchanged absent a separate publication request.
5. **Final combined verification:** rebase/prepare against intended current bases,
   resolve conflicts and run the applicable native, Document feature matrix,
   viewer/browser, frontend, WASM and host smoke checks. Revalidate any changes
   after the file-review checkpoint. One contribution does not imply one commit
   across two repositories; neither repository may be committed/pushed here.

Create review packages in a fresh task-specific directory outside `target/`, with
an index distinguishing generated synthetic inputs from authorized practice data.
Do not modify private originals or include them in a public fixture package without
redistribution permission. Exported files are future implementation deliverables;
this design pass generates no CAD files or passing exchange claims.

The Explorer/OCS load-time investigation is independent. Preserve its exact
dependency/input/output baseline and warm caches. No builds, fixture replacement,
dependency updates, cache cleanup or measurements run in its checkout. Before
heavy Rust/WASM/Studio builds, coordinate a quiet window with that task; while
measurements run, continue documentation and small checks. Do not gate semantic
design on completing performance research and do not trigger a benchmark rerun.

## Context and sources

The relevant open issues were reviewed on 26 October 2026:

- [#2: additional typed families](https://github.com/OpenAEC-Foundation/ifccad/issues/2)
  supports bounded native semantics; no general payload framework is added.
- [#13: associative updates](https://github.com/OpenAEC-Foundation/ifccad/issues/13)
  separates stored relations from future update/identity processing.
- [#17: construction requirements](https://github.com/OpenAEC-Foundation/ifccad/issues/17)
  motivates further dimensions; viewport associativity and joint annotation
  remain later work under the roadmap, not implied by this slice.
- [#11: placement](https://github.com/OpenAEC-Foundation/ifccad/issues/11) and
  [#12: directed segment proof](https://github.com/OpenAEC-Foundation/ifccad/issues/12)
  do not authorize new placement representations or a proof-backend project here.

These issues provide context, not normative requirements. The roadmap still
governs sequencing. The opencaddocument conversion direction is the user's
decision for this future work, not a claim about the current converter route.

Technical references used in the discussion:

- [Current Dimension support](../dimension.md) and
  [stored graphics](graphics-design.md).
- [ODA DWG specification, section 20.4.19](https://www.opendesign.com/files/guestdownloads/OpenDesign_Specification_for_.dwg_files.pdf#page=119).
- [Autodesk DIMASSOC](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C0B96256-A911-4B4D-85E6-EB4AF2C91E27.htm)
  and [point-role enumeration](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-AcDbDimAssoc__PointType.html).
- [Autodesk ArcDimension methods](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbArcDimension.html)
  and [reserved isPartial method](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-AcDbArcDimension__isPartial.html).
- [acadrust dimension definitions at the inspected base](https://github.com/hakanaktt/acadrust/blob/bd10efc2c54dfe1e6288c5f10486dce9be0d7f32/src/entities/dimension.rs)
  and [DWG writer](https://github.com/hakanaktt/acadrust/blob/bd10efc2c54dfe1e6288c5f10486dce9be0d7f32/src/io/dwg/dwg_stream_writers/object_writer/entities.rs).
