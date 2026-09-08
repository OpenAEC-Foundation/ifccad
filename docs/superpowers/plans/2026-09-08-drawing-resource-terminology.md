# Drawing Resource Terminology Implementation Plan

> Execute with `superpowers:executing-plans` in this task. Repository conventions prohibit delegation. Do not commit, push or merge without new authorization.

**Goal:** Apply the approved drawing vocabulary and enforce representation consistency within each Drawing.

**Architecture:** IFCX discovery, local schema checks and graph validation own this change. IFCDR logical data and codecs retain their contract. Package navigation and construction adopt the new names together.

**Tech Stack:** Existing Rust workspace, serde_json, jsonschema, JSON conformance assets; no additional dependency.

**Spec:** [Approved design](../specs/2026-09-08-drawing-resource-terminology-design.md).

## Constraints

- Work on `drawing-resource-terminology`, based on `83fcdb8`.
- IFCX overlay becomes 0.8.0. Drawing core and IFCPR remain 0.2.0; IFCDR registry/mapping remain 0.7.0. Candidate suite stays 1.1.0.
- Preserve historical schemas and frozen conformance collections.
- Keep source identity opaque and separate from locations. Change sample IDs and default output filename, never rewrite user-supplied identities while loading.
- Model/paper layout kinds and ModelSpace scope names retain their meanings. No paper export, inline resources, new CAD entities or preservation functionality.
- Unrelated IFCX extensions stay open. Recognized retired representation nodes block strict loading explicitly.

## 1. Contract and graph tests

- [x] Add failing schema tests in `tests/ifcx_overlay_schema.rs` for the 0.8 overlay, `DrawingRepresentation`, `attributes.resource`, role drawing and missing descriptors; retain historical test helpers.
- [x] Add focused graph tests in `src/package/read/graph.rs` for valid shared representation, wrong representation, same-resource/different-node mismatch, and suppression after missing/wrong-kind references.
- [x] Add discovery tests in `src/package/read/discovery.rs` for explicit retired-vocabulary diagnostics while unknown extension nodes remain accepted.
- [x] Observe failures before implementing behavior.

## 2. Switch production vocabulary and rules

- [x] Add `schemas/ifcx/ifccad-overlay-0.8.0.json` and a normative `drawing-resource-contract-0.8.0.md`. Keep the drawing-core 0.2 reference. Require the drawing role.
- [x] Rename resource discovery, graph target types, binding maps and navigation types/helpers under `src/package/read`; update public reexports in `src/package/mod.rs` and `read/mod.rs`.
- [x] Graph validation compares valid Drawing/layout representation paths and reports `IFCCAD_PACKAGE_BINDING_INVALID` with relationship context. Check node types before emitting equality errors.
- [x] Discovery reports `IFCCAD_PACKAGE_VOCABULARY_UNSUPPORTED` for the retired node type, even when no Drawing references it.
- [x] Update `src/package/write/ifcx.rs` default filename/descriptor and related builder constants. Preserve IFCDR version 0.7.0 and caller identities.
- [x] Rename code/test consumers together. Historical schema tests must still exercise their historical vocabulary.

## 3. Candidate and production roundtrips

- [x] Migrate active package fixture descriptor keys/node types/roles, renew sample identities and linked IFCPR IDs consistently if changed, and recompute resource checksums while preserving deliberate negative cases.
- [x] Replace only the candidate overlay copy. Copy the normative package contract alongside it.
- [x] Add valid shared model/paper package, invalid differing representation package, and unsupported retired-vocabulary package to the candidate manifest.
- [x] Test production typed navigation across two layouts, arbitrary resource IDs/URIs, writer determinism and converter semantic roundtrips.
- [x] Update asset/manifest assertions and retain deferred IFCPR cases unchanged.

## 4. Review, documentation and verification

- [x] Update README, converter README, compatibility/provenance, spec status and ROADMAP progress without changing milestone order. Record remaining inline/reporting/measurement work.
- [x] Review all old-name search hits: only historical schemas/docs or explicit retired-vocabulary tests should retain the old node/API names. Real geometry concepts and ModelSpace scopes remain.
- [x] Run `cargo test --doc --workspace` while changing public API consumers.
- [x] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] Check `git diff --check`, unchanged frozen assets and no generated artifacts. Record actual results and leave changes uncommitted.

## Verification results

- Workspace tests: 295 passed, 1 existing ignored test, zero failures.
- Formatting check and workspace Clippy with warnings denied passed.
- All three workspace doc tests passed.
- Frozen conformance, historical schemas, IFCDR/IFCPR contracts and dependencies unchanged.
- Implementation verified on `drawing-resource-terminology`; the user subsequently authorized committing and integrating it into `main`.
