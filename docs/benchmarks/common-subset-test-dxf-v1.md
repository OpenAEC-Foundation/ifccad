# Four-format common-subset comparison: test.dxf

Accepted 2026-10-02, run `transparency-test-dxf-v1`, optimized Rust profile,
unmodified opencadcodec revision `d96e3fa2fe5acbeac966f1db4c01142618bf9c79`.
Detailed evidence: [test-dxf-results-v1.json](../../benchmarks/size-exchange/test-dxf-results-v1.json).
The [common-subset experiment](common-subset-size-exchange-v1.md) defines the
semantic comparison, runtime scaffold bridge and known-profile transfer boundary.

## Source and retained content

Original `test.dxf`: 33,288,016 bytes, SHA-256
`74d8d7b861cda0e3859879c8149df2344a457f5c9cefce6a144a7cf52a4fbe9a`.
Both passes independently load this original DXF and prepare through
CadDocument → OcdrawDocument → CadDocument → IfcxCadDocument → CadDocument.
Preparation uses Allow with complete stage diagnostics retained; all emitted
branches use strict production readback and exact semantic comparisons.

| Retained family | Entities |
| --- | ---: |
| LINE | 8,737 |
| LWPOLYLINE | 5,525 |
| CIRCLE | 263 |
| INSERT | 85 |
| Total | 14,610 |

There are 35,566 polyline vertices, 9,815 direct Model entities, 357 layers,
153 authored local block definitions plus two reserved CAD block records,
and 38 CAD line-type records including selection scaffolding. Blocks stay shared.
Unsupported content is removed before the common reference is frozen, with
35,370 / 4,901 / 12,021 diagnostics in the three preparation stages. These
diagnostic counts include modifications and propagated losses; they are not
counts of removed entities. No additional numeric stabilization rounds were needed.

## Sizes

Decimal units: 1 kB = 1,000 bytes; 1 MB = 1,000,000 bytes.
JSON compact variants remove whitespace while preserving values and integer tokens.

| Representation | Raw bytes | gzip bytes | Raw MB | gzip kB |
| --- | ---: | ---: | ---: | ---: |
| OCDraw compact JSON | 4,356,704 | 498,723 | 4.357 | 498.7 |
| IFCX-CAD compact JSON | 7,906,416 | 623,332 | 7.906 | 623.3 |
| DXF | 5,796,711 | 562,709 | 5.797 | 562.7 |
| DWG | 750,501 | 678,474 | 0.751 | 678.5 |
| OCDraw pretty JSON | 9,213,849 | 549,734 | 9.214 | 549.7 |
| IFCX-CAD pretty JSON | 17,023,200 | 724,843 | 17.023 | 724.8 |

gzip level 9, empty filename and mtime 0; decompression must reproduce every
original byte. All six decompressed variants pass their production readers
and exact semantic checks. Two full generations have identical artifact and
gzip hashes, reference snapshots and preparation diagnostics. Source provenance
was rechecked after completion. The nine synthetic cases also pass in this run.

IFCX uses its installed shared profile, not a self-contained schema bundle.
Its shared schema alone costs 11,720 raw / 999 gzip bytes if separately transferred.
External gzip remains a transport probe, not a supported native compressed encoding.

## Transparency correction and regression evidence

DWG readback exposes 14 `AcCmTransparency` layer EED records, all exactly
duplicating typed explicit layer transparency: packed values 33554559/33554483
for transparency amounts 128/204. The IFCX source classifier now accepts only
a resolved Layer, exact application name, one decoded Integer32, and exact
equality to the canonical DXF packed value of the mapped explicit transparency.
Unknown applications, wrong owners, undecodable/additional values, mismatches
and noncanonical flags remain diagnosed losses. No source repair or mutation
is performed. Core formats and the codec dependency are unchanged.

The positive actual-DWG strict-native-roundtrip regression failed before the
correction and passes afterward. Negative payload/owner/application tests,
mismatched typed/raw transparency and existing arbitrary object-XDATA tests pass.
fmt, workspace clippy with warnings denied, workspace tests, workspace doc tests,
15 exchange example tests and six Python tests pass.

The separate `transparency-foundation-v1` rerun passes both generations for
the original ten cases; all raw/gzip artifact hashes and semantic snapshot
hashes equal the preceding accepted foundation/synthetic run.

## Reproduction and interpretation

```text
python scripts/size_exchange.py --run NEW_FRESH_RUN_NAME --release --practice path/to/test.dxf
```

Use the original file with the recorded source hash, the documented pinned
dependencies and a fresh run directory. The runner selects DXF by extension,
regenerates both passes from the source and records complete provenance.
The full result is retained outside `target/`; generated artifacts remain local.

DWG is smallest before additional gzip. With gzip, compact OCDraw is smallest:
20.0% below compact IFCX-CAD and 26.5% below DWG. Compact IFCX-CAD is 8.1% below
DWG, while DXF is 9.7% below compact IFCX-CAD. This larger subset is approximately
7.5 times the foundation subset's entity count. It confirms that JSON plus
gzip remains competitive here, while the IFCX advantage over compressed DWG
is smaller than in the foundation drawing.

These are equal reduced contents, not the original complete CAD drawing.
Missing text, dimensions, hatches, layouts and other unsupported semantics
are absent from all four outputs. The results concern these implementations
and this corpus; they do not establish universal format ratios, isolate pure
columnar savings, or identify the largest cause of DWG's remaining overhead.
