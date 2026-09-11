# JSON / DXF / DWG baseline implementation

> Execute in this task using executing-plans, without delegation. Implementation
> and commit/push of the working branch are authorized. Merging requires a
> separate user instruction.

**Goal:** Reproducible size accounting and exact, bounded semantic exchange checks.

**Architecture:** A development example in ifccad-convert owns recipes, adapters,
semantic projections and reporting. It reuses production models, codecs and
converters; approved follow-up fixes are recorded below. Every output is read
through its production reader.

**Tech stack:** Rust, pinned cadcodec, serde_json and sha2 development dependencies.

**Specification:** [Approved design](../specs/2026-09-11-json-size-baseline-design.md).

## Constraints

- Existing checkout and size-baseline branch; no subagents.
- Preserve conformance fixtures and existing converter loss classifications.
- Exact geometry/property comparisons; Reject on conversion losses.
- Separate direct size measurements from conversion-chain artifacts.
- Fresh run directories under target; preserve failed runs for diagnosis.
- Report incomplete results honestly; do not mark milestone complete on failure.

## Steps

- [x] Add a small writer/reader/converter preflight to expose DWG limitations
  before expanding the corpus. Run a focused test of geometry, layer appearance,
  visibility and Reject conversion. Retain structured failure evidence.
- [x] Add benchmark modules under examples/size_baseline/ for typed recipes,
  direct IFCCAD/CAD construction and semantic projections. Freeze nine cases
  and formulas in benchmarks/size/corpus-v1.json. Use integer and binary-fraction
  coordinates and fixed metadata.
- [x] Implement explicit fixture inventories, unique-file accounting, hashes,
  component sums and nullable ratios. Focused tests cover duplicate paths,
  missing files, overflow, inline counting and empty cases.
- [x] Implement all four direct outputs and both complete conversion chains,
  validating every boundary and retaining diagnostics and assessment gaps.
- [x] Implement two fresh runs, deterministic comparison and provenance,
  including a local copy of Cargo.lock. Test failure propagation and refusal
  to overwrite existing outputs.
- [x] Execute the corpus twice. Produce curated JSON and a readable Markdown
  report, or an explicitly incomplete diagnostic report if actual failures
  prevent a successful baseline. Link generated artifacts locally.
- [x] Review implementation and tests; run formatter check, Clippy for all
  workspace targets and workspace tests. Review roadmap exit criteria and
  update README/ROADMAP only to reflect demonstrated completion.

## Verification and delivery

The user approved consolidating the experiment into one final report and
omitting earlier diagnostic comparisons. The delivered measurements use
cadcodec 0.5.4 at `2f2cd25` with two local DWG fixes. Every controlled exchange
and repeatability check passes. Formatter, Clippy and 335 workspace tests pass;
one existing test is ignored. Six codec regression tests also pass.

The final report is `docs/benchmarks/size-baseline-v1.md`, with detailed data in
`benchmarks/size/results-v1.json` and the frozen `corpus-v1.json` inventory.
Patch setup is documented in `patches/cadcodec-upstream/README.md`. The measured
run and development diagnostics remain local under `target/size-baseline/`.
The report uses one DWG variant. Compression is an explanatory probe only;
production compressed output becomes a formal variant after implementation.

Core cached geometry extents are treated as derived values while drawing
limits remain diagnosed. The upstream public-model audit and import rounding
expectation are reflected in the converter coverage documentation/tests.
The shared pin still lacks the two local DWG fixes. At milestone closure the
documented patched run was accepted as the scoped initial measurement reference;
upstream integration remains separate codec maintenance. The recorded automatic
`baseline_accepted` flag is unchanged and does not claim a passing unpatched run.

## File map

- crates/ifccad-convert/examples/size_baseline.rs and size_baseline/: experiment.
- benchmarks/size/corpus-v1.json: recipes and fixture inventory.
- benchmarks/size/results-v1.json: complete successful measured run.
- docs/benchmarks/size-baseline-v1.md: the complete report.
- patches/cadcodec-upstream/: remaining DWG patches and reproduction setup.
- README.md and ROADMAP.md: current status.

## Verification commands

User-approved dependency update: pin the latest reviewed upstream commit
`2f2cd25832db298524fb5eb36ced5a438a877e95` (0.5.4), audit public source surfaces,
adapt the transparency quantization expectation, and verify new drawing-variable
loss reporting. Run the full workspace checks and both generations of the corpus
with the normal pin. Rebase and separately verify the remaining DWG patches;
retain the successful patched result without treating it as an accepted baseline.
Compact-JSON production output is not part of this update. Commit/push of the
working branch was subsequently authorized; merge remains separate.

Run focused example tests while developing, then the example with a new output
directory. Final checks:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
