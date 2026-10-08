# Preservation direction

Status: typed-spline pilots implemented separately for OCDraw and IFCCAD in their provisional 0.1.0 contracts. Broader providers remain follow-up designs; this does not publish or freeze the supported contract. ROADMAP remains authoritative; active schemas and tests remain the source of truth for existing behavior.

## Purpose and boundaries

Preservation retains source information a native drawing cannot yet express. Native content remains editable; retained source content carries capture-time restoration requirements. Saving and reopening a drawing must not require a live original CadDocument.

OCDraw and IFCCAD have separate models, schemas and conversion routes. Their cores remain independent of opencadcodec. The adapter interprets source data and restoration evidence; the file codec transports versioned payload bytes. The ceiling is information the source codec actually retains, not arbitrary lossless DWG/DXF exchange.

Preservation is separate from native semantic expansion and external IFC associations. The retired IFCPR package resource is not revived.

## Selected design direction

A drawing has one preservation collection with source contexts, independently identified records and an allocation watermark. Records distinguish complete source content, supplements to native content, and shared storage. Categories include entities, objects, tables, drawing/layout state and shared data; all use one envelope.

Each record identifies its source namespace/key, distinguishes codecTyped from codecOpaque, and carries a provider-defined versioned payload. It stores source-reference bindings and predicates with capture-time baselines. Predicate/payload bytes remain opaque to the core.

A complete drawable source entity is represented by an opaque entity with a drawing-wide entity ID, preservation-record reference and exactly representable native common properties: layer, appearance/line-pattern scale and visibility. Its ID participates in ordinary ownership and draw order. Unsupported common values remain in the snapshot without invented defaults; optional native layer/appearance records that distinction. Objects, layout/plot supplements and shared sections have no draw-order position. Opaque entities have no native geometry.

Dependencies point from record to requirement. No persistent reverse dependency lists or mutable validity flags are required. Evaluate actual restoration requirements at export or on demand. A missing/changed soft dependency makes restoration ineligible without erasing the snapshot. Opaque-entity/record links and present native property references remain hard references; ordinary native reference edits must remain structurally consistent.

Unknown provider payloads/predicates survive core read/write unchanged. Unknown core fields and envelope versions retain strict rejection/unsupported-version behavior. An adapter checks required predicates and context rather than trusting a qualified label.

Native values are authoritative. An old complete source record cannot overwrite native edits. Raw DWG reuse requires its source identity/version and original typed guards. Shared AcDs storage may require a section/document bundle including non-entity associations; it cannot be treated as an isolated entity blob by default.

Initial OCDraw JSON transport uses Base64 for byte values and a registered opaque entity stream. The logical model contains bytes, not Base64, arbitrary JSON or CAD runtime objects. Compression, sidecars and chunking are separate future decisions.

Opaque geometry makes complete bounds unavailable, including through nested block occurrences. Such scopes use null bounds with completeness derived from content; unaffected fully native scopes retain strict finite-enclosure rules. Inspection must show the native capability gap.

## First slice and follow-up

The initial OCDraw slice and independent IFCCAD adaptation: snapshot every available CadDocument Spline variant in ModelSpace, PaperSpace and supported local block definitions. No degree, knot, weight, flag or control/fit-point profile filters capture. Preserve all source parameters/common data, expose known common properties natively and retain mixed source order. There is no native spline evaluator, bounds calculation or geometry editing operation.

Capture is opt-in and separate from restoration qualification. Ownership/coordinate context and any residual references must be safely reconstructed for export; unresolved attached objects or raw context may prevent restoration while the full available spline snapshot remains stored. Current native layer/appearance/visibility and line-pattern changes are authoritative, so ordinary layer/default/pattern edits do not automatically invalidate spline parameters. Opaque extra data can have stricter qualified reuse rules.

Simple open cubic splines are initial test fixtures, not an input whitelist. Storage tests cover broad variants/bit patterns; actual DXF/DWG tests compare the direct codec chain with the preservation chain in every supported owner kind. Codec restrictions are reported independently of successful capture.

Original entity DWG records are not included in that typed snapshot. Their omission is reported separately from interpreted semantic retention. Full original/private/shared codec storage remains later work, associated with [opencadcodec issue #91](https://github.com/HakanSeven12/opencadcodec/issues/91).

Spline is the first end-to-end pilot for the preservation model. The implemented pilot covers core storage, native common properties, dependencies and restoration evidence, with multiple spline variants and all supported scopes. Keep the codec-specific snapshot implementation isolated and replaceable in its adapter. Broad expansion to other entity families and raw/private/shared codec storage waits for an agreed direction from #91 or a separately approved alternative; a complete upstream implementation is not required to finish the pilot. Integrating it later requires audited payload versioning and continued read support or verified migration for stored pilot snapshots.

IFCCAD adapts the same concepts in a separate profile slice, using its paths, composed attributes and ordered typed layout/block vectors. OCDraw implementation does not establish IFCCAD support or foreign IFCX source-graph writeback.

[Native spline semantics #4](https://github.com/OpenAEC-Foundation/ifccad/issues/4), [irregular typed content #2](https://github.com/OpenAEC-Foundation/ifccad/issues/2), and physical/measurement issues [#3](https://github.com/OpenAEC-Foundation/ifccad/issues/3) / [#6](https://github.com/OpenAEC-Foundation/ifccad/issues/6) retain their independent scope. No measurement or dependency update is authorized by this design.

## Local execution specifications

The complete working specifications are retained locally, outside Git, under docs/superpowers/specs:

- 2026-10-06-preservation-model-design.md: full architectural model, lifecycle, storage and IFCCAD adaptation.
- 2026-10-06-ocdraw-spline-preservation-design.md: full available spline field transport, supported scopes, editable native common properties, restoration conditions, API/codec changes and verification.

These local working documents record the approved design ground and implementation plan; historical proposal/status wording does not override current code and schemas. Exact implemented contracts and verification scope are recorded in the active schemas, candidate conformance and converter coverage.

## Implemented interfaces and evidence

The core exposes OcdrawPreservation with an independent checked record-ID
watermark, DrawingOpaqueEntity, typed targets/bindings/conditions/payloads,
ValidatedOcdraw.preservation()/opaque_entities(), and core-only builder helpers.
Logical byte vectors have no provider runtime, Base64 or arbitrary-JSON fields;
canonical Base64 belongs only to the JSON mapping. Scope lists retain mixed order.

Capture remains opt-in through OcdrawPreservationCapture::SupportedTyped;
RestoreSupported is the export default. Current native properties are
authoritative. Actual missing/unqualified source context receives independent
restoration evidence, and Reject still refuses actual omitted live content.
Unknown provider bytes and soft missing/cyclic dependencies remain transportable.

The [spline snapshot contract](../crates/ocdraw-convert/docs/SPLINE-SNAPSHOT-V1.md)
records every public field, payload shape, mandatory predicates, known typed
reference rebinding, omitted raw records, and the actual DXF/AC1032 DWG matrix.
Library-only CadDocument inputs use cadDocument provenance; the file inspector
records the explicitly known DXF/DWG origin and version before core encoding.

The inspector's Spline-brongegevens bewaren option applies only to the OCDraw
CAD route. It reports opaque counts, source provenance, unavailable bounds and
separate capture/restoration evidence. The CLI supports --preserve-splines for
cad/export-cad. Native opening and IFCCAD conversion do not require this option.
## Independent IFCCAD mapping

IFCCAD owns its envelopes, paths, validation and restore predicates. Its ordered
owner vectors contain `IfccadEntity::Native(IfccadNativeEntity)` or
`IfccadEntity::Opaque(IfccadOpaqueEntity)`. Strong required native properties stay
on the native variant. Opaque layer and the complete appearance/scale bundle
are optional; visibility remains independently editable. No geometry/default
layer/appearance is invented. The codec-only `cad-preservation` companion shares
only the audited opencadcodec byte DTO with both converters. Existing spline
payload v2/current 063c106 and v1/legacy fe69506 bytes remain compatible.

The IFCX mapping uses `ifccad::opaqueEntity`, `ifccad::preservation` and
`ifccad::preservationRecord`; see the active IFCCAD experimental contract.
The record watermark lives in `IfccadIdCounters`, independently of entity IDs.
Unknown provider bytes and soft missing/cyclic dependencies remain transportable.
Complete live records match their opaque subjects; detached archives have no
export obligation. The original IFCX graph stays immutable, and graph-aware CAD
conversion still diagnoses independent foreign information.

IFCCAD predicate version 1 names are `openaec.ifccad.splineEntityBinding`,
`openaec.ifccad.splineCoordinateContext` and
`openaec.ifccad.sourceReferenceBinding`. Entity baselines retain the complete
record and owner paths. Model/block meaning uses the drawing unit; blocks also
check insertion unit. Paper meaning is the exact reduced rational metres per
coordinate from effective plot mapping. Unknown is explicit. Equivalent mappings
remain eligible, changed meaning refuses, and reverting restores eligibility.
Renames, draw-order edits, block base/instance transforms and common-property
edits do not refresh baselines. Bounds distinguish Empty/Complete/Unavailable;
opaque-only children never use the empty-block origin fallback. Dormant same-owner
exclusive opaque clips are legal; active opaque clip boundaries are rejected.

Known source references bind through actual allocated CAD targets, including
forward typed XDATA handles. Inherited ByLayer/ByBlock handles use Drawing-role
bindings with provider source keys `byLayer:<hex>`/`byBlock:<hex>`; this avoids
inventing ordinary native pattern IDs for codec scaffold tables. Unresolved or
changed required bindings and unsupported attached/raw/application context refuse
restoration while keeping source bytes. No provider registry, generic SCC restore,
raw source record replay or native spline evaluator is added.

Library capture defaults to Disabled; RestoreSupported is the export default,
with explicit Skip and located loss under Allow/Reject. The explorer forwards its
existing CAD-input preservation choice through either route. Its inspector shows
opaque records and separately captured/restored evidence, plus unassessed geometry.
CLI CAD-input commands accept `--drawing-format ifccad|ocdraw` and
`--preserve-splines`; default remains OCDraw. Native input follows its extension.

On this pin, Uniform/SquareRoot fit-only parameterization survives snapshot storage
and typed restoration, and the qualified DWG chain retains it. Current DXF physical
readback returns Chord (0). Both routes report the separately located
`TARGET_CODEC_SPLINE_PARAMETERIZATION_LOSS`; successful typed restoration does not
qualify that curve's DXF geometry. The codec pin, local patches and viewer codec are
unchanged by this pilot; [codec PR #99](https://github.com/HakanSeven12/opencadcodec/pull/99)
is a separate adoption step. Bounds/numeric evidence remain incomplete for opaque
geometry even when its parameters restore.
