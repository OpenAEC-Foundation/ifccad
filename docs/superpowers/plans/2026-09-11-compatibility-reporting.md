# Compatibility Reporting Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans in this task. Do not delegate: the repository requires explicit user authorization for subagents. Track execution with the checkboxes below.

**Goal:** Add precise classifications, assessment completeness and bounded conversion summaries to the existing reports.

**Architecture:** Diagnostic producers classify evidence; package validation records assessment gaps separately. Converter outcomes summarize existing loss evidence within documented coverage. Existing strict-loading and conversion policies remain unchanged.

**Tech Stack:** Existing Rust workspace, serde and JSON conformance assets. No additional dependency.

**Spec:** [Approved reporting specification](../specs/2026-09-11-compatibility-reporting-design.md).

## Global constraints

- Execute on a new `compatibility-reporting` branch based on current main, preserving the approved uncommitted spec and this plan in the existing checkout.
- No preflight scan, new content checks, partial drawing access or preservation functionality.
- Preserve diagnostic codes, severity, messages, source locations and existing conversion loss policies.
- Preserve is_valid() and strict typed-package eligibility; incomplete assessment is not itself a new error.
- IFCX remains 0.9.0, drawing core 0.2.0, IFCDR registry/mapping 0.7.0, IFCPR 0.2.0 and candidate suite 1.1.0.
- Historical schemas and conformance/1.0.0 remain immutable. Manifest schema v2 is a separate new contract.
- Core must not depend on cadcodec. No commits, merges or pushes without new authorization.
- Review each deliverable locally; no subagents. Finish all authorized implementation and verification before reporting completion.

## File map and interfaces

- `src/diagnostic.rs`: add public PackageDiagnosticCategory and a category field to PackageDiagnostic; preserve current serialization names and add category in camelCase.
- `src/package/read/assessment.rs` (new): assessment conclusions, gap evidence and aggregation.
- `src/package/read/diagnostic.rs`: retain detailed report API and expose assessment().
- `src/package/read/validation.rs`, `loader.rs`, `schema.rs`, `error.rs`: collect evidence at the existing validation boundaries and classify errors.
- `src/ifcdr/codec/json/diagnostic.rs`, `physical.rs`, plus package graph/binding/appearance producers: explicit diagnostic classifications.
- `src/package/read/mod.rs`, `src/package/mod.rs`, `src/lib.rs`: reexports as appropriate to existing public paths.
- `crates/ifccad-convert/src/assessment.rs` (new), import/export outcome modules and lib.rs: shared converter-local summary types, independently of CAD-specific diagnostics.
- `crates/ifccad-convert/src/import/COVERAGE.md` (new): current import assessment boundary; export/COVERAGE.md remains the export basis.
- `conformance/next/reporting-contract-v1.md` (new), `manifest-schema-v2.json` (new), manifest.json and `src/conformance/manifest.rs`: vocabulary and expected report fields.
- Public report tests, package conformance tests and existing converter tests: assertions on both old outcomes and new summaries.

Proposed core interfaces, owned by the files above:

```rust
pub enum PackageDiagnosticCategory {
    ContractViolation,
    UnsupportedContent,
    ExecutionBlocked,
}
pub enum PackageValidity { Valid, Invalid, NotFullyAssessed }
pub enum AssessmentCompleteness { Complete, Incomplete }
pub enum AssessmentGapReason {
    UnsupportedProfile,
    UnavailableInput,
    ExecutionLimit,
    ContentNotAssessable,
    PreservationSemanticsNotAssessed,
}
pub struct AssessmentGap {
    pub resource_id: Option<ResourceId>,
    pub resource_uri: Option<String>,
    pub location: Option<String>,
    pub reason: AssessmentGapReason,
}
```

PackageAssessment owns validity, completeness and a deterministically ordered
gap list behind read-only getters. PackageValidationReport::assessment() returns
&PackageAssessment. Default reports must not fabricate Complete evidence;
validation finalization constructs a completed assessment explicitly.

## Task 1: Classify existing diagnostic producers

- [x] Inventory every PackageDiagnostic construction and stable code in core. Record code, meaning and category in reporting-contract-v1.md. Treat the code name as an identifier, not a semantic classifier.
- [x] Add tests for known geometry/reference errors, unsupported version/stream, byte limits and PackageOpenError. Assert exact categories alongside unchanged severity/codes. Run focused tests and observe missing category support.
- [x] Add the category enum and field, implement PackageOpenError::category(), and assign categories explicitly at all producers. Update struct literals in tests without changing their scenario or old expectations.
- [x] Handle schema profile-selection disagreements at the schema/validation boundary. For an unsupported version, its fixed-version schema diagnostic is UnsupportedContent; unrelated known rule failures stay ContractViolation. Do not blanket-reclassify all schema errors of that resource.
- [x] Audit ambiguous codes such as IFCDR_UNIT_UNSUPPORTED against the actual active contract. A value forbidden by a known enum is ContractViolation unless the profile itself is unknown. Missing files, rejected paths, malformed JSON and checksums need deliberate documented treatment; preserve their existing operational result.
- [x] Test the unsupported-version case both alone and with an independent duplicate-node violation. Expected categories are UnsupportedContent alone versus a mixture containing ContractViolation.
- [x] Run `cargo test --lib` and existing public-package tests. All old loading decisions and diagnostic text/code assertions must still pass.

## Task 2: Record completeness and expose package conclusions

- [x] Add aggregation tests in assessment.rs for this truth table before implementing it:

| Proven contract violation | Assessment gaps | Validity | Completeness |
| --- | --- | --- | --- |
| no | none | Valid | Complete |
| yes | none | Invalid | Complete |
| no | present | NotFullyAssessed | Incomplete |
| yes | present | Invalid | Incomplete |

- [x] Implement PackageAssessment aggregation with explicit completion evidence. Conceptually:

```rust
let validity = if has_contract_violation {
    PackageValidity::Invalid
} else if assessment_complete {
    PackageValidity::Valid
} else {
    PackageValidity::NotFullyAssessed
};
```

- [x] Collect gaps where work is actually skipped: unavailable entrypoint/resource, loading limit, unsupported profile, schema/body failure preventing dependent checks, and current IFCPR semantic coverage. A completed rejection of known malformed input can establish Invalid while dependent checks remain incomplete.
- [x] Make current IFCPR coverage an explicit validation-coverage declaration. Add gaps only for encountered IFCPR content; a drawing-only package must not inherit an irrelevant preservation gap. Explain schema, identity and link checks separately from unassessed preservation semantics.
- [x] Preserve existing errors for failed I/O through Result. Never return a default or empty report as though loading succeeded after PackageOpenError.
- [x] Populate gap origins with the same inline/external source mapping as diagnostics. Deduplicate repeated declarations of one physical source; retain distinct inline locations. Keep deterministic ordering.
- [x] Keep permitted unrelated IFCX extensions open. Document assessment scope as the applicable IFCCAD-owned contract; make no assertion that arbitrary extension semantics were assessed.
- [x] Add `tests/public_package_assessment.rs`: valid minimal drawing; known contract error; unsupported version; IFCPR without errors but incomplete coverage; invalid-plus-incomplete; inline/external equivalence. Assert is_valid() and validated_package() remain identical to the prior behavior in each scenario.
- [x] Run `cargo test --test public_package_assessment` and `cargo test --test inline_resources`. Review the public API docs so complete assessment is never confused with full preservation transfer.

## Task 3: Conversion summaries from existing evidence

Converter-local interfaces:

```rust
pub enum TransferConclusion {
    LossDetected,
    NoLossDetected,
    NotFullyAssessed,
}
pub enum TransferScope { SelectedDrawingToCadDocument, PublicCadDocumentToPackage }
pub enum TransferCoverage { CompleteWithinScope, Incomplete }
```

TransferAssessment owns conclusion, scope, coverage and documented limitations
with read-only getters. ImportOutcome and ExportOutcome expose
transfer_assessment() -> &TransferAssessment. Construct it when the existing
operation completes, not by rescanning source data at accessor time.

- [x] Audit the existing import code against logical drawing fields and write import/COVERAGE.md. Identify actual represented semantics, nonsemantic identities and areas whose fidelity is not currently assessed. Do not add runtime conversion checks to close gaps.
- [x] Retain export's pinned public-CadDocument boundary and existing COVERAGE.md exclusions. Do not claim to cover hidden/raw CAD state.
- [x] Add tests for loss diagnostics, no diagnostics with complete scoped coverage, and no diagnostics with incomplete coverage. Implement this precedence:

```rust
let conclusion = if has_recorded_loss {
    TransferConclusion::LossDetected
} else if coverage == TransferCoverage::CompleteWithinScope {
    TransferConclusion::NoLossDetected
} else {
    TransferConclusion::NotFullyAssessed
};
```

- [x] Keep limitations visible even when LossDetected takes precedence. Import gaps must conservatively yield Incomplete when existing checks cannot establish scoped fidelity; do not manufacture new detailed loss diagnostics from a missing coverage guarantee.
- [x] Update outcome constructors and reexports. Preserve existing into_parts() tuple shapes and other consuming APIs; the new assessment is inspectable before consumption and adds no required caller arguments.
- [x] Preserve all Result error paths. Document UnsupportedDrawingStructure, insertion failures, InvalidSourceStructure and LossRejected as failed/blocked attempts. For LossRejected, existing retained loss diagnostics remain evidence of detected loss, not evidence that an output was returned.
- [x] Extend export loss-policy tests to prove Allow/Reject outcomes have not changed. Extend import rounding/fallback tests to assert LossDetected and the selected-drawing scope. A package load does not receive any transfer assessment.
- [x] Run `cargo test -p ifccad-convert` and `cargo test --doc --workspace`.

## Task 4: Versioned conformance expectations

- [x] Add reporting-contract-v1.md with category vocabulary, completeness/validity rules, transfer precedence and bounded scope. Define serialized enum values in camelCase matching Rust serialization.
- [x] Copy candidate manifest-schema-v1.json to a new manifest-schema-v2.json and add optional expected.packageAssessment and diagnostic.category fields. packageAssessment includes validity, completeness and gaps with the core shape above; closed enums reject unknown values. Frozen v1 assets remain untouched.
- [x] Add an optional top-level manifestVersion (integer); absence means v1, explicit 2 selects v2. Require it to be 2 in the v2 schema. Keep suiteVersion as the independent collection version.
- [x] Extend ExpectedOutcome with an optional typed package assessment and parse manifest versions explicitly. Preserve v1 shapes and parsing of the frozen 1.0.0 suite in addition to the active 1.1.0 candidate; unsupported versions still error. Do not run obsolete packages through the current reader as a compatibility claim.
- [x] Add parser/schema tests for v1 without new fields, v2 with typed expectations, unsupported versions and malformed enum values. Test actual frozen manifest parsing without editing it.
- [x] Add manifestVersion: 2 to the candidate and representative assessment/category expectations. Keep case IDs, old diagnostic expectations and the five deferred IFCPR cases. Enrich existing cases rather than cloning a large fixture corpus.
- [x] Extend the production conformance runner to assert new fields when present. Preserve its old projection of diagnostic fields for v1 expectations.
- [x] Update candidate asset tests to include the new reporting document/schema and validate candidate v2 plus frozen v1 against their corresponding schemas.
- [x] Run focused conformance parsing, loading, assets and package tests. No converter preflight operation is added to the manifest vocabulary.

## Task 5: Documentation, review and final verification

- [x] Update README, compatibility matrix, provenance, converter docs and ROADMAP. Explain legacy is_valid() separately from assessment().validity(), and why IFCPR incompleteness does not itself block a previously loadable package.
- [x] Document both conversion scopes and the meaning of NoLossDetected. Keep private/raw source exclusions and retained coverage gaps visible.
- [x] Audit every current diagnostic producer for explicit classification and every early return for correct assessment completion. Check that no branch classifies unknown-profile content as invalid under that profile without known evidence.
- [x] Compare old/new strict-loading and conversion outcomes across existing tests. Check deterministic serialization/order of new data and unchanged inline/external equivalence.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo test --workspace`.
- [x] Check `git diff --check`, unchanged format-body schemas, frozen collections and dependencies; no target/ artifacts may enter version control.
- [x] Update spec status and actual plan progress, report test counts and remaining limitations, and leave changes uncommitted.

## Plan review

Tasks 1–2 cover reader classification and explicit assessment without changing
loading gates. Task 3 covers conversion summaries without new checks. Task 4
publishes the language-neutral reporting vocabulary and preserves old manifests.
Task 5 verifies compatibility and updates milestone status. Measurement work,
schema cleanup and preservation modeling are excluded.

## Execution evidence (2026-09-11)

Implemented in this task on `compatibility-reporting`, based on main `43ad394`.
Implementation and review stayed in this task without subagents. After review,
the user approved committing and integrating into remote `main`. Reader and converter API
tests first failed on the absent assessment interfaces, then passed with the
implementation. The unknown-version regression initially revealed accidental
null-field insertion in the test fixture; correcting the fixture isolated the
intended unsupported-profile scenario.

Final verification: formatter check passed; workspace Clippy with warnings
denied passed; workspace tests passed (323 passed, 0 failed, 1 existing ignored).
Focused documentation tests passed (3). Diff whitespace check passed; format-body
schemas, frozen conformance assets and dependency manifests/lockfile are unchanged.
The v2 candidate has representative valid, invalid-plus-incomplete, unsupported
and unavailable-input expectations; frozen v1 parsing/schema checks still pass.

Review refinements: unknown IFCPR body-version mismatches retain current-schema
evidence without claiming body invalidity, while independent envelope and graph
rules remain applicable. Gaps retain actual source origins and known resource
IDs, and dependent checks skipped after graph/binding/schema failures remain
explicitly incomplete. Import fidelity coverage remains conservative as approved.
Only initial size measurements remain for milestone 2; entity expansion and full
IFCPR semantics remain later work.
