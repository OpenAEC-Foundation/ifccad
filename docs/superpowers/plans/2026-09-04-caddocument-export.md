# CadDocument Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Export a cadcodec `CadDocument` into a valid in-memory IFCCAD package with exact initial geometry coverage, complete deterministic loss diagnostics, and reproducible DXF/IFCCAD chain artifacts.

**Architecture:** `ifccad` gains per-property IFCDR appearance bindings while continuing to own package construction. `ifccad-convert::export` classifies source data as emit, skip, or fatal and inserts accepted values directly into `PackageBuilder`; it retains only diagnostics, deduplication state, layer keys, and handle-to-entity mappings. Automated chains construct controlled documents in code and cross real cadcodec DXF read/write boundaries; separate manual acceptance runs use local Prototype samples.

**Tech Stack:** Rust 2021, `ifccad`, `ifccad-convert`, pinned cadcodec/acadrust revision `a0f7d444f1607bc4b2c881060cbe7ea1014253cb`, serde JSON, Cargo tests and clippy.

**Spec:** `docs/superpowers/specs/2026-09-04-caddocument-export-design.md`

## Global Constraints

- Work on `feature/caddocument-export`; do not use `codex`, `p24`, or an unnecessary `ifccad/` branch prefix.
- The public function is `cad_document_to_package(&CadDocument, PackageOptions, ExportOptions) -> Result<ExportOutcome, ExportError>`.
- Export one true model-space drawing and internal resource ID `drawing-main` only.
- Native geometry is limited to exactly representable `LINE` and straight `LWPOLYLINE` instances.
- No hidden approximation, coordinate rescaling, replacement layer, block explosion, paper-space export, IFCPR, ZIP codec, or schema terminology change.
- `ExportLossPolicy::Allow` is the default; `Reject` returns the complete safely detectable loss list and no package.
- Source handle numbers are not semantic by themselves; lost relationships or attached semantic content are loss.
- Produced packages must load strictly with zero package-validation diagnostics.
- Follow red-green-refactor for every behavior change and commit each independently reviewable task.

---

### Task 1: Per-property core appearance bindings

**Files:**
- Modify: `src/package/write/types.rs`
- Modify: `src/package/write/state.rs`
- Modify: `src/package/write/builder.rs`
- Modify: `src/package/write/mod.rs`
- Modify: `src/package/mod.rs`
- Modify: `src/ifcdr/write/mod.rs`
- Modify: `src/ifcdr/write/encoder.rs`
- Modify: `tests/public_package_builder.rs`
- Modify: `tests/package_writer_roundtrip.rs`

**Interfaces:**
- Produces `AppearanceMode::{ByLayer, Explicit, ByBlock}`.
- Produces `EntityAppearance { appearance, color_mode, opacity_mode, line_pattern_mode, line_weight_mode }` with `by_layer()`, `by_block()`, and `explicit(AppearanceKey)` constructors.
- `LineDefinition` and `PolylineDefinition` continue to accept `EntityAppearance` by value.
- `ModelSpaceBuilder::{add_line, add_polyline}` continue returning the final `EntityId`.

- [ ] **Step 1: Write the failing public builder tests**

Replace old enum construction and add a mixed binding case:

```rust
let mixed = EntityAppearance {
    appearance: Some(style),
    color_mode: AppearanceMode::ByLayer,
    opacity_mode: AppearanceMode::Explicit,
    line_pattern_mode: AppearanceMode::ByBlock,
    line_weight_mode: AppearanceMode::Explicit,
};
```

Assert that a foreign explicit `AppearanceKey` returns
`PackageBuildError::ForeignAppearanceKey`, that an explicit mode without a key
returns `PackageBuildError::AppearanceDefinitionMissing`, and that invalid
bindings do not advance entity IDs.

- [ ] **Step 2: Run the focused tests and verify RED**

Run: `cargo test --test public_package_builder --test package_writer_roundtrip`

Expected: compile failure because `AppearanceMode` and the structured
`EntityAppearance` do not exist.

- [ ] **Step 3: Implement the public binding types and writer state**

Use this public shape in `types.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceMode { ByLayer, Explicit, ByBlock }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityAppearance {
    pub appearance: Option<AppearanceKey>,
    pub color_mode: AppearanceMode,
    pub opacity_mode: AppearanceMode,
    pub line_pattern_mode: AppearanceMode,
    pub line_weight_mode: AppearanceMode,
}
```

Add an internal deduplicated `AppearanceBindingEntry` collection to
`DrawingState`. IDs 0 and 1 remain the reserved all-by-layer and all-by-block
bindings. Custom bindings start at 2; their optional IFCX appearance path is
resolved independently from `AppearanceKey` numbering. Store the resolved
`AppearanceId` in each `PendingEntity` so later encoding cannot disagree with
the mapping returned by `add_line`/`add_polyline`.

- [ ] **Step 4: Encode all four binding modes independently**

Extend `IfcdrAppearanceBindingInput` with an optional IFCX path and four
numeric mode fields. Encode mode values exactly as IFCDR 0.5 defines them:
ByLayer `0`, Explicit `1`, ByBlock `2`. Emit custom bindings in first-use order
and keep the two reserved bindings first.

- [ ] **Step 5: Run focused tests and verify GREEN**

Run: `cargo test --test public_package_builder --test package_writer_roundtrip`

Expected: all tests pass; strict reader assertions observe the mixed modes and
their explicit appearance reference.

- [ ] **Step 6: Run core regression tests and commit**

Run: `cargo test -p ifccad`

Commit: `feat(writer): support mixed appearance inheritance`

---

### Task 2: Export public contract

**Files:**
- Create: `crates/ifccad-convert/src/export/mod.rs`
- Create: `crates/ifccad-convert/src/export/options.rs`
- Create: `crates/ifccad-convert/src/export/outcome.rs`
- Create: `crates/ifccad-convert/src/export/entity_mapping.rs`
- Create: `crates/ifccad-convert/src/export/diagnostic.rs`
- Create: `crates/ifccad-convert/src/export/conversion.rs`
- Modify: `crates/ifccad-convert/src/lib.rs`
- Create: `crates/ifccad-convert/tests/public_export_api.rs`

**Interfaces:**
- Produces `ExportOptions`, `ExportLossPolicy`, `ExportOutcome`,
  `ExportDiagnostic`, `ExportDiagnosticSource`, `ExportAction`,
  `ExportLossReason`, `SourceStructureProblem`, `ExportEntityMapping`, and
  `ExportError` at crate root.
- `ExportOutcome` owns `EncodedPackage`, diagnostics, and mapping.

- [ ] **Step 1: Write the failing crate-root API test**

```rust
use ifccad_convert::{
    cad_document_to_package, ExportEntityMapping, ExportLossPolicy,
    ExportOptions, ExportOutcome,
};

#[test]
fn default_export_policy_allows_reported_loss() {
    assert_eq!(ExportOptions::default().loss_policy, ExportLossPolicy::Allow);
}
```

Also type-check the agreed function signature without invoking it and assert an
empty mapping's `len`, `is_empty`, `target_entity_id`, and ordered `iter` API.

- [ ] **Step 2: Run the API test and verify RED**

Run: `cargo test -p ifccad-convert --test public_export_api`

Expected: unresolved export symbols.

- [ ] **Step 3: Implement options, outcome, mapping, and errors**

Use these central shapes:

```rust
pub struct ExportOptions { pub loss_policy: ExportLossPolicy }
pub enum ExportLossPolicy { Allow, Reject }

pub enum ExportError {
    InvalidSourceStructure { problems: Vec<SourceStructureProblem> },
    LossRejected { diagnostics: Vec<ExportDiagnostic> },
    PackageBuild(#[from] ifccad::package::PackageBuildError),
    InternalInvariant { message: String },
}
```

Represent each diagnostic as one source item, one action, and a non-empty
ordered list of typed reasons. Provide `ExportDiagnostic::is_loss()`; it returns
true for all initial variants. Mapping storage is `BTreeMap<Handle, EntityId>`.
Mirror `ImportOutcome` with `package()`, `diagnostics()`, `entity_mapping()`,
`into_package()`, and `into_parts()`.

- [ ] **Step 4: Add a temporarily minimal function boundary**

Declare `cad_document_to_package` in `export::conversion` only after adding a
test that calls it with a structurally invalid document and expects
`InvalidSourceStructure`. Its first implementation may perform only that
preflight; it must not return a fabricated successful package.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p ifccad-convert --test public_export_api`

Commit: `feat(convert): define caddocument export contract`

---

### Task 3: Structural preflight, units, layers, and appearances

**Files:**
- Modify: `crates/ifccad-convert/src/export/conversion.rs`
- Create: `crates/ifccad-convert/src/export/structure.rs`
- Create: `crates/ifccad-convert/src/export/units.rs`
- Create: `crates/ifccad-convert/src/export/appearance.rs`
- Create: `crates/ifccad-convert/src/export/layers.rs`
- Modify: `crates/ifccad-convert/src/export/mod.rs`
- Create: `crates/ifccad-convert/tests/export_drawing.rs`

**Interfaces:**
- `inspect_model_space(&CadDocument) -> Result<ModelSpaceInfo<'_>, Vec<SourceStructureProblem>>` returns the model-space block handle and exactly related layout name.
- `map_length_unit(i16) -> (IfcdrLengthUnit, Option<ExportLossReason>)` never rescales coordinates.
- `add_layers(&CadDocument, &mut DrawingBuilder, &mut ExportContext)` fills a case-insensitive CAD-layer-to-`LayerKey` map.
- Appearance helpers return either an exact writer binding or typed reasons that prevent exact emission.

- [ ] **Step 1: Write failing structure and unit tests**

Cover one exact model-layout relationship, zero matches, multiple matches, null
or missing model block record, all seven supported units, and an unsupported
unit mapping to `Unitless` plus `UnsupportedUnit`.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p ifccad-convert --test export_drawing structure units`

Expected: export cannot yet build a drawing.

- [ ] **Step 3: Implement preflight and units minimally**

Use `header.model_space_block_handle` and `ObjectType::Layout.block_record` as
the authoritative relationship. Aggregate all independently detectable
preflight failures before returning. Copy the matched layout name exactly.

- [ ] **Step 4: Write failing layer and appearance tests**

Construct a document with layer `0`, an empty RGB layer, an invisible layer,
and entity appearances combining ByLayer/ByBlock/Explicit properties. Assert
source layer order, empty-layer retention, `off || frozen => visible=false`, ACI
index plus RGB preservation, true color, opacity, named line pattern, numeric
line weight, and binding deduplication. Add cases for unsupported layer state and
an unrepresentable required appearance.

- [ ] **Step 5: Implement layer and appearance conversion**

Insert every exactly representable layer and its explicit appearance. Bundle
unsupported auxiliary layer properties into one `PartiallyExported` diagnostic.
Skip a layer with an unrepresentable required appearance; do not invent a
fallback. Deduplicate `AppearanceDefinition` and structured entity bindings in
stable first-use order.

- [ ] **Step 6: Verify GREEN, strict-load the empty-geometry package, and commit**

Run: `cargo test -p ifccad-convert --test export_drawing`

Commit: `feat(convert): export drawing metadata and layers`

---

### Task 4: Exact entities, mapping, and loss policy

**Files:**
- Create: `crates/ifccad-convert/src/export/entities.rs`
- Modify: `crates/ifccad-convert/src/export/conversion.rs`
- Modify: `crates/ifccad-convert/src/export/diagnostic.rs`
- Create: `crates/ifccad-convert/tests/export_entities.rs`
- Create: `crates/ifccad-convert/tests/export_loss_policy.rs`

**Interfaces:**
- `classify_entity(&EntityType, ModelSpaceInfo, &ExportContext) -> EntityDecision` returns `EmitLine`, `EmitPolyline`, `Skip`, or `Fatal`.
- Emission records the returned `EntityId` against the source `Handle` immediately.
- `LossRejected` owns the same ordered diagnostics an `Allow` run would return.

- [ ] **Step 1: Write failing exact line/polyline tests**

Assert emission for finite 2D zero-thickness positive-unit-Z lines and for
straight zero-width zero-elevation lightweight polylines. Assert whole-entity
skip with bundled reasons for nonzero Z, thickness, unsupported normal, bulge,
vertex/constant width, PLINEGEN, too few vertices, non-finite values, and
unrepresentable appearance.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p ifccad-convert --test export_entities`

Expected: no entity classifier or emitted mappings.

- [ ] **Step 3: Implement exact entity classification and emission**

Require `common.owner_handle == model_space_block_handle`. Diagnose paper-space,
block-owned, missing/unknown ownership, and every unsupported entity type.
Never explode or approximate. Visit cadcodec storage order, assign sequential
IFCDR IDs only to emitted entities, and populate `ExportEntityMapping` from the
IDs returned by `ModelSpaceBuilder`.

- [ ] **Step 4: Write failing policy tests**

Build one document containing multiple independently lossy items. Run `Allow`
and retain its ordered diagnostics. Run `Reject` and assert its
`LossRejected.diagnostics` equals the complete `Allow` list and that no package
is returned. Assert a fully supported document returns zero diagnostics.

- [ ] **Step 5: Implement the policy gate and verify GREEN**

Apply the gate only after the safe content scan. Return fatal structural errors
before the loss-policy decision. Let unexpected builder/capacity/encoding errors
stop immediately as `PackageBuild`, never as diagnostics.

Run: `cargo test -p ifccad-convert --test export_entities --test export_loss_policy`

- [ ] **Step 6: Commit**

Commit: `feat(convert): export exact model-space entities`

---

### Task 5: Pinned semantic coverage contract

**Files:**
- Create: `crates/ifccad-convert/src/export/COVERAGE.md`
- Create: `crates/ifccad-convert/src/export/coverage.rs`
- Modify: `crates/ifccad-convert/src/export/conversion.rs`
- Modify: `crates/ifccad-convert/src/export/diagnostic.rs`
- Create: `crates/ifccad-convert/tests/export_coverage.rs`

**Interfaces:**
- Coverage is explicitly pinned to cadcodec revision `a0f7d444f1607bc4b2c881060cbe7ea1014253cb`.
- `scan_document_semantics` adds deterministic document/table/object/common-entity loss reasons not handled by native geometry conversion.

- [ ] **Step 1: Write failing representative coverage tests**

Exercise non-default header semantics, non-layer tables, block definitions,
layouts/paper space, objects, XDATA, reactors, extension dictionaries, materials,
visual styles, plot styles, and named/color-book identity. Assert one bundled
diagnostic per source item and stable summary diagnostics for unsupported
collections. Assert bare handle-number replacement is absent from diagnostics.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p ifccad-convert --test export_coverage`

Expected: semantic source content is not yet fully diagnosed.

- [ ] **Step 3: Write the complete pinned coverage matrix**

For every public `CadDocument` collection, every `EntityCommon` field, every
layer field, the relevant header fields, and every `EntityType`/`ObjectType`
variant at the pinned revision, record one of `Exact`, `PartialLoss`,
`SkippedLoss`, `NonSemantic`, or `FatalIfInconsistent`. Include the exact
diagnostic reason or invariant for every non-exact row and link cadcodec #30.

- [ ] **Step 4: Implement the coverage scan and verify GREEN**

Use exhaustive matches where Rust exposes enums. Where cadcodec exposes open
collections or private internals, use explicit counts/field checks and document
the bounded guarantee. Do not inspect or serialize raw private data as a proxy
for semantic coverage.

Run: `cargo test -p ifccad-convert --test export_coverage --test export_loss_policy`

- [ ] **Step 5: Commit**

Commit: `feat(convert): report pinned cad semantic coverage`

---

### Task 6: Automated chains and review artifact generator

**Files:**
- Create: `crates/ifccad-convert/tests/support/mod.rs`
- Create: `crates/ifccad-convert/tests/support/documents.rs`
- Create: `crates/ifccad-convert/tests/support/artifacts.rs`
- Create: `crates/ifccad-convert/tests/export_chains.rs`
- Create: `crates/ifccad-convert/tests/export_chain_artifacts.rs`
- Modify: `crates/ifccad-convert/Cargo.toml`

**Interfaces:**
- Test builders produce `supported_model_space_document()` and `loss_heavy_document()`.
- An ignored artifact test writes deterministic outputs below
  `target/chain-artifacts/`; optional manual sample paths come from the
  task-specific `IFCCAD_MANUAL_SAMPLES` environment variable.
- Manual artifact metadata records source path, SHA-256, and diagnostics.

- [ ] **Step 1: Write the failing supported and loss-heavy chain tests**

For both builders run `CadDocument -> DXF bytes -> CadDocument -> IFCCAD ->
CadDocument -> DXF bytes -> CadDocument`. Compare semantic projections rather
than handles or bytes. Assert no export diagnostics for supported content; for
loss-heavy input assert the exact complete `Allow` list and equivalent `Reject`
list. Strict-load every produced package with zero package diagnostics.

- [ ] **Step 2: Write the failing IFCCAD-originated chain test**

Start at `bundled_conformance_root()/packages/valid/minimal-no-preservation`,
then run `IFCCAD -> CadDocument -> DXF -> CadDocument -> IFCCAD` and compare the
strictly loaded final drawing projection with the initial projection.

- [ ] **Step 3: Verify RED, then implement shared builders/projections**

Run: `cargo test -p ifccad-convert --test export_chains`

Expected RED: missing shared chain builders and/or remaining conversion gap.
Implement only test support needed to make the real chain assertions pass.

Add a determinism assertion that exports the same reloaded source twice with
identical options and compares the ordered diagnostics, entity mapping, file
paths, and bytes returned by `EncodedPackage::files()`.

- [ ] **Step 4: Add the artifact writer test-first**

First add a regular test that invokes `support::artifacts::write_chain_artifacts`
in a temporary root and asserts `source.dxf`, `ifccad/package.ifcx.json`,
`roundtrip.dxf`, and `diagnostics.txt`. Verify it fails before implementing the
helper. Then add an ignored `write_review_artifacts` test that invokes the same
helper for `target/chain-artifacts/` and refuses to overwrite an existing
artifact root. Add `sha2 = "0.10"` as a dev-dependency for manual source hashes.

- [ ] **Step 5: Verify automated chains and generate controlled artifacts**

Run: `cargo test -p ifccad-convert --test export_chains`

Run: `cargo test -p ifccad-convert --test export_chain_artifacts -- --ignored --nocapture`

Expected: supported, loss-heavy, and minimal-no-preservation subdirectories in
`target/chain-artifacts/`.

- [ ] **Step 6: Profile and run real Prototype samples**

Use the ignored test's read-only inspection/export path on the local sample root.
Choose at least one successfully decoded simple DXF and one successfully decoded
semantically rich DXF/DWG by typed content. Start with
`nextgis/line_2013.dxf` and `libre/sample_2018.dxf`, but replace either only when
the recorded profile shows it does not satisfy its role. Write selected outputs
under `target/chain-artifacts/manual/` and record exact source paths and SHA-256.

- [ ] **Step 7: Commit**

Commit: `test(convert): cover dxf ifccad export chains`

---

### Task 7: Documentation and full verification

**Files:**
- Modify: `crates/ifccad-convert/README.md`
- Modify: `crates/ifccad-convert/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-09-04-caddocument-export-design.md` only if implementation reveals an explicitly approved correction

**Interfaces:**
- Documents import/export/encode/write terminology, exact native coverage,
  policy behavior, diagnostics, mapping, and separate storage.

- [ ] **Step 1: Add public documentation examples and verify them**

Show `PackageOptions`, `ExportOptions::default()`, `ExportLossPolicy::Reject`,
diagnostic inspection, entity mapping, `into_package()`, and the subsequent
`write_directory` call. State that filesystem failures are `PackageWriteError`,
not `ExportError`.

Run: `cargo test --doc --workspace`

- [ ] **Step 2: Run formatting and lint verification**

Run: `cargo fmt --all -- --check`

Run: `cargo clippy --workspace --all-targets -- -D warnings`

- [ ] **Step 3: Run the complete workspace suite**

Run: `cargo test --workspace`

Expected: all unit, integration, public API, strict package, and chain tests pass
without warnings.

- [ ] **Step 4: Regenerate final review artifacts and inspect the manifest**

Run the ignored artifact test once after the final tests. Confirm every reported
start/intermediate/end path exists, every final IFCCAD package strict-loads, and
the manual diagnostics match the visibly expected omissions.

- [ ] **Step 5: Commit**

Commit: `docs: document caddocument export workflow`

- [ ] **Step 6: Prepare the user handoff**

Provide clickable absolute links to the controlled source and roundtrip DXFs,
the generated IFCCAD directories, the selected Prototype source files and their
roundtrip DXFs, and the IFCCAD-originated start/end packages. Summarize intentional
loss per lossy file so the OCS visual comparison has an explicit expectation.
