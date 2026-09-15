# Polyline preparation and observed placement reuse

Measured 2026-09-15 against unmodified cadcodec revision
`2f2cd25832db298524fb5eb36ced5a438a877e95`.

## Practice-file inventory

The sibling prototype corpus contains 464 DWG/DXF files. Excluding 133 generated
files under `converted/` leaves 331 originals: 200 DWG and 131 DXF. Readers
returned documents for 319 (189 DWG, 130 DXF); eleven old DWG versions are
unsupported and one early DXF fails parsing. The
[detailed inventory](../../benchmarks/spatial/placement-inventory-v1.json)
records every original's relative path, SHA-256, counts or reader error.

These are file-level observations, not 331 independent projects: many samples
are format/version variants, and one pair is byte-identical. Reader success is
not a claim of complete CAD fidelity. Counts cover stored LWPOLYLINE entities
across model space, paper space and block definitions, without expanding INSERTs.
The 779 other POLYLINE entities and other planar entity kinds are outside this
optimization's scope.

| Observation | Count |
| --- | ---: |
| Readable files containing LWPOLYLINE | 115 |
| LWPOLYLINE entities | 17,179 |
| Vertices | 148,983 |
| Identity placements | 15,786 |
| Elevated placements, normal (0,0,1) | 1,393 |
| Other normals | 0 |

The funderingsherstel drawing contains four LWPOLYLINE entities (16 vertices),
all at identity; three belong directly to model space. Both DWG and DXF agree.
The larger `3bm/test.dxf` contains 15,054 polylines, 85,785 vertices and 95 exact
normal/elevation tuples: 13,667 identity polylines and 1,387 elevated ones.
Only 3,826 polylines belong directly to model space. Independent reading of raw
ASCII DXF group codes agrees with the reader's placement counts for all three
3bm DXFs. Nearby but unequal elevations remain distinct; no tolerance merges
were used. With these normal vectors, distinct elevation tuples correspond
exactly to distinct IFCCAD placements.

## Bounded implementation

IFCCAD-to-CAD prepares rational origin, axes, target origin and affine
coefficients once per non-direct polyline. Each vertex's input and rounded
output are converted to rational operands once. Direct CAD-compatible
placements skip the unused projection preparation entirely. The arbitrary-axis
algorithm, nearest rounding, exact squared residual, tolerance policy, errors
and diagnostics are unchanged. CAD-to-IFCCAD and core bounds/point evaluation
are unchanged.

Preparation has a private, per-polyline lifetime. There is no cross-polyline
cache or encoding change. Keeping preparation separate leaves room for later
reuse without adding cache policy or public API now.

## Timing and numerical checks

A fresh sequential before/after release measurement uses the existing
`spatial_accuracy` workload: 10,000 two-vertex polylines per case, two warmups
and five measured iterations per operation. No build, tests or corpus sweep ran
concurrently. The frozen before binary and optimized binary are identified by
SHA-256 in the [detailed results](../../benchmarks/spatial/preparation-results-v1.json).
An earlier exploratory run overlapped the corpus sweep and is excluded.

Median IFCCAD-to-CAD time, milliseconds:

| Case | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Identity (direct path) | 72.20 | 40.08 | 44.5% |
| Large shifted coordinates | 381.72 | 281.34 | 26.3% |
| Oblique stress case | 2482.52 | 1877.23 | 24.4% |
| Oblique with tight bounds | 2619.87 | 1659.57 | 36.7% |

The identity improvement is the most relevant to the observed corpus. Unchanged
CAD-to-IFCCAD median time for that case was 58.2 versus 57.1 ms. Other unchanged
operations varied too: non-identity CAD-to-IFCCAD medians improved by roughly
6–9% across runs. These are single local paired observations, not guaranteed
speedups or whole-project conversion measurements. No additional oblique tuning
is proposed from these timings.

All eight emitted DXF/DWG files are byte-identical before and after (hashes in
the detailed results). All four geometry assessments are identical, including exact
maximum deviation bounds and rounded-entity counts. A focused regression checks
nearest ties-to-even in both directions at 2^53, the unrounded residual,
cancellation to zero, multiple vertices reusing preparation and range overflow.
Existing strict spatial exchange tests cover both file codecs and both loss
policies. The full unmodified-dependency workspace passes formatting, Clippy
with warnings denied and 379 tests (three opt-in tests excluded). The
[controlled size/exchange experiment](size-baseline-v1.md), using its documented
CAD dependency patches, also passes: all 180 generated artifacts and all
measurements are unchanged from the preceding run.

## Follow-up decision

Do not prioritize further oblique-placement optimization until representative
practice drawings contain many such placements. Retain the synthetic oblique
cases as correctness and stress checks. The corpus demonstrates substantial
reuse of identity/elevated placements, but those already use the direct path;
repetition alone does not justify a shared cache. Reconsider that separately
when actual workloads and profiling show enough remaining preparation cost.

## Reproduction

Build `cargo build -p ifccad-convert --example placement_inventory`, then run
`placement_inventory INPUT OUTPUT_JSON` for each original file. Exclude
`converted/`; record relative paths and file SHA-256 values. The recorded sweep
used separate processes with a 30-second reader timeout; none timed out. The
[example](../../crates/ifccad-convert/examples/placement_inventory.rs) counts
raw normal/elevation tuples by exact equality, treating signed zeros as equal.
No source CAD files are copied into this repository.
