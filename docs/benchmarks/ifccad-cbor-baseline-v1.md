# IFCCAD JSON and base-CBOR: small initial baseline

Final scope: on 2026-10-16 the user agreed to omit test.dxf from this initial
baseline because conversion preparation was disproportionately expensive.
The four recipes and foundation practice drawing have qualified results; see the
[five-case summary](ifccad-cbor-results-summary-v1.md). The originally approved
six-case protocol and failed-attempt history below remain for provenance.
No test.dxf size or timing result qualified, and no additional run is implied.

Status: reduced six-case protocol approved on 2026-10-16. The initial run
qualified five cases; `test.dxf` failed conversion before encoding. See the
[partial results](ifccad-cbor-results-v1.md). A separately authorized retry excludes
only dimension `0xEF354` from an in-memory copy of `test.dxf`; it does not alter
the original file, converter behavior or tolerances. The first retry attempt
stopped before execution after ten minutes waiting for unrelated Cargo/test
activity. A subsequent quiet-host attempt excluded that one entity but failed
conversion on dimension `0xEF3B7` with the same geometry error; see the
[retry evidence](ifccad-cbor-results-test-dxf-v1.md). No test.dxf encoding
size/timing result has qualified. The subsequently authorized dimension-error
rule excluded 31 dimensions, then failed on a different LWPOLYLINE error; see
[dimension-exclusion attempt](ifccad-cbor-results-test-dxf-v2.md). Main and
existing accepted OCDraw reports remain untouched.

## Purpose

Establish a useful first observation of file size for current IFCCAD JSON,
whitespace-compact JSON and base-CBOR, with indicative production read/write
timings. This is a small initial comparison, not exhaustive entity coverage,
scaling research or a general format-performance claim.

## Six cases

| Case | Input / size | Reason |
| --- | --- | --- |
| practice-foundation | Original Funderingsherstel DWG | Natural mixture of geometry, text, dimensions, hatches and blocks |
| practice-test-dxf | Original 3bm/test.dxf | Large practice drawing with substantial polyline geometry |
| lines-10000 | 10000 XY lines | Repeated simple records and coordinate/default overhead |
| polylines-10000 | 10000 four-vertex planar polylines | Vertex arrays and many small paths |
| text-1000 | 1000 objects, alternating TEXT/MTEXT | Strings, shared styles and modest rich-text structure |
| hatches-1000 | 1000 hatches cycling solid/pattern/gradient | Nested boundaries and richer records |

One fixed size per recipe. No separate cases for every entity kind, extra scale
series, mixed scaled recipe or empty-drawing case in this initial round. Other
families are observed through whatever the current practice conversion retains;
missing families are reported rather than implying complete coverage. Add later
cases only if observations leave a concrete question unresolved.

Generate fresh typed documents with deterministic metadata, IDs, coordinates and
strings. Use mm for synthetic recipes; retain practice units. Four shared layers,
fixed shared styles and integer/simple binary-fraction coordinates suffice. Keep
blocks shared in practice inputs. Recipes use no example-file copying, random
source, coordinate quantization, tessellation or timing-dependent metadata.

## Original sources

The originals exist below source-root label `IFCCAD Prototype`; the following
identities were confirmed during design. This was file inspection, not a CAD
conversion or measurement run.

| Case | Relative original path | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| practice-foundation | DXF DWG samples/3bm/2705_model Funderingsherstel - Constructie - Sheet - CP-21 - Constructietekening.dwg | 239012 | 24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5 |
| practice-test-dxf | DXF DWG samples/3bm/test.dxf | 33288016 | 74d8d7b861cda0e3859879c8149df2344a457f5c9cefce6a144a7cf52a4fbe9a |

Start each practice case from its original source using the current selected
codec, direct cad_document_to_ifccad_document, Allow for diagnosed restrictions
and SupportedTyped preservation. Structural/numeric failures remain hard errors.
Retain complete conversion diagnostics and counts by kind/owner before and after
conversion. Count stored entities/vertices and shared definitions without expanding
block instances. Include opaque/preservation byte totals when present. No common
OCDraw intersection and no silent omission to improve a size result.

The `test.dxf` retry has one explicit exception authorized after the initial
failure: exclude dimension `0xEF354` (handle 979796), whose geometry assessment
reported a 543.9999999999972 mm deviation against the default 1e-9 mm tolerance.
Record the original and filtered conversion inventories and the exclusion in
both independent preparations. Other conversion errors remain failures. This
measures the retained filtered drawing, not the full original drawing. The source
inventory's `polylineVertices` counts LWPOLYLINE vertices only; the native
inventory counts both supported polyline kinds.

After the one-entity retry failed on a second dimension, the user authorized
excluding all dimensions with this specific geometry failure for this measurement.
The explicit `point2-proven-exceedance` policy applies only to `practice-test-dxf`:
an original CAD DIMENSION in the Drawing domain, deviation assessment at vertex
index 2, ProvenExceedance with the default geometry tolerance. The actual
production error must identify the entity; no generic error-text parsing or
all-dimension filtering is used. Other failures still reject the case. Every
removed entity and complete failure reason is recorded, together with original
and filtered inventories. Removal/reconversion is untimed preparation and is
repeated independently from original source bytes. Production semantics and
tolerances remain unchanged.

After that attempt reached a different LWPOLYLINE failure, the user approved
benchmark-only exclusion of any failed entity that the conversion error identifies
unambiguously. The `identified-entity` policy uses a typed CAD source handle
(including an occurrence's leaf) or the explicit source `entity/0xHEX` location
on preparation/structure errors and fatal precision diagnostics. It never guesses
CAD handles from target IFCCAD IDs or free error text. The source entity must
still exist before each removal, so every retry makes progress. Global errors,
unidentified sources and strict encoding/readback failures still reject the case.
All exclusions include kind, handle, owner and full production error text; original
and filtered inventories remain separate. This is a filtered-content experiment,
not evidence of complete source retention. No production behavior changes.

The [placement inventory](placement-preparation-v1.md) and
[historical large-DXF report](common-subset-test-dxf-v1.md) describe older coverage;
the new conversion inventory must establish what is actually retained now.

## Same document, three artifacts

Prepare one validated frozen IfccadDocument per case. Production JSON encode/readback
may normalize its documented unordered definition/set order once during preparation;
record that normalization and reject content/numeric changes. Preparation and CAD
conversion remain outside native codec timing.

From the frozen document write production JSON and production base-CBOR (revision
2). Derive compact JSON by removing whitespace outside strings only, preserving
all tokens, strings, escapes and number spellings. Read all three artifacts with
the production reader, strict shared validation and full typed comparison with
an exact numeric-bit guard. Keep IDs, watermarks, owner order and preservation.
Built-in writer validation/readback remains enabled. No public semantic fingerprint
or JSON bridge is added to the binary product codec.

## Minimal measurements

File size is the main result. Report bytes for all three artifacts and CBOR's
ratio to compact JSON. Original CAD sizes identify inputs; they are not equal-content
comparison rows when conversion has diagnosed losses.

For an indicative timing result, use the existing release profile and own worktree
cache. Compile first. Use preloaded bytes and the prepared document: one warmup,
five samples per operation, median and observed range. Rotate variant order; keep
returned-value disposal outside the timed call. Retain raw samples and qualify
outputs outside timing. No separate timing framework, filesystem throughput,
cold-start, memory profiling or statistical performance claim.

Time production opening for JSON, compact JSON and CBOR. Time production writing
for JSON and CBOR only: compact JSON has no separate production writer. Compact
JSON filtering is auxiliary preparation, not a third equivalent writer. JSON
opening also retains the IFCX source graph; CBOR returns the validated drawing.
Label these as complete production-route costs, not equal token-parsing work.

Generate each case once for the main artifacts. One untimed repeat generation
checks deterministic bytes/content; it is not a second timing campaign. Practice
repetition starts from the original source again. Do not require identical timings.
Do not run competing builds during timed sampling or change dependencies/profiles.

## Small manual tool and retained evidence

Use one IFCCAD-focused manual example in ifccad-convert with a small recipe/helper
module. Avoid modifying or executing the historical four-format/gzip runner.
Core storage remains independent of CAD runtimes. Unit tests use tiny documents
for correctness and fabricated durations for statistics, without benchmark runs.

A run takes an explicit selected case set, original source paths and a fresh
output directory. The runner can generate a unique run name. Refuse overwrites;
never run unselected cases or add measurements to CI/completion gates. Build and
qualification do not themselves run the controlled experiment.

Retain a concise table plus matching detailed JSON: source hashes, selected cases,
actual converted inventories/losses, artifact hashes/sizes, exact readback outcomes,
timing samples and tool/machine identity. Capture dirty/untracked source, lockfile,
configuration and selected dependency/patch hashes before and after. Preserve
applicable reproduction evidence outside target/ and leave earlier accepted
reports/results intact. Failed cases stay explicit and out of accepted rows.

Use current aligned codec 325fa98d2b881f48e106a830d5cf161624f63131 with its explicit
local main patches; do not call this an unmodified-codec result. Use focused
correctness checks during tool development. The full gate for the final integration
candidate remains pending; no main merge is required for a later authorized run.

Compression, columns, DWG/DXF output comparisons, broader family recipes and
additional scales remain later choices driven by the first observations. This
addresses a small part of issue #6, not the chunking or associative-update work.
Actual execution requires an explicit measurement request for the selected scope.

## Reproduction

From the cbor-base worktree, using its recorded dependency/configuration:

```text
python scripts/ifccad_cbor_baseline.py --cases lines-10000,polylines-10000,text-1000,hatches-1000,practice-foundation,practice-test-dxf --source-root PATH_TO_IFCCAD_PROTOTYPE --run NEW_RUN_NAME
```

The launcher builds release first, qualifies original source hashes, waits for
other Cargo/compiler processes, captures input identity and executes exactly the
selected cases. Retained report paths are create-new; a subsequent measurement
needs an explicit request and a fresh report version as well as a fresh run name.
The selected run's source/config/input archive is retained outside target/.

For the explicitly authorized single-case retry, use fresh output paths:

```text
python scripts/ifccad_cbor_baseline.py --cases practice-test-dxf --source-root PATH_TO_IFCCAD_PROTOTYPE --run NEW_RETRY_NAME --omit-test-dxf-entity EF354 --result benchmarks/ifccad-cbor/NEW_RESULTS.json --report docs/benchmarks/NEW_REPORT.md
```

This flag supports only the named exclusion and selects no additional cases.

For the subsequently approved error-class exclusion, use instead
`--omit-test-dxf-dimension-errors point2-proven-exceedance`, again selecting only
`practice-test-dxf` with fresh run, detailed result and report paths. The original
failed attempts and the five qualified cases remain retained unchanged.

For the final approved entity-error scope, use
`--omit-test-dxf-entity-errors identified-entity`, again with only
`practice-test-dxf` and fresh output paths.

The optional preparation tooling can reuse a source-specific exclusion manifest
with `--exclusions benchmarks/ifccad-cbor/exclusions-test-dxf-v1.json`. The Python
launcher verifies the original source hash, production source/schema inputs,
selected dependency, configuration and toolchain against the recorded proof.
The Rust preparation checks each source handle, kind and owner before removal.
A cheap degenerate-bulge candidate check only permits removal after shared
production primitive preparation rejects it as invalid geometry. Independent
repeat generation starts from original bytes and reapplies the final fixed
exclusion list; it compares the filtered document, diagnostics and encoded bytes,
without rediscovering the excluded errors one by one. This batching code passed
focused tests; the real-file attempt was stopped during its release build when
test.dxf was removed from scope. No practical speed improvement was measured.
