# AGENTS.md

## Project context

IFCCAD is an open exchange format that combines IFC-style project and building
semantics with CAD drawings in an IFCX-based package architecture.

An IFCCAD package contains an IFCX semantic graph, one or more IFCDR drawing
resources, and optionally IFCPR resources for source information that cannot
yet be represented natively without loss.

Start with:

- `README.md` for the project purpose, package architecture, current
  capabilities, roadmap summary, and repository layout;
- `ROADMAP.md` for authoritative development sequencing, milestone status,
  dependencies, and exit criteria;
- `docs/vision.md` for long-term use cases, design principles, and success
  criteria;
- `crates/ifccad-convert/README.md` for conversion terminology and the boundary
  between IFCCAD and cadcodec `CadDocument`;
- `crates/ifccad-convert/src/export/COVERAGE.md` for the pinned cadcodec export
  coverage and loss-classification contract.

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

Open GitHub issues provide roadmap and future-design context, but are not
normative requirements. Before designing changes that affect format
architecture, schema evolution, physical encoding, preservation, or conversion
coverage, review the relevant open issues and identify overlaps or conflicts.
Do not expand the current task merely because a related issue exists.

## Architecture boundaries

- The core `ifccad` crate owns the format model, package validation, package
  construction, IFCDR encoding, and directory-package storage.
- The core crate must remain independent of cadcodec and other CAD runtimes.
- The `ifccad-convert` companion crate owns conversion between validated IFCCAD
  drawings and cadcodec `CadDocument`.
- Keep conversion, logical package construction, physical encoding, and
  filesystem storage as separate responsibilities.
- Do not silently approximate or discard source semantics. Represent them,
  diagnose the loss, preserve them through an approved preservation mechanism,
  or reject structurally inconsistent input.
- Keep IFCX extension points open where the schemas intentionally permit
  additional fields or unknown node types.
- Numbered conformance collections are immutable. Develop contract changes in
  the active schemas and `conformance/next`; never modify a released collection
  in place.

## Working conventions

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
  version control.

## Verification

Use focused tests while developing.

Before reporting a change to Rust code, schemas, or conformance fixtures as
complete, run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

When editing public Rust documentation or examples, use
`cargo test --doc --workspace` as a focused intermediate check.

New writer or converter output must be loaded through the production reader and
pass strict package validation. Conversion tests should compare semantic
content rather than unstable handles or serialized byte layouts, except where
byte determinism is itself the contract under test.

## Repository authority

- Do not commit, push, merge, publish, or create a release unless the user
  explicitly asks.
- A request to implement or verify a change does not by itself authorize any of
  those repository operations.
- Do not commit credentials, tokens, local absolute paths, or generated review
  artifacts.
