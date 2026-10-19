# Initial IFCCAD JSON / base-CBOR baseline

The [final five-case summary](../../docs/benchmarks/ifccad-cbor-results-summary-v1.md)
contains the qualified initial results. The user agreed to defer test.dxf after
conversion preparation proved too expensive for this first comparison. The
original failed attempts remain retained; there is no accepted test.dxf size or
timing row. Do not run that case again without an explicit measurement request.

Separate explicitly requested foundation-only follow-ups retain the
[gzip and current CAD roundtrip comparison](../../docs/benchmarks/ifccad-foundation-compression-roundtrip-v1.md)
and [Zstd versus gzip probe](../../docs/benchmarks/ifccad-foundation-zstd-vs-gzip-v1.md).
The later [compression/decompression timing probe](../../docs/benchmarks/ifccad-foundation-compression-speed-v1.md)
measures only gzip9/Zstd3,9,19 on the same retained CBOR artifact, in memory.
Those measurements use external compression; neither follow-up measures the later
optional native Zstd transport. See the [current encoding routes](../../docs/experiments/ifccad-cbor.md).
Timing results concern the recorded Python APIs and one fixed warm artifact,
not general native-library or CAD performance.

Manual experiment only, never a CI/completion gate. The approved small scope is
four fixed-size recipes and two original practice drawings. See the
[protocol and reproduction instructions](../../docs/benchmarks/ifccad-cbor-baseline-v1.md).

`corpus-v1.json` records the six cases, fixed recipe rules and original source
hashes. Production readback, complete typed/numeric equality and one untimed
repeat qualify artifacts. Size is primary; five timing samples are indicative.
The current patched codec/configuration must be recorded; old accepted OCDraw
and common-subset results do not measure this encoding or retained content.

The Rust example `cbor_baseline` in ifccad-convert owns recipes, inventory,
qualification and timings. The Python launcher captures provenance and retains
reproduction inputs and results outside target/. Tests use tiny correctness
documents and fabricated statistics, without controlled measurement runs.

The [initial partial report](../../docs/benchmarks/ifccad-cbor-results-v1.md)
and matching `results-v1.json` retain five qualified cases and one original-DXF
conversion failure. The explicitly authorized dimension-0xEF354 exclusion is
implemented in benchmark preparation only. After an initial attempt exceeded
the wait limit for unrelated Cargo/test activity, a quiet-host retry confirmed
the exclusion but failed conversion on dimension `0xEF3B7` with the same error.
See the [retry report](../../docs/benchmarks/ifccad-cbor-results-test-dxf-v1.md)
and `results-test-dxf-v1.json`. No test.dxf size/timing result has qualified.

The subsequently approved dimension-error rule excluded 31 dimensions before
conversion failed on a different LWPOLYLINE error (`0xF1439`): see the
[next attempt](../../docs/benchmarks/ifccad-cbor-results-test-dxf-v2.md) and
`results-test-dxf-v2.json`. Every excluded dimension and its error is recorded;
the different polyline failure remains fatal under that rule.
