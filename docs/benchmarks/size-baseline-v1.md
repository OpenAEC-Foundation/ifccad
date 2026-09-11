# IFCCAD, DXF and DWG: controlled size and exchange experiment

**Result, 2026-09-11:** all direct readbacks and complete conversion chains pass
for the controlled line/polyline corpus. Two independent generations produce
identical bytes. This document contains the experiment's method, results,
interpretation and reproduction instructions; [results-v1.json](../../benchmarks/size/results-v1.json)
contains the corresponding detailed measurements and provenance.

## Purpose and scope

The experiment measures current IFCCAD JSON output size against text DXF and
native compressed DWG, and checks exchange of the supported drawing semantics.
It establishes a repeatable reference for future changes. It does not select
a new physical encoding or measure speed, memory, arbitrary CAD entities or
compatibility with external CAD applications.

The measured implementation uses cadcodec/acadrust 0.5.4, commit
`2f2cd25832db298524fb5eb36ced5a438a877e95`, plus the two
[local DWG fixes](../../patches/cadcodec-upstream/README.md) for current lineweight
and MEASUREMENT. The table therefore has one DWG variant. The normal dependency
pin does not yet include these fixes; issues
[#41](https://github.com/HakanSeven12/cadcodec/issues/41) and
[#42](https://github.com/HakanSeven12/cadcodec/issues/42) track them. These are
verified local results. On 2026-09-11 this run was accepted as the scoped initial
measurement reference for milestone 2. The recorded `baseline_accepted: false`
is unchanged: the experiment's automatic gate does not accept local overrides
as an unpatched upstream baseline. Upstream integration and a repeat run remain
separate codec maintenance; they do not block the established logical model.

## Corpus and method

The [versioned inventory](../../benchmarks/size/corpus-v1.json) defines nine
deterministic generated drawings and three unchanged package fixtures. All
generated drawings use millimetres, one model layout, XY lines and straight
polylines. Short polylines have four vertices; long ones have 128. The mixed
case exercises interleaved entity order, layers, visibility, open/closed
polylines and explicit/ByLayer appearance. Fractional coordinates use exact
binary fractions. The empty case retains a layer and exposes fixed overhead.

| Drawing | Lines | Polylines | Polyline vertices |
| --- | ---: | ---: | ---: |
| empty | 0 | 0 | 0 |
| lines-100 | 100 | 0 | 0 |
| lines-10000 | 10,000 | 0 | 0 |
| short-polylines-100 | 0 | 100 | 400 |
| short-polylines-10000 | 0 | 10,000 | 40,000 |
| long-polylines-10 | 0 | 10 | 1,280 |
| long-polylines-1000 | 0 | 1,000 | 128,000 |
| mixed-1000 | 500 | 500 | 2,000 |
| fractional-lines-1000 | 1,000 | 0 | 0 |

Each drawing is constructed directly from the same typed recipe in four forms:
external IFCCAD, inline IFCCAD, text DXF and DWG (both CAD formats AC1032).
The CAD preparation fixes dates and allocates missing layer handles. IFCCAD
uses the current pretty-JSON writer without compression; DWG uses its normal
native compression. Production readers load every output. Separately, complete
IFCCAD → DXF → IFCCAD and IFCCAD → DWG → IFCCAD chains exercise both converters,
with Reject enabled on return and strict validation of the final IFCCAD package.

Comparisons check every entity in order: exact XY geometry, closure, layer,
visibility, appearance modes/values, drawing unit and all layers, including
unused ones. IFCCAD storage modes also preserve ordered entity IDs. CAD handles
and internal appearance labels are outside this semantic comparison. The
corpus exercises only its declared values; passing does not establish fidelity
for untested opacity quantization or broader import/preservation semantics.

All sizes are actual complete-file bytes. Each physical file or shared blob
counts once. Inline resource content counts inside IFCX, never again as IFCDR.
Conversion-chain artifacts are separate from the four directly measured outputs.
Per-file hashes and IFCX/IFCDR/IFCPR/blob subtotals are in the result JSON.

## Size results

| Drawing | External IFCCAD, bytes | Inline IFCCAD, bytes | DXF, bytes | DWG, bytes |
| --- | ---: | ---: | ---: | ---: |
| empty | 4,899 | 6,266 | 47,078 | 20,438 |
| lines-100 | 16,380 | 26,887 | 60,438 | 21,846 |
| lines-10000 | 1,224,739 | 2,126,246 | 1,444,724 | 167,417 |
| short-polylines-100 | 25,918 | 43,435 | 79,498 | 22,070 |
| short-polylines-10000 | 2,243,867 | 3,845,384 | 3,396,624 | 187,415 |
| long-polylines-10 | 41,463 | 69,380 | 114,414 | 23,414 |
| long-polylines-1000 | 3,898,705 | 6,540,222 | 7,026,611 | 256,225 |
| mixed-1000 | 187,842 | 324,709 | 312,593 | 41,670 |
| fractional-lines-1000 | 154,948 | 246,455 | 215,971 | 46,750 |

Ratios below divide the complete IFCCAD package size by the complete CAD file
size. A value below 1 means the IFCCAD output is smaller.

| Drawing | External / DXF | External / DWG | Inline / DXF | Inline / DWG |
| --- | ---: | ---: | ---: | ---: |
| empty | 0.104 | 0.240 | 0.133 | 0.307 |
| lines-100 | 0.271 | 0.750 | 0.445 | 1.231 |
| lines-10000 | 0.848 | 7.315 | 1.472 | 12.700 |
| short-polylines-100 | 0.326 | 1.174 | 0.546 | 1.968 |
| short-polylines-10000 | 0.661 | 11.973 | 1.132 | 20.518 |
| long-polylines-10 | 0.362 | 1.771 | 0.606 | 2.963 |
| long-polylines-1000 | 0.555 | 15.216 | 0.931 | 25.525 |
| mixed-1000 | 0.601 | 4.508 | 1.039 | 7.792 |
| fractional-lines-1000 | 0.717 | 3.314 | 1.141 | 5.272 |

The empty files show substantial fixed CAD overhead. With many entities or
vertices, pretty JSON is much larger than DWG. External IFCCAD remains smaller
than this text-DXF output in every measured case. These are properties of the
selected writers and generated corpus, not universal format rankings.

### Fixture accounting

Fixtures exercise package accounting and expected validation assessments;
they are not converted into the CAD comparison files above.

| Fixture | IFCX bytes | IFCDR bytes | IFCPR bytes | Blob bytes | Total bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| minimal-no-preservation | 3,467 | 4,056 | 0 | 0 | 7,523 |
| inline-drawing | 9,819 | 0 | 0 | 0 | 9,819 |
| source-archive | 4,119 | 4,056 | 4,269 | 94 | 12,538 |

The first two fixtures are native packages with complete valid assessments.
The source-archive fixture includes preservation data and one shared 94-byte
blob. Its expected assessment is incomplete for IFCPR semantics, so its total
must not be presented as a native-only package size.

## Exchange and repeatability

| Executed check | Result |
| --- | --- |
| External IFCCAD production readback | 9/9 pass in each generation |
| Inline IFCCAD production readback | 9/9 pass in each generation |
| DXF production readback | 9/9 pass in each generation |
| DWG production readback | 9/9 pass in each generation |
| IFCCAD → DXF → IFCCAD, Reject and strict final validation | 9/9 pass in each generation |
| IFCCAD → DWG → IFCCAD, Reject and strict final validation | 9/9 pass in each generation |
| Expected fixture assessments | 3/3 pass in each generation |
| Direct output hashes and sizes | All 36 outputs identical between generations |
| Full generated artifact inventory | Identical between generations |

The converter recomputes IFCDR bounds from emitted geometry; cached CAD extents
are derived data. Drawing limits and other unsupported settings retain loss
diagnostics. No loss policy was relaxed for these chains. The import report
still declares its broader coverage limitations; exact recipe comparisons
establish agreement only for the values exercised here.

Verification of the measured implementation: formatter and Clippy pass;
335 workspace tests pass, with one existing ignored test. Six additional
cadcodec regression tests pass, covering DXF ordering and the two DWG fixes.

## Explanatory checks: whitespace and compression

These small checks explain the observed sizes; they are not additional
supported IFCCAD encodings or formal benchmark variants.

Inline IFCDR sits deeper inside IFCX JSON. Pretty printing adds indentation to
every nested coordinate line. Removing optional whitespace with compact JSON
serialization from the mixed drawing gives the following totals; reparsing
confirms the JSON values remain equal:

| Mixed drawing | Pretty JSON bytes | Compact JSON bytes |
| --- | ---: | ---: |
| External | 187,842 | 63,489 |
| Inline | 324,709 | 63,378 |

Thus the measured inline overhead comes almost entirely from formatting.
Inline storage does not intrinsically require more data than external storage.

For a separate compression check, each external JSON file from the first
generation was compressed individually with Python gzip, level 9, mtime 0.
Totals include gzip headers and exclude an archive container. The same gzip
settings were also applied to each complete DWG file. Decompression was
checked byte-for-byte against every original JSON and DWG file.

| Drawing | Pretty JSON bytes | JSON + gzip bytes | DWG bytes | DWG + gzip bytes |
| --- | ---: | ---: | ---: | ---: |
| lines-10000 | 1,224,739 | 56,390 | 167,417 | 96,772 |
| short-polylines-10000 | 2,243,867 | 85,270 | 187,415 | 104,867 |
| long-polylines-1000 | 3,898,705 | 70,589 | 256,225 | 141,246 |

DWG benefits from additional gzip despite its native compression. Compressed
IFCCAD still remains smaller in these examples when gzip is applied to both.
That is
strong evidence that uncompressed text and its repetition account for much
of this measured gap. It does not isolate every layout/encoding effect or
predict compression of real drawings: the generated corpus is very repetitive.
This compares the selected writers and file contents, not the smallest size
either format could achieve with other encoders or compression algorithms.

Keep compression as this explanatory check for now. Once IFCCAD implements a
supported compressed encoding or container, add its actual production output
as a benchmark variant, including all metadata/container bytes, production
readback, semantic checks and repeatability. Runtime and memory comparisons
and a practical corpus remain separate future extensions.

## Reproduce and inspect

Prepare the checkout using the [patch instructions](../../patches/cadcodec-upstream/README.md).
From the repository root, choose a new run name and run:

```text
cargo run --config patches/cadcodec-upstream.toml -p ifccad-convert --example size_baseline -- --run size-v1-review --cargo-config patches/cadcodec-upstream.toml
```

Omit `--case` for the full corpus. Adding `--case mixed-1000` produces a labelled
partial run. Existing directories are never overwritten. Failed comparisons,
unexpected assessments, missing files or nondeterminism return failure while
retaining local diagnostic output; no size thresholds are asserted.

The retained measured run is `target/size-baseline/upstream-2f2cd25-patched-v2/`.
Its `artifacts.md` indexes `first/` and `second/`, with each case's IFCCAD
directories, `drawing.dxf`, `drawing.dwg` and separate chain artifacts.
Provenance records compiler/host, repository revision and dirty state, exact
dependencies, source/corpus hashes, patched codec source hash and Cargo.lock
hash; the active lockfile and source manifests are retained locally. Exact
byte reproduction requires the same source and dependency resolution. Run
different Cargo configs sequentially because they share the root lockfile.

Only this report, its detailed result JSON and the corpus inventory form the
curated experiment. Development runs and generated files remain local under
`target/`; they are not additional published comparisons.
