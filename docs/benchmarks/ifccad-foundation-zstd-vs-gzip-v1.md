# Foundation CBOR: experimental Zstd versus gzip size probe

Requested on 2026-10-19; only the existing foundation CBOR artifact was compressed.
No CAD conversion, corpus rerun, speed measurement or compressed-format integration.

Input: 383443 bytes; SHA-256
`391fa55223f87f9821d8782b9ace5553725fe6d17a18aedd6cb4df2b915b7c68`.
This is exactly the strict production-qualified `practice-foundation` artifact
from `cbor-small-20261016-v1`. All four decompressions reproduced those bytes
exactly, retaining its existing native content/readback qualification. No claim
of qualification for later model/code revisions is implied.

| External compression | Bytes | Smaller than gzip9 |
| --- | ---: | ---: |
| gzip level 9 | 60666 | 0.0% |
| zstd level 3 | 56782 | 6.4% |
| zstd level 9 | 54679 | 9.9% |
| zstd level 19 | 48238 | 20.5% |

Zstd19 is the smallest tested output, 20.5% below gzip9 for this input.
There is no compression/decompression speed result, peak-memory result, trained
dictionary test, chunking comparison or general CAD workload conclusion. A codec
level is an algorithm-specific setting: gzip9 and Zstd9 do not imply equal work.

Runtime:Python 3.14.7, Zstd 1.5.7, zlib-ng 1.3.1; full versions and binary hashes
are in detailed results. Zstd uses one-shot compression, checksum/content-size
enabled, zero workers (single-threaded), no dictionary. gzip uses level 9, mtime 0
and no filename. gzip bytes and SHA match the earlier accepted external probe.

[Matching detailed results](../../benchmarks/ifccad-cbor/foundation-zstd-vs-gzip-v1.json)
retain parameters, input/output hashes and byte-exact decompression outcomes.
The input copy, compressor script and all compressed artifacts are retained
outside target in
.superpowers/measurements/ifccad-cbor/foundation-zstd-vs-gzip-20261019-v2/.
The first attempt completed compression/decompression but failed while recording
metadata for built-in zlib; it is not accepted evidence. The metadata handling
was corrected and a fresh successful run produced the results above.

Reproduction on Python3.14 with compression.zstd available:

```text
python scripts/ifccad_cbor_compression_probe.py --input INPUT.ifccad.cbor --sha256 391fa55223f87f9821d8782b9ace5553725fe6d17a18aedd6cb4df2b915b7c68 --out FRESH_EVIDENCE_DIRECTORY
```

The manual script refuses a mismatching source hash and existing evidence
directory. It is not invoked by tests or CI. Later measurements require an
explicit request. Native storage remains unchanged; all compression here is
an optional external experiment.
