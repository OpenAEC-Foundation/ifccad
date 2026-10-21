# Native SOLID and TRACE

Design revised on 21 October 2026 after the OCS investigation and the user's
decision to admit every finite local corner combination. Both the convex-only
and later simple-nonzero-area validity restrictions are superseded. The revised
design is implemented and verified. Readers, schemas and conversion use this contract.
OCDraw is parked and is outside this slice.

## Purpose and sequencing

Preserve the filled primitives used by CAD drawings and dimension arrowheads
without converting their object type to Hatch or a wide polyline. SOLID and TRACE
are distinguishable native entity kinds sharing one geometry value and validation
kernel. TRACE's legacy authoring command does not make its existing geometry opaque.
No dedicated TRACE authoring UI or general preservation mechanism is required.

### TRACE terminology and authoring boundary

The native Trace kind represents the legacy DXF `TRACE` / DWG `AcDbTrace`
entity. It does not represent the collaborative Trace feature introduced in
AutoCAD 2022. Autodesk explicitly distinguishes the two in its
[DXF reference](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm).
The modern feature manages review/markup content and needs its own use case and
design; native Trace coverage makes no claim to retain that feature.

The old drawing command constructed wide straight segments and their joins.
Its usual authoring purpose differs from SOLID's filled faces, but both stored
entity descriptors have four corner slots, a normal and signed extrusion
thickness. Identical descriptors can therefore look identical. Their distinct
CAD type remains worth retaining. See Autodesk's
[legacy-command explanation](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/The-Old-TRACE-command-does-not-work-from-AutoCAD-2022-and-above.html)
and [Trace object reference](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-ActiveX-Reference/files/GUID-7113C9E5-6875-4ABB-A4DC-68F6C09C4D7D.htm).

`TRACEWID` is a drawing/database authoring setting; it is not a width or
centerline field in a stored TRACE entity. The corner descriptor already records
the resulting footprint. Do not reconstruct a centerline, regenerate joins or
apply `TRACEWID` to imported corners. Changing a nondefault `header.trace_width`
currently produces ordinary source-field loss; preserving that editor default
would be separate drawing-state work. It is not missing entity geometry and
does not depend on the later polyline-width slice. Extrusion thickness, geometric
width and appearance lineweight remain distinct. Host `FILLMODE` can affect
display without changing the descriptor; the existing unassessed display claim
continues to apply.

This is a preparatory slice before worked-out Dimension graphics, following the
stable migration/Hatch boundary in [ROADMAP.md](../../ROADMAP.md). It adds ordinary
entities to Model, Paper and local blocks. The later
[Dimension graphics mapping](../dimension/graphics-design.md) reuses their payloads
and CAD preparation without allocating ordinary IDs for graphics parts.
This slice does not introduce Dimension picture capture, new selectors, an update
engine, polyline widths, native 3DSOLID/mesh/B-rep semantics or xformop placement.

Source review also found a separate follow-up case: sample_AC1032's
`_ArchTick` marker is a LWPOLYLINE with constant width 0.15, not a SOLID/TRACE.
The precursor therefore does not by itself make that complete red picture
representable. The user has deferred polyline widths until after the bounded
Dimension graphics slice and assigned that red Dimension to the width slice's
acceptance. Width support is not a prerequisite of the bounded graphics plan;
its existing unsupported-content rules apply. No automatic conversion of that
marker to a filled primitive is selected here. See the
[roadmap follow-up](../../ROADMAP.md#planned-polyline-width-follow-up).

## Model and shared geometry

The public IFCCAD model adds:

- `IfccadEntityKind::Solid(IfccadFilledQuadrilateral)`;
- `IfccadEntityKind::Trace(IfccadFilledQuadrilateral)`.

`IfccadFilledQuadrilateral` contains required `placement: IfccadPlacement`,
required `corners: [[f64; 2]; 4]` and `thickness: f64`, defaulting physically to
positive zero. Corners retain their four stored slots; no variable-length array,
triangle flag, closing fifth vertex or separate cached outline is stored.
The entity discriminant retains the object kind. There is no native codec
`is_trace` flag or generic source-kind metadata on the geometry value.

The identity-free core geometry kernel gains a filled-quadrilateral view/value
with the same corners, thickness and validated coordinate frame, usable through
`GeometryRef`/`OwnedGeometry`. That geometry does not contain the Solid/Trace
distinction, layer, owner, IDs or a CAD runtime type. Core validation/bounds share
one implementation; `ifccad-convert` chooses the CAD entity type explicitly.
No new crate or generic geometry-payload registry is introduced.

An optional identity-free surface-cover analysis can recognize a simple polygon
and its admissible triangle/diagonal cover for stronger conversion evidence.
Failure to find such a cover is an ordinary Unassessed result, not invalid native
geometry or a conversion failure. This transient analysis has no IFCX/CBOR
encoding, entity identity, stored validity flag or stored triangulation.

The four slots follow CAD storage order. In one-based notation the visible
quadrilateral boundary is **1,2,4,3**. Repeating slots 3/4 is the conventional
triangle spelling, not the only accepted spelling: an exact duplicate at any
adjacent position of the cyclic boundary can describe the same triangle.
Both winding directions are legal. Corner slots are never sorted, swapped,
deduplicated or snapped during native IO. A temporary analysis view may coalesce
numerically equal adjacent boundary positions, including last/first; that view
does not change the stored four slots. Equality is exact numeric component
equality, including geometric equivalence of positive/negative zero; their stored
float bits remain independently preserved. The codec's epsilon-based triangle
helper must not choose native topology or discard slot 4.

Let placement be origin O, X/Y axes U,V and derived N = U cross V, computed with
the existing exact stored-axis semantics. Base and extruded vertices are:

```text
B_i = O + U * corners[i].x + V * corners[i].y
T_i = B_i + N * thickness
```

The stored meaning is the ordered CAD SOLID/TRACE corner descriptor with its
placement and signed extrusion coefficient. A simple nonzero-area boundary has
the ordinary filled polygon/sweep interpretation. A crossing, retraced or
collapsed boundary remains the supplied descriptor; IFCCAD does not invent one
universal fill, repair, triangulation, empty-fill or lower-dimensional rendering
rule for it. In particular, zero area does not mean that the entity can be dropped.
Thickness is a signed local-Z coefficient, not screen lineweight or polyline
width. Axes are not renormalized on read. This is not a B-rep/volume-analysis API,
and appearance does not imply separate stroked contour entities.

## Native validity and derived interpretation

Every combination of the four finite local XY corners is supported: convex,
concave, either winding, crossing, touching/overlapping or retraced edges,
adjacent/non-adjacent repeats, collinear states and complete collapse to a line
or point. There is no required area, simplicity, orientation or minimum number
of distinct points. A triangle convention or renderer epsilon is not a validity
rule. No corner is reordered, removed, snapped or rewritten on storage/conversion.

Validation checks fixed field/array structure, finite coordinates/thickness,
the existing valid placement frame, and an available finite base/top envelope.
Normal identity, ownership/reference and unknown-field rules still apply. This
removes topology restrictions; it does not relax binary64/frame/enclosure limits
or add spatial corners with independent local Z values. Near-coincident and
exactly equal points remain as supplied, including their native float bits.

The [OCS investigation](solid-trace-cad-behaviour.md) provides evidence that the
previously excluded finite states are retained by CAD. None needs a preservation
wrapper or another native entity kind. Native validity and support do not depend
on finding a fill or predicting a particular host's display.

Optional exact analysis may recognize simple/nonzero-area covers, duplicate
patterns or degeneracy. It is derived information only. It must not mutate the
descriptor, become a save gate, or label an unusual finite combination as malformed,
unsupported, lossy or eligible for SkipInvalidEntities merely because of topology.
The same applies to a missing common surface cover after conversion rounding.

Non-finite values, an invalid normal/frame, malformed structure, unavailable
finite enclosure and failure of the required coordinate-accuracy proof retain
their existing typed errors/policies. Unequal source OCS-Z values or genuinely
unsupported source metadata remain separate conversion boundaries. Allow/Reject
and Abort/SkipInvalidEntities are not activated by crossing or collapse alone.

Normal entity identity, exactly-one Model/Paper/block owner, draw order, local
layer/pattern references, visibility and allocation watermark rules are unchanged.
Solid/Trace use the existing `nextEntityId`; there is no new resource domain.
They are not newly eligible Hatch sources, viewport clip boundaries or Dimension
point selectors. Those eligibility sets remain explicitly unchanged in this slice.

Native diagnostics use `IFCCAD-FILLED-001` for non-finite intrinsic values
and `IFCCAD-FILLED-002` for invalid placement or unavailable finite enclosure,
retaining field/node locations and existing typed reports. Generic identity,
reference, ownership and physical-read errors retain their current categories.

## IFCX attributes and example

An ordinary entity retains `/cad/dD/eE`, one normal owner entry and the existing
`ifccad::entity` appearance/layer/visibility/scale value. Add exactly one of:

- `ifccad::geom::solid`;
- `ifccad::geom::trace`.

Both are closed Objects with required `corners` (exactly four arrays, each exactly
two Real coordinates) and optional `thickness` (Real, default positive zero).
Both require `ifccad::geom::placement` on the same node. Placement remains the
complete origin/X-axis/Y-axis value. There is no second embedded placement, normal,
fill flag, Hatch payload, source-kind member or `isTrace` Boolean.
Null, unknown members, wrong array lengths and combined native geometry roles fail.
The schema/supplemental declarations share the same value definition while
keeping both attribute names registered. Unknown core fields remain rejected.

This illustrative fragment assumes an existing drawing, layer 0 and ordinary
owner link. It is a node fragment, not a standalone drawing file:

```json
{
  "path": "/cad/d1/e20",
  "attributes": {
    "ifccad::entity": {"layer": "/cad/d1/layer/0"},
    "ifccad::geom::placement": {
      "origin": [0, 0, 5], "xAxis": [1, 0, 0], "yAxis": [0, 1, 0]
    },
    "ifccad::geom::solid": {
      "corners": [[0, 0], [20, 0], [0, 10], [0, 10]],
      "thickness": -2
    }
  }
}
```

Changing only the geometry attribute name to `ifccad::geom::trace` changes the
native entity kind; it does not alter these geometry parameters. Positive-zero
thickness may be omitted. Explicit negative zero remains present under the exact
default rule. IFCX Same-path LaterWins replaces a complete geometry attribute
value, not individual corners. Leaving the old Solid attribute while adding Trace
is an invalid two-payload entity, not a type-change transaction. Source-graph
deletion/replacement APIs remain deferred; fresh writers emit one selected type.

## CBOR codes

The current ordinary kind codes 0..13 keep their meaning. Reserve:

| Code | IfccadEntityKind | Variant body |
| ---: | --- | --- |
| 14 | Solid | record field 0=value_0, IfccadFilledQuadrilateral |
| 15 | Trace | record field 0=value_0, IfccadFilledQuadrilateral |

The existing ordinary native entity contextual kind/geometry rule applies:
the `geometry` slot contains this selected body. Standalone kind values keep the
ordinary `{0: variantCode, 1: body}` envelope. These are explicit codes, not Rust
enum ordinals. The shared value record is:

| Code | Field | Physical rule |
| ---: | --- | --- |
| 0 | placement | Required existing IfccadPlacement record |
| 1 | corners | Required fixed array of four fixed XY arrays, all coordinates binary64 |
| 2 | thickness | Binary64; default positive 0.0, omitted only for exact default bits |

Four slots remain required for triangles. No null, coordinate integer shortcuts,
unknown fields, repeated keys, shorter floats or inferred Z values are added.
Negative zero/subnormal bits and supplied corner order survive strict CBOR
readback. Full-width ordinary IDs keep the existing uint64 route.
The graphics-kind code space later uses the same explicitly registered 14/15
bodies; it does not enable ordinary entity identity inside a graphics part.

CBOR remains the single mutable unreleased version 1, updated in place along
with its inventory and conformance/next. Zstd transport remains version 1 around
the same CBOR payload. No historical compatibility mapping, version bump or new
transport/UI mode belongs to this slice. An open versioning issue is future
context, not authorization to change current version policy.

## CAD preparation and object-kind retention

`cad-geometry-convert` adds bounded ID-free filled-primitive preparation and
geometry evidence; `ifccad-convert` binds the existing common entity state and
maps the kind. The selected codec represents both as `EntityType::Solid`, with
`is_trace=false` for SOLID and `is_trace=true` for TRACE. Import must inspect that
flag, and export must set it deliberately. Do not trust the entity enum variant
or a `SOLID`-only reporting string to identify both types.

Source preparation reads all four raw corner fields, normal, thickness and common
state. It never calls epsilon-based `corners()`/`is_triangle()` to drop slot 4.
The first CAD profile requires all four source OCS Z coordinates to agree exactly
as numeric values. A mismatch is unsupported nonplanar source geometry, not an
instruction to project the corners into a common plane.

Use the existing CAD arbitrary-axis preparation. Choose native U/V from the
source OCS, origin from its common elevation and keep the ordered source XY
coordinates and signed thickness. Reconstruct source base and top vertices with
the actual source-normal/OCS semantics, and compare the prepared native geometry.
The existing source-normal normalization diagnostics apply if raw normal magnitude
cannot be retained by the native frame. No AA storage variant or relaxed numerical
limit is introduced to make the preparation succeed.

Target preparation uses the current native frame and existing CAD OCS factoring,
preserving the four slots and signed thickness. It must prove the complete
base/top correspondence with the actual emitted normal, corner elevation and
thickness. Required rounding follows the same hard geometry policy as other
primitives. If the target representation/setters cannot retain the profile, it
is refused or diagnosed under the appropriate existing error/loss boundary;
there is no fallback to Hatch, polyline or an exploded mesh.

SOLID exports as SOLID, TRACE as TRACE in ASCII DXF, binary DXF and AC1032 DWG.
The codec already has distinct DXF names and DWG type paths. That source capability
is not a passing conversion qualification: production-byte tests must establish
it independently. Native storage is broader than any unqualified target version.
Earlier native drafts and CAD byte identities are not compatibility promises.
Reader-synthesized defaults already collapsed into `CadDocument` remain the
codec-boundary limitation; this adapter does not claim to audit missing raw tags
that the reader no longer exposes or add its own replacement corner defaults.

This support applies equally to ordinary Model/Paper entities and all local
block definitions, including unused ones, with the existing source metadata
inventory. It is not a special exception limited to marker blocks. Invalid
scalars are checked even when another source field is unsupported. Unsupported
XDATA/common metadata retain current loss handling; object-kind support does
not qualify every possible source field.

The existing opt-in preservation pilot remains separate. Neither type becomes
opaque merely because TRACE has a legacy authoring history. Polyline width,
generic external payloads and preservation of arbitrary unknown attributes are
not implicitly introduced. TRACE editing tools and automatic TRACE-to-polyline
conversion are outside this slice.

## Storage, geometry evidence and display guarantees

| Guarantee | Required meaning |
| --- | --- |
| Native storage | Both encodings strictly retain kind, four ordered XY slots, placement and signed thickness, including native float/default rules, for every finite topology |
| Coordinate accuracy | All four base and four top positions must pass the existing exact/domain-specific hard numerical limit, including every retained occurrence |
| Parameter envelope | Exact/outward min/max of all eight positions; finite, conservative and defined even for crossing or wholly collapsed records |
| Filled surface/sweep | Proven only when an independently checked common simple-polygon cell cover exists; otherwise Unassessed, while storage and coordinate evidence remain supported |
| Display | Always unassessed by this format/converter slice; no cross-viewer pixel, outline, lighting or fill-equivalence claim follows from any geometric status |

Native geometry bounds enclose every base and top corner, with exact/outward
coordinate evaluation. This is the descriptor's parameter envelope, not a rule
that fills its convex hull or predicts its rendered footprint. It remains defined
for a crossing, line or point state and conservatively encloses the usual simple
polygon/sweep when that interpretation applies. At zero thickness the top coincides
geometrically with the base. Negative thickness and arbitrary valid placements
require the same full enclosure; thickness is never replaced by abs(thickness).
Enclosure overflow is a native geometry error. Supplied scope bounds stay optional
and conservative; explicit recomputation remains atomic and does not run on save.

The mandatory numerical proof pairs all four base and all four top slots in owner
coordinates, including repeated/collapsed slots. Their exact rationale evaluation,
finite target construction and deviation assessment remain hard requirements;
unknown fill cannot bypass a coordinate proof failure. Full-surface evidence is
an additional, optional guarantee requiring
one common corner-index triangulation covering both source and actual prepared
target bases. For boundary 1,2,4,3, the diagonal choices are stored slots 1-4
(triangles 1,2,4 and 1,4,3) or 2-3 (triangles 1,2,3 and 2,4,3). A concave quad
admits only its interior diagonal; never use a fixed diagonal or its convex hull
without proving that it covers the polygon. Straight-through collinear corners
can produce a zero-area cell within a nonzero-area polygon. Adjacent duplicates
use the same surviving corner-index triangle while all four pairs remain retained.

Intersect the admissible choices for source and target. If either descriptor has
no simple cover, rounding changes the relevant duplicate/index relationships,
or valid simple bases have no common cover, surface coverage is Unassessed.
Conversion still succeeds when required coordinate proof and the other source
contracts pass, including under Reject. Do not reject, skip or alter the record
to obtain a surface proof. Rounding can change derived topology or host behaviour;
the coordinate-rounding diagnostic must state its scope and makes no topology
or display guarantee. Exact mode still refuses any nonzero coordinate residual.
Never claim a full-fill bound from eight anchors alone. Shared triangle/prism barycentric
parameterizations then make the maximum vertex-deviation bound cover the entire
surface/sweep. This small exact decomposition is proof machinery, not a Dimension
display generator or stored mesh. Arbitrary Hatch fill remains unassessed.

Expose the coverage explicitly in conversion evidence, not in native storage:
per retained filled entity/occurrence, record its native owner/entity identity,
`positionScope=FourBaseAndFourTop`, and `surfaceCoverage=Proven|Unassessed`.
The source/target pair's optional common cover determines this last value;
even exact corner correspondence alone does not promote an unknown fill.
`IfccadGeometryAssessment::filled_surfaces()` returns those typed records.
The companion API uses `IfccadFilledSurfaceAssessment` with `source()` and
`coverage()` getters, and `IfccadFilledSurfaceCoverage::{Proven, Unassessed}`.
They are derived conversion results with canonical native owner/entity or
occurrence identities, assembled for retained mapped pairs after successful
coordinate assessment; they are not new core-model or physical-encoding types.
An Unassessed surface is also represented through the existing unassessed-entity
route, so `is_complete()` cannot report complete geometry from anchors alone.
Partial surface coverage is not a semantic-loss diagnostic and does not by itself
trigger Reject or a partial-open omission. The viewer presents coordinate status,
surface coverage and `displayEvaluation=unassessed` separately, for every topology.
Its JSON evidence spells these as `geometry.filledSurfaces[]` records with
`source`, `positionScope:"FourBaseAndFourTop"`, `surfaceCoverage:"Proven"|"Unassessed"`
and `displayEvaluation:"unassessed"`; IDs use the existing decimal-string route.
This changes the evidence API/presentation, not the native file format.

The tested OCS build displays conventional triangles, concave quads and the
straight-through-corner case with thickness. Its alternate adjacent-duplicate
triangle spellings retain thickness in files but fall back to a base outline in
the tested shaded/isometric display. Keep the original four slots and report this
host limitation separately; do not canonicalize the triangle, zero thickness,
reject valid native geometry or patch OCS to make the preview look different.

Submit the proof in Model/Paper/block units and every retained occurrence using
the existing domain-specific tolerance engine, including reflected/nonuniform
nested INSERTs. Transform the base and top points through the full occurrence
map: recomputing an extrusion from transformed base axes can give a different
shape under scale or reflection. Retaining a block is preferable to inventing
a flattening operation for a sheared extrusion. No new general transform model
is selected. Definition-local proof alone is insufficient.

The core record describes a filled CAD entity, not a separate stroked line-pattern boundary.
Common appearance choices remain stored and validated even when a leaf is
inactive for a filled display. Native bounds do not invent an outline stroke or
text/marker footprint. External applications' fill-display/plot behavior remains
separately qualified by existing appearance/header coverage.

## Applications and verification

The IFCCAD viewer/browser/Explorer recognize both kinds, inspect their four
corners, placement and signed thickness in Dutch/English, and report separate
kind names. Maintain the distinction between the filled SOLID primitive and
Solid Hatch. Example/catalogue sources add ordinary triangles, quads and extruded
positive/negative cases, with Model/Paper/block occurrences. The current catalogue
count of 14 is updated only when implementation actually adds the kinds.

The primitive bounds, serializers, source-aware foreign/numeric audits and
conversion output all need real support; labels alone do not qualify the slice.
Native download retains source bytes; fresh IFCX/CBOR output uses strict validated
production readback. CAD preview uses the same emitted bytes as download. No
OpenCADStudio patch, new rendering engine or TRACE drawing-tool UI is required.

| Area | Required implementation evidence |
| --- | --- |
| Identity/type | Ordinary ownership/order/watermarks and full-width IDs; Solid and Trace survive as distinct native and CAD types |
| Values | Conventional and adjacent-duplicate triangles, convex/concave quads, straight-through collinear corners, boundary 1,2,4,3, both windings, near-coincident distinct slots, signed thickness and exact native float bits |
| All combinations | Crossing, retraced, touching, all repeat patterns, collinear, line and point states remain valid and survive strict native/CAD readback; no omission or loss because fill coverage is unknown |
| Rejection | Null/unknown fields, wrong array lengths, non-finite scalars, invalid frame and unavailable finite enclosure; unequal source OCS-Z remains a separate mapping boundary |
| Source profile | Unequal OCS corner Z stays unsupported; no epsilon collapse or zeroing; malformed source and invalid normal retain typed failures |
| Encodings | IFCX/supplemental schema, logical validator, CBOR codes 14/15 and value codes 0/1/2 agree; strict production readback, source audits and optional Zstd |
| Geometry | Mandatory eight-position bounds/accuracy for all topologies; optional common-cell full-surface proof; no common cover yields explicit Unassessed rather than refusal; unchanged hard coordinate failures, units and occurrence limits |
| Occurrences | Model, Paper with fixed/unknown physical mapping, local/unused blocks, nonzero block bases and reflected/nonuniform nested instances |
| CAD bytes | Literal SOLID and TRACE source fixtures; actual ASCII/binary DXF and AC1032 DWG type retention, corners/normal/elevation/thickness and common state |
| Independent checks | Read raw DXF entity names/corner tags and a separately checked DWG object type so matching codec errors cannot hide type loss; independently evaluate corner order/OCS geometry |
| Failure policy | Allow/Reject separate from Abort/SkipInvalidEntities; entity-local omissions retain evidence, shared structural failures remain fatal, no substituted primitive |
| Eligibility | New kinds remain ineligible as Hatch sources, viewport clipping entities and Dimension selectors unless a separate contract changes that |
| Applications | Both localized types, property/source inspection and numerical evidence, representative examples and applicable frontend/WASM checks; retain and disclose the alternate-triangle thickness display limitation in OCS |

Implementation updates active IFCX/supplemental schemas, registry/profile rules,
shared kernels/API/validation, shared-geometry and converter coverage, CBOR revision-1
inventory and conformance/next. Numbered conformance and parked legacy stay intact.
Use focused checks, then the final candidate gate: workspace boundary check, fmt,
workspace Clippy with -D warnings and workspace tests. Qualify feature-enabled
Zstd and frontend/build/smoke paths as changed. Record complete input/dependency
provenance. No automatic benchmark or size/exchange run is authorized.

## Issue overlap and rationale

Relevant issues were reviewed on 21 October 2026. [#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2)
supports a bounded typed family, not a universal representation registry.
[#11](https://github.com/OpenAEC-Foundation/ifccad/issues/11) leaves AA placement
as an investigation; this slice uses existing frames. [#13](https://github.com/OpenAEC-Foundation/ifccad/issues/13)
keeps associative updates separate. Existing bounds semantics and color/source
resolution boundaries stay unchanged; no newly eligible associations are added.
[#19](https://github.com/OpenAEC-Foundation/ifccad/issues/19) proposes package/release
year-week versions. That is distinct from the CBOR wire mapping revision; it does
not select a change to the active mutable CBOR-v1 policy in this slice.

The alternatives were native paired kinds, Hatch substitution and opaque
preservation. The user selected native support for both kinds, sharing geometry.
That retains inspectable shape/type and permits ordinary drawing use as well as
Dimension graphics. The Hatch-substitution proposal is superseded. No source-kind
hint or new opaque preservation family is needed.

Primary CAD references: [SOLID](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
and [TRACE](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm).
The inspected codec is base `325fa98` with the existing explicit Dimension patch.
Its four raw corner fields and `is_trace` flag support this design; its epsilon
triangle helper and base-only `bounding_box()` are not the required validation,
thickness enclosure or proof. No new dependency patch has been selected or tested.
