# IFCCAD reporting contract v1

This contract describes evidence from attempted package loading, validation and
conversion. It adds no format-body fields, acceptance rules or preflight scan.
Assessment covers the applicable IFCCAD-owned contract; permitted unrelated
IFCX extension semantics are outside that scope.

## Diagnostic categories

`category` is independent of `severity` (`error`, `warning`, `info`):

| Category | Evidence |
| --- | --- |
| `contractViolation` | A known applicable rule is violated. |
| `unsupportedContent` | A required profile or content cannot be assessed by this implementation. |
| `executionBlocked` | Required input or execution capacity was unavailable. |

Existing codes, messages, context, resource identity and source pointers remain.
Error severity still blocks strict loading. Category is assigned from the
producer's rule, never from diagnostic prose or substrings in a code.

The following table is the current primary reader's reviewed classification.
Codes use the prefix shown in the first column plus the listed suffix.

| Prefix | Suffixes | Category / meaning |
| --- | --- | --- |
| `IFCCAD_PACKAGE_` | `CHECKSUM_MISMATCH` | `contractViolation`: external bytes disagree with their declared digest |
| `IFCCAD_PACKAGE_` | `ENTRYPOINT_INVALID`, `JSON_INVALID` | `contractViolation`: known entrypoint/JSON structure is malformed; dependent assessment is incomplete |
| `IFCCAD_PACKAGE_` | `PATH_INVALID` | `contractViolation`: URI violates the package path contract; target content is not assessed |
| `IFCCAD_PACKAGE_` | `ENTRYPOINT_MISSING`, `RESOURCE_MISSING` | `executionBlocked`: the required file/input is unavailable; no claim about the missing content's validity |
| `IFCCAD_PACKAGE_` | `RESOURCE_LIMIT_EXCEEDED`, `TOTAL_LIMIT_EXCEEDED` | `executionBlocked`: reader byte limit, not a format size constraint |
| `IFCCAD_PACKAGE_` | `VOCABULARY_UNSUPPORTED` | `unsupportedContent`: retired representation vocabulary |
| `IFCCAD_PACKAGE_` | `SCHEMA_INVALID` | Context-dependent, as described below |
| `IFCCAD_PACKAGE_` | `NODE_PATH_DUPLICATE`, `NODE_REFERENCE_MISSING`, `NODE_REFERENCE_TYPE_MISMATCH` | `contractViolation`: known graph identity/reference rule |
| `IFCCAD_PACKAGE_` | `RESOURCE_ID_DUPLICATE`, `RESOURCE_ID_MISMATCH`, `TARGET_RESOURCE_MISSING` | `contractViolation`: known resource identity/link rule |
| `IFCCAD_PACKAGE_` | `TIMESTAMP_INVALID`, `BINDING_INVALID`, `APPEARANCE_INVALID`, `LAYER_NAME_DUPLICATE` | `contractViolation`: known timestamp, binding, appearance or layer rule |
| `IFCCAD_IFCDR_` | `VERSION_UNSUPPORTED`, `STREAM_SCHEMA_UNSUPPORTED` | `unsupportedContent`: unavailable version or stream mapping |
| `IFCCAD_IFCDR_` | `UNIT_UNSUPPORTED` | `contractViolation`: value outside the supported contract's closed length-unit enum |
| `IFCCAD_IFCDR_` | `STRUCTURE_INVALID`, `DIRECTORY_INVALID` | `contractViolation`: known physical shape, field, directory or pool-range rule |
| `IFCCAD_IFCDR_` | `REFERENCE_MISSING`, `ENTITY_ID_INVALID`, `ENTITY_ID_DUPLICATE`, `ENTITY_ORDER_INVALID` | `contractViolation`: known reference/identity/order rule |
| `IFCCAD_IFCDR_` | `BOUNDS_INVALID`, `GEOMETRY_INVALID`, `POLYLINE_INVALID`, `APPEARANCE_INVALID` | `contractViolation`: known logical geometry/appearance rule |

`SCHEMA_INVALID` normally reports `contractViolation`. A string-valued mismatch
at a fixed profile selector (IFCX header version, resource descriptor version,
or IFCPR header version) reports `unsupportedContent`. Missing/non-string
selectors remain malformed known structure. IFCPR body-rule errors obtained
with schema 0.2.0 against an explicitly unknown body version are unsupported
profile evidence, not invalidity under that unknown version. Package envelope
format and resource-ID requirements remain independently applicable. The
separate IFCX source, graph and checksum checks retain their own categories.
IFCDR stops before interpreting an unknown version's body. Known independent
errors can therefore coexist with unsupported content.

`PackageOpenError` is `executionBlocked` and remains a failed `Result` rather
than a fabricated validation report.

## Package assessment

Reports contain `assessment` with `validity`, `completeness` and `gaps`:

| Error-level contract violation | Assessment completed without gaps | Validity | Completeness |
| --- | --- | --- | --- |
| no | yes | `valid` | `complete` |
| yes | yes | `invalid` | `complete` |
| no | no | `notFullyAssessed` | `incomplete` |
| yes | no | `invalid` | `incomplete` |

Completion requires explicit evidence that the assessment flow finished. A
default report is incomplete. `is_valid()` retains its legacy meaning: no
error diagnostics. Neither it nor `validated_package()` means that every
preservation semantic or arbitrary IFCX extension was assessed. Incompleteness
alone does not introduce a new loading error.

Each gap contains nullable `resourceId`, `resourceUri` and RFC 6901 `location`,
plus one reason:

- `unsupportedProfile`: unavailable version, vocabulary or stream mapping;
- `unavailableInput`: required input could not be loaded;
- `executionLimit`: byte/capacity limit prevented assessment;
- `contentNotAssessable`: malformed structure or missing validation proof
  prevents dependent checks, or no completed assessment exists;
- `preservationSemanticsNotAssessed`: current IFCPR coverage limitation.

External content uses its resource URI and local pointer; inline content uses
the containing document URI and full content pointer. Gaps are sorted by
resource ID, URI, pointer, then reason in the order above (null before strings),
and exact duplicates removed. Repeated declarations of one physical source do
not repeat its content assessment; distinct inline pointers remain distinct.
Detailed diagnostics retain their existing sort order.

Current IFCPR coverage checks JSON schema, descriptor/header identity, external
checksums and drawing links. It does not fully assess blob contents, payload
ranges, record dependencies or projection semantics. Each encountered loaded
IFCPR source records this coverage gap. Drawing-only packages do not inherit it.
Future preservation implementation must update this coverage declaration.

## Transfer assessment

A completed conversion returns a summary alongside its existing diagnostics
and entity mapping. Its language-neutral values are:

- `scope`: `selectedDrawingToCadDocument` or `publicCadDocumentToPackage`;
- `coverage`: `completeWithinScope` or `incomplete`;
- `conclusion`: `lossDetected`, `noLossDetected` or `notFullyAssessed`;
- `limitations`: descriptions of scope exclusions and remaining assessment gaps.

Recorded loss takes precedence: `lossDetected` retains coverage and limitations.
Without recorded loss, complete scoped coverage gives `noLossDetected`;
incomplete coverage gives `notFullyAssessed`. An empty diagnostic list alone
never proves losslessness. Summaries are constructed from existing evidence
when the operation completes, not by scanning again on access.

The Rust converter's enums correspond to these values; it does not introduce a
serialized converter report format. Its [import coverage](../../crates/ifccad-convert/src/import/COVERAGE.md)
is incomplete; its [export coverage](../../crates/ifccad-convert/src/export/COVERAGE.md)
is complete within the pinned public CadDocument boundary, excluding private
runtime state and original raw CAD bytes. These are distinct operation scopes.

Failed conversions retain their existing typed errors. `LossRejected` retains
detected loss diagnostics but produces no output. Unsupported layout structure,
invalid source structure, insertion failures and package-build errors are also
failed attempts. `Allow`/`Reject` policies and returned `into_parts()` components
are unchanged. Package loading carries no transfer assessment.

## Conformance expectations

`manifestVersion` and `suiteVersion` are independent. An absent manifest version
means v1 (explicit 1 is equivalent for the parser); 2 selects the new
`manifest-schema-v2.json`, which requires `manifestVersion: 2`. The frozen
1.0.0 manifest still parses in its original shape. This is not a promise that
the current reader accepts historical package bodies.

V2 allows optional `expected.packageAssessment` (the complete assessment shape)
and optional `category` on expected diagnostics. V1 expectations retain their
old field projection and cannot use those v2 fields. Unknown enum values and
manifest versions are rejected. The five deferred IFCPR package cases remain
deferred; this change does not add a compatibility-scan operation.
