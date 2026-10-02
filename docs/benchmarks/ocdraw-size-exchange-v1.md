# OCDraw initial JSON size and exchange experiment

## Purpose and boundary

This controlled experiment measures the initial standalone OCDraw JSON writer
and actual DXF/DWG codec outputs for fixed primitive recipes. It checks semantic
exchange through production readers and repeatability in two fresh generations.
It does not estimate typical CAD file sizes, preservation overhead, compression
quality or whole-drawing fidelity.

The primitive recipes in `benchmarks/size/corpus-v1.json` are reused: empty,
line and polyline scales, mixed primitives, fractional/spatial lines and
shifted/elevated/oblique planar polylines. Historical package/preservation fixture
entries, inline/external resource alternatives and package directory overhead
are excluded. Block, layout and viewport coverage uses focused conversion and
conformance tests; those families are absent from the measured primitive corpus.

OCDraw output is standalone pretty JSON with direct ordered scope entity lists,
central typed streams and no stream directory. DXF is the actual text writer
output. DWG is the actual normally compressed codec output. Ratios therefore
compare complete emitted representations, not compression algorithms. No byte
layout or CAD handles are used as semantic equality evidence.

## Reproduction

Use the unmodified cadcodec revision `5b682ed66ea2c89be8142c8dd83d83774fc3de08`
pinned in Cargo.lock, without Cargo patch configuration. From repository root:

```text
cargo run --offline -p ocdraw-convert --example size_baseline -- --run YOUR_FRESH_RUN_NAME
```

Each run creates a new directory below `target/size-baseline/`; an existing run
name is rejected. Optional `--case CASE_ID` restricts investigation and is not
full-corpus acceptance evidence. Run different Cargo configurations sequentially
because they share Cargo.lock. Keep detailed reports together with their results.

The tool writes original CAD variants and the chains OCDraw -> CadDocument ->
DXF/DWG -> CadDocument -> OCDraw, loads every native output through the production
reader, and compares typed primitive semantics. Numerical tolerance failures are
hard errors; only explicit CAD parameterization diagnostics are accepted in the
controlled chain. Both fresh generations must pass and have identical measured
artifacts/hashes. Source inventory, working-tree deletions, Cargo.lock/dependency
provenance, source-to-output comparisons and conversion diagnostics remain in
the detailed result. Uncommitted source bytes are identified by a source manifest
hash; a base Git revision alone does not identify that working tree.

## Retained evidence

The final report table and matching detailed results are stored separately from
the historical package baseline. The historical accepted
[package experiment](size-baseline-v1.md) and
[results](../../benchmarks/size/results-v1.json) remain retained and must not be
used as standalone OCDraw measurements.

## Accepted initial JSON measurement

Run `ocdraw-document-counts-20261002-112932`, measured on 2026-10-02:
all 12 primitive cases passed production readback and semantic exchange;
both fresh generations passed repeatability. The pinned cadcodec dependency
was unmodified. This evidence exercises the complete `OcdrawDocument` builder,
shared validator, common encoder and logical CAD conversion wrappers. All
measured artifacts, hashes, byte counts and exchange evidence are identical
to the previously accepted `ocdraw-document-20261002-104515` run.

This replaces the prior applicable standalone measurement from 2026-10-02;
the historical package baseline is unchanged. Custom/complex line patterns,
blocks, authored layouts and viewports are absent from this fixed corpus and
use focused strict-readback/conversion tests. Sparse identities and advanced
allocation history likewise have dedicated document and conformance tests.
Malformed stream counts and missing or short columns have a bounded reader
regression test; they are not part of this valid-drawing measurement corpus.
These measurements do not certify those absent families or editing sessions.

[Matching detailed results](../../benchmarks/size/ocdraw-results-v1.json)
contain the dirty working-tree source manifest, dependency provenance,
per-artifact hashes, diagnostics and stage comparisons. The initial contract
is still a candidate; this measurement does not publish a supported version.

| Case | OCDraw bytes | DXF bytes | DWG bytes |
| --- | ---: | ---: | ---: |
| empty | 945 | 46978 | 20854 |
| lines-100 | 29272 | 60338 | 22262 |
| lines-10000 | 2851323 | 1444626 | 168218 |
| short-polylines-100 | 43679 | 79398 | 22486 |
| short-polylines-10000 | 4350520 | 3396526 | 187933 |
| long-polylines-10 | 56358 | 114314 | 23830 |
| long-polylines-1000 | 5718362 | 7026512 | 257964 |
| mixed-1000 | 385817 | 312494 | 42117 |
| fractional-lines-1000 | 314536 | 215872 | 47198 |
| spatial-line-1000 | 286313 | 187664 | 42213 |
| elevated-1000 | 739444 | 376972 | 38917 |
| tilted-shifted-1000 | 741471 | 408112 | 38982 |

Larger line/short-polyline recipes have more pretty-JSON bytes than text DXF;
other primitive recipes differ. Compressed DWG is smaller in every measured
case. These observations apply only to this fixed corpus and these emitted
representations. Layout, block and viewport exchange uses the focused tests,
including explicit rejection of known inconsistent DWG block markers; those
families are not evidence supplied by this table.

Relative to the prior applicable run, all measured data is unchanged.
The new document lifecycle changes the Rust construction/conversion boundary;
it retains the emitted primitive representations in this controlled corpus.
The retained provenance records the uncommitted source manifest and removed
legacy decoding projection. This is size/exchange evidence, not a runtime
performance or memory-use measurement.
