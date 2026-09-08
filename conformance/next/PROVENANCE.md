# IFCCAD Conformance Suite 1.1.0 Candidate Provenance

- Source repository: private IFCCAD reference repository
- Source commit: `9f1c4078a760da631c8553fc5aaa474b8134b95d`
- Suite version: `1.1.0` candidate (`conformance/next`)
- Imported on: 2026-08-05
- Candidate basis: immutable IFCCAD conformance suite `1.0.0`
- Identity and header migration: explicit IFCX resource IDs, IFCPR `0.2.0`, ID-based drawing links, and the required minimal IFCCAD package header
- Included schemas: IFCX overlay `0.7.0`, drawing core `0.2.0`, IFCDR logical registry and normative contract `0.7.0`, registry meta-schema v2, JSON mapping `0.7.0` with meta-schema v1, and IFCPR `0.2.0`
- Base-contract reduction (2026-09-08): retained line/polyline, scope, layer/appearance bindings and overrides, and entity-order definitions; removed other prototype definitions from the candidate. Drawing resources and their descriptor checksums were renewed; frozen collections and the IFCPR schema were not changed.
- Added small unsupported-version and unsupported-stream cases. See [compatibility and validation limits](COMPATIBILITY.md).
- Logical-model migration (2026-09-08): renewed drawing resources and descriptor checksums for 0.7.0. Added polyline-minimum, missing/non-enclosing/conservative bounds, unused-invalid-override, and closed-color-field cases. Frozen assets remain unchanged.
- Mapping clarification (2026-09-08): added the normative JSON mapping language v1 document, referenced by the mapping and meta-schema; made XY pool arity explicit in the meta-schema. Existing resource range behavior is unchanged.
- Excluded paths: Python source, Python tests, sample DWG/DXF files, generated reports, and unrelated documentation

## Licensing

OpenAEC Foundation contributes the selected language-neutral schemas, manifests,
fixtures, vectors, scenarios, and blobs to `ifccad` under the Mozilla Public
License 2.0. The adjacent `LICENSE` file contains the applicable license text.
