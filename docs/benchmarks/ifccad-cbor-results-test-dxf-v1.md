# IFCCAD test.dxf retry: conversion failure

No cases qualified and no encoding size/timing results were produced.
Dimension `0xEF354` was explicitly excluded from the in-memory conversion input;
original source bytes remain unchanged. Conversion then failed on dimension
`0xEF3B7` (979895), with the same reported geometry deviation.

Run `cbor-test-dxf-skip-ef354-20261016-v2`; 2026-10-16T16:46:46.766454+00:00. Size is the primary result; timings are indicative.

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

Failed `practice-test-dxf`: geometric accuracy failed in Drawing, coordinates DrawingUnit(Millimetre): GeometryFailure { source: CadEntity { handle: Handle(979895), kind: "DIMENSION" }, vertex_index: Some(2), stage: DeviationAssessment, requested_tolerance: GeometryTolerance(Default), resolved_tolerance: Some(DistanceInterval { lower: 9.999999999999999e-10, upper: 1e-9 }), deviation: Some(DistanceInterval { lower: 543.9999999999972, upper: 543.9999999999972 }), reason: ProvenExceedance }
Current selected codec is locally patched; exact source/config/dependency identities,
source hashes, diagnostic records, sample ranges and raw samples are retained in
[matching detailed results](../../benchmarks/ifccad-cbor/results-test-dxf-v1.json).
Applicable source/config/input snapshots and selected dependency patch are retained
outside Git under `.superpowers/measurements/ifccad-cbor/cbor-test-dxf-skip-ef354-20261016-v2/`.

Scope/reproduction: [six-case protocol](ifccad-cbor-baseline-v1.md).
No compression, DWG-size equivalence, cold-start, memory or general performance claim.
Old OCDraw/common-subset reports remain unchanged. Full integration verification of
the changed worktree remains pending; these are scoped measurement correctness results.
