# Foundation CBOR: experimental compression and decompression timing

Explicitly requested on 2026-10-19. Scope is only the same 383443-byte qualified
foundation CBOR artifact and gzip 9 / Zstd 3, 9, 19 used by the earlier
[size probe](ifccad-foundation-zstd-vs-gzip-v1.md). No CAD conversion, corpus rerun,
new compression variant or native compressed-storage implementation.

| Compression | Compressed bytes | Compress median ms | Decompress median ms |
| --- | ---: | ---: | ---: |
| gzip 9 | 60666 | 7.5624 | 0.5226 |
| zstd 3 | 56782 | 0.8699 | 0.5506 |
| zstd 9 | 54679 | 4.0079 | 0.5793 |
| zstd 19 | 48238 | 121.9260 | 0.6102 |

Observed min-max milliseconds (ranges, not confidence intervals):

| Compression | Compress range ms | Decompress range ms |
| --- | ---: | ---: |
| gzip 9 | 7.3534-10.7563 | 0.4426-0.7063 |
| zstd 3 | 0.8143-1.3788 | 0.4836-0.8976 |
| zstd 9 | 3.8708-5.8456 | 0.5197-0.7732 |
| zstd 19 | 117.8373-134.8544 | 0.5191-0.8605 |

For this artifact/API, Zstd3 compression is 8.7 times faster than gzip9
and its file is 6.4% smaller. Zstd19 compression is
140.2 times slower than Zstd3 for an additional
8544 bytes saved. It is not the obvious interactive-save choice solely on size.

Decompression medians are all approximately 0.5-0.6 ms and observed ranges overlap.
These samples do not establish a useful decompression-speed winner. This is a
Python one-shot API comparison, including wrapper/allocation costs and zlib-ng;
it is not a general statement about native Rust/C codec throughput.

Protocol: 3 warmups and 31 recorded samples per codec/direction, rotating all 8
operations by round. Inputs and prior compressed outputs are loaded/qualified
before timing. Each call creates its normal one-shot codec context and output
buffer. Byte comparison and returned-buffer disposal happen after the timer.
Every compression reproduces the previous compressed bytes; every decompression
reproduces the exact original bytes. Filesystem IO, CBOR parsing/validation and
CAD conversion are excluded. No batching or baseline-overhead subtraction.
Checksums remain enabled; no dictionary or worker threads. Throughput in detailed
results is decimal MB/s based on uncompressed bytes for both directions.

Runtime: Python 3.14.7, Zstd 1.5.7 and zlib-ng 1.3.1 on Windows 11. CPU identity,
clock, runtime binary hashes and sample values are retained. CPU affinity is not
pinned; normal scheduling, garbage collection and power management apply. No
Cargo/compiler activity was detected immediately before or after timing. This
is one process run with one warm artifact, not a cold-start, memory or general
CAD-workload benchmark. No end-to-end file-opening/save times are measured.

[Matching detailed results](../../benchmarks/ifccad-cbor/foundation-compression-speed-v1.json)
retain all 31 raw samples, median/min/max/p10/p90, input/output hashes, options,
machine/runtime provenance and script SHA. Exact input, compressed outputs, probe
source and matching JSON remain outside target under
.superpowers/measurements/ifccad-cbor/foundation-compression-speed-20261019-v1/.

Reproduction with the retained preceding size-probe directory:

```text
python scripts/ifccad_cbor_compression_speed_probe.py --baseline PATH_TO_FOUNDATION_ZSTD_VS_GZIP_20261019_V2 --out FRESH_EVIDENCE_DIRECTORY
```

The manual runner refuses mismatching input/output bytes, unexpected variants,
existing output directories and detected competing build activity. Runtime
binaries must match the preceding size probe for acceptance. It is never called
by tests or CI; new measurements require an explicit user request. Earlier
accepted size/roundtrip reports remain retained unchanged.
