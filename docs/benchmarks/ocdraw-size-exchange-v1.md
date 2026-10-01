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

Run `ocdraw-polyline-layout-fixes-20261001c`, measured on 2026-10-01:
all 12 primitive cases passed production readback and semantic exchange;
both fresh generations passed repeatability. The pinned cadcodec dependency
was unmodified. This evidence covers the final ordered-ownership JSON mapping
without a stream directory, with signed/fractional polyline coordinate pools
and source-driven paper layouts. This replaces the applicable standalone
measurement from 2026-09-30; the historical package baseline is unchanged.

[Matching detailed results](../../benchmarks/size/ocdraw-results-v1.json)
contain the dirty working-tree source manifest, dependency provenance,
per-artifact hashes, diagnostics and stage comparisons. The initial contract
is still a candidate; this measurement does not publish a supported version.

| Case | OCDraw bytes | DXF bytes | DWG bytes |
| --- | ---: | ---: | ---: |
| empty | 826 | 46978 | 20854 |
| lines-100 | 27815 | 60338 | 22262 |
| lines-10000 | 2721166 | 1444626 | 168218 |
| short-polylines-100 | 39981 | 79398 | 22486 |
| short-polylines-10000 | 4000322 | 3396526 | 187933 |
| long-polylines-10 | 55810 | 114314 | 23830 |
| long-polylines-1000 | 5683164 | 7026512 | 257964 |
| mixed-1000 | 365282 | 312494 | 42117 |
| fractional-lines-1000 | 301379 | 215872 | 47198 |
| spatial-line-1000 | 273156 | 187664 | 42213 |
| elevated-1000 | 704246 | 376972 | 38917 |
| tilted-shifted-1000 | 706273 | 408112 | 38982 |

Larger line/short-polyline recipes have more pretty-JSON bytes than text DXF;
other primitive recipes differ. Compressed DWG is smaller in every measured
case. These observations apply only to this fixed corpus and these emitted
representations. Layout, block and viewport exchange uses the focused tests,
including explicit rejection of known inconsistent DWG block markers; those
families are not evidence supplied by this table.
