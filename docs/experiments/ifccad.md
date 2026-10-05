# IFCX-native CAD node experiment

The [document lifecycle](ifccad-document-lifecycle.md) separates the complete
immutable IFCX source from the editable typed CAD projection. `IfccadDocument`
is the central input/output for shared validation, encoding and direct CAD
conversion. Merging an edited projection into a larger source graph remains
deferred; CAD-profile encoding produces a fresh file.

Status: experimental profile, 2026-10-01. This is an opt-in IFCX alpha drawing with a separate provisional schema module, independent of the standalone OCDraw contract. Historical released IFCCAD conformance collections remain unchanged. The [contract](../../schemas/ifccad/experimental-contract-0.1.0.md), [schema module](../../schemas/ifccad/ifccad-profile-0.1.0.ifcx), [drawing fixture](../../examples/ifccad/hello-cad.ifcx), [nested-block fixture](../../examples/ifccad/hello-nested-blocks.ifcx), [paper-layout fixture](../../examples/ifccad/hello-paper-layouts.ifcx), and [strict-readback tests](../../tests/ifccad_native.rs) form the reproducible result.

## Finding

### Direct conversion proof (2026-10-01)

The experimental [IFCCAD converter](../../crates/ifccad-convert/README.md)
now maps the validated projection directly to/from opencadcodec, without an IFCDR
intermediate. Its bounded Model subset covers lines, circles, straight polylines,
units, layers, stored appearance and shared/nested local blocks. Exact XY
translation and CAD setter checks reject rounding/clamping. Extra source fields
and foreign IFCX information are diagnosed. Default Allow returns a supported
subset with explicit loss evidence; Reject refuses diagnosed semantic losses.
Structural and numeric precision failures remain errors under both policies.

In-memory tests retain owner order, unused declarations, signed nonuniform block
scale, rotation/base/unit and large u64 identity mappings. Every generated IFCX
passes the production writer/reader. The same primitive corpus passes semantic
exchange through unmodified pinned opencadcodec DXF and DWG (AC1032). Nested blocks
with nonzero base pass both DXF and DWG using opencadcodec revision
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`. The former nonzero DWG block-base
reader defect is resolved. Full inventory traversal is required: the ordinary CAD
entity iterator hides these structural markers.

The direction-specific coverage contracts record default scaffold handling and
unsupported semantics. Full plot conversion, broader oblique parameterization, broader
CAD/source preservation and optimized encoding remain
deferred. These probes support suitability of the tested IFCCAD subset;
they provide no broad losslessness, size or performance claim.

### Partial practice conversion (2026-10-01)

Default Allow was applied independently to the DWG and DXF funderingsherstel
practice drawing identified in the
[placement inventory](../benchmarks/placement-preparation-v1.md). Source hashes
remain DWG `24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5`
and DXF `054317ae053be20d9c0f9c8650b5070f76826a7eede9690cd9c572700ea6149c`.
Each route went through the production IFCX writer/reader, then CadDocument,
the pinned DXF writer/reader and the production IFCX writer/reader again.

Both retained 21 layers, 127 local definitions, 1,574 LINE, 207 CIRCLE,
164 INSERT and three LWPOLYLINE entities across Model and definitions. Text,
dimensions, hatches, arcs, images, viewports and Paper content were omitted
with evidence. Those three Model polylines now retain nondefault `plinegen` as continuous
pattern generation. Seven named patterns (including Continuous), signed lengths,
unused definitions, drawing/entity scales and name-based references survive
the final DXF readback. One finite declared period disagreement in the DXF
source Hidden definition is diagnosed and derived from its elements. Appearance
adaptations, unsupported metadata and partial nested definitions were also diagnosed. This is a partial drawing,
not a source-preserving or complete presentation of the original sheet.

Semantic comparisons cover geometry, units, layer names/appearance, entity modes,
named block targets/base/unit and owner order, pattern definitions/lengths/scales
and polyline generation, excluding regenerated IDs.
The external DXF codec introduces one additional binary64-step rotation change
at Model entity index 778 (absolute error `2.220446049250313e-16` radians) in each
route. The probe records it separately and verifies that it exactly matches the
pinned degree/radian conversion. All other compared semantics agree; this is
not an exact numerical DXF roundtrip. No tolerance is added to the converter.
Generated files and detailed reports remain local artifacts outside Git.

### IFCX-native representation

The subsequent Paper viewport slice stores a Model reference, Paper frame,
orthographic/perspective camera, signed depth clipping, seven render modes,
independent display controls and frozen layers in the native profile. Active
Paper clipping currently accepts analytic circles and closed straight polylines,
including concave boundaries; dormant references retain ownership and identity.
See the [contract](../../schemas/ifccad/experimental-contract-0.1.0.md#paper-viewports)
and [example](../../examples/ifccad/hello-viewports.ifcx).

In-memory, DXF and DWG exchange tests use explicit development codec repairs for
clipping activation/boundaries, DXF angle units and independent off state.
[Patch provenance](../../patches/opencadcodec-viewports/README.md) distinguishes
these results from the unmodified production pin. Independent perspective
arithmetic and hand-authored DXF provide camera reference evidence; no AutoCAD
rendering equivalence is established. Native storage does not depend on these
codec repairs. Ellipses and bulges still need native IFCCAD geometry support;
OCDraw's independently expanded clip contract does not grant that support here.
Full plot settings and a viewport rendering API remain later work. Historical
practice and size results below retain their original tested coverage and pins.

**IFCX nodes are a plausible authoritative representation for the tested CAD subset.** Every independent entity has an addressable path, ordinary IFCX `children` establish ownership, numeric child keys convey drawing order, and direct attributes retain precise geometry, unit, layer and appearance modes. One block definition serves two differently placed and styled instances without cloning its entity nodes or using `inherits`. A foreign IFCX node can reference a CAD entity. The generic IFCX graph remains extensible while a strict CAD-profile reader catches the conditional rules that IFCX's current per-attribute schemas cannot express alone.

The proof also shows that a CAD profile remains necessary even if a shared IFCX geometry vocabulary emerges: IFCX alpha does not prescribe CAD draw order, unique layout/block ownership, layer-0 behavior, ByLayer/ByBlock resolution, or an exact mapping of all CAD primitives. Those rules are explicit here rather than presumed to come from `inherits`. The size and parsing cost of one node per entity deserve a broader controlled comparison before choosing a production physical encoding.

| Criterion | Observed result | Evidence / limit |
| --- | --- | --- |
| Identity and composition | Passed for the proof | Paths survive ordered readback; the default reader uses later values per key, including geometry and draw order. An explicit `RejectConflicts` option supports development checks. Duplicate JSON keys in one object fail. CAD validation checks the composed result. |
| CAD meaning | Passed for tested primitives | Strict writer readback retains drawing unit, XYZ values, placement, line, planar polyline, circle, scope order, layers, four appearance modes and values. |
| Reuse and occurrence | Passed for two nested definitions | The inner definition is referenced from an outer definition, which has two model occurrences. Strict readback retains definition targets, child order, transforms, layer `0`, and ByLayer/ByBlock/Explicit modes. Effective appearance is not computed by this reader. |
| Multiple layouts | Passed for one Model and two Paper layouts | Each scope keeps its own draw order and entities have one owner. A3 in mm and Letter in inches coexist with a cm model. Shared layers and nested block definitions are reused; the paper instance's explicit scale of 10 survives without unit conversion by the reader. |
| IFCX integration | Passed locally | Unknown nodes and non-CAD attributes survive loading; another node may refer to a CAD path. The one versioned CAD schema import resolves offline; general or remote import resolution is not implemented. |
| Validation | Passed for tested failures | Missing placement, duplicate ownership, child-key gap, invalid unit, unsupported geometry, missing schema and unused definition cycle fail. The reader is strict for this bounded profile, not a full IFCX validator. |
| Geometry alignment | Documented, not established by interchange | Local circle aims at IFC4 analytic circle semantics; finite line segment differs from unbounded `IfcLine`; polyline bulges and widths have no tested mapping. |
| CAD coverage | Partial | Core supports Model and Paper layouts with explicit tab order, separate coordinate units and optional physical media; the converter covers their bounded geometry subset through DWG/DXF. Viewports, plot settings, annotation, complex text/shape line patterns, indexed color identity, hatch, spline and preservation remain outside this experiment. |

The [upstream IFCX alpha TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp) currently models a node as a path plus optional `children`, `inherits`, and `attributes`, with `schemas` describing individual attribute values. This prototype uses that shape and reads repeated path fragments. Its compact path convention and numeric child keys are local profile choices while IFCX path conventions and graph semantics evolve. The [upstream gap register](ifcx-upstream-gaps.md) records these and other assumptions, their broader uses, and relevant buildingSMART issues. A [composition probe](ifcx-composition-probe.md) found that the original reader rejected conflicts that the upstream composer resolved with the later value; the current reader follows that observed direction. The 0.1.0 identifier remains provisional, without a compatibility promise during this alpha experiment. The probe also found that the then-current slash-containing paths did not appear as roots in the upstream composer. The current paths omit the earlier angle brackets but still contain `/`; upstream expansion needs a fresh check. The schema import uses `urn:example`, a registered namespace for experiments; it is not a production publication address. The upstream examples are preliminary; compatibility with a later IFCX release requires renewed validation.

In a 2026-09-30 viewer check, a temporary copy of the then-current bracketed fixture with the profile schemas inlined and no imports loaded but showed only the artificial `root` in the model tree. Replacing all node paths and references with temporary slashless aliases made the tree visible, matching the earlier composer probe. Those aliases violated the local CAD profile; the viewer still gave limited insight into the CAD content. The two temporary files were removed after this check. This is a historical viewer interoperability finding, not a failed strict CAD readback of the current fixture.

## OCDraw migration (2026-10-01)

The experiment now builds on the standalone OCDraw repository layout. Core IFCX
code is in `src/ifccad`, exported as `ocdraw::ifccad`; its separate converter
is in `crates/ifccad-convert`. Geometric frame/transform validation uses
OCDraw's shared types and length-unit validation reads OCDraw's bundled logical
registry. IFCX nodes, composition, schema imports and CAD profile rules remain
independent of OCDraw's logical drawing model and JSON codec. The converter
maps directly to opencadcodec, without passing through an OCDraw drawing.

The mechanical migration retained strict conversion. A subsequent independent
Allow/Reject implementation now supports partial conversion, whole-entity
omission, explicit appearance adaptations and nested block loss propagation.
It does not depend on `ocdraw-convert` or extract a shared conversion crate. No
profile version increase, browser route or new physical encoding is introduced.

## Physical encoding observations

Before adding paper metadata to the schema, the pretty-printed drawing fixture was **7,806 bytes** and imported the reusable **9,552-byte** schema module; counting both files once gave **17,358 bytes**. A separate synthetic drawing with 1,000 ordered XYZ lines was **731,205 bytes**, or **740,757 bytes** with that schema module counted once; one debug-build strict read of the drawing took **34 ms** on this machine in a single run. These are historical exploratory observations using the unwrapped path convention; the current schema module is larger after adding Paper layouts. The reader resolves the known experimental schema from its bundled definitions; that timing excludes filesystem or network schema loading. These are not benchmark distributions or product thresholds. The writer itself also strict-reads before returning.

The existing [IFCCAD/DXF/DWG size baseline](../benchmarks/size-baseline-v1.md) measures different controlled recipes, millimetres and different drawing metadata. It cannot be directly divided into the figures above to claim a fair ratio. In particular, the mixed fixture includes block and circle semantics absent from the baseline corpus. A later matched experiment should generate the same typed recipe through every writer, strict-read each output, include full package overhead, compare pretty and compact JSON, and apply identical stated compression settings to each complete file. DWG's native compression must be labelled separately. Compressed bytes alone do not prove a usable encoding.

[CBOR (RFC 8949)](https://www.rfc-editor.org/rfc/rfc8949.html) could encode this same logical IFCX object/map/array graph with binary numbers and shorter structural overhead. No CBOR writer, reader, schema-version behavior, or strict readback is implemented here. It is therefore only a candidate physical encoding, not a supported IFCX CAD variant. A production choice needs a decoder and the same semantic validation, deterministic mapping rules, complete-file measurements and exact readback before comparing it with compressed JSON, DXF and DWG.

## Recommendation

The [persistent ID allocation design](ifccad-id-management.md) was implemented
and verified locally on `ifccad-id-management` on 2026-10-02. It keeps compact
node paths and stores per-domain watermarks for native editing. Native
read/edit/write retains IDs and advanced counters; fresh CAD imports start a
new allocation history. Independent concurrent allocation remains deferred.

Continue the IFCX-native model as an experiment, especially for addressable CAD entities linked to BIM/project data. Keep the profile explicit and provisional alongside the independent OCDraw contract. Next decisive work is representative CAD coverage (more curves, plot settings and appearance forms), an independent implementation of the profile, and a semantically matched size/CPU study. If buildingSMART publishes exact IFCX geometry attributes, replace local geometry keys selectively where their units, placement and roundtrip meaning agree; preserve CAD-specific role, layer, order and appearance semantics in a CAD profile.

### Named line-pattern slice (2026-10-01)

The profile now stores drawing-owned `ifccad::linePattern` nodes with u64 path
identities, names, optional descriptions and signed-length arrays. Layers and
explicit entities refer to paths; ByLayer/ByBlock remain modes. Scales default
to 1 and planar polyline generation defaults to perSegment. These are explicit
CAD profile rules; G1/G3/G4 track path resolution, fragment replacement and
cross-node validation assumptions. The experimental schema URI/version remains
0.1.0 without a compatibility promise. Existing examples were migrated.

Core tests cover Unicode case folding, missing/wrong references, ownership,
reserved names, invalid lengths/scales, whole-pattern fragment replacement and
u64 IDs. Converter tests exercise simple fractional/dot patterns, unused named
empty definitions, both real DXF/DWG routes, exact numeric/default handling and
complex text/shape fallback. Allow replaces each entire complex definition with
an empty named pattern and diagnoses it once; Reject refuses those losses.
The model stores intent and does not resolve appearance or render patterns.
The standalone OCDraw contract/converter remains unchanged.

### IFCCAD & OCDraw Explorer route (2026-10-01)

The existing explorer now opens `.ifcx`/`.ifcx.json`, presents composed IFCX nodes
and graph information, downloads original IFCX bytes and previews/exports the
CAD projection. CAD input can choose OCDraw or direct IFCCAD. This uses the
existing independent reader/converter; shared adapter code only writes/checks
physical CAD files. Actual CAD readback must convert to strict-readable IFCX
before a download is exposed. Nonzero DWG block bases now pass this transfer.
Original graph inspection/native download retain
foreign information; CAD projection diagnoses omissions. Browser processing,
size limits and cancellation remain active. Viewer/WASM tests cover both native
routes and named-pattern DXF/DWG roundtrips; no general IFCX renderer or exact
external-codec numeric fidelity is claimed. Deployment is a separate action.
