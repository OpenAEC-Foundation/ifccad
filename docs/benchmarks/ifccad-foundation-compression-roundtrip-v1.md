# Foundation IFCCAD compression and CAD roundtrip size probe

Explicitly requested on 2026-10-16; one foundation drawing only. No test.dxf,
synthetic-corpus rerun, compression performance measurement or native compressed
format implementation. gzip is an external size probe, level 9, mtime0. Every
decompression reproduced its original bytes exactly.

The starting point is the exact 383443-byte CBOR artifact from
`cbor-small-20261016-v1`, SHA-256
`391fa55223f87f9821d8782b9ace5553725fe6d17a18aedd6cb4df2b915b7c68`.
Production CBOR load/validation -> existing logical IFCCAD-to-CAD conversion ->
current DWG and text-DXF writers -> production CAD readers -> IFCCAD conversion
with SupportedTyped preservation -> strict drawing validation. Existing Allow
policy and all located conversion diagnostics remain in detailed results.

| Representation | Native bytes | External gzip9 bytes |
| --- | ---: | ---: |
| IFCCAD CBOR | 383443 | 60666 |
| Compact IFCCAD JSON | 1295418 | 92005 |
| DWG from the same IFCCAD document | 139662 | 121627 |
| DXF from the same IFCCAD document | 954584 | 84157 |

CBOR+gzip is 56.6% smaller than the roundtrip DWG without extra gzip,
50.1% smaller than DWG+gzip and 27.9% smaller than DXF+gzip.
It is 34.1% smaller than compact JSON+gzip.

Both CAD inventories after physical readback equal the inventory supplied to
the writers, including kind and owner counts: 2711 stored CAD entities. Both
native reimports pass strict drawing validation and return 2709 IFCCAD entities,
including 87 dimensions. The extra CAD entities are part of CAD representation;
no full arbitrary-ID-independent semantic equivalence certification was performed.
Inventory equality alone does not prove preservation of every property.

This compares outputs starting from the same currently retained IFCCAD document.
It is more controlled than comparing against original source file sizes, but
conversion diagnostics and format-specific omissions/modifications still apply.
The initial source import already omitted unsupported original content. The old
October2 common-subset numbers used fewer semantic families and are not rows
in this table. Original source DWG 239012 / DXF 1314182 bytes remain identification
context, not equivalent-content reference representations.

The current selected codec is revision 325fa98d2b881f48e106a830d5cf161624f63131
with existing explicit local dimension-bit patches. Release profile/configuration
are unchanged. Source, dependency, configuration and toolchain identities were
captured before/after and matched. Only this selected size probe ran, once; no
repeatability or general compression/performance claim is implied.

[Matching detailed results](../../benchmarks/ifccad-cbor/foundation-compression-roundtrip-v1.json)
retain input/output hashes, inventories, full conversion diagnostics and provenance.
The probe source, source/config/dependency snapshots, logs, exact roundtrip outputs,
gzip outputs and matching JSON remain outside target under
`.superpowers/measurements/ifccad-cbor/foundation-compression-roundtrip-20261016-v1/`.

Reproduction with the recorded configuration and initial CBOR artifact:

```text
cargo build --offline --release -p ifccad-convert --example cbor_foundation_size_probe
target/release/examples/cbor_foundation_size_probe.exe INPUT.ifccad.cbor FRESH_OUTPUT_DIRECTORY
```

Python gzip.compress(data,compresslevel=9,mtime=0) then byte-exact gzip.decompress
checks produced the external compression rows. Python/zlib versions are recorded
in detailed results. New measurements still require explicit user request.
No supported compressed IFCCAD file container was added. Full CBOR integration
verification remains pending; the selected production readbacks do not replace
the full workspace gate.
