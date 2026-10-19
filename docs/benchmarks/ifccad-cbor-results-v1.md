# Initial IFCCAD JSON / base-CBOR measurement

Partial run: five of six selected cases passed. The original `test.dxf`
conversion failed before encoding; it has no accepted measurement row here.
The five qualified results below remain retained unchanged.

The user subsequently agreed to defer test.dxf. The
[final five-case summary](ifccad-cbor-results-summary-v1.md) collects the applicable
results and limitations without rerunning these cases.

Run `cbor-small-20261016-v1`; 2026-10-16T16:26:03.527951+00:00. Size is the primary result; timings are indicative.

| Case | JSON bytes | Compact JSON bytes | CBOR bytes | CBOR / compact |
| --- | ---: | ---: | ---: | ---: |
| lines-10000 | 4601962 | 2060452 | 623378 | 0.303 |
| polylines-10000 | 9547462 | 3405952 | 2283378 | 0.670 |
| text-1000 | 1737886 | 735642 | 249427 | 0.339 |
| hatches-1000 | 1908388 | 572144 | 326824 | 0.571 |
| practice-foundation | 2847682 | 1295418 | 383443 | 0.296 |

Median native API time, milliseconds; 1 warmup + 5 samples, no filesystem IO.

| Case | JSON open | Compact open | CBOR open | JSON write | CBOR write |
| --- | ---: | ---: | ---: | ---: | ---: |
| lines-10000 | 101.217 | 99.608 | 32.918 | 145.872 | 36.922 |
| polylines-10000 | 170.155 | 168.232 | 80.814 | 256.969 | 98.671 |
| text-1000 | 35.865 | 36.144 | 24.961 | 64.177 | 40.824 |
| hatches-1000 | 939.258 | 884.172 | 829.749 | 1669.798 | 1727.937 |
| practice-foundation | 138.201 | 146.667 | 129.990 | 287.158 | 232.577 |

Production writers include validation and mandatory readback. JSON opening also retains
the IFCX source graph; CBOR returns the CAD document. Compact JSON has no dedicated
production writer. These are complete route costs, not pure token-parser comparisons.

Every accepted case passed strict production readback, full typed/numeric comparison
and one untimed independent repeat. All original CAD content is not necessarily retained.

`practice-foundation` retained 2709 entities;
diagnostic actions: `{"Modified": 2243, "Omitted": 3664, "RoundedWithinTolerance": 92}`.
Retained kinds: `{"Arc": 73, "BlockInstance": 165, "Circle": 207, "Dimension": 87, "HatchPattern": 12, "HatchSolid": 72, "Line": 1651, "MText": 433, "PlanarPolyline": 4, "Viewport": 5}`.

Failed `practice-test-dxf`: geometric accuracy failed in Drawing, coordinates DrawingUnit(Millimetre): GeometryFailure { source: CadEntity { handle: Handle(979796), kind: "DIMENSION" }, vertex_index: Some(2), stage: DeviationAssessment, requested_tolerance: GeometryTolerance(Default), resolved_tolerance: Some(DistanceInterval { lower: 9.999999999999999e-10, upper: 1e-9 }), deviation: Some(DistanceInterval { lower: 543.9999999999972, upper: 543.9999999999972 }), reason: ProvenExceedance }
Current selected codec is locally patched; exact source/config/dependency identities,
source hashes, diagnostic records, sample ranges and raw samples are retained in
[matching detailed results](../../benchmarks/ifccad-cbor/results-v1.json).
Applicable source/config/input snapshots and selected dependency patch are retained
outside Git under `.superpowers/measurements/ifccad-cbor/cbor-small-20261016-v1/`.

Scope/reproduction: [six-case protocol](ifccad-cbor-baseline-v1.md).
No compression, DWG-size equivalence, cold-start, memory or general performance claim.
Old OCDraw/common-subset reports remain unchanged. Full integration verification of
the changed worktree remains pending; these are scoped measurement correctness results.
