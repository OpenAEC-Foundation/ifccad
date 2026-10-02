# Common-subset compressed size/exchange experiment v1

This experiment compares OCDraw, IFCX-CAD, DXF and DWG with the same retained
CAD meaning. It does not compare full format capability or predict application
adoption. The source corpus and driver are under
[`benchmarks/size-exchange`](../../benchmarks/size-exchange/README.md).

## Method and assumptions

Preparation starts at the original synthetic recipe or practice DWG each run:

```text
CadDocument -> OcdrawDocument -> CadDocument -> IfcxCadDocument -> CadDocument
```

The last document is a candidate, not an equality guarantee. Four independent
writers produce native OCDraw, native IFCX-CAD, DXF and DWG. Actual production
readers load every output. Reject conversion and an exact common semantic
snapshot check prove equality before any size is accepted. Unknown source
semantics and foreign IFCX graph data cannot be silently projected away.
Synthetic preparation accepts no semantic losses. Practice preparation permits
losses before freezing, with the complete stage diagnostics in the result JSON.

The benchmark adapter allocates missing layer identities after OCDraw import,
restores the codec's empty default paper scaffold, and resets the generated
measurement selector to the codec default. These bridge differing runtime
defaults; they do not preserve an authored paper layout or its entities.
Authored names, unused definitions, reference targets, entity order, units,
geometry, transforms, patterns, descriptions, scales and appearance modes
remain in the comparison. Allocation IDs, handles, counters and bookkeeping
dates are excluded. CAD output uses AC1032 with fixed dates.

Practice may require numeric stabilization because DXF converts rotation
between radians and degrees. At most three diagnosed preparation rounds are
allowed; nonnumeric changes and new losses fail. After freezing the candidate,
every branch must match exactly. Core converter tolerances are not modified.

Gzip uses level 9, mtime zero and an empty filename. Decompression must reproduce
every original byte, followed by production readback. JSON is measured as the
current writer output and as compact JSON; integer tokens remain exact. Tables
use compact JSON gzip for OCDraw/IFCX, DXF gzip, and both native DWG and DWG gzip.
Raw and pretty JSON sizes are in the detailed result.

Transfer assumes a receiver with the shared OCDraw registry and IFCX-CAD schema
already installed. The IFCX schema's standalone raw/gzip cost is reported
separately. Adding it to each transfer models a different delivery assumption.
Gzip does not become an official file encoding through this experiment.
The exact imported experimental profile is `urn:example:ifccad:0.1.0`, from
`schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx`; its bytes are hashed
in the detailed results. This is the current alpha profile, not a released
standard. The driver requires Python 3.11 or newer.

Two independent passes must match every artifact hash, snapshot hash and
preparation diagnostic.
Diagnostic lists are sorted by their complete content, preserving duplicates;
their enumeration order through the CAD runtime's hash map has no meaning.
Provenance records source file hashes and deletions, dirty state, lock hash,
original practice hash, runtimes and pinned codec
revision. Local codec patches are excluded. Original private drawings and local
absolute source paths are not copied into the accepted report.

## Initial results

The matching detailed results are retained in
[`results-v1.json`](../../benchmarks/size-exchange/results-v1.json).
Accepted run: `approved-v2`, 2026-10-02. All ten cases pass both independent
generations; all native, compact and gzip artifact hashes, semantic snapshot
hashes and canonically ordered diagnostics match.

All table values are bytes; OCDraw/IFCX gzip uses compact JSON.

| Case | OCDraw gzip | IFCX gzip | DXF gzip | DWG raw | DWG gzip |
|---|---:|---:|---:|---:|---:|
| empty | 789 | 471 | 6833 | 20755 | 16670 |
| lines-100 | 1817 | 2236 | 8130 | 22483 | 18141 |
| lines-10000 | 53690 | 149528 | 104154 | 202799 | 123653 |
| fractional-lines-1000 | 13557 | 25011 | 24809 | 45315 | 35646 |
| straight-polylines-100 | 2281 | 3088 | 9045 | 22677 | 18384 |
| straight-polylines-10000 | 81069 | 225646 | 168331 | 223080 | 133774 |
| circles-1000 | 7277 | 14277 | 14686 | 38523 | 28370 |
| nested-shared-blocks | 1412 | 928 | 7178 | 21331 | 17210 |
| named-line-patterns | 2674 | 3507 | 9308 | 23379 | 19087 |
| practice-foundation | 39605 | 47881 | 48022 | 86417 | 75256 |

The practice candidate retains 1,574 LINE, 207 CIRCLE, 164 INSERT and
3 LWPOLYLINE entities, 21 layers, 127 authored block definitions and 7 named
line patterns. CAD inventories also include two reserved block records and
the ByLayer/ByBlock pattern sentinels. The original has 501 MTEXT, 87 linear
dimensions, 85 HATCH, 73 ARC, 3 IMAGE and 6 VIEWPORT entities; these families
are excluded from the current common subset. Some supported geometry is also
omitted during preparation, as detailed in the loss diagnostics.

The only physical numeric preparation change is model instance 778 rotation,
from `1.570796326794893` to `1.5707963267948932`, then
`1.5707963267948934` radians. Two rounds reach the exact common fixed point.
No core tolerance or production converter behavior changed.

For the practice subset, compact OCDraw+gzip is 39,605 bytes; compact
IFCX-CAD+gzip is 47,881 bytes, so OCDraw is about 17.3% smaller. Current
pretty writer output+gzip is 48,440 / 56,361 bytes respectively. The large
line/polyline recipes favor OCDraw more strongly, while IFCX-CAD is smaller
for the empty and small shared-block recipes. These results describe current
encodings and this corpus, not a general ranking of format capability.

The reusable IFCX schema costs 11,720 raw / 999 gzip bytes,
separate from the table's known-profile transfer assumption. Python 3.14.7
and zlib 1.3.1.zlib-ng produced this run.

## Reproduction and interpretation

Use the command in the corpus README and a new run name. The original foundation
DWG is identified by SHA-256
`24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5`
and is 239,012 bytes. File paths are supplied explicitly and remain local.

The nine synthetic cases isolate primitive count, coordinate variation,
polylines, circles, shared/nested blocks and line patterns. Their compression
ratios depend on their deliberately repetitive geometry. The practice row is
the supported common subset of one drawing; it is not a lossless representation
of the original complete drawing. Report subset growth separately from encoding
savings on future runs. Text, dimensions, hatches, paperspace/viewports and
other excluded families need representative recipes before this experiment
can make claims about them. Existing accepted benchmark reports are unchanged.

## Transparency follow-up verification

On 2026-10-02, `transparency-foundation-v1` reran all ten cases in the optimized
profile after the narrowly guarded IFCX layer-transparency XDATA correction.
Both complete generations pass. Every raw/gzip artifact hash and semantic
snapshot hash is identical to the preceding accepted run; the retained
`benchmarks/size-exchange/results-v1.json` now records this run's current source
provenance. Historical standalone and legacy benchmark references are unchanged.
The separately accepted larger DXF practice comparison is documented in
[common-subset-test-dxf-v1.md](common-subset-test-dxf-v1.md).
