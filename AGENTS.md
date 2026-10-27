# AGENTS.md

## Project context

IFCCAD is the active CAD drawing profile using IFCX. The root Rust package is
`ifccad`, with public model/IO types exported directly; implementation remains
under `src/ifccad`. `ifccad-convert` and the CAD companions own direct CAD exchange.
The viewer/browser/Explorer support IFCCAD only. OCDraw is parked under
`legacy/ocdraw`, excluded from ordinary builds/tests and active dependencies.
Do not restore parity, import archived helpers or add an OCDraw compatibility
feature. Restore the complete pre-migration checkpoint only for explicitly
requested legacy work; see `docs/ocdraw-parking.md` and the archive manifest.
Underlying IFCX syntax/extensions retain their names. CBOR revision 1 and optional
native Zstd transport revision 1 remain experimental; default WASM excludes Zstd.
The first native source alpha, 2026.44.0, publishes IFCCAD profile 0.1.0,
CBOR mapping revision 1 and Zstd transport revision 1. Those tagged contracts
are fixed. Subsequent normative changes advance the appropriate identifiers
under the [release policy](docs/releases.md). Update the single active mapping
and conformance/next together; historical mappings/readers are not mandatory.
Earlier experimental files and future alpha releases have no general
compatibility guarantee.

Start with:

- `README.md` for the project purpose, drawing architecture, current
  capabilities, roadmap summary, and repository layout;
- `ROADMAP.md` for authoritative development sequencing, milestone status,
  dependencies, and exit criteria;
- `docs/vision.md` for long-term use cases, design principles, and success
  criteria;
- `crates/ifccad-convert/README.md` for conversion terminology and the boundary
  between IFCCAD and opencadcodec `CadDocument`;
- `crates/ifccad-convert/README.md` for pinned codec coverage and loss classification.

For measurement or encoding work, also read
`docs/benchmarks/ocdraw-size-exchange-v1.md` for the standalone controlled experiment, its limits,
and reproduction instructions. Import changes must also consult
`crates/ifccad-convert/README.md`. Historical OCDraw contracts live in the archive.

For performance work, also read `docs/benchmarks/placement-preparation-v1.md`
for the current preparation measurements and practice-file inventory. Prefer
optimizations justified by measured representative workloads. Keep synthetic
stress cases for correctness and numerical boundaries; their cost alone does
not establish a practical priority. Further oblique-placement tuning and shared
placement preparation remain deferred until practice data and profiling justify them.

Use the following source-of-truth order:

- `schemas/` and `conformance/` define the language-neutral format contract;
- Rust source and tests define the current Rust API and implementation behavior;
- relevant documents under `docs/superpowers/specs/` explain approved design
  decisions and rationale, but are not a substitute for current code, schemas,
  or tests.

`ROADMAP.md` is the source of truth for development sequencing and milestone
status. The roadmap order expresses architectural dependencies even when
milestones overlap. Do not couple later work to unresolved earlier boundaries.
When proposed work crosses milestones, identify the dependency and trade-off
explicitly.

Dated specs and plans record the scope and status of their implementation
slice. Their historical "remaining work" statements do not override the current
roadmap or reopen completed milestones.

Open GitHub issues provide roadmap and future-design context, but are not
normative requirements. Before designing changes that affect format
architecture, schema evolution, physical encoding, preservation, or conversion
coverage, review the relevant open issues and identify overlaps or conflicts.
Do not expand the current task merely because a related issue exists.

## Local Superpowers workflow documents

`docs/superpowers/` stays local and outside Git. Its relevant designs and plans
are working documents for the Superpowers workflow: consult and maintain them
during the agent task to retain agreed decisions, execution steps and progress.
They are not disposable merely because they are untracked.

Record lasting contracts, architecture decisions and development conventions in
the regular repository documentation as well. After a task is complete, local
documents may be cleaned up once they contain no unique information needed for
follow-up work. Retain designs and plans still needed by active or upcoming work.

## Architecture boundaries

- The core `ifccad` crate owns the drawing model, shared validation, typed
  construction, encoding, and standalone file storage.
- The core crate must remain independent of opencadcodec and other CAD runtimes.
- The `ifccad-convert` companion crate owns conversion between validated IFCCAD
  drawings and opencadcodec `CadDocument`.
- Keep conversion, logical package construction, physical encoding, and
  filesystem storage as separate responsibilities.
- Add new entity semantics to the language-neutral logical contract and shared
  semantic validation; keep physical field/range rules in the codec mapping.
  Exercise reader and writer backings through the shared logical access rather
  than duplicating semantic rules in each direction.
- Do not silently approximate or discard source semantics. Represent them,
  diagnose the loss, preserve them through an approved preservation mechanism,
  or reject structurally inconsistent input.
- Unknown core fields are rejected. Future preservation or external semantic
  links need a concrete approved extension design before implementation.
- Numbered conformance collections are immutable. Develop contract changes in
  the active schemas and `conformance/next`; never modify a released collection
  in place.

## Working conventions

- Create Git worktrees inside the repository-local `.worktrees/` directory by
  default. Keep that directory ignored, and use another worktree location only
  when the user explicitly requests it.
- After merging a completed task into main, always clean up its repository-local
  worktree and delete the merged local task branch. The user gives standing
  authorization for this cleanup; no separate per-task removal confirmation is
  needed. A merged worktree must not be kept solely because it contains local files.
- Before removal, verify that the task is complete, its commits are in main,
  and no ongoing work or process needs the worktree. Preserve active worktrees,
  even if their current commits are technically merged. Never infer inactivity
  from commit age alone or discard unrelated/unmerged changes.
- Select local material before archiving; do not copy all ignored/untracked
  files by default. Keep only unique information needed for active/upcoming
  work, reproduction of applicable accepted evidence, or a concrete unresolved
  problem. Ignored status, file type and commit age alone do not establish value.
- Preserve unrecoverable source material and user-authored files, designs/plans
  with still-needed unique decisions, and applicable benchmark provenance. For
  accepted measurements, retain the documented report and matching detailed
  results plus any exact source manifest, dependency lock/configuration or input
  snapshot needed for reproduction that is not already retained elsewhere.
  Keep raw generated measurement files only when the contract requires them or
  they are needed to investigate a specific discrepancy.
- Remove reproducible build/site/WASM output, caches, temporary test files,
  intermediate migration scripts and resolved-debugging output when they contain
  no unique required information and no process uses them. Discard duplicate or
  superseded runs only after verifying that the applicable replacement and its
  matching evidence are retained. Keep detailed logs only for unresolved issues
  or evidence not adequately recorded in the retained task summary/report.
- Completed local designs/plans may be removed once their lasting decisions and
  relevant verification are recorded in regular repository documentation or a
  concise retained task summary. Preserve working documents still needed by
  active/upcoming work. Compare content before treating a main-local or Git copy
  as a duplicate; do not overwrite existing main-local files.
- Move the selected material into an ignored, task-specific archive below
  `.superpowers/worktree-archives/<task>/` in the local main checkout, retaining
  relative paths. Record an inventory with each retained item's reason and a
  brief account of discarded categories. Archives stay outside Git. Inspect
  unclassified material rather than indefinitely retaining it wholesale; if its
  value or ownership remains uncertain, preserve that limited material and
  report the uncertainty without retaining the entire completed worktree.
- Run `pwsh -NoProfile -File scripts/cleanup_local.ps1` at task resumption and
  after integration to audit IFCCAD and separate codec worktrees/dependency
  checkouts. Use `-Sizes` at task retirement to account for cache and other
  target content; sizes are informational, with no hard budgets or build stops.
  Explicitly select the completed task with `-Apply
  -RemoveWorktree <name>` after relocating local material; the script must not
  force deletion of changes or unknown ignored content. Do not clean caches in
  other active worktrees. `-CleanBuildCache` selects only main by default; select
  another checkout explicitly with `-CacheCheckout <name-or-key>`. After task
  retirement, also account for its main/dependency caches: clean those no longer
  needed, or record the concrete ongoing task that still needs their warm cache.
  Apply the same completion cleanup to codec tasks after integration into their
  own main line; never compare their commits against IFCCAD main. Primary codec
  repositories need separate review before removal because they can own shared
  Git metadata or selected dependencies. See `docs/local-maintenance.md`.
  If cleanup remains blocked, report the exact blocker,
  last commit date and preserved-material location, and resolve recoverable file
  blockers rather than silently retaining the entire worktree.
- Keep work in one task by default. Do not delegate to subagents unless the
  user explicitly requests parallel work.
- Do not use `codex/` or a redundant `ifccad/` prefix in branch names.
- Preserve unrelated user changes and existing uncommitted work.
- Avoid ambiguous public type names when domain or direction context is needed
  to understand a selective import.
- Update nearby documentation and coverage contracts when behavior,
  architecture, or supported source semantics change.
- Keep the shorter roadmap summary in `README.md` synchronized whenever
  milestone names, order, or intent change in `ROADMAP.md`.
- Treat generated files below `target/` as local artifacts. Do not add them to
  version control. Put source material, issue attachments, and lasting design
  notes outside `target/`; retain accepted benchmark reports and matching
  detailed results as documented. The cleanup script removes only recognized
  build-cache entries with `-Apply -CleanBuildCache` and reports other `target/`
  content for review. Run cache cleanup only when no build or test process is
  using those directories.
- When updating opencadcodec, review public-model changes against the active IFCCAD converter
  coverage contract, including added fields on existing types. Keep local
  codec patches explicit and tied to a base revision; do not edit the Cargo
  cache or treat patched results as evidence for the unmodified dependency.

## Verification

Use focused tests while developing. A worktree may be ready for integration
with focused checks recorded and full verification explicitly pending; do not
describe that state as fully verified or complete.

Before reporting changes to Rust code, schemas, or conformance fixtures as
complete, run the full gate on the final integration candidate:

```text
python scripts/check_active_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Prepare the candidate against the intended current main, resolve conflicts, and
run relevant regressions before the full gate. When several tasks are explicitly
authorized for combined integration, verify their final combined candidate once;
do not require a separate full gate for every intermediate integration step.
Combining tasks or preparing a candidate does not authorize merging or pushing.

Reuse a successful full gate when transferring an unchanged verified candidate
to main. First compare all verification inputs: source, manifests, tests, schemas,
fixtures, examples, relevant scripts/configuration, Cargo.lock, selected external
dependency contents/patches, toolchain, target, features and relevant environment.
A matching commit or clean tracked diff alone is insufficient: ignored local
configuration/dependencies and untracked inputs also matter. Record the comparison
and the original evidence; if equivalence cannot be established, rerun the gate.
Content changes from conflict resolution, main advancement or dependency/config
changes require fresh verification of the resulting candidate. Changes only to
prose with no build/test inputs affected do not invalidate Rust verification.

Record the verified commit/tree (or a content manifest including dirty/untracked
inputs), dependency/lock/config identity, tool versions and relevant environment,
commands, exit codes, durations and log paths in the task's local verification
summary. Old logs without this provenance remain historical evidence, not a
reusable passing gate. A failed or incomplete run never counts as a pass.

Fix a failed gate with focused reproduction first, then rerun the full gate once
the final candidate is ready. Do not repeatedly run the workspace suite against
an unchanged known failure. Use `--no-fail-fast` only when collecting the remaining
failures is needed, not as the default integration command.

When editing public Rust documentation or examples, use
`cargo test --doc --workspace` as a focused intermediate check. An unrestricted
successful `cargo test --workspace` already includes enabled library doctests;
do not repeat them immediately without a change or missing coverage to verify.
Prose-only documentation/instruction changes require diff and applicable link/
content checks, not Rust builds, unless the changed files are build/test inputs.

Keep Cargo target directories, profiles, incremental settings and dependency
selection stable within a task. Preserve warm per-worktree caches; do not switch
to a shared target directory or change build settings during integration merely
to try to speed up a run. Coordinate expensive full workspace runs across active
tasks to avoid competing builds; retain focused checks while developing.
Frontend or other checks required by the actual change remain required.

New writer or converter output must be loaded through the production reader and
pass strict drawing validation. Conversion tests should compare semantic
content rather than unstable handles or serialized byte layouts, except where
byte determinism is itself the contract under test.

Controlled size/exchange measurements are manual research tools, never automatic
completion gates. Run or rerun a measurement, including any selected subset, only
when the user explicitly requests it. Changes to main, the corpus, writers, codecs,
converters or dependencies do not themselves authorize a rerun. Do not add automatic
measurement runs to CI or development workflows. Focused correctness, strict-readback
and conversion tests remain required independently of size measurements.
When the user requests a measurement, use the documented dependency configuration
and a fresh run directory, and run only the requested scope. Run different Cargo
configurations sequentially because they share Cargo.lock. Retain the last accepted
report and its matching detailed result JSON until a new applicable run replaces
them; identify their recorded coverage and provenance rather than implying they
measure every later revision. Add representative recipes for new semantic families
only within an explicitly requested measurement scope.
Compression probes explain observations; add compressed output as a formal
variant only when production encoding and readback exist.

## Repository authority

- Do not commit, push, merge, publish, or create a release unless the user
  explicitly asks.
- A request to implement or verify a change does not by itself authorize any of
  those repository operations.
- Do not commit credentials, tokens, local absolute paths, or generated review
  artifacts.
