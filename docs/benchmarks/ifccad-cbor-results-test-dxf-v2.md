# IFCCAD test.dxf retry: dimensions excluded, polyline failure

No encoding size/timing results qualified. The approved benchmark-only rule
excluded 31 dimensions with point-2 proven geometry exceedance, reducing the
conversion input from 54598 to 54567 stored entities. Every exclusion and its
complete failure text is retained in the detailed results (sourceRead.exclusions).
The original file remains unchanged. Conversion subsequently failed on
LWPOLYLINE `0xF1439` (988217), SourceEvaluation/CadAxisEvaluationFailed.
This different error was not included in the approved dimension-only rule.

Run `cbor-test-dxf-dimension-exclusions-20261016-v1`; 2026-10-16T16:54:04.767614+00:00. Size is the primary result; timings are indicative.

| Case | JSON bytes | Compact JSON bytes | CBOR bytes | CBOR / compact |
| --- | ---: | ---: | ---: | ---: |

Median native API time, milliseconds; 1 warmup + 5 samples, no filesystem IO.

| Case | JSON open | Compact open | CBOR open | JSON write | CBOR write |
| --- | ---: | ---: | ---: | ---: | ---: |

Production writers include validation and mandatory readback. JSON opening also retains
the IFCX source graph; CBOR returns the CAD document. Compact JSON has no dedicated
production writer. These are complete route costs, not pure token-parser comparisons.

Every accepted case passed strict production readback, full typed/numeric comparison
and one untimed independent repeat. All original CAD content is not necessarily retained.

Failed `practice-test-dxf`: geometric accuracy failed in Drawing, coordinates DrawingUnit(Millimetre): GeometryFailure { source: CadEntity { handle: Handle(988217), kind: "LWPOLYLINE" }, vertex_index: None, stage: SourceEvaluation, requested_tolerance: GeometryTolerance(Default), resolved_tolerance: Some(DistanceInterval { lower: 9.999999999999999e-10, upper: 1e-9 }), deviation: None, reason: CadAxisEvaluationFailed }
Current selected codec is locally patched; exact source/config/dependency identities,
source hashes, diagnostic records, sample ranges and raw samples are retained in
[matching detailed results](../../benchmarks/ifccad-cbor/results-test-dxf-v2.json).
Applicable source/config/input snapshots and selected dependency patch are retained
outside Git under `.superpowers/measurements/ifccad-cbor/cbor-test-dxf-dimension-exclusions-20261016-v1/`.

Scope/reproduction: [six-case protocol](ifccad-cbor-baseline-v1.md).
No compression, DWG-size equivalence, cold-start, memory or general performance claim.
Old OCDraw/common-subset reports remain unchanged. Full integration verification of
the changed worktree remains pending; these are scoped measurement correctness results.
