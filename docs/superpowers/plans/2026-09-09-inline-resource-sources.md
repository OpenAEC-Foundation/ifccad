# Inline Resource Sources Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan in this task. Repository conventions prohibit subagents unless the user explicitly requests them. Track progress with the checkboxes below.

**Goal:** Read inline and external IFCDR/IFCPR through one resource pipeline and let the writer select IFCDR storage explicitly.

**Architecture:** Separate resource identity, source selection and diagnostic origin in package code. Retain shared resource validation and keep storage choices outside the logical IFCDR model. Both filesystem loading and in-memory writer validation must use the same source resolution rules.

**Tech Stack:** Existing Rust workspace, serde_json, jsonschema and SHA-256; no new dependencies.

**Spec:** [Approved inline resource design](../specs/2026-09-09-inline-resource-design.md).

## Global constraints

- Create branch `inline-resource-sources` from current main when execution starts; preserve the uncommitted approved spec and this plan. Use the existing checkout, as preferred by the user.
- IFCX overlay 0.9.0; drawing core 0.2.0; IFCDR registry/mapping 0.7.0; IFCPR 0.2.0; candidate suite 1.1.0.
- External source requires uri/checksum and forbids content. Inline requires object content and forbids uri/checksum, including null.
- IFCDR descriptor is attributes.resource; IFCPR descriptor is attributes.preservation.
- Preserve opaque ResourceIds and all entity IDs and logical relationships.
- Keep historical schemas and conformance/1.0.0 unchanged. Historical-schema cleanup is a separate task.
- No binary content, container, automatic threshold, arbitrary-file opening API, new entity, or expanded IFCPR semantic validation.
- No commit, push or merge without separate authorization. Checkpoints below are review points, not instructions to commit.

## File responsibilities

- `schemas/ifcx/ifccad-overlay-0.9.0.json`: descriptor source alternatives.
- `schemas/ifcx/resource-source-contract-0.9.0.md`: normative source, identity and origin rules; references drawing-resource-contract-0.8.0.md for relationships.
- `src/package/read/source.rs` (new): source identity and diagnostic origin; no filesystem I/O or semantic validation.
- `src/package/read/discovery.rs`: source choice and descriptor locations.
- `src/json_resource.rs`: represent parsed content without fabricated paths or bytes.
- `src/package/read/model.rs`, `loader.rs`, `validation.rs`, `bindings.rs`: resolve sources, validate once and bind by ResourceId.
- `src/ifcdr/read/resource.rs`: source-neutral reader backing, retaining the typed logical model.
- `src/package/write/types.rs`, `state.rs`, `builder.rs`, `ifcx.rs`: per-drawing storage option and output assembly.
- `src/package/mod.rs`, `src/package/read/mod.rs`, `src/package/write/mod.rs`: module registration and public writer enum reexport.
- `tests/inline_resources.rs` (new): production load cases and equivalence across source forms.
- `tests/ifcx_overlay_schema.rs`, `tests/package_writer_roundtrip.rs`, `tests/package_writer_filesystem.rs`: schema and writer coverage.
- `conformance/next`, its asset/manifest tests and nearby documentation: language-neutral evidence and supported profile.

## Task 1: Source contract and schema

- [x] Add a 0.9.0 validator helper in `tests/ifcx_overlay_schema.rs`, registering drawing-core 0.2.0 through the existing offline Registry pattern.
- [x] Add table-driven tests for drawing and preservation descriptors: valid external; valid object content; neither source; both; null/string/array content; external missing checksum; inline checksum string/null. Start from the existing complete valid package document and mutate only one descriptor per case.
- [x] Run `cargo test --test ifcx_overlay_schema overlay_0_9`; expect failure because the new schema is absent.
- [x] Copy overlay 0.8.0 to 0.9.0, update ID/title and use this source-choice shape in both descriptors. Keep common metadata outside the alternatives and keep additionalProperties false:

```json
{
  "properties": {"content": {"type": "object"}},
  "oneOf": [
    {"required": ["uri", "checksum"], "not": {"required": ["content"]}},
    {"required": ["content"], "not": {"anyOf": [
      {"required": ["uri"]}, {"required": ["checksum"]}
    ]}}
  ]
}
```

- [x] Remove uri/checksum from the shared required arrays, retaining their existing property validators. Do not validate resource bodies twice by embedding their schemas here.
- [x] Write the normative source contract from the approved spec: exclusive source alternatives, byte checksums, identity rules, body validation and diagnostic origins. Explicitly retain 0.8.0 drawing relationships.
- [x] Rerun the focused schema tests and historical overlay tests. Do not switch the production reader until source resolution is implemented.

## Task 2: Source and origin representation

**Internal interface:** Add these types in `src/package/read/source.rs`. Use the source key for caches; never expose it as ResourceId.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ResourceSourceKey {
    External(String),
    Inline { document_uri: String, pointer: String },
}

pub(crate) struct ResourceOrigin {
    pub(crate) document_uri: String,
    pub(crate) pointer: String,
}
```

External origin has its file URI and empty pointer. Inline origin has package.ifcx.json and the content pointer. Add `ResourceOrigin::locate(&self, local: Option<&str>) -> String`: absent/empty local returns the origin pointer, otherwise concatenate the base with the resource-local JSON Pointer.

- [x] Write unit tests for empty/external origin, inline root and `/header/resourceId`. Example expected result:

```rust
assert_eq!(origin.locate(Some("/header/resourceId")),
    "/data/3/attributes/resource/content/header/resourceId");
```

- [x] Run `cargo test --lib package::read::source` to observe the missing implementation, then implement and rerun.
- [x] Replace ResourceDeclaration's mandatory external URI with source key plus explicit descriptor/source-field locations. Keep checksum optional internally, but accepted declarations must satisfy the source rules.
- [x] Add discovery tests rejecting ambiguous sources without choosing a URI or constructing a missing-file error. Presence of fields matters, including null values. Preserve retired vocabulary detection.
- [x] For invalid descriptors, leave primary schema diagnostics to the overlay; do not emit a usable declaration for an invalid alternative. Preserve independently reportable metadata errors.
- [x] Refactor LoadedJsonResource to distinguish external file-backed content from inline parsed content. Keep external bytes available for checksums; inline construction accepts a Value directly and no fake path or byte buffer. Update test constructors and IFCDR reader backing consumers together.
- [x] Run `cargo test --lib`; existing external-only behavior must remain green before wiring inline packages.

## Task 3: Shared loading and validation

- [x] Add `tests/inline_resources.rs` helpers that copy existing valid candidate packages to isolated test directories. To inline one descriptor, remove uri/checksum and assign parsed resource JSON to content. Keep resource and entity IDs unchanged.
- [x] Add a production loading test using a minimal package whose IFCDR file is absent and content is inline; assert a strict package, expected entities and external_uri() == None. Run `cargo test --test inline_resources` and observe failure before switching production behavior.
- [x] In loader/validation/model, cache by ResourceSourceKey. External sources call the existing bounded loader; inline sources retrieve the object from the entrypoint pointer. Do not serialize/reparse that object. Extract common source resolution used by both load_directory_package and validate_encoded_package; only their external-byte providers differ.
- [x] Switch `src/package/read/schema.rs` to overlay 0.9.0. Keep external checksum verification on stored exact bytes. Validate source content through existing IFCDR and IFCPR validators.
- [x] Replace URI-keyed semantic identity, IFCPR-target lookup and descriptor-error deduplication with declaration/source identity. Preserve typed resource maps keyed by ResourceId and existing unavailable-target suppression.
- [x] Update `bindings.rs` kind-conflict and declaration-order handling. Same ID/kind/external URI may repeat; same ID with different inline pointers, mixed sources or different kinds must report duplicate identity. Different IDs pointing to one external file retain header mismatch checks.
- [x] Translate body diagnostic origins once at the package boundary, including IFCPR link errors. Descriptor diagnostics retain their existing entrypoint pointer; do not prefix them with content a second time.
- [x] Add source-pair tests (external/external, inline/inline, both mixed combinations), malformed bodies, mismatched IDs, unknown IFCDR versions/streams and missing preservation targets. Compare codes/severity and exact pointers, not entire rendered diagnostic messages.
- [x] Assert inline and external semantic snapshots match: units, bounds, scopes, entity IDs/order, geometry, visibility, layers and appearance. Include the shared model/paper-layout fixture.
- [x] Add small configured-limit tests inside loader tests: entrypoint bytes count once, exact total limit succeeds, one-byte-too-small fails, and inline retrieval does not increase loaded-byte accounting. Keep per-file limits unchanged.
- [x] Run `cargo test --test inline_resources` and `cargo test --lib package::read`; fix regressions before proceeding.

## Task 4: Explicit writer storage choice

**Public interface:** In write/types.rs, reexported through package:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawingResourceStorage {
    #[default]
    External,
    Inline,
}
```

Add `DrawingBuilder::set_resource_storage(&mut self, storage: DrawingResourceStorage)` returning (). Store the option in DrawingState, initialized to External; do not add a required DrawingOptions field.

- [x] Add a writer test that selects Inline, finishes the package and asserts files().len() == 1, absence of resources/drawing.ifcdr.json and presence of attributes.resource.content without uri/checksum. Observe failure with `cargo test --test package_writer_roundtrip`.
- [x] Implement the enum, state and setter. Adapt assembly so both choices consume the same validated resource/JSON mapping; inline embeds the encoded object. Prefer passing the JSON Value from the codec assembly to avoid encode-then-parse just for embedding, while keeping the codec independent of package storage policy.
- [x] Keep external filename and byte-checksum behavior unchanged. Ensure finish() runs production in-memory package validation for either choice.
- [x] Test default and explicit External equivalence; repeated Inline determinism; explicit External output matches the default. Preserve supplied IDs and semantic content across modes.
- [x] Extend filesystem tests to reload the one-file inline directory with load_directory_package. Extend converter roundtrip coverage to import that validated drawing and compare existing supported semantics.
- [x] Update public API docs, replacing stale external/model-space-only descriptions. Run `cargo test --doc --workspace` and focused writer/converter tests.

## Task 5: Candidate conformance and documentation

- [x] Copy overlay 0.9.0 and resource-source-contract-0.9.0.md into conformance/next; replace only the candidate overlay 0.8.0 copy. Keep drawing-resource-contract-0.8.0.md and drawing-core 0.2.0.
- [x] Add persistent fixtures for the four IFCDR/IFCPR source combinations, duplicate inline identity, simultaneous sources, inline checksum and malformed inline body. Derive from current fixtures without changing body semantics or checksums of remaining external files.
- [x] Register exact expected diagnostics in manifest.json. Keep the existing five deferred IFCPR cases deferred; no new preservation validity claims. Update manifest count/order assertions and candidate asset equality checks.
- [x] Run `cargo test --test ifccad_package_conformance`, `cargo test --test ifccad_conformance_assets` and `cargo test --test ifccad_manifest_loading`.
- [x] Update README, converter README, compatibility/provenance and ROADMAP. Explain per-file limits for inline entrypoints, writer default/choice, absence of inline checksums and limited IFCPR checking. Record inline normalization complete only after verification; reporting and measurements remain milestone 2 work.

## Task 6: Final review and verification

- [x] Review remaining external_uri, URI-keyed caches, LoadedJsonResource bytes/path access and checksum calls; each remaining use must be genuinely external-specific. Confirm logical access and CAD conversion have no storage branches.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo test --workspace`.
- [x] Check `git diff --check`, unchanged frozen assets and dependencies, and absence of generated target artifacts in tracked/untracked deliverables.
- [x] Mark actual plan progress and spec implementation status. Report validation evidence and any limitations; leave changes uncommitted until explicitly requested.

## Plan self-review

The source contract is covered by task 1; source/origin separation by task 2;
normalization, identities, IFCPR, diagnostics and limits by task 3; writer API
and production roundtrips by task 4; candidate/documentation requirements by
task 5; repository verification by task 6. Schema cleanup remains excluded.

## Execution record

Implemented in this task without subagents. The first production-reader test
failed on the old source contract before implementation; the writer API test
failed before adding the enum/setter. A duplicate-ID origin regression was
also reproduced before correcting per-source diagnostic translation. Schema
matrix and origin unit tests supplemented these integration-led red/green cycles.

Discovery retains existing external metadata-error reporting: the overlay
reports a missing/malformed checksum, while source discovery can still locate
an unambiguous external body for independent diagnostics. Competing alternatives
never select a source. This preserves existing external conformance outcomes.

Source-relative body diagnostics are translated at each validator boundary;
package binding diagnostics are translated separately. Inline content owns a
parsed JSON value without file bytes or a path; the typed IFCDR reader backing
and logical access remain unchanged. The encoder exposes its constructed JSON
value to package assembly so inline output needs no encode/reparse roundtrip.

Verification: 308 workspace tests passed, 1 existing test ignored; formatting, Clippy with warnings denied and all three doc tests passed. Historical contracts, frozen collections and dependencies are unchanged. No commit, merge or push performed.
