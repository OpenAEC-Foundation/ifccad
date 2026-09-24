# IFCCAD Conformance Suite 1.1.0 Candidate Provenance

- Source repository: private IFCCAD reference repository
- Source commit: `9f1c4078a760da631c8553fc5aaa474b8134b95d`
- Suite version: `1.1.0` candidate (`conformance/next`)
- Imported on: 2026-08-05
- Candidate basis: immutable IFCCAD conformance suite `1.0.0`
- Identity and header migration: explicit IFCX resource IDs, IFCPR `0.2.0`, ID-based drawing links, and the required minimal IFCCAD package header
- Included schemas: IFCX overlay `0.9.0`, drawing core `0.2.0`, IFCDR logical registry and normative contract `0.7.0`, registry meta-schema v2, JSON mapping `0.7.0` with meta-schema v1, and IFCPR `0.2.0`
- Base-contract reduction (2026-09-08): retained line/polyline, scope, layer/appearance bindings and overrides, and entity-order definitions; removed other prototype definitions from the candidate. Drawing resources and their descriptor checksums were renewed; frozen collections and the IFCPR schema were not changed.
- Added small unsupported-version and unsupported-stream cases. See [compatibility and validation limits](COMPATIBILITY.md).
- Logical-model migration (2026-09-08): renewed drawing resources and descriptor checksums for 0.7.0. Added polyline-minimum, missing/non-enclosing/conservative bounds, unused-invalid-override, and closed-color-field cases. Frozen assets remain unchanged.
- Mapping clarification (2026-09-08): added the normative JSON mapping language v1 document, referenced by the mapping and meta-schema; made XY pool arity explicit in the meta-schema. Existing resource range behavior is unchanged.
- Excluded paths: Python source, Python tests, sample DWG/DXF files, generated reports, and unrelated documentation

## Licensing

OpenAEC Foundation contributes the selected language-neutral schemas, manifests,
fixtures, vectors, scenarios, and blobs to `ifccad` under the Mozilla Public
License 2.0. The adjacent `LICENSE` file contains the applicable license text.

- Drawing terminology migration (2026-09-08): IFCX overlay and normative drawing-resource contract 0.8.0; IFCDR remains 0.7.0. Renewed representation descriptors, sample identities, linked IDs and checksums. Added shared model/paper layouts, representation mismatch and retired-vocabulary cases. Frozen collections remain unchanged.

- Inline-source migration (2026-09-09): overlay and normative source contract 0.9.0, retaining drawing relationship contract 0.8.0. Added IFCDR/IFCPR source combinations, conflicting sources, inline checksum, duplicate identity and unsupported inline-body cases. Resource body contracts and frozen collections are unchanged.

- Reporting update (2026-09-11): added reporting contract v1 and manifest schema v2 with optional category/assessment expectations. Existing case IDs and five deferred IFCPR checks remain; format-body schemas, frozen collections and suite version are unchanged.

- Scope/block migration (2026-09-22): active IFCDR 0.9.0 and IFCX overlay 0.11.0;
  registry meta-schema v4, mapping meta-schema v3 and mapping language v3.
  Generic scopes now have typed ownership; local definitions and instance
  transforms are explicit. Resource-global IDs and scope-local order remain.
  Candidate fixtures were mechanically migrated, refreshing only previously
  matching checksums so intentionally incorrect hashes remain incorrect.
  Added block defaults, signed uniformity, subnormal scale, missing targets,
  unused cycles, null transform and inline/external numerical-proof gaps.
  Additional cases cover non-neutral frames, conservative/non-enclosing bounds,
  missing/null/zero transform members, unknown scope kinds, full-fold name
  collisions, duplicate definitions and layout/scope-kind mismatch.
  Unicode 17 default full case folding C/F data and its license are pinned in
  the schema snapshots. No numbered release was modified.

- Layout/viewport/plot candidate (2026-09-23): added IFCDR 0.10.0,
  registry meta-schema v5, JSON mapping meta-schema/language v4, IFCX overlay
  0.12.0 and drawing core 0.3.0 as immutable-by-name snapshots in this
  unpublished collection. Added a strict writer-produced paper layout fixture
  with complete effective plot settings, one viewport with dormant lens/clip
  values, independent state flags, two override child rows and a second
  viewport using an active self-intersecting straight boundary. An invalid
  mutation opens that boundary. Further strict
  writer output adds two paper scopes with distinct
  media/plot settings and no viewport. Another valid case adds two drawings
  with independent layouts sharing the same resource, Layer and Appearance
  nodes. A model-only writer case has empty definition lists. Invalid cases
  cover paper-Limits, plot quality and scale constraints, folded layer/layout
  names, drawing-list closure, viewport dimensions/duplicate overrides and
  incomplete viewport child-partition mutations. Modified resource checksums were
  recomputed. At that stage, the previous 0.9.0/0.11.0 fixtures remained for read
  compatibility; no numbered collection changed.

- Layout condition completion (2026-09-23): added 38 independently named
  candidate package cases, bringing the manifest to 125 cases. The fixtures
  are reproducible with `scripts/generate_layout_conformance.ps1` and cover
  unused definitions, every supported plot area, conditional media/scale
  values, paper-scope selection, viewport ownership/view/clip/override rules,
  complete child ranges, and omission versus null. The production package
  reader checks each diagnostic expectation and strict-view availability;
  focused typed-reader and writer-readback tests check retained semantics.
  See [the condition and CAD evidence matrix](LAYOUT-MATRIX.md). This addition
  changes no schema or numbered release.

- Current-version fixture migration (2026-09-24): migrated the 67 remaining
  IFCDR 0.9.0 package cases to 0.10.0, including the required IFCX Drawing
  layer/appearance lists, renamed layout limits flag, and independent Layer
  state. Matching external checksums were refreshed; deliberately incorrect
  checksums retain their negative-test meaning. Two earlier layout-binding
  negatives now expect the additional uncovered-scope error of the active
  contract. Explicit 0.5.0/0.7.0 unsupported-version probes and the invalid
  999.0.0 inline body remain intentional. The reader's separate 0.9.0 support,
  schema snapshots, and numbered conformance releases were not changed.
