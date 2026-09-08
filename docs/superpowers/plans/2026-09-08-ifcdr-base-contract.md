# IFCDR Base Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking. Execute sequentially in this task; AGENTS.md prohibits delegation unless the user explicitly requests it.

**Goal:** Establish the reduced IFCDR 0.6.0 JSON reference baseline before separating the logical drawing model from its encoding.

**Architecture:** Keep the existing JSON reader/writer structure during this reduction. Switch its active registry and overlay, renew candidate fixtures, remove prototype-only entity paths, and make unsupported-content diagnostics block strict views without false dependent errors. Retain the existing limited IFCPR checks and the CAD conversion fidelity boundary.

**Tech Stack:** Rust workspace, serde_json, JSON Schema through jsonschema, SHA-256 through sha2, versioned JSON conformance assets.

**Spec:** [Approved design](../specs/2026-09-08-ifcdr-base-contract-design.md).

## Global Constraints

- Active IFCDR registry: `0.6.0`.
- IFCDR resource schema ID: `ifccad.ifcdr.resource.v0.6.0`.
- IFCX composite overlay: `0.6.0`.
- IFCX drawing core: `0.2.0`.
- IFCDR registry meta-schema: `ifccad.ifcdr.registry.v1`.
- IFCDR stream directory: `ifccad.ifcdr.streamDirectory.v1`.
- IFCPR: `0.2.0`.
- Conformance candidate: `1.1.0` in `conformance/next`.
- Retained streams: `line`, `polyline`, `entityOrder`, `entityOrderEntry`.
- Retained tables: `scope`, `layerBinding`, `appearanceBinding`, `appearanceOverride`.
- Retained stream/table schema IDs, defaults, and field semantics stay unchanged.
- No legacy IFCDR reader/writer or migration layer; old versions are unsupported.
- No codec separation, new geometry, inline resources, representation renaming, enum-system redesign, physical optimization, or full preservation implementation in this plan.
- Preserve all of `conformance/1.0.0` and existing historical versioned schemas byte-for-byte.
- Preserve unrelated work, including the approved specification and roadmap changes already in the worktree.
- Keep core independent of cadcodec; preserve conversion, construction, encoding, and filesystem boundaries.
- Do not commit, push, publish, or create a release. Do not add artifacts from `target/` to version control.
- No subagents. At execution, inspect worktree state before deciding whether isolation is needed; never use a `codex/` or redundant `ifccad/` branch prefix.
- Use focused failing/passing tests per change and the required workspace checks at the end.

## File responsibilities and execution order

| Area | Files | Responsibility |
| --- | --- | --- |
| Contract | New registry/overlay in `schemas/` and matching candidate copies | Exact new supported vocabulary. |
| Candidate fixtures | `conformance/next/packages/`, schema inventory, provenance | Current valid and invalid inputs, retaining test intent. |
| Registry loader | `src/ifcdr/read/registry.rs` | Load and internally cross-check the active registry. |
| Resource validator | `src/ifcdr/read/validation.rs` | Physical checks, supported-schema detection, semantic-validation prerequisites. |
| Typed entities | `src/ifcdr/read/entity.rs`, `resource.rs`, module re-exports | Remove prototype-only public/internal views and text-order exception. |
| Package analysis | `src/package/read/validation.rs`, `bindings.rs`, `schema.rs` | Overlay selection and resource-dependent package checks. |
| Writer | `src/ifcdr/write/encoder.rs`, `src/package/write/ifcx.rs` | New versions and reduced payload inventory. |
| Converter | `crates/ifccad-convert/src/import/` | Remove obsolete IFCDR unmodeled-entity handling; keep other loss reporting. |
| Evidence | Existing Rust tests plus targeted new cases | Verify retained behavior, unsupported boundaries, and deterministic roundtrips. |
| Documentation | Candidate compatibility/provenance, README, roadmap, converter documentation | Report actual post-change support and deferred work. |

Task 1 is one coordinated contract-switch checkpoint: the registry, reader,
writer, and candidate fixture versions must move together. Its intermediate
red tests are expected; do not leave a half-switched baseline as the completed
deliverable. Tasks 2–5 depend on that baseline and execute in order.

## Task 1: Switch the active JSON baseline to the reduced contract

**Files:**

- Create: `schemas/ifcdr/registry-0.6.0.json`, `schemas/ifcx/ifccad-overlay-0.6.0.json`.
- Create matching copies: `conformance/next/schemas/ifcdr/registry-0.6.0.json`, `conformance/next/schemas/ifcx/ifccad-overlay-0.6.0.json`.
- Remove candidate copies only: `conformance/next/schemas/ifcdr/registry-0.5.0.json`, `conformance/next/schemas/ifcx/ifccad-overlay-0.4.0.json`, `conformance/next/schemas/ifcx/ifccad-overlay-0.5.0.json`.
- Modify: `src/ifcdr/read/registry.rs`, `src/package/read/schema.rs`, `src/ifcdr/write/encoder.rs`, `src/package/write/ifcx.rs`.
- Modify: applicable JSON resources, descriptors, and package indexes under `conformance/next/packages/`; `conformance/next/PROVENANCE.md`.
- Tests: `tests/ifccad_conformance_assets.rs`, `tests/ifcx_overlay_schema.rs`, `tests/ifccad_package_conformance.rs`, and current-version literals in the existing reader/writer/converter tests.

**Interfaces:**

- Keep `canonical_registry() -> &'static IfcdrRegistry` and `validate_ifcdr(LoadedIfcdrResource) -> ValidationOutcome<LoadedIfcdrResource>` unchanged.
- Keep public package and conversion entry points unchanged.
- Produce a new-version candidate consumed by `fixture_source()` and `bundled_conformance_root()`; the latter still resolves `conformance/next` with suite version `1.1.0`.

- [x] Inspect `git status --short`, read the approved spec and AGENTS.md, and run the focused baseline:

```text
cargo test --workspace --test ifccad_package_conformance
cargo test --workspace --test package_writer_roundtrip
```

Record pre-existing failures rather than attributing them to this change. Keep
any temporary transformation artifacts under `target/` and out of version control.

- [x] Add a failing registry inventory test inside `src/ifcdr/read/registry.rs`:

```rust
#[test]
fn active_registry_has_only_base_profile_definitions() {
    let registry = canonical_registry();
    assert_eq!(registry.ifcdr_version(), "0.6.0");
    let streams = registry.streams().iter()
        .map(|stream| stream.name()).collect::<BTreeSet<_>>();
    assert_eq!(streams, BTreeSet::from([
        "line", "polyline", "entityOrder", "entityOrderEntry",
    ]));
    let tables = registry.tables().iter()
        .map(|table| table.name()).collect::<BTreeSet<_>>();
    assert_eq!(tables, BTreeSet::from([
        "scope", "layerBinding", "appearanceBinding", "appearanceOverride",
    ]));
    assert!(validate_registry_cross_references(registry).is_empty());
}
```

Run `cargo test -p ifccad active_registry_has_only_base_profile_definitions`.
Expect failure on the current version/inventory, not a test compilation error.

- [x] Derive the new registry from the historical registry, retaining exactly
  the spec's tables and streams. The JSON transformation is:

```rust
registry["ifcdrVersion"] = serde_json::json!("0.6.0");
registry["resource"]["schemaId"] =
    serde_json::json!("ifccad.ifcdr.resource.v0.6.0");
registry["streams"].as_array_mut().unwrap().retain(|stream| {
    matches!(stream["name"].as_str(),
        Some("line" | "polyline" | "entityOrder" | "entityOrderEntry"))
});
registry["tables"].as_array_mut().unwrap().retain(|table| {
    matches!(table["name"].as_str(),
        Some("scope" | "layerBinding" | "appearanceBinding" | "appearanceOverride"))
});
```

Here `registry` is the parsed `serde_json::Value` of the existing file; this is
an asset-generation operation, not a runtime filter. Preserve retained entry
order and schema IDs. Create the new overlay from 0.5.0, changing its own `$id`
and title to 0.6.0 and the IFCDR geometry descriptor's version constant to 0.6.0.
Do not change IFCPR or the drawing-core reference. Write LF-terminated UTF-8 JSON
and copy exact bytes into the candidate. Explicitly inspect target paths before
removing the three named superseded candidate files.

- [x] Switch the embedded registry/overlay paths and their version assertions
  to 0.6.0. The writer changes are restricted to:

```rust
// In the IFCDR header and IFCX geometry descriptor:
"version": "0.6.0"
// Delete these root entries from IFCDR writer output:
// "namedUcsBindings": [],
// "dimensionOverrideTable": [],
```

Update the registry cache test's expected counts from 12/29 to 4/4 and the
writer test that currently expects an empty dimension table. Keep the existing
stream directory, payload columns, appearance modes, and filename conventions.

- [x] Renew each candidate drawing resource: set its otherwise-valid IFCDR
  header version to 0.6.0, remove the discarded support tables listed in the
  spec, and update referring IFCX geometry descriptor versions. Remove
  `streams.textRuns` if present. Preserve intentional malformed fields and
  identifier/reference mismatches. Do not repair a negative test's defect.

For each rewritten resource, compute the checksum from final stored bytes:

```rust
use sha2::{Digest, Sha256};
let checksum = format!("sha256:{:x}", Sha256::digest(&resource_bytes));
```

Update matching descriptor checksums only when they were previously correct.
Keep intentionally incorrect checksums incorrect. Check every candidate
`package.json` resource index and linked descriptor, including multi-resource
and preservation fixtures. Do not modify semantic fingerprint vectors just
because pretty-printed JSON bytes changed.

- [x] Audit current-version references with:

```text
rg -n '0\.5\.0|0\.6\.0|dimensionOverrideTable|namedUcsBindings' src tests crates/ifccad-convert conformance/next
```

Update valid inline test inputs and expected current versions. Existing tests
use 0.6.0 as an unsupported version: change those negative values to 99.0.0,
and add an explicit 0.5.0 rejection case. Do not globally replace references in
historical schema tests. Add a new overlay-0.6 test by cloning the existing
overlay-0.5 validation test pattern, retaining the old-version schema tests.

- [x] Update candidate file inventory/equality tests to require only the new
  active overlay/registry plus the retained drawing core, meta-schema, and
  IFCPR schema. Preserve historical bootstrap equality and frozen-byte checks.
  Update candidate provenance to describe its new contract inventory.

- [x] Run the coordinated checkpoint:

```text
cargo test -p ifccad active_registry
cargo test -p ifccad --test ifccad_conformance_assets
cargo test -p ifccad --test ifcx_overlay_schema
cargo test -p ifccad --lib
cargo test -p ifccad --test ifccad_package_conformance
cargo test -p ifccad --test package_writer_roundtrip
cargo test -p ifccad-convert --test export_chains
```

Expected: existing supported cases use 0.6.0 and pass; the known five IFCPR/
projection deferrals remain explicit. Diagnose mismatches from actual output,
especially negative fixtures masked by version/checksum mistakes.

## Task 2: Remove prototype entity paths from the typed model and converter

**Files:**

- Modify: `src/ifcdr/read/entity.rs`, `src/ifcdr/read/resource.rs`, `src/ifcdr/read/mod.rs`, `src/ifcdr/mod.rs`.
- Modify: `crates/ifccad-convert/src/import/conversion.rs`, `diagnostic.rs`, `entity_mapping.rs`.
- Tests: `tests/public_conversion_api.rs`, `crates/ifccad-convert/tests/minimal_conversion.rs`, and adjacent unit tests.

**Interfaces:**

- `IfcdrEntityRef<'a>` retains only `Line(Line)` and `Polyline(PolylineRef<'a>)`.
- Remove `UnmodeledEntityRef`, `UnmodeledStreamRef`, and their exports/accessors.
- `ImportDiagnostic` retains its line-pattern and line-weight diagnostics;
  remove only `UnmodeledEntitiesSkipped` and its aggregation.

- [x] Add a compile-enforced public API test helper to
  `tests/public_conversion_api.rs`:

```rust
fn base_entity_id(entity: IfcdrEntityRef<'_>) -> EntityId {
    match entity {
        IfcdrEntityRef::Line(line) => line.entity_id(),
        IfcdrEntityRef::Polyline(polyline) => polyline.entity_id(),
    }
}

#[test]
fn public_entity_api_is_exhaustive_for_the_base_profile() {
    let directory = bundled_conformance_root()
        .join("packages/valid/minimal-no-preservation");
    let loaded = load_directory_package(directory).unwrap();
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let layout = drawing.layouts().next().unwrap();
    let ids = layout.representation().resource()
        .entities(layout.scope().id()).map(base_entity_id)
        .map(|id| id.get()).collect::<Vec<_>>();
    assert_eq!(ids, [1, 2, 3, 4]);
}
```

Run `cargo test -p ifccad --test public_conversion_api` and verify the intended
non-exhaustive-match failure caused by `Unmodeled`.

- [x] Remove the enum variant and its type; remove the wildcard construction
  of `UnmodeledEntityRef` from the entity iterator. The remaining internal
  dispatch may use `unreachable!("validated base-profile object stream")` for
  an invariant violation because the exact registry inventory is tested and
  unsupported input cannot obtain validation evidence.

- [x] Replace the text-specific `directly_ordered` exemption with unconditional
  insertion for every registered object entity:

```rust
directly_ordered.entry(scope).or_default().insert(id);
```

Keep object-versus-child filtering: `entityOrderEntry` is still a child stream.
Remove the unused stream-view enumeration and change its test to verify scope
entity order rather than classify order streams as unmodeled.

- [x] Remove the obsolete import conversion/mapping match arms and diagnostic
  fields/methods. In `DiagnosticAccumulator::finish`, start the retained output
  with `let mut diagnostics = Vec::new();`, then keep the existing sorted
  line-pattern and line-weight aggregation. Remove only tests for the deleted
  diagnostic; retain display and aggregation tests for the remaining variants.
  Remove `unmodeled_schema` from `EntityProjection` and corresponding match arms
  from all test helpers. Do not change CAD export unsupported-entity handling.

- [x] Verify the focused checkpoint and residual references:

```text
cargo test -p ifccad --test public_conversion_api
cargo test -p ifccad ifcdr::read
cargo test -p ifccad-convert --lib
cargo test -p ifccad-convert --test minimal_conversion
cargo test -p ifccad-convert --test export_loss_policy
rg -n 'Unmodeled|unmodeled|ownerKind' src/ifcdr crates/ifccad-convert/src/import tests/public_conversion_api.rs
```

Expected: tests pass; the final search has no obsolete code references. No
runtime IFCDR compatibility path is added to compensate for the removed API.

## Task 3: Report unsupported content without dependent false errors

**Files:**

- Modify/test: `src/ifcdr/read/validation.rs`.
- Modify/test: `src/package/read/validation.rs`, `src/package/read/bindings.rs`.
- Test: `tests/ifccad_package_conformance.rs` and new candidate negative cases in Task 4.

**Interfaces:**

- Keep the public loading, reporting, and strict-view interfaces unchanged.
- Add private `has_unsupported_streams: bool` to `ResourceValidator`.
- Extend `analyze_resource_bindings` and `validate_ifcpr_drawing_resource_ids`
  with `unavailable_ifcdr_resource_ids: &BTreeSet<ResourceId>`; all other
  parameters and return types remain unchanged. This set means declared but
  not validated; it never means safe to expose through typed views.

- [x] In the existing IFCDR validation test module, add:

```rust
#[test]
fn unsupported_stream_does_not_create_order_or_orphan_errors() {
    let mut value = fixture_source().value().clone();
    value["streamDirectory"]["streams"].as_array_mut().unwrap()
        .push(serde_json::json!({
            "name": "hatch", "schema": "example.hatch.v1",
            "role": "object", "count": 1,
            "columns": ["entityId", "scopeId"]
        }));
    value["streams"]["hatchStream"] = serde_json::json!({
        "count": 1, "entityId": [5], "scopeId": [0]
    });
    value["header"]["nextEntityId"] = serde_json::json!(6);
    value["streams"]["entityOrderEntryStream"]["entityId"]
        .as_array_mut().unwrap().push(serde_json::json!(5));
    value["streams"]["entityOrderEntryStream"]["count"] = serde_json::json!(5);
    value["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(5);
    for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
        if entry["name"] == "entityOrderEntry" {
            entry["count"] = serde_json::json!(5);
        }
    }
    let outcome = validate_value("drawing.ifcdr.json", value);
    assert!(outcome.validated().is_none());
    assert_eq!(outcome.diagnostics().iter().map(|d| d.code.as_str())
        .collect::<Vec<_>>(), ["IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED"]);
}
```

Run `cargo test -p ifccad unsupported_stream_does_not_create_order_or_orphan_errors`.
Expect failure from the current orphan/order diagnostic cascade.

- [x] Initialize the new boolean to false. When a stream name/schema is
  unsupported, set it true and emit the existing unsupported code with
  `streamName` and `schemaId` string context when available. Replace the hardcoded
  0.5.0 message with the active registry version. Keep a known schema attached
  to the wrong registered name as a directory error.

After stream structure checking and before entity/index/reference validation:

```rust
if validator.has_unsupported_streams {
    return EvidenceOutcome::failure(validator.diagnostics);
}
```

Guard orphan-payload reporting with `!self.has_unsupported_streams`; the JSON
mapping of unknown payloads is not known, so do not guess it from stream names.
Continue checks on independently understood columns and tables. Keep orphan
reporting and all entity/reference checks for fully supported resources.

- [x] Add a test for a supported `line` directory entry with an unknown schema
  ID and assert the same support blocker, no strict evidence, and the two
  context values. Keep the existing unsupported-version early return and test
  both 0.5.0 and 99.0.0. Preserve a malformed-known-column test and the real
  orphan-payload test to prove that suppression is not global.

```rust
#[test]
fn unsupported_line_schema_identifies_the_attempted_schema() {
    let mut value = fixture_source().value().clone();
    let entry = value["streamDirectory"]["streams"].as_array_mut().unwrap()
        .iter_mut().find(|entry| entry["name"] == "line").unwrap();
    entry["schema"] = serde_json::json!("example.line.v99");
    let outcome = validate_value("drawing.ifcdr.json", value);
    assert!(outcome.validated().is_none());
    let diagnostic = outcome.diagnostics().iter().find(|diagnostic| {
        diagnostic.code == "IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED"
    }).unwrap();
    assert_eq!(diagnostic.context.get("streamName"), Some(
        &PackageDiagnosticContextValue::String("line".to_owned())));
    assert_eq!(diagnostic.context.get("schemaId"), Some(
        &PackageDiagnosticContextValue::String("example.line.v99".to_owned())));
}
```

- [x] After building the validated-resource map in package validation, derive:

```rust
let unavailable_ifcdr_resource_ids = package.declarations.iter()
    .filter(|declaration| declaration.kind == ResourceKind::Ifcdr)
    .map(|declaration| declaration.resource_id.clone())
    .filter(|id| !validated_ifcdr_resources.contains_key(id))
    .collect::<BTreeSet<_>>();
```

Pass this set into both linkage checks. For IFCX preservation links in
`analyze_resource_bindings`, keep proven links, skip unavailable declared
targets, and emit a missing-target diagnostic only for a genuinely absent
target. In `validate_ifcpr_drawing_resource_ids`, add the unavailable-set check
alongside the existing validated-map and duplicate-report suppression checks.
Do not insert unavailable targets into any proven binding map. Existing layout
validation already skips representations without a validated resource; retain
and test that behavior. Resource identity/checksum/graph errors remain visible.

- [x] Add a package regression using a temporary copy of the renewed
  `unrepresented-packed` fixture. Set its IFCDR header and referring descriptor
  to 99.0.0 and recompute only the IFCDR resource checksum. Assert no strict
  package, presence of `IFCCAD_IFCDR_VERSION_UNSUPPORTED`, and absence of
  `IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING` and dependent layout-scope errors.
  Repeat with the mixed unknown-stream mutation above. Keep the existing
  missing-target IFCPR test to establish that truly absent IDs still fail.

Use the existing `TestDirectory`, fixture-copy routines, and
`sha256_checksum(&bytes)` in the package validation test module. Do not add
a new public test utility or a production fixture migrator.

- [x] Verify:

```text
cargo test -p ifccad ifcdr::read::validation
cargo test -p ifccad package::read::validation
cargo test -p ifccad package::read::bindings
cargo test -p ifccad --test public_package_loading
cargo test -p ifccad --test ifccad_package_conformance
```

Expected: support blockers prevent strict views, while known invalid structure
and genuinely missing resources retain their diagnostic contracts.

## Task 4: Prove the reduced profile and preserve meaningful conformance

**Files:**

- Modify/test: `tests/package_writer_roundtrip.rs`, `tests/public_conversion_api.rs`, `src/ifcdr/read/streams/line.rs`, `src/ifcdr/read/streams/polyline.rs`.
- Add candidate cases: `conformance/next/packages/invalid/unsupported-ifcdr-version/`, `conformance/next/packages/invalid/unsupported-ifcdr-stream/`.
- Modify: `conformance/next/manifest.json`.
- Retain/check: `tests/ifccad_package_conformance.rs`, `tests/ifcpr_schema.rs`, existing conversion-chain and filesystem tests.

**Interfaces:**

- Use the existing manifest `validatePackage` operation and diagnostic-code/
  severity expectations; no manifest-schema revision.
- Candidate case category `invalid` denotes failure of this runner's strict
  validation, not proof that unsupported content violates every contract.
  Explain this limitation in compatibility documentation.

- [x] Add a writer contract assertion to the existing production roundtrip
  test after writing its package:

```rust
let bytes = std::fs::read(target.join("resources/model-space.ifcdr.json")).unwrap();
let resource: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
assert_eq!(resource["header"]["version"], "0.6.0");
assert!(resource.get("namedUcsBindings").is_none());
assert!(resource.get("dimensionOverrideTable").is_none());
```

Keep the existing reader reload, semantic assertions, and deterministic-output
test. Verify coverage for empty, line-only, polyline-only, and mixed resources;
add only the missing cases using the existing `representative_builder` and
public builder patterns. Each case must load through the production reader.

- [x] In the existing stream-view unit tests, compare omitted visibility with
  explicit true using this fixture mutation:

```rust
let mut value = fixture_source().value().clone();
value["streams"]["lineStream"].as_object_mut().unwrap().remove("visible");
value["streams"]["polylineStream"].as_object_mut().unwrap().remove("visible");
for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
    if entry["name"] == "line" || entry["name"] == "polyline" {
        entry["columns"].as_array_mut().unwrap()
            .retain(|column| column != "visible");
    }
}
```

Validate the result and compare typed entity IDs, point sequences, closed state,
and visibility to an otherwise equivalent fixture with explicit true values.
Keep directory columns consistent so this tests default semantics rather than
a malformed directory. Preserve existing appearance-override tests, including
per-property modes and IFCX identity references.

- [x] Add a looped negative test inserting each removed empty root table into
  a supported fixture; require `IFCCAD_IFCDR_STRUCTURE_INVALID` at that field.
  Test removed `streams.textRuns` separately as unsupported/orphan payload
  according to the existing closed directory rules. Do not assert a legacy
  translation or silently ignore empty prototype data.

```rust
#[test]
fn removed_empty_tables_are_not_silently_accepted() {
    for table in [
        "textStyleBindings", "dimensionStyleBindings", "hatchPatternBindings",
        "namedUcsBindings", "dimensionOverrideTable", "characterFormatTable",
        "paragraphFormatTable",
    ] {
        let mut value = fixture_source().value().clone();
        value[table] = serde_json::json!([]);
        let outcome = validate_value("drawing.ifcdr.json", value);
        assert!(outcome.validated().is_none(), "{table}");
        let location = format!("/{table}");
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some(location.as_str())
        }), "{table}: {:?}", outcome.diagnostics());
    }
}
```

Place this test in `src/ifcdr/read/validation.rs` next to the other fixture-value
validation tests, so it can use the existing private test helpers.

- [x] Create the two small candidate cases from the renewed minimal package,
  with coherent descriptor versions/checksums. For the version case use 0.5.0;
  for the stream case use the synthetic directory/payload mutation from Task 3.
  Capture actual diagnostics, confirm each is independently justified, then
  record code and severity expectations in the manifest. Names containing
  `unsupported` and descriptions must make the support limitation explicit.
  Assert `validated_package().is_none()` in a Rust test as well as checking the
  manifest diagnostics.

- [x] Verify that old negative fixture meanings survive: duplicate IDs,
  missing scope/layer references, invalid ranges/lengths/order, resource
  identity mismatches, and malformed scalar cases must reach their intended
  checks. Retain all five named deferrals in
  `DEFERRED_VALIDATE_PACKAGE_CASES`. Keep IFCPR schema files unchanged, and
  compare physical checksums independently from semantic fingerprint vectors.

- [x] Run:

```text
cargo test -p ifccad --test package_writer_roundtrip
cargo test -p ifccad --test package_writer_filesystem
cargo test -p ifccad --test ifccad_package_conformance
cargo test -p ifccad --test ifcpr_schema
cargo test -p ifccad --test ifccad_fingerprints
cargo test -p ifccad ifcdr::read
cargo test -p ifccad-convert --test dxf_roundtrip
cargo test -p ifccad-convert --test export_chains
cargo test -p ifccad-convert --test export_loss_policy
```

Expected: the new profile roundtrips; unsupported CAD source entities still
produce explicit losses; no broader entity-family positive corpus is introduced.

## Task 5: Publish accurate local support documentation and verify the workspace

**Files:**

- Create: `conformance/next/COMPATIBILITY.md`.
- Modify: `README.md`, `ROADMAP.md`, `conformance/next/PROVENANCE.md`, `crates/ifccad-convert/README.md`.
- Review/update if affected: `crates/ifccad-convert/src/export/COVERAGE.md`, public documentation in `src/package/read/model.rs` and `diagnostic.rs`, and crate/module docs.
- Update completion state only after verification: approved spec and this plan.

**Interfaces:**

- No new Rust APIs. Document current success/reporting limitations without
  promising complete validity/support/transfer classification.
- No milestone status change: milestone 2 remains current and incomplete.

- [x] Write the compatibility matrix from the approved spec, replacing future
  wording with verified results. Include IFCDR 0.6.0 read/write, old-version
  rejection, unsupported-stream behavior, the one-model-layout writer subset,
  supported conversions, open IFCX read behavior, and limited IFCPR checks.

Use this explicit qualification:

```text
A strict load succeeds only for content supported by the implemented checks.
Unsupported versions or streams block the strict typed view. Such a blocker
does not establish invalidity under an unsupported contract. The current
manifest category and report success flag do not yet encode all independent
validity, operation-support, and lossless-transfer claims.

IFCPR 0.2.0 is loaded and checked against its schema, resource identity,
resource-file checksum, and supported drawing-resource links. Full payload,
dependency, projection, and lossless-preservation validation is not provided.
The converter does not transfer preservation content.
```

- [x] Document the deliberate `Unmodeled`/import-diagnostic API removal and
  the lack of migration support. Describe preserved CAD export loss reporting
  accurately. Keep the pinned export coverage contract unchanged unless an
  actual statement became inaccurate; do not weaken its source-coverage rules.
  Link the compatibility document from the README. Keep the roadmap's accepted
  profile reduction; replace only the first-slice implementation status when
  its acceptance criteria are verified.

- [x] Record retained reader/writer semantic gaps for the next design, including
  the builder-only minimum polyline vertex check, bounds semantics, and the
  retained JSON-specific appearance override representation. Do not change
  those rules as an incidental cleanup. Keep future geometry, codec, enum,
  terminology, inline-resource, and benchmark work explicitly outstanding.

- [x] Verify public documentation and formatting:

```text
cargo test --doc --workspace
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Inspect each result. Fix failures attributable to this work and rerun the
relevant failing checks, then obtain passing required checks for the final
state. Do not claim success from a plan or from earlier test output.

- [x] Audit scope and historical preservation:

```text
git diff --check
git diff --name-only -- conformance/1.0.0
git diff --name-only -- schemas/ifcdr/registry-0.5.0.json schemas/ifcpr/schema-0.2.0.json conformance/next/schemas/ifcpr/schema-0.2.0.json
git status --short
```

The two historical/schema-specific queries must be empty. Inspect remaining
changes against the spec; generated `target/` artifacts must not be staged.
Confirm old active schemas still exist and only their superseded candidate
copies were removed. Check local Markdown links and candidate schema equality.

- [x] Mark only completed steps, summarize verified behavior and intentional
  compatibility/API changes, and identify codec separation as the next design
  task. Leave the work uncommitted and keep milestone 2 open.

## Spec coverage and handoff

| Specification requirement | Plan coverage |
| --- | --- |
| Versioned reduced registry and candidate equality | Task 1 |
| Preserve retained defaults/semantics and general validation | Tasks 1, 2, 4 |
| Remove prototype-only model and converter paths | Task 2 |
| Unsupported content, no strict view, no dependent false errors | Task 3 |
| Writer/converter alignment and deterministic production roundtrips | Tasks 1, 4 |
| Retain IFCPR schema/checks with explicit limitations | Tasks 3, 4, 5 |
| Renew active fixtures; preserve historical collections and negative intent | Tasks 1, 4, 5 |
| Compatibility matrix and roadmap alignment | Task 5 |
| Mandatory final verification and milestone boundary | Task 5 |

Execute this plan in the current task using `superpowers:executing-plans` when
implementation is requested. Do not ask for a delegation choice: the existing
project instruction already selects sequential work without subagents. This
plan is not an authorization to commit or publish.

## Execution record — 2026-09-08

Completed in branch `ifcdr-base-contract`, without commits or publication.
The reduced registry, reader, writer, converter, and candidate fixtures now use
IFCDR 0.6.0. Milestone 2 remains open for the logical-model and codec boundaries.

Focused tests first demonstrated the old registry inventory, the extra public
entity variant, unsupported-stream diagnostic cascades, and false missing
preservation targets. Their corresponding checks pass after implementation.
The production-reader roundtrips cover line-only, polyline-only, mixed, empty,
and deterministic writer output. The candidate manifest now has 27 cases; its
loading test was updated for the two added unsupported-content cases.

Final verification passed: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, and `cargo test --doc --workspace`.
`git diff --check` passed. Historical conformance 1.0.0, historical active
schemas, and both IFCPR 0.2.0 schema copies are unchanged. The five existing
IFCPR/projection conformance deferrals remain documented in the compatibility
matrix. No preservation transfer or codec separation was added.

Test placement and helper API spelling follow the existing implementation;
some focused checks were batched into library/integration runs. No design
scope changes were required. Implementation and documentation remain
uncommitted for review.
