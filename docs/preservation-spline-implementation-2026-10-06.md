# OCDraw spline preservation implementation evidence

Date: 2026-10-06. Baseline `0d462a3514040fb7e2183c25023499970a8184d2`.
Implementation and the native-edit coverage follow-up are complete. Integration
into remote main was authorized on 2026-10-06. No release, dependency-version
update or size/performance measurement was performed.
The initial 0.1.0 contract remains provisional. IFCCAD's model, schemas and
converter and all numbered conformance collections are unchanged.

## Delivered slice

- Generic owned preservation sources/records, independent checked record IDs,
  closed typed references/payloads/conditions and ordinary opaque entity identity,
  nullable native common properties, ownership and mixed draw order.
- Shared logical validation, canonical JSON byte transport, exact uint64
  persistence, production readback and candidate conformance. Soft missing/cyclic
  dependencies and unknown provider content remain storage-valid.
- Direct/transitive unavailable bounds with native validation and transactional
  recomputation, including unused definitions, nested/repeated occurrences and
  opaque-only children. No native spline evaluator, tessellation or guessed bounds.
- Complete interpreted Spline/Common/XDATA field transport, including Serde-skipped
  public fields and exact binary64 bits. Original raw records are omitted with
  explicit storageSupplementOmitted evidence; raw/private/shared replay is absent.
- Opt-in capture, authoritative native common edits, on-demand guarded restoration,
  constructed forward reference rebinding, independently reported missing/context/
  payload/predicate failures and Allow/Reject/Skip behavior. No input variant whitelist.
- CLI/browser opt-in, provenance, opaque counts, unavailable bounds, separate
  capture/restoration and geometry completeness evidence. Known file origin/version
  is supplied by the file adapter; library-only inputs use cadDocument provenance.

The detailed [payload, field and qualification contract](../crates/ocdraw-convert/docs/SPLINE-SNAPSHOT-V1.md)
and both converter coverage contracts record the supported behavior. Core storage
remains independent of opencadcodec. Unknown geometry is not assessed as exact.

## Qualified CAD exchange

Direct and preservation chains use the same readers/writers and explicit AC1032
target. The preservation chain encodes and production-loads OCDraw before creating
fresh CAD content. Source instances/caches are not needed for restoration.

| Fixture/profile | DXF | AC1032 DWG |
| --- | --- | --- |
| Independently hand-authored open cubic | Direct/preservation equivalent | Direct/preservation equivalent |
| Mixed native/cubic ModelSpace | Equivalent owner-relative order and semantics | Equivalent |
| Two named Paper layouts | Equivalent by layout identity | Equivalent by layout identity |
| Local and unused block definitions | Equivalent | Equivalent |
| Repeated and nested block occurrences | Equivalent | Equivalent |
| Quadratic | Equivalent; one spline read | Equivalent; one spline read |
| Rational weights | Equivalent; one spline read | Equivalent; one spline read |
| Closed/periodic | Equivalent; one spline read | Equivalent; one spline read |
| Spatial control points/normal | Equivalent; one spline read | Equivalent; one spline read |
| Fit-only | Equivalent; one spline read | Equivalent; one spline read |
| Mixed control/fit/tangents | Equivalent; one spline read | Equivalent; one spline read |

Carrier tests separately exercise unusual/nonfinite parameter sets, all common
fields, source option presence/enum identity, exact IDs, and all 256 transparency
bytes. Reference/native-edit/context tests remain separate from geometry variants.
Original handles, codec-reserved Paper block names, redundant entity_mode and
whole-file bytes are not equivalence oracles. These results qualify the stated
fixtures/configuration, not every source variant, later revision or whole drawing.

## Dependency provenance

Both converter pins remain `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`.
Ignored local Cargo configuration selects the existing documented viewport patch
checkout, on this same base. Its only modified files have these SHA256 values:

| File | SHA256 |
| --- | --- |
| src/entities/viewport.rs | 6EB5E04AF03D43103F1B4FF638808A3551FBEF1EF0F66A6F78489CBA6B25E241 |
| src/io/dxf/reader/section_reader.rs | B10938421575C21B158038980FE458C5CA7EC3E4D7643EA34E6301140513AFC7 |
| src/io/dxf/writer/section_writer.rs | 344AF75E7236AAD4F2E435E3201CC0D1A24AAF124F5F679B8B84636543E362F6 |

See `patches/opencadcodec-viewports/README.md` for reproduction. Patched verification
does not establish unmodified-upstream behavior. The copied local Cargo.lock
retains package versions, with only direct dependency edges for already locked
Base64/Serde declarations. No Cargo cache or codec source was patched by this task.

## Review decisions and remaining boundary

Work followed the local plan in one task without subagents; final review was an
explicit self-review. The ignored plan ledger retains RED/GREEN and gate logs.

- A repository-local worktree and PowerShell ledger bookkeeping honor the user's
  location/no-delegation rules; no commit-dependent bookkeeping or deletion occurs.
- Adapter-private functions use small typed inputs and existing final maps instead
  of speculative context/framework objects. Future signature changes stay local.
- Missing pattern targets remain opaque and unavailable for restore; a provable
  source name/handle contradiction stays fatal. Native pattern rules are unchanged.
- Paper comparison uses layout identity; ModelSpace references use scope-role/header identity even after layout renames. Operational reserved names are excluded,
  while semantic owner/order and parameters remain compared.
- Missing optional source fields cannot silently become None/default. The source
  carrier requires their explicit presence and tests the Serde behavior.
- Numerical review does not invent a point for an opaque child. It tests an actual
  native matrix-preparation overflow and preserves transactional recomputation.
- Build cache access was needed to complete matching wasm-bindgen tooling; package
  versions were not upgraded. Generated WASM remains ignored local output.

The initial native-edit coverage audit corrected an overly broad completion claim.
The follow-up now covers the listed native edits and the full durable physical
exchange chain; see the current matrix below. Intentional restrictions
remain: unqualified raw/attached/
application contexts, unsupported non-entity/supplement providers and cyclic restore
groups may be stored but not restored. Broad providers/private/shared storage await
upstream #91 direction or a separately approved alternative. IFCCAD preservation,
native spline semantics, copy qualification, GC/dedup and alternative encodings
remain separate follow-ups.

## Verification

All final checks passed:

| Check | Result |
| --- | --- |
| cargo fmt --all -- --check | Pass |
| cargo clippy --workspace --all-targets -- -D warnings | Pass |
| cargo test --workspace | Pass |
| cargo test --doc --workspace | Pass |
| Explorer npm test | 33 pass, 4 existing Linux deployment tests skipped on Windows |
| Viewer CLI opt-in literal fixture | Strict readback; one opaque entity; DXF/AC1032 provenance |
| Browser release WASM build and actual smoke | Pass, including spline/native/DXF/DWG and existing OCDraw/IFCCAD profiles |

The ignored execution ledger retains the log inventory and RED/GREEN review
evidence. These are correctness/readback checks; no benchmark was run.

## Native-edit coverage audit — 2026-10-06

Tests concretely exercise post-capture visibility, color, entity pattern scale,
layer rename, order reversal with forward typed XDATA references, model layout
rename, coordinate-unit change/reversion, owner move, native reference deletion,
opaque deletion/archive retention, missing/unknown/malformed conditions and a
needed record cycle. The durable helper encodes/production-loads before the common
edits and drops the original CadDocument. Common-edit assertions check new CAD
common values, unchanged degree/knots and an unchanged preservation collection.

Those initial tests have now been extended by
`crates/ocdraw-convert/tests/ocdraw_spline_edit_exchange.rs`. Every positive case
captures and production-loads OCDraw, applies native edits, explicitly recomputes
bounds, encodes and production-reopens the edited drawing, restores into a fresh
CadDocument, and uses the real DXF and AC1032 DWG writers/readers. The fixture has
ModelSpace, a Paper layout, local/unused definitions and nested instances. The
snapshot collection and capture-time baselines must remain identical; all spline
parameter fields are compared against the independent direct codec chain to
account for its physical normalization. Ownership and mixed order are checked.

| Native edit | After edited save/reopen | Actual DXF | Actual AC1032 DWG |
| --- | --- | --- | --- |
| Layer reassignment | Current layer wins | Pass | Pass |
| Layer color/opacity/weight/pattern defaults | Defaults change; spline ByLayer modes remain | Pass | Pass |
| Pattern definition | Edited ordered pattern and spline reference survive | Pass | Pass |
| Pattern rename | Current name/reference survives | Pass | Pass |
| Drawing-global pattern scale | Current scale survives | Pass | Pass |
| Color/opacity/weight/visibility/entity scale | Current common properties win | Pass | Pass |
| Complete native appearance supplied after absence | Source Default appearance is explicitly replaced | Pass | Pass |
| Unrelated native line geometry | Edited endpoint survives; spline parameters stay unchanged | Pass | Pass |
| Block base point | Current base point; unchanged local spline data | Pass | Pass |
| Instance placement and scale | Current nested occurrence state; unchanged spline data | Pass | Pass |
| Paper layout name and mixed draworder | Semantic owner/order survives | Pass | Pass |
| Delete original layer and reassign native properties | Missing original soft dependency does not override new native properties | Pass | Pass |

The delete/clear case also passes production save/reopen: present hard references
first make the drawing invalid, clearing/reassigning them restores storage validity,
while the original missing soft dependency gives MissingDependency. Reject refuses
the live drop; Allow's actual DXF/DWG outputs omit those splines with evidence.

Fourteen independent condition mutations are checked after save/reopen: each
mandatory predicate missing separately, duplicate entity predicate, future predicate
version, wrong entity target role, changed record baseline, malformed bytes, invalid
unit baseline, reference target/slot/key/role mismatches, missing required reference
condition despite a qualified label, and future payload version. Only the affected
spline is skipped under Allow; Reject refuses the loss. A separate unit change/save/
revert/save test regains eligibility and completes actual DWG readback.

One test additionally writes through production filesystem storage, drops the edited
document and encoder, reopens the file through `load_ocdraw_file`, and completes
DWG writing/readback. No graphic editor or editable validated-reader backing is
required: callers consume `ValidatedOcdraw::into_document()` and edit the complete
owned logical document. The opaque spline's geometry remains uneditable natively.

The new target contains 16 tests, including the 14-case condition matrix. Its scope
is the documented fixtures, source codec configuration and implemented provider;
it does not certify arbitrary raw/application context or all future CAD variants.
