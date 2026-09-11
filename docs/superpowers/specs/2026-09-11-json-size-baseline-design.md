# Initial IFCCAD JSON, DXF and DWG size baseline

Date: 2026-09-11

Status: Implemented and verified. The successful patched run is accepted as
milestone 2's scoped initial measurement reference. See the
[complete experiment report](../../benchmarks/size-baseline-v1.md).
The default upstream pin still lacks the two documented DWG fixes; their
integration remains separate codec maintenance. This does not claim a passing
unpatched upstream baseline. The final report consolidates the successful run;
development diagnostics remain local.
Corpus choice approved: existing fixtures and
deterministically generated drawings; real-world drawings follow later.
Comparative scope updated after review: include both DXF and DWG using the
already pinned cadcodec writers.

## Purpose and scope

Establish a repeatable accounting method and compare the current IFCCAD JSON
representation with DXF and DWG for equivalent controlled drawings. Component
accounting explains where size differences arise. This completes the remaining measurement slice of roadmap
milestone 2 once implemented and verified. The benchmark is observational, not
a file-size requirement or a format conformance rule.

[Issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6) describes a
larger program. This slice implements its first fixture/generated-case baseline.
Real-world coverage grows in milestone 3; alternative IFCCAD compression and
binary experiments, timings, peak memory and container measurements remain in
the later roadmap measurement work. Ratios apply to this corpus and these
specific writers, not to representative CAD practice or every DWG producer.
Issue #6 remains open after this slice.

Issues #1 and #7 supply the already implemented source/model boundaries used by
the experiment. Issues #2–#4 concern later payload, chunking and geometry work;
the baseline does not depend on resolving them. Historical issue descriptions
do not override the current roadmap or completed architecture.

## Approach

Use a standalone Rust example in ifccad-convert, built on the production package
builder/reader and the existing cadcodec dependency. This keeps generation,
measurement and validation in one command without adding a CAD runtime to core.
It is development tooling, not a new public library API.

The final cadcodec pin `2f2cd25832db298524fb5eb36ced5a438a877e95` exposes
both DxfWriter::write_to_vec and DwgWriter::write_to_vec, plus both readers.
Its DWG writer selects the version from CadDocument.version and includes AC1032
roundtrip tests. Use DxfVersion::AC1032 explicitly for both CAD outputs, textual
DXF via DxfWriter::new, and the normal DWG writer with its native compression.
Do not use the diagnostic no-compression DWG entry point. Record these settings
and the exact writer revision in the measurement results.

Fixtures alone would mostly measure small package overhead. A real-world corpus
would add representativeness but currently includes semantics outside the native
subset. The selected combination of existing fixtures and controlled generated
cases measures overhead, growth and storage placement with known content.

Measure exact stored bytes. The current writer uses pretty JSON for IFCX and
IFCDR; that output is the reference. Keep fixture formatting unchanged and label
fixture measurements separately from generated writer output. Minification and
compression would be separate future experiment variants with their own byte
integrity and equivalence checks. Label the comparison accurately: current
uncompressed pretty IFCCAD JSON versus textual DXF versus normal compressed
DWG. The ratios measure these outputs; they do not isolate a compression gain
or prove a lower bound on future IFCCAD storage.

## Initial corpus

Use these existing fixtures without editing or copying them into conformance:

| Fixture under conformance/next/packages/valid | Purpose |
| --- | --- |
| minimal-no-preservation | Existing small native drawing |
| inline-drawing | Existing inline counterpart |
| source-archive | Existing preservation-inclusive accounting, with metadata and shared blob bytes separate |

Add nine generated logical drawings, each emitted as external IFCCAD, inline
IFCCAD, textual DXF and DWG:

| Family | Entity counts | Geometry and purpose |
| --- | --- | --- |
| Empty | 0 | Fixed package, layer and appearance overhead |
| Lines | 100; 10,000 | Regular finite XY endpoints; growth per line |
| Short polylines | 100; 10,000 | Four vertices per polyline |
| Long polylines | 10; 1,000 | 128 vertices per polyline; pool growth |
| Mixed | 1,000 | Alternating 500 lines and 500 four-vertex polylines in draw order |
| Fractional lines | 1,000 | Reproducible varied coordinates with fractional values, to expose numeric-text sensitivity |

This yields 36 generated output variants and three fixture-accounting rows.
Each logical case has one DXF and one DWG reference shared by its two IFCCAD
comparisons. Existing fixtures retain accounting-only rows: their complete
package or preservation semantics are not assumed transferable to CAD.
Counts are
small enough for routine local runs and stay within production loading limits.
They are controlled test cases, not a performance stress test.

Generated cases use millimetres, one model layout, fixed package/resource IDs,
author, timestamp, names and insertion order. Basic families use one visible
layer and shared ByLayer appearance. The mixed case uses four layers, alternates
ByLayer and a shared explicit appearance, and includes open/closed polylines
and invisible entities according to fixed rules. Geometry and supporting data
are identical across the four output variants. Choose exactly representable
shared appearance values (opaque RGB colors, supported line weights and line
patterns) so the benchmark does not depend on unassessed color metadata or
opacity quantization.

A common typed benchmark recipe drives both PackageBuilder and CadDocument
construction. This avoids treating one serialized format as the authoritative
source and allows both IFCCAD storage modes without changing converter options.
Recipe adapters are measurement code, not a new general conversion API.

The generator fixes and documents its coordinate formulas, integer sequence
and seed in corpus version 1. It uses no clock, platform randomness,
transcendental functions or implicit coordinate rounding. Changes to formulas,
counts, metadata or selected fixture contents require an explicit corpus
revision. No new entity semantics are introduced.

## Accounting rules

An IFCCAD row describes one complete directory-package representation; a CAD
row describes one complete DXF or DWG file:

- Record every included file's package-relative path, role, exact byte count
  and SHA-256. Count each physical file once, including a blob referenced by
  several preservation records.
- Report IFCX bytes, external IFCDR bytes, external IFCPR metadata bytes, blob
  bytes and total bytes. The total equals the sum of the listed file lengths.
- Inline bodies are already part of IFCX bytes. External-resource byte columns
  are zero when no corresponding separate file exists. Do not estimate inline
  body lengths by reserializing them or add them again to the total.
- Record the storage mode, native-only versus preservation-inclusive case kind,
  drawing/scope counts, line count, polyline count and polyline vertex count.
  A vertex count means stored polyline vertices; line endpoints are separate.
- Total bytes per entity is defined for nonempty native-only rows. IFCDR bytes
  per polyline vertex is defined only for external native polyline-only rows;
  it includes IFCDR supporting-data overhead. Use null when a ratio is not
  meaningful, including empty or preservation-inclusive cases.
- Count logical file bytes, not disk allocation or directory metadata. There is
  no container in these measurements, so container overhead is not applicable.
- CAD rows record complete file bytes, hash, writer/version, shared case identity
  and semantic verification result. Do not strip CAD headers or standard tables
  to improve ratios; their storage is part of the selected writer's output.
- For each verified generated case and IFCCAD storage mode report
  IFCCAD-total/DXF-bytes and IFCCAD-total/DWG-bytes. Preserve integer byte counts
  alongside derived ratios. Empty drawings may have total-file ratios, while
  their bytes-per-entity remains null. Fixture-accounting rows have no CAD ratio.

For fixtures, a reviewed corpus inventory explicitly lists included resources
and blobs. Exclude the conformance helper package.json, README files and other
test infrastructure. Check the listed files exist; do not infer full IFCPR
resource discovery or validation from this inventory.

The source-archive fixture gets its own preservation-inclusive row. Component
subtotals do not become a hypothetical native-only package size by subtracting
IFCPR and blobs: its IFCX graph still contains preservation declarations. Use
actual native-only packages for native-only totals.

## Validation and repeatability

Build each generated drawing from its recipe through PackageBuilder and
CadDocument, write all four outputs to fresh paths, and reload them with the
production IFCCAD reader and cadcodec's DxfReader/DwgReader respectively.
Generated/native-only IFCCAD cases must
produce strict typed access and a complete valid assessment. Compare the
loaded drawing against its generation recipe: geometry, IDs, order, units,
layer and appearance relationships, visibility and polyline closed flags.
All four outputs must reproduce the recipe's supported drawing semantics.
Use benchmark-local semantic projections for comparison, covering every generated
entity in drawing order, its exact XY coordinates, layer, appearance modes and
values, visibility, polyline vertices/closed flag and drawing unit. Include
recipe-defined layers even if empty. Numeric handles and format-specific default
tables are technical storage details; preserve semantic references while allowing
those identifiers to differ. IFCCAD IDs must agree across its storage modes.

Do not infer equivalence from entity counts or an empty converter diagnostic
list. Do not introduce coordinate tolerances or omit a generated property merely
to make a comparison pass. If a pinned reader/writer changes the test content,
diagnose the mismatch and leave that case's comparative result unavailable.
No successful full-corpus baseline is claimed until every generated case passes.
This is a controlled check through the pinned implementation, not independent
certification that every CAD application accepts its output.

Also exercise the production converter for the nine generated recipes:
IFCCAD -> drawing_to_cad_document -> textual DXF/DWG -> the corresponding CAD
reader -> cad_document_to_package -> the production IFCCAD reader. Use Reject
for export and retain all import/export diagnostics and assessment limitations.
Verify the recipe-defined semantics at each boundary, including the final
IFCCAD drawing. Compare entities by semantic order and relationships, allowing
new handles/IDs when a conversion explicitly reallocates them. Independent
writer/readback checks alone must not be labelled an end-to-end conversion test.

The direct recipe outputs remain the baseline size comparisons. Keep chain
outputs separately labelled so rewritten CAD metadata or conversion effects
are not mixed into those size ratios. A chain failure is reported by case,
direction and first differing property. It prevents a successful full-corpus
exchange conclusion; do not silently discard extra CAD metadata or relax the
converter loss policy to obtain a passing result. Existing aggregate import
coverage can remain incomplete while the explicit recipe properties are proven
equal; report both findings without promoting that bounded check to a general
losslessness guarantee.

Existing fixtures also pass the production reader. Record their package
assessment. The source-archive fixture is expected to retain the current
preservationSemanticsNotAssessed gap; byte accounting does not claim to prove
preservation validity or source-restoration fidelity.

Repeat generation and accounting twice under the same build and corpus. Fix
CAD header dates and other controllable generated metadata as well as IFCCAD
metadata. File lists, bytes, hashes, counts and deterministic measurement fields
must agree for all four variants. Nondeterminism in an external writer is an
investigation result, not permission to silently rewrite its serialized bytes.
Compare typed logical values directly; byte hashes identify measured artifacts
and are not a new semantic fingerprint format.

Record corpus version/hash, repository revision and dirty state, rustc version,
resolved dependency versions and the active Cargo.lock digest. Preserve the run's
lockfile and provenance alongside local outputs. Cargo.lock is currently ignored
by the library repository; this experiment does not change that policy. Exact
byte comparisons require the same generator and serialization dependencies.
Avoid timestamps and local absolute paths in the deterministic result body.

A missing file, failed build/load, unexpected assessment, semantic mismatch,
overflow or nondeterministic result fails the command. Report the case and
cause; do not substitute zeros or present a partial run as a successful baseline.

## Deliverables and verification

- A runnable companion-crate example at crates/ifccad-convert/examples/size_baseline.rs, with a small
  versioned corpus inventory under benchmarks/size/.
- Generated packages, repeated-run artifacts and environment details below a
  newly created target/size-baseline/ run directory. Existing run directories
  are never overwritten or cleaned automatically.
- A machine-readable initial baseline and a concise report under benchmarks/size/
  and docs/benchmarks/, with exact reproduction instructions, component totals,
  inline/external and IFCCAD/DXF/DWG comparisons and limitations. Curated baseline results are
  versioned; generated package files remain local artifacts.
- Focused tests for accounting sums, shared-file deduplication, inline counting,
  zero-entity ratios and failure handling; production-reader semantic checks for
  all four output variants of generated cases. Baseline numbers are observations, not brittle size thresholds
  in ordinary correctness tests.
- Run the full corpus twice and the repository's formatter, Clippy and workspace
  tests. Review milestone 2 exit criteria before marking it complete, and keep
  README and ROADMAP status synchronized.

No format-body schema, conformance fixture, converter coverage, encoder behavior
or resource-selection policy changes are required. The specification and
implementation remain on size-baseline until repository integration is requested.

## Report delivery

Deliver a readable Markdown report at docs/benchmarks/size-baseline-v1.md and
machine-readable results at benchmarks/size/results-v1.json. Present the report
in the task with a short conclusion and clickable links. Large generated files
remain in the local run directory and are linked for manual inspection rather
than committed as part of the report.

The report contains:

1. The measured corpus, exact writer versions/settings and the overall outcome.
2. A comparison table per generated drawing: entity/vertex counts, external and
   inline IFCCAD totals, DXF bytes, DWG bytes, and the two reference ratios for
   each IFCCAD storage mode. State units explicitly; preserve exact byte counts.
3. IFCX/IFCDR/IFCPR/blob accounting, with preservation-inclusive fixtures separate.
4. An exchange table identifying each executed direct readback and conversion
   route, its semantic-check result and any diagnostics or unassessed areas.
   For a failure, show the first differing field and source/output artifact paths.
5. The bounded interpretation: fixed overhead, growth with entities/vertices,
   source placement, current pretty JSON versus normal CAD writer output, and
   findings that cannot yet be generalized to real-world CAD applications.
6. Reproduction commands and provenance, plus a local artifact index for
   inspecting the generated IFCCAD directories, DXF and DWG files.

A failed run may produce an explicitly incomplete diagnostic report, but cannot
replace the accepted baseline or mark milestone 2 complete. The task summary
must distinguish measured size results, successful exchanges and unresolved
failures; one aggregate pass label must not hide unexecuted routes.
