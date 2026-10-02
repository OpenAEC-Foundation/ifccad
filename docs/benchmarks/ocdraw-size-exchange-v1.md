# OCDraw initial JSON size and exchange experiment

Measurements are repeated only on explicit user request, for the full experiment
or a requested selection. A new main revision or corpus/converter/writer/codec/
dependency change does not trigger a measurement. This is never an automatic
completion gate; focused correctness and strict-readback tests remain required.
The accepted evidence below retains its recorded revision and coverage.

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

Use the unmodified opencadcodec revision `d96e3fa2fe5acbeac966f1db4c01142618bf9c79`
pinned in Cargo.toml and resolved in Cargo.lock, without Cargo patch configuration. From repository root:

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

## Accepted current JSON measurement

Run `model-io-followup-20261002`, measured on 2026-10-02:
all 12 primitive cases passed production readback and semantic exchange;
both fresh generations passed repeatability. The pinned opencadcodec dependency
was unmodified, with no local override. This evidence exercises the harmonized
`OcdrawBuilder`, complete logical document, shared validator, common encoder,
logical CAD conversion and encoded/source convenience routes.

All measured results, including native OCDraw, DXF and DWG byte counts and
hashes, are identical to the preceding accepted
`model-io-harmonization-20261002` run, which also matched the preceding
`opencadcodec-update-20261002-140309` run. This final follow-up moves shared unit
mapping into `units` and aligns encoded-value traits, retaining the exact unit
values/order and byte-equality contract. The harmonization changes Rust API
names and module responsibilities, not the emitted representations or this
corpus's semantic exchange. The detailed provenance uses the upstream dependency
key `opencadcodec`; preceding reports used the local alias `cadcodec`.

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
| empty | 945 | 47107 | 20918 |
| lines-100 | 29272 | 60467 | 22326 |
| lines-10000 | 2851323 | 1444755 | 168282 |
| short-polylines-100 | 43679 | 79527 | 22550 |
| short-polylines-10000 | 4350520 | 3396655 | 188029 |
| long-polylines-10 | 56358 | 114443 | 23894 |
| long-polylines-1000 | 5718362 | 7026641 | 258028 |
| mixed-1000 | 385817 | 312623 | 42181 |
| fractional-lines-1000 | 314536 | 216001 | 47262 |
| spatial-line-1000 | 286313 | 187793 | 42277 |
| elevated-1000 | 739444 | 377101 | 38981 |
| tilted-shifted-1000 | 741471 | 408241 | 39046 |

Larger line/short-polyline recipes have more pretty-JSON bytes than text DXF;
other primitive recipes differ. Compressed DWG is smaller in every measured
case. These observations apply only to this fixed corpus and these emitted
representations. Layout, block and viewport exchange uses the focused tests,
including explicit rejection of known inconsistent DWG block markers; those
families are not evidence supplied by this table.

In the preceding dependency-update run, direct OCDraw bytes/hashes were unchanged
relative to `ocdraw-document-counts-20261002-112932`. That update made direct DXF
129 bytes larger in each case and direct DWG 64 bytes larger except
short-polylines-10000, which is 96 bytes larger. CAD hashes changed in that update; semantic
readback and repeatability passed. The current harmonization run retains those
updated CAD hashes exactly. These are observed complete-file differences,
not compression or runtime-performance conclusions. The retained provenance
identifies the uncommitted source manifest and renamed dependency at its exact
revision. Historical package and placement evidence retains its original pins.
