# Compatibility reporting improvements

Date: 2026-09-11

Status: Implemented and verified; integration into remote `main` approved.

## Purpose and boundaries

Complete milestone 2's distinction between contract validity, operation support
and transfer fidelity by extending existing reports and typed errors. This is
not a new scanning subsystem or a text-rendering project.

Report evidence from attempted package loading/validation and the existing
conversion functions. Do not predict unexecuted conversions. Preserve loading
gates, converter loss policies, supported entity content and IFCPR validation
behavior. Incomplete assessment alone must not introduce a new loading blocker.

ROADMAP.md is authoritative. Open issues #1, #7 and #8 provide resource,
contract-boundary and compatibility context; their historical descriptions do
not reopen completed architecture. Issue #6's measurements remain a separate
step. No current open issue requires expanding this reporting work into format
semantics, preservation implementation or a preflight API.

## Existing foundation

PackageDiagnostic already contains code, severity, resource identity, source
location, structured context and human-readable text. PackageValidationReport
collects these and defines is_valid() as absence of error diagnostics.

ImportDiagnostic already identifies line-pattern fallback and line-weight
rounding. ImportError identifies blocked imports. ExportDiagnostic records
loss source, action and reasons; ExportError distinguishes invalid source
structure, rejection by loss policy, package construction and internal errors.
The export COVERAGE.md bounds its guarantee to the pinned public CadDocument
model. These types and detailed messages remain the primary evidence.

## Reader diagnostic categories

Add an explicit machine-readable category for diagnostics emitted by the
reader, separate from severity:

| Category | Meaning | Examples |
| --- | --- | --- |
| ContractViolation | A known contract rule is violated. | Invalid coordinates, duplicate IDs, broken known references, mismatching checksum. |
| UnsupportedContent | Content is outside the implementation's understood profile. | Unsupported resource version, stream schema or retired vocabulary. |
| ExecutionBlocked | The attempt could not finish because of an execution constraint. | Resource/total byte limit, filesystem access failure. |

Keep existing codes, locations and text. Use an explicit reviewed classification
of existing diagnostic producers/codes; never infer category by matching prose
or solely by searching a code for words such as INVALID or UNSUPPORTED.
For example, a value outside a known enum can be a contract violation even
when an old diagnostic code happens to use the word UNSUPPORTED.

Context-dependent schema diagnostics need classification at their source or
an explicit package-level interpretation. A fixed-version schema disagreement
caused by an unsupported declared version must not become proof that the
resource violates its own unknown contract. Keep the detailed schema evidence
and loading outcome, but classify profile selection separately from known
content violations. Independently established errors remain reportable.

PackageOpenError remains an ordinary Result error when inspection could not
start or proceed due to I/O. Expose its classification consistently without
fabricating an empty successful validation report. Existing converter error
variants remain authoritative; the three reader categories do not replace
converter-specific loss-policy or internal-failure distinctions.

## Assessment completeness and validity

Record assessment completeness independently from found errors. The report
must identify unassessed areas and reasons, including a ResourceId when known.
This evidence is recorded by the validation flow, not reconstructed solely
from the absence of diagnostics.

Examples of incomplete assessment are an unsupported resource profile,
unavailable input, a loading limit or the currently unimplemented IFCPR
semantic checks. Report current IFCPR coverage as schema/identity/links checked,
with remaining preservation semantics unassessed. This is a property of the
current validation coverage, not an immutable rule that all IFCPR is incomplete.
When coverage is extended, update the coverage declaration and its tests.

Completeness concerns the applicable IFCCAD-owned contract and explicitly
states that scope. Permitted unrelated IFCX extensions are not declared invalid
just for being unknown, nor is their meaning claimed to have been validated.

Expose a structured validity conclusion with these meanings:

- Invalid: at least one established ContractViolation exists.
- Valid: the applicable assessment completed and no contract violation exists.
- NotFullyAssessed: neither validity nor invalidity is established because
  assessment is incomplete.

An Invalid conclusion may coexist with incomplete assessment: one proven
violation is sufficient for invalidity, but does not mean every other part was
checked. Therefore retain completeness and its reasons alongside the conclusion.
UnsupportedContent and ExecutionBlocked alone do not establish Invalid.

Keep is_valid() with its current documented semantics for existing callers;
do not silently change it or the strict typed-package gate in this step.
The new conclusion is the precise contract statement. Documentation must make
clear that legacy is_valid() answers whether implemented checks emitted errors.
For example, a package with currently limited IFCPR checks can retain a strict
typed view while its full assessment is NotFullyAssessed.

## Conversion summaries

Add a compact structured transfer assessment to successful ImportOutcome and
ExportOutcome, retaining their existing diagnostics, output and entity mapping.
It identifies the operation and assessed scope and exposes:

- LossDetected: existing diagnostics establish at least one loss.
- NoLossDetected: assessment completed within the explicitly documented scope
  and no loss was found.
- NotFullyAssessed: coverage is insufficient to conclude NoLossDetected and
  there is no established loss.

Retain coverage limitations alongside the summary even when LossDetected takes
precedence. NoLossDetected must never follow from an empty diagnostic list
alone: it also requires an explicit coverage basis for that direction.

For CadDocument-to-IFCCAD use the existing pinned export coverage contract;
private/raw CAD codec state lies outside its guarantee. For Drawing-to-CadDocument
document the existing converter's actual assessed semantics. Do not assert that
all logical Drawing metadata is covered merely because line and polyline
conversion succeeds. If current diagnostics cannot account for a semantic area,
report limited coverage rather than adding new conversion checks in this task.
Reviewing/documenting current coverage is required; expanding it is not.

A Drawing conversion does not assess transfer of its entire containing package
or IFCPR source payloads. Loading alone has no transfer conclusion. Result errors
remain errors: an aborted attempt must not be presented as a completed conversion.
LossRejected can report that loss was detected and policy blocked output; it
does not mean a lossy output was returned. Do not generate a NoLossDetected
conclusion for operations that failed before assessment completed.

Core report concepts must remain independent of cadcodec. Converter-specific
scope and summaries belong in ifccad-convert; shared vocabulary must not become
a dependency from the core crate onto a CAD runtime.

## Conformance, documentation and compatibility

Publish the report vocabulary and aggregation rules as language-neutral
conformance documentation. Extend the mutable manifest format with versioned,
optional expected assessment fields and diagnostic categories, preserving
loading of frozen v1 manifests. Use a new manifest schema version for the
candidate rather than editing the released v1 schema contract in place.

Assert new structured outcomes for representative package cases; retain exact
existing detailed diagnostic expectations and deferred IFCPR cases. Do not
rename historical invalid/unsupported case identifiers to simulate a better
runtime conclusion. Conversion unit/integration tests assert transfer summaries
alongside the current loss diagnostics and policy outcomes.

IFCX overlay remains 0.9.0, drawing core 0.2.0, IFCDR registry/mapping 0.7.0 and
IFCPR 0.2.0. Serialized drawing/preservation bodies and writer output are unchanged.
Candidate suite remains 1.1.0. Historical schema cleanup and size measurements
are separate changes. Update compatibility documentation, public API examples
and ROADMAP progress when implementation is verified.

## Required evidence

- A valid supported IFCDR package reports complete assessment and Valid.
- A known reference/geometry violation reports ContractViolation and Invalid.
- An unsupported version without an independent known violation reports
  UnsupportedContent and NotFullyAssessed, including its schema-profile evidence.
- A loading limit reports ExecutionBlocked and incomplete assessment; I/O errors
  cannot be mistaken for successful reports.
- A package with IFCPR and no detected errors reports the remaining assessment
  gap while preserving its existing loading eligibility.
- A package containing both proven violations and unassessed content reports
  Invalid plus incomplete assessment and the reasons.
- Inline and external source forms yield equivalent conclusions with accurate
  origins; no new semantic validation or source restrictions are introduced.
- Every current reader diagnostic producer has a reviewed classification.
- Conversion summaries preserve known loss evidence, respect loss policy and
  their coverage boundary, and never infer full fidelity from no diagnostics.
- Frozen conformance collections and their schemas remain unchanged.

Use focused tests during implementation; final verification is cargo fmt --all
-- --check, cargo clippy --workspace --all-targets -- -D warnings, and cargo test
--workspace. Run cargo test --doc --workspace when public examples change.

Implement on a separate `compatibility-reporting` branch after specification
review and implementation planning. Work stays in this task without subagents.
Commit, merge and push require separate explicit authorization.
