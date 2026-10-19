# IFCCAD base-CBOR: five qualified initial cases

Final measurement scope on 2026-10-16: four generated recipes and the original
foundation DWG. The user agreed to leave test.dxf out after conversion preparation
proved disproportionately expensive. No additional controlled measurement was run.

The five independently qualified rows come from run `cbor-small-20261016-v1`.
That historical six-case attempt exited 1 because test.dxf failed before encoding;
its five successful cases each passed strict production readback, full typed/numeric
comparison and an untimed independent repeat. The historical failed row remains
retained, and is not represented as a successful sixth case.

| Case | Pretty JSON bytes | Compact JSON bytes | CBOR bytes | Smaller than compact JSON |
| --- | ---: | ---: | ---: | ---: |
| 10000 lines | 4601962 | 2060452 | 623378 | 69.7% |
| 10000 polylines | 9547462 | 3405952 | 2283378 | 33.0% |
| 1000 text objects | 1737886 | 735642 | 249427 | 66.1% |
| 1000 hatches | 1908388 | 572144 | 326824 | 42.9% |
| Foundation practice drawing | 2847682 | 1295418 | 383443 | 70.4% |

Indicative median production API times, milliseconds (one warmup, five samples):

| Case | Compact JSON open | CBOR open | JSON write | CBOR write |
| --- | ---: | ---: | ---: | ---: |
| 10000 lines | 99.6 | 32.9 | 145.9 | 36.9 |
| 10000 polylines | 168.2 | 80.8 | 257.0 | 98.7 |
| 1000 text objects | 36.1 | 25.0 | 64.2 | 40.8 |
| 1000 hatches | 884.2 | 829.7 | 1669.8 | 1727.9 |
| Foundation practice drawing | 146.7 | 130.0 | 287.2 | 232.6 |

CBOR is 33-70% smaller than whitespace-compact JSON in these five cases, without
compression or columns. Read/write differences depend on content. Hatch timing
ranges overlap substantially; the sample does not establish a clear hatch writing
speed gain. These timings are indicative, not statistically established claims.

JSON opening also retains an IFCX source graph; CBOR returns the CAD document.
Production writing includes validation/readback. Compact JSON has no dedicated
production writer. Filesystem IO and returned-value disposal are outside timing.

The foundation conversion retained 2709 stored entities, including 87 dimensions.
Its diagnostics document omissions and modifications; these sizes compare the
same converted drawing, not the complete original DWG. Diagnostic counts are not
entity-loss counts. Natural mixed coverage remains limited to this one practice
drawing; no DWG-size equivalence or broad format-performance claim follows.

The production source, recipes, dependency files, configuration, toolchain, target
and relevant environment were compared against the later retry inputs and remained
identical. Only the manual driver and example tests changed during retry preparation.

[Matching original detailed results](../../benchmarks/ifccad-cbor/results-v1.json)
have SHA-256 `92916f969e64a3c3c47cbd323edec075a6d7e3bb06600a8cd7be1452a27f303d` and retain raw samples, ranges,
inventories, diagnostics, artifact hashes and exact source/config/dependency manifests.
The selected codec base is `325fa98d2b881f48e106a830d5cf161624f63131` with the
explicit existing local dimension-bit patches; this is not an unmodified-codec run.
Reproduction snapshots and matching evidence remain outside Git under
`.superpowers/measurements/ifccad-cbor/cbor-small-20261016-v1/`.

Scope and reproduction: [protocol](ifccad-cbor-baseline-v1.md).

## Deferred test.dxf

The original DXF remains unchanged. The early attempts established separate
dimension and polyline conversion failures, retained in the
[one-dimension retry](ifccad-cbor-results-test-dxf-v1.md) and
[dimension-error-class retry](ifccad-cbor-results-test-dxf-v2.md).
The broader attempt preserved 387 individually observed conversion failures in
`benchmarks/ifccad-cbor/exclusions-test-dxf-v1.json`, with source and production
provenance. It was stopped during preparation, before codec timing. A subsequent
batching attempt was stopped during compilation at the user-directed scope change.
Neither attempt produced accepted test.dxf sizes or timings.

Full workspace integration verification remains pending. The focused correctness
tests and Clippy checks qualify the manual tooling slice, not the entire changed
CBOR worktree. No commit, merge or push was performed.
