# IFCDR Logical Model Implementation Plan

> **For agentic workers:** Use `superpowers:executing-plans` to implement this plan task-by-task, in this task. Repository instructions prohibit delegation unless the user explicitly requests it. Steps use checkboxes for tracking. Do not commit, push, or merge without explicit authorization.

**Goal:** Implement the approved encoding-neutral IFCDR contract, shared validation, and JSON reference codec while retaining separate reader and writer backings.

**Architecture:** A crate-internal resource-access trait supplies typed, safe access to decoded columns and prepared builder data. Shared semantic validation creates immutable evidence; the JSON encoder accepts only validated resources. Package validation adds IFCX binding and graph checks, and directory storage remains separate.

**Tech Stack:** Rust workspace, existing serde/serde_json and JSON-schema infrastructure, versioned contract assets, and the pinned cadcodec companion crate. No new runtime dependency is required.

**Spec:** [Approved logical-model design](../specs/2026-09-08-ifcdr-logical-model-design.md).

## Completion record — 2026-09-08

- [x] Task 1: versioned logical registry, normative rules, JSON mapping and overlay.
- [x] Task 2: shared typed collection access, logical diagnostics and immutable proof.
- [x] Task 3: checked JSON decoding into typed columns.
- [x] Task 4: prepared writer backing and supplied-ID allocation.
- [x] Task 5: deterministic encoding of validated resources from either backing.
- [x] Task 6: production reader/writer and candidate collection migrated together.
- [x] Task 7: obsolete paths removed; compatibility and contract evidence updated.
- [x] Task 8: self-review and full workspace verification completed.

Actual final checks: `cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets -- -D warnings` passed;
`cargo test --workspace` passed with 286 tests and one existing ignored test.
`cargo test --doc --workspace` separately passed (three examples).
`git diff --check` passed. Frozen conformance files and dependency manifests
are unchanged; generated verification material remains ignored under `target/`.
No commit, push or merge was performed.

Implementation refinements:

- Typed decoded storage lives in `read/decoded.rs`; physical checks live in
  `codec/json/physical.rs`. Superseded reader storage and encoder files are removed.
- `orders()` enumerates every declaration, alongside `order(scope)` lookup,
  allowing unknown and duplicate scope-order rows to be checked.
- Mapping completeness is checked against the embedded contract assets in
  tests; the codec materializes physical metadata from those fixed assets.
- Final package validation reuses the production reader in memory after
  resource encoding. Logical resource proof is required before encoding, and
  no package is returned until graph, binding, schema and checksum checks pass.
  This currently repeats resource decoding/validation; avoiding that work can
  be considered when measurements justify it.
- Completion aggregation is tested with two independent missing references.
  Independent identity/bounds errors are tested directly at the shared logical
  boundary, because builder bounds are computed rather than supplied.
- Both prepared and decoded backings execute the same semantic comparison
  helper. Tests include mixed order, ID gaps, degenerate geometry, metadata,
  safe lookups, unused overrides, and multiple scopes in the codec.
- Review found an unused line-pattern override reference escaping package
  validation. A failing production-reader test reproduced it; all declared
  override references now receive the existence check.

The detailed checklist below is retained as the original execution recipe;
its illustrative intermediate commands are not a transcript. The completed
task list and actual verification results above record execution status.

## Approved review refinements — completed 2026-09-08

The user approved three follow-up points: specify the mapping forms, move the
resource writer backing to `ifcdr/write` without package dependencies, and
document the two validation wrappers while retaining their names.

- Added `schemas/ifcdr/json-mapping-v1.md` and its candidate copy, with mapping
  and meta-schema references. It specifies index/range formulas, XY pool
  ordering, allowed overlap/unused pool data, and strict child-range coverage.
  The meta-schema now requires two distinct XY pool columns. Tests cover those
  rules and the candidate document inventory.
- `ifcdr/write/prepare.rs` now owns `IfcdrWriteInput`, `IfcdrWriteEntity` and
  `PreparedIfcdrResource`. The package adapter resolves IFCX identities and
  transfers owned geometry into resource input. The lower module imports no
  package state, package builder types, path generator or JSON types. Tests
  construct and encode a resource using only resource data, including multiple
  scopes; existing package/converter roundtrips remain covered.
- Documented `ValidatedIfcdr<R>` and `ValidatedIfcdrResource` at their type
  definitions; updated the spec and README for the final module boundary.
- Reviewed open issue context: these refinements implement the boundaries in
  #7 and preserve #3's separation from future chunking. Vocabulary (#8), inline
  resources (#1), and additional entity families remain outside this change.

Final follow-up checks passed: formatting, workspace Clippy with warnings
denied, workspace tests (291 passed, one existing ignored test), separate doc
tests (three passed), and `git diff --check`. Frozen conformance material is
unchanged and generated files remain ignored. No repository integration was
performed. This refinement supersedes the original recipe's placement and
ownership of the prepared model inside `package/write`.

## Global constraints

- Work on `ifcdr-logical-model`; check the current branch and user changes before execution. Use the worktree skill at execution time to assess isolation without discarding the existing specification or plan.
- IFCDR and composite overlay become `0.7.0`; drawing core and IFCPR stay `0.2.0`. The candidate collection remains unpublished `1.1.0`.
- Preserve historical schema files and every file under `conformance/1.0.0`.
- Keep lines, straight XY polylines, and the four existing supporting tables. No new CAD entity family, inline loading, naming migration, or preservation transfer.
- Keep the package builder's one drawing/model layout. Resource decoding and encoding retain multiple scopes.
- Keep the core independent of cadcodec. No JSON values or source pointers in shared semantic access or validation.
- Preserve supplied identities, next ID, order, bounds, modes, metadata, and valid unused overrides. No geometry repair or silent field loss.
- No binary codec, incremental editing, deletion, ID reservation, or whole-resource fingerprint algorithm.
- Required completion checks: formatting, workspace Clippy with warnings denied, and workspace tests. Use doc tests while changing public APIs/examples.
- General release/versioning policy is outside this plan, as agreed after spec review.

## Execution strategy and file boundaries

Tasks 1–5 introduce and test the new contract and internal path without switching the production reader's supported version. Task 6 switches the reader, writer, candidate collection, and converter consumers together. This avoids a temporary advertised combination of a 0.7 reader and 0.6 writer. Temporary internal coexistence is migration scaffolding, not shipped old-version support; remove it in task 7.

| Location | Responsibility |
| --- | --- |
| `schemas/ifcdr/registry-0.7.0.json`, `registry-meta-schema-v2.json` | Logical vocabulary, types, defaults, references, constraints |
| `schemas/ifcdr/json-mapping-0.7.0.json`, `json-mapping-meta-schema-v1.json` | JSON header, directory, payload names, omission and sequence packing |
| `schemas/ifcdr/logical-contract-0.7.0.md` | Normative semantic rules with named invariants |
| `schemas/ifcx/ifccad-overlay-0.7.0.json` | Package descriptor version boundary |
| `src/ifcdr/logical/{mod,access,types,validation,diagnostic}.rs` | Shared access, domain records, validation and logical diagnostics |
| `src/ifcdr/logical/tests.rs` | Resource contract tests and small candidate fixtures |
| `src/ifcdr/codec/{mod,json/mod,json/mapping,json/decode,json/encode,json/diagnostic}.rs` | JSON-specific interpretation and diagnostic source mapping |
| `src/ifcdr/read/{resource,entity,streams/*}.rs` | Decoded typed column storage and public validated views |
| `src/package/write/prepare.rs` | Prepared builder backing and row/order indexes |
| `src/package/write/{builder,state,types,error,ifcx}.rs` | Construction, IDs, completion, package assembly |
| `src/package/read/{appearance,bindings,validation,navigation,model,schema}.rs` | Typed projections and package-level proof integration |
| `tests/ifcdr_logical_contract.rs` | Logical registry/mapping consistency tests |
| Existing tests, converter tests and `conformance/next` | Production integration and published evidence |

Create modules only when their task adds exercised behavior. Keep the existing `Validated<T>` mechanism; do not introduce an independent proof framework.

## Task 1: Publish and test the logical registry and JSON mapping

**Files:** Create the six schema/contract files listed above. Create `tests/ifcdr_logical_contract.rs`. Read `schemas/ifcdr/registry-0.6.0.json`, existing meta-schema, and `tests/ifcx_overlay_schema.rs` as migration sources. Do not switch candidate copies or runtime constants yet.

**Interfaces:** The registry uses logical names and named invariants. The mapping identifies those names rather than duplicating their types/defaults. Use the following concrete structure for column metadata and its corresponding JSON mapping:

```json
{
  "name": "visible",
  "valueType": "boolean",
  "default": true
}
```

```json
{
  "logical": "line.visible",
  "payload": "streams.lineStream.visible",
  "omission": "logicalDefault"
}
```

The registry top level contains `registrySchemaVersion`, `ifcdrVersion`, `types`, `resource`, `tables`, `streams`, and `invariants`. The mapping contains `mappingSchemaVersion`, `ifcdrVersion`, `resource`, `directory`, `tables`, and `streams`. Meta-schemas close these metadata objects and validate the declared forms. Represent structured sequence packing with named offset/count/pool fields, not an executable path language.

- [ ] Add schema tests loading files relative to `CARGO_MANIFEST_DIR` with the existing `jsonschema` API. Assert exactly the four retained streams and tables, a logical visibility default of true, and no JSON payload keys or `jsonValue` in the logical registry.
- [ ] Add a mapping-completeness test: every mapped property resolves to a logical definition; every stored logical property is mapped exactly once or has a documented derived representation. Deliberately duplicate a mapping and remove `line.layerId` in cloned test inputs; both must fail the checker.
- [ ] Run `cargo test --test ifcdr_logical_contract`; expect missing-file failures before adding assets.
- [ ] Build the logical registry from retained 0.6 definitions. Replace physical polyline/order offsets with logical sequences; define color, enum domains, optional bounds, and local/IFCX reference categories. Keep scope kind/flags as the existing numeric metadata domains.
- [ ] Build the JSON mapping with the current payload names and directory layout. Encode absence of logical bounds as required JSON `bounds: null`; retain current appearance mode codes 0/1/2. Require exact integer decoding for IDs.
- [ ] Write the semantic document with stable invariant names: `entity.identity`, `entity.next-id`, `reference.local`, `order.coverage`, `geometry.finite`, `polyline.vertex-count`, `bounds.enclosure`, `appearance.values`, and `appearance.source`. Describe the spec's rules for each, including empty resources, invisible geometry, scope bases, inactive overrides, and external-reference responsibilities.
- [ ] Add overlay 0.7.0 and the schema IDs from the spec. Run `cargo test --test ifcdr_logical_contract` and `cargo test --test ifcx_overlay_schema`; expect the new assets to validate while current production tests retain the 0.6 baseline.

Example regression assertion in the asset test, after parsing `registry` as `serde_json::Value`:

```rust
assert_eq!(registry["ifcdrVersion"], "0.7.0");
let serialized = serde_json::to_string(&registry).unwrap();
for physical_key in ["payloadPath", "payloadKey", "jsonValue", "vertexOffset"] {
    assert!(!serialized.contains(physical_key), "{physical_key} leaked into logical registry");
}
```

## Task 2: Introduce shared safe access and resource semantics

**Files:** Create `src/ifcdr/logical/{mod,access,types,validation,diagnostic,tests}.rs`; modify `src/ifcdr/mod.rs` and `src/ifcdr/types.rs`. Reuse/adapt `src/validated.rs`. Read existing checks in `src/ifcdr/read/{entity,validation}.rs`.

**Interfaces:** Implement crate-internal `IfcdrResourceAccess` using these operations:

```rust
trait IfcdrResourceAccess {
    type Lines<'a>: IfcdrLinesAccess where Self: 'a;
    type Polylines<'a>: IfcdrPolylinesAccess where Self: 'a;

    fn resource_id(&self) -> &ResourceId;
    fn unit(&self) -> IfcdrLengthUnit;
    fn next_entity_id(&self) -> u64;
    fn bounds(&self) -> Option<Bounds2d>;
    fn scopes(&self) -> &[IfcdrScope];
    fn layers(&self) -> &[IfcdrLayerBinding];
    fn appearances(&self) -> &[IfcdrAppearanceBinding];
    fn overrides(&self) -> &[IfcdrAppearanceOverride];
    fn lines(&self) -> Self::Lines<'_>;
    fn polylines(&self) -> Self::Polylines<'_>;
    fn orders(&self) -> &[IfcdrScopeOrder];
    fn order(&self, scope: u32) -> Option<&[u64]>;
}

trait IfcdrLinesAccess {
    fn len(&self) -> usize;
    fn get(&self, row: usize) -> Option<IfcdrLineRow>;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

trait IfcdrPolylinesAccess {
    type Polyline<'a>: IfcdrPolylineAccess where Self: 'a;

    fn len(&self) -> usize;
    fn get(&self, row: usize) -> Option<Self::Polyline<'_>>;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

trait IfcdrPolylineAccess {
    fn entity(&self) -> IfcdrEntityRow;
    fn closed(&self) -> bool;
    fn vertex_count(&self) -> usize;
    fn vertex(&self, index: usize) -> Option<Point2>;
}
```

These are typed collection and row projections of per-kind logical columns, not a requirement to store row objects. Associated borrowed view types let each backing implement collection access without allocating a new collection or copying vertices. The traits remain crate-internal and statically dispatched; a generic plugin registry or boxed collection framework is not needed. The table slices cover small shared metadata; entity/vertex storage remains backing-specific.

Keep a collection view bound while borrowing a polyline from it:

```rust
let polylines = resource.polylines();
if let Some(polyline) = polylines.get(0) {
    for index in 0..polyline.vertex_count() {
        let point = polyline.vertex(index);
        assert!(point.is_some());
    }
}
```

This example assumes validated input; candidate validation reports an inaccessible vertex rather than asserting. Validation and encoding traverse these collections. Future entity-specific details belong on their own views, not on `IfcdrResourceAccess`. Adding a supported entity family may add one typed collection accessor. Retain the existing mixed entity iteration in scope draw order alongside this per-kind access.

Define all referenced records in `logical/types.rs`:

- `IfcdrScope`: `id`, `kind`, `flags`: u32; `name`: String; `base`: Point2.
- `IfcdrLayerBinding`: `id`: u32; `ifcx_layer`: String.
- `IfcdrAppearanceBinding`: `id`: u32; `ifcx_appearance`: Option<String>; `modes`: [u32; 4] in color/opacity/pattern/weight order; `override_id`: Option<u32>.
- `IfcdrAppearanceOverride`: `id`: u32; `color`: Option<IfcdrColor>; `opacity`, `line_weight`: Option<f64>; `ifcx_line_pattern`: Option<String>.
- `IfcdrColor`: `rgb`: [u8; 3]; optional `IfcdrIndexedColor { system: String, index: u64 }` and `IfcdrNamedColor { catalog: String, name: String }`. Both may coexist.
- `IfcdrEntityRow`: `entity_id`: u64; `scope_id`, `layer_id`, `appearance_id`: u32; `visible`: bool.
- `IfcdrLineRow`: `entity`: IfcdrEntityRow; `start`, `end`: Point2.
- Polyline metadata and vertex access are supplied by `IfcdrPolylineAccess`; no shared owned polyline/vertex-vector record is required.

Use representable raw IDs/modes in candidates so duplicate, zero and unknown domain values can be diagnosed safely. Move the existing writer `AppearanceMode` enum into the shared types and re-export it at its current package path in this task. Add shared fallible domain helpers `appearance_mode(u32) -> Option<AppearanceMode>`, `length_unit(&str) -> Option<IfcdrLengthUnit>`, and `rgb_channel(u64) -> Option<u8>`; these own allowed domains when decoding cannot construct a typed value.

Produce `IfcdrDiagnostic { code: &'static str, resource_id: ResourceId, collection: &'static str, row: Option<usize>, property: &'static str, message: String }`, without JSON locations. Preserve existing codes where their meaning still applies; add codes for polyline minimum and bounds violations. Use the existing diagnostic representation only at package/codec boundaries.

Produce `IfcdrCandidate<R>` with private data and `ValidatedIfcdr<R> = Validated<IfcdrCandidate<R>>`. Its `ValidationTarget` implementation has unit context and logical diagnostics. Expose crate-internal:

```rust
fn validate_resource<R: IfcdrResourceAccess>(
    resource: R,
) -> ValidationOutcome<IfcdrCandidate<R>>;
```

Evidence owns resource-wide entity and table indexes plus per-scope order indexes. Only `validate_resource` creates the proof; add read-only access through the target. No blanket proof conversion from the access trait.

- [ ] Implement a test-only backing with one scope 0, layer 0 linked to `layer-0`, appearance 0 with four ByLayer modes, line ID 1 from (0,0) to (1,1), order [1], next ID 2, and bounds (0,0)–(1,1). Call its constructor `line_candidate()` and its backing type `TestIfcdrResource`; keep vectors mutable for negative tests.
- [ ] Exercise collection `len`, `is_empty`, and checked `get` on both backing implementations as they become available. Cover empty collections, out-of-range rows and polyline vertices, and interleaved scope order independently of collection order. Access checks include `assert_eq!(candidate.lines().len(), 1)` and `assert!(candidate.lines().get(1).is_none())` for `line_candidate()`.
- [ ] Add tests for zero/duplicate IDs, next ID equal to max, missing local references, incomplete/wrong-scope/duplicate order, minimum vertices, finite geometry, and bounds. Use independently broken line ID and bounds in one case to assert multiple diagnostics.
- [ ] Run `cargo test --lib ifcdr::logical`; expect failing assertions or missing functions before implementation.
- [ ] Implement validation in dependency order: metadata/tables, entities/geometry, references, order, bounds. Skip dependent checks when their index or coordinate inputs are invalid. Bound all candidate access and use checked arithmetic; do not panic on a claimed row/vertex that cannot be retrieved.
- [ ] Validate every non-null override including unused values. Keep external target resolution out of this function. Check finite scope bases without applying them as transforms.
- [ ] Implement `geometric_bounds<R: IfcdrResourceAccess>(&R) -> Result<Option<Bounds2d>, Vec<IfcdrDiagnostic>>` in the shared validation module. Use it both for enclosure validation and later builder preparation; empty geometry yields None. Geometry validity does not depend on supplied bounds, so preparation can call this helper before setting its own bounds.
- [ ] Run `cargo test --lib ifcdr::logical`; all new semantic cases must pass. Include these concrete acceptance assertions in tests:

```rust
let mut candidate = line_candidate();
candidate.next_entity_id = 40;
candidate.lines[0].end = candidate.lines[0].start;
let (proof, diagnostics) = validate_resource(candidate).into_parts();
assert!(proof.is_some());
assert!(diagnostics.is_empty());

let mut candidate = line_candidate();
candidate.lines[0].entity.entity_id = 0;
let (proof, diagnostics) = validate_resource(candidate).into_parts();
assert!(proof.is_none());
assert!(!diagnostics.is_empty());
```

## Task 3: Decode JSON into typed columns and map diagnostics

**Files:** Create `src/ifcdr/codec/{mod,json/mod,json/mapping,json/decode,json/diagnostic}.rs`; add typed storage in `src/ifcdr/read/streams/store.rs`. Modify `src/ifcdr/mod.rs`. Keep the old production route until task 6.

**Interfaces:** `DecodedIfcdrResource` implements task 2's trait with SoA storage for line properties and polyline row properties, typed point pools, and ordered entity sequences. A private checked constructor establishes physical row/range consistency. Domain validity still comes from shared validation.

```rust
fn decode_json(
    uri: &str,
    value: &serde_json::Value,
) -> Result<DecodedIfcdrResource, Vec<PackageDiagnostic>>;
```

`json/diagnostic.rs` maps logical collection/row/property to source pointers using the JSON mapping, adding the resource URI outside the logical diagnostic. The raw source tree may remain with package inspection data; semantic views only read typed storage.

- [ ] Add codec tests built from a minimal 0.7 JSON resource with the exact line candidate from task 2. Test omitted visibility, bounds null, full-width integer IDs above 2^53, array length mismatches, pool overflow, wrong primitives, unknown fields in nested color metadata, unsupported versions and unknown stream schemas.
- [ ] Run `cargo test --lib ifcdr::codec::json::decode`; expect failure before the new decoder exists.
- [ ] Load the mapping using typed serde metadata structs confined to the codec. Cross-check its named logical targets against the registry. Move physical directory, closed-field, primitive, length and range checks from the current reader into this boundary.
- [ ] Reject unsupported versions before interpreting version-dependent fields. For unsupported streams collect independent physical diagnostics, return no full resource, and suppress derived missing-order/orphan errors. Preserve ordinary errors for real orphan payloads in understood resources.
- [ ] Decode finite-capable numeric storage, IDs without float conversion, metadata, modes, sequences, and defaults. Invoke shared scalar conversion helpers for unit/RGB domains. Collect independent decode errors; do not manufacture valid values to continue semantic proof construction.
- [ ] Add a test that mutates a decoded line's ID/order/bounds through test-only construction and calls task 2 validation, proving these checks are not JSON-only. Run `cargo test --lib ifcdr::codec` and the logical suite.

Concrete high-ID roundtrip input fragment for decoder tests:

```json
{
  "entityId": [9007199254740993],
  "scopeId": [0],
  "x1": [0], "y1": [0], "x2": [1], "y2": [1],
  "layerId": [0], "appearanceId": [0]
}
```

Set header next ID to 9007199254740994 and order to the same entity ID; assert equality as u64, not through `as_f64`.

## Task 4: Prepare writer state and support supplied entity IDs

**Files:** Create `src/package/write/prepare.rs`; modify `src/package/write/{mod,state,builder,error,types}.rs`, `src/ifcdr/types.rs`, and `tests/public_package_builder.rs`.

**Interfaces:** `PreparedIfcdrResource` owns the completed `DrawingState`, resolved table records, per-kind indexes into pending entities, scope order, bounds, and next ID. It implements task 2 access without copying every polyline's points. Split `NodePaths` preparation from JSON assembly only as needed to resolve logical IFCX identities.

```rust
fn prepare_drawing(
    drawing: DrawingState,
    paths: &NodePaths,
) -> Result<PreparedIfcdrResource, Vec<IfcdrDiagnostic>>;
```

Keep existing `add_line` / `add_polyline`; add `add_line_with_id(id: EntityId, definition: LineDefinition)` and `add_polyline_with_id(id: EntityId, definition: PolylineDefinition)` on the same model-space builder, with the existing `Result<EntityId, PackageBuildError>` return type. Make `EntityId::new(u64) -> Option<EntityId>` public. Add `DuplicateEntityId { id: EntityId }` to construction errors.

- [ ] Add allocator tests: automatic 1; supplied 10; automatic 11; supplied unused 5; automatic 12. Duplicate 10 and u64::MAX insertion fail without changing state. Supply IDs only after validating the definition so rejected inserts do not consume IDs.
- [ ] Run `cargo test --test public_package_builder`; expect missing-method failures for new tests.
- [ ] Store next ID and used IDs in `DrawingState`. Replace `entities.len()+1` with checked allocation. Reject a supplied maximum ID because no representable next ID can follow it. No ID reuse or reservation feature.
- [ ] Implement prepared per-kind indexes, separate order [insertion sequence], normalized visibility, bindings, and bounds. Preserve raw construction ownership and borrow vertices through access; preparation is not JSON serialization.
- [ ] Apply the shared contract suite to prepared resources. Cover empty state, mixed types, equal endpoints, duplicate vertices, closed two-vertex polylines, and stable IDs. Immediate builder validation calls shared geometry/appearance helpers rather than copying constraints.
- [ ] Run `cargo test --test public_package_builder` and `cargo test --lib package::write::prepare`. Add doc examples for supplied IDs and run `cargo test --doc --workspace`.

Allocator assertions use the existing public test setup to obtain a model-space builder and a cloned valid line definition:

```rust
assert_eq!(model.add_line(definition.clone()).unwrap().get(), 1);
assert_eq!(model.add_line_with_id(EntityId::new(10).unwrap(), definition.clone()).unwrap().get(), 10);
assert_eq!(model.add_line(definition.clone()).unwrap().get(), 11);
assert_eq!(model.add_line_with_id(EntityId::new(5).unwrap(), definition.clone()).unwrap().get(), 5);
assert_eq!(model.add_line(definition).unwrap().get(), 12);
```

## Task 5: Encode only validated logical resources

**Files:** Create `src/ifcdr/codec/json/encode.rs`; modify `src/ifcdr/codec/json/mod.rs`. Port packing/checksum code from `src/ifcdr/write/encoder.rs` without retaining its semantic preparation.

**Interfaces:** Move/reuse `IfcdrEncodeError` and `EncodedIfcdrResource` at the codec boundary. The artifact has resource ID, bytes and checksum; any retained bounds field is `Option<Bounds2d>` copied from input.

```rust
fn encode_json<R: IfcdrResourceAccess>(
    resource: &ValidatedIfcdr<R>,
) -> Result<EncodedIfcdrResource, IfcdrEncodeError>;
```

- [ ] Add encode tests for the validated prepared backing and validated decoded backing. Add a two-scope resource containing both entity types, ID gaps, next ID 100, conservative bounds, nonzero scope bases/flags, and color carrying indexed and named metadata. Include valid unused overrides.
- [ ] Run `cargo test --lib ifcdr::codec::json::encode`; expect missing encoder or preservation failures.
- [ ] Implement serialization through shared access. Pack logical sequences into checked JSON offsets/counts. Preserve scope/table metadata and supplied next ID/bounds. Do not call allocation, bounds accumulation, geometry normalization, or appearance resolution from the encoder.
- [ ] Emit required null bounds for empty resources and deterministic table/stream output. Omit visibility only when all rows have the logical default. Report u32 packing limits as encoding failures, without truncation.
- [ ] Encode, decode and validate each test resource; compare every logical property through shared access, including modes and metadata rather than rendered appearance only. Assert repeated encoding bytes are identical.
- [ ] Run `cargo test --lib ifcdr::codec`. Test compilation must make it impossible to call `encode_json` with a bare candidate; keep proof constructors private and inspect signatures as part of review.

Roundtrip test core, using the explicit multi-scope fixture constructed in this task:

```rust
let (proof, errors) = validate_resource(decoded).into_parts();
assert!(errors.is_empty());
let proof = proof.unwrap();
let first = encode_json(&proof).unwrap();
let second = encode_json(&proof).unwrap();
assert_eq!(first.bytes, second.bytes);
let json: serde_json::Value = serde_json::from_slice(&first.bytes).unwrap();
assert_eq!(json["header"]["nextEntityId"], 100);
assert_eq!(json["scopeTable"].as_array().unwrap().len(), 2);
```

## Task 6: Switch production loading, completion, and candidate assets together

**Files:** Modify `src/ifcdr/read/{mod,resource,entity,streams/*}.rs`; `src/package/read/{appearance,bindings,validation,navigation,model,schema}.rs`; `src/package/write/{builder,error,ifcx,types}.rs`; `src/package/mod.rs`; candidate manifest/schema copies/fixtures; `tests/{public_package_loading,public_package_builder,package_writer_roundtrip,ifccad_conformance_assets,ifccad_package_conformance,ifcx_overlay_schema}.rs`; affected converter consumers/tests.

**Interfaces:** The public `IfcdrResourceRef::bounds()` returns `Option<Bounds2d>`. Public entity/scopes/appearance views borrow typed backing rather than JSON maps. Retain useful existing public names through re-exports while moving the owned color/mode types to the logical boundary; widen indexed color storage to u64 so reader metadata is not narrowed by the writer.

Keep `PackageBuilder::finish(self) -> Result<EncodedPackage, PackageBuildError>` and add `PackageBuildError::Validation { diagnostics: Vec<PackageDiagnostic> }`. Immediate construction failures retain their existing variants. Shared logical diagnostics adapt into this collection; package binding diagnostics append in deterministic order. Error collections must remain inspectable rather than only formatted into a string.

- [ ] Update public acceptance tests to expect IFCDR 0.7.0, empty `None` bounds, typed colors, and preserved entity IDs. Add one intentionally malformed local reference plus independent invalid bounds to internal completion tests; require both diagnostics and no artifact.
- [ ] Run focused reader/writer integration tests and record expected failures under the old route.
- [ ] Route resource loading through `decode_json`, then `validate_resource`; retain source JSON only for package inspection/physical diagnostics. Rewrite entity and point iterators to access typed rows/pools, and make invalid lookup inputs return absence rather than panic.
- [ ] Refactor package appearance checks to use typed resource modes/overrides and typed projections of IFCX attributes. Keep external-reference existence, node kind, layer-name uniqueness, and explicit-source availability checks at package level. Preserve override precedence and IFCX extension fields in the original graph. Use shared value validators for selected IFCX appearance values.
- [ ] Change `finish` to prepare, run shared resource validation, perform package binding/graph checks, and encode only on success. Extract the current graph/appearance validation entry points to accept assembled in-memory package data; do not write a temporary directory in production to validate the builder. Existing schema/checksum checks may run after in-memory encoding where bytes are required. Physical assembly must not create a competing semantic validator.
- [ ] Switch runtime contract constants, overlay references, and IFCX descriptor versions together. Ensure all builder/converter packages reload through `load_directory_package` in tests.
- [ ] Copy the new active contract assets into `conformance/next`, remove superseded candidate registry/overlay copies, update candidate asset inventory/provenance, and renew current fixtures. Do not alter historical `schemas/` files or frozen collections.
- [ ] Recompute checksums from actual renewed resource bytes. Preserve negative-case intent and use unsupported old versions only in explicitly unsupported tests. Do not change canonical-value fingerprints solely because JSON bytes changed.
- [ ] Add candidate cases for invalid polyline minimum, bounds, inactive override values, and typed color unknown fields. Retain existing IFCPR limited checks and deferred cases. Ensure multiple-scope fixtures exercise production-reader re-encoding through the internal codec tests plus package reload.
- [ ] Run `cargo test --test package_writer_roundtrip`, `cargo test --test public_package_loading`, `cargo test --test ifccad_package_conformance`, and `cargo test -p ifccad-convert`. Inspect every failure; do not relax validation or remove fidelity assertions to accommodate the new storage.

Concrete empty-resource acceptance assertion, adapting the existing empty writer fixture:

```rust
let encoded = package.finish().unwrap();
let bytes = encoded.file("resources/model-space.ifcdr.json").unwrap();
let json: serde_json::Value = serde_json::from_slice(bytes).unwrap();
assert!(json.get("bounds").unwrap().is_null());
assert_eq!(json["header"]["version"], "0.7.0");
assert_eq!(json["header"]["nextEntityId"], 1);
```

## Task 7: Remove obsolete semantic paths and complete contract evidence

**Files:** Remove superseded `src/ifcdr/write/{mod,encoder}.rs` and move/delete obsolete parts of `src/ifcdr/read/{registry,validation,codes}.rs`; update module declarations. Modify logical/codec tests, `tests/ifccad_conformance_assets.rs`, `conformance/next/COMPATIBILITY.md`, and nearby API/conversion docs.

**Interfaces:** Only the new codec owns JSON resource interpretation. Public package APIs continue to use package diagnostics; logical validators remain JSON-free. Old IFCDR versions have no production decoder or migration layer.

- [ ] Add architecture checks by inspecting imports and signatures: `rg -n 'serde_json|LoadedJsonResource|/streams/' src/ifcdr/logical` must find no production dependency on JSON. Test-only JSON fixtures belong under codec tests instead.
- [ ] Remove the old JSON-backed semantic getters and independent entity/reference/geometry checks after all consumers use the new path. Preserve codec-specific checks by moving them rather than deleting their negative tests.
- [ ] Verify both backings execute the same semantic comparison helper and validation cases. Include bounds over invisible entities, duplicate final vertices, color metadata above u32 where supported, and negative diagnostic aggregation. Retain checked converter narrowing/loss behavior for CAD domains that are smaller than IFCDR's.
- [ ] Update compatibility documentation to 0.7.0 and explain the deliberate tightening of polylines, empty bounds and stored override values. Keep validity/support/transfer report redesign and full IFCPR validation explicitly outstanding.
- [ ] Update the spec status to implemented only after task 8 succeeds. Add its link to the milestone 2 progress paragraph in `ROADMAP.md`; keep milestone status Current. Update the README's current capability/version statements as needed, without changing milestone intent or implying inline support.
- [ ] Run `cargo test --test ifccad_conformance_assets`, `cargo test --test ifccad_fingerprints`, and `cargo test --doc --workspace`. Confirm frozen-byte checks and existing fingerprint vectors still pass.

## Task 8: Verify the complete change and prepare review

**Files:** Whole diff, approved spec, this plan, and actual verification output. Keep generated review/probe files under ignored `target/`.

- [ ] Check the diff against every acceptance requirement in the spec. Pay particular attention to immutable proofs, table/override preservation, inactive values, full-width IDs, scope bases, empty bounds, and encoding without mutation.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run `cargo test --workspace`.
- [ ] If a check fails, fix its cause and rerun the affected checks. Repeat broader checks only when the fix warrants it. Record actual commands/results; do not describe a planned check as passed.
- [ ] Verify `git diff --check`, no changes under `conformance/1.0.0`, and no generated `target/` artifacts in the change. Review that no CAD runtime dependency entered the core crate.
- [ ] Update implementation status and checked plan items to match the evidence. Report the behavior changes, intentional 0.6 incompatibility, relevant tests, and remaining milestone 2 items. Leave commit/push/merge to explicit user authorization.

## Self-review coverage map

| Approved requirement | Tasks |
| --- | --- |
| Logical registry/mapping/document split and version boundary | 1, 6 |
| Separate backings, shared access, immutable resource proof | 2–5 |
| Resource IDs, supplied entity IDs, high-water allocation, order | 2, 4, 5 |
| Polyline/line degeneracy, closure, visibility, bounds | 2–6 |
| Typed appearance, inheritance, unused overrides, IFCX extensions | 2, 3, 5, 6 |
| Physical vs semantic diagnostics and dependency suppression | 2, 3, 6 |
| Full decoded-resource encoding beyond builder subset | 3, 5, 6 |
| Consuming completion with aggregated final errors | 4, 6 |
| Production-reader roundtrips and converter fidelity | 6, 8 |
| Frozen assets, limited IFCPR, documentation and scope discipline | 6–8 |

Plan review: each new cross-task interface is defined above; existing public methods are retained unless an explicit changed signature is listed. Implementation proceeds inline under repository conventions. No execution-mode question or delegation is needed.
