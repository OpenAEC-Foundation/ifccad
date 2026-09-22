# Block codec boundary qualification

The converter currently selects the unmodified cadcodec revision
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`, without a local codec patch.
These observations characterize the dependency separately from the converter's
local-definition and ordinary-instance support.

`block_codec_boundary` exercises independent DXF and DWG roundtrips of a
definition named `Door`, base point `(2,0,0)`, line `(2,0,0)..(3,0,0)`,
all insertion-unit codes 0 through 24, and an anonymous flag independent of
the name. Inserts use position `(3,4,5)`, normal `(1,2,3)`, angle `0.7`, and
scales `(2,3,-4)` or `(-2,-2,-2)`.

## Observed limitations

- DWG materializes a public Block marker with `base_point=ZERO` rather than
  the nonzero value retained on BlockRecord. Its name/owner agree, but its
  base point does not. `dwg_document_builder::OBJ_BLOCK` explicitly constructs
  this zero; it is not a source inconsistency. DXF deliberately omits these
  structural marker entities. Do not claim nonzero-base DWG export is qualified
  under the marker-consistency check until this discrepancy is resolved.
  Tracked upstream as [cadcodec #52](https://github.com/HakanSeven12/cadcodec/issues/52).
- The DXF BLOCKS writer emits the record base point but omits description
  group 4. Its roundtrip therefore loses `BlockRecord.description`; DWG
  preserves it in the same case. A separate entity-marker writer does emit
  group 4, but that is not the ordinary block-record serialization path.
- Public Insert scale setters replace every magnitude below `1e-12` with
  positive `1e-12`. Both `+1e-13` and `-1e-13` lose their exact value before
  serialization, even for an empty definition. The DWG document builder also
  calls these setters when materializing decoded scale values. Native IFCCAD
  must not inherit this constraint or treat the change as exact conversion.

## Coordinate inspection

Both codec writers serialize `insert_point` directly; the DWG reader builds
the public Insert from that point without an OCS transformation. DXF serializes
rotation in degrees; DWG stores radians. The public `get_transform` evaluates
`OCS * translation * rotation * scale`, despite a misleading WCS field comment.
Thus the public insertion point is used in OCS by that helper; it must not be
copied directly into an owning-scope placement origin for oblique normals.
Block base-point subtraction is a separate operation.

Independent sampled owning-scope assertions now use the hand-derived axes
`u=(-2,1,0)/sqrt(5)`, `v=(-3,-6,5)/sqrt(70)`, `w=(1,2,3)/sqrt(14)` and
reference sin/cos literals. Empty-definition roundtrips also confirm the
already-clamped scale. These tests are sampled semantic checks, not a certified
whole-domain numerical proof. Marker tests now explicitly characterize the DWG
inconsistency and DXF absence; explodable and uniform-scaling flags roundtrip.

## Converter policy and exchange evidence

The converter rejects contradictory present marker name/owner/base-point data
as `InvalidSourceStructure` under both loss policies. Absence is allowed; a
zero marker is not guessed to mean absent. Native-to-CAD conversion constructs
consistent records and markers. `block_exchange` exercises production package
readback and the actual codecs: DXF with nonzero base and DWG with zero base
return successfully; DWG with nonzero base is explicitly rejected on return
because of #52. DXF description loss remains visible in these assertions
([cadcodec #49](https://github.com/HakanSeven12/cadcodec/issues/49)).

Scale setters are checked after construction. Changed values give
`BlockTargetLimitation`, also for empty definitions. Non-neutral native frames
produce an explicit `BlockParameterizationChanged` diagnostic; actual target
geometry is assessed, not assumed equivalent from the extracted angle alone.
Occurrence-space assessment carries correlated source-minus-target intervals
through each nested transform and reports the instance path and leaf. An
export regression shows locally acceptable rounding failing after 1e9 outer
scaling. These checks do not certify arbitrary downstream CAD applications or
recover source fields already lost before the CadDocument boundary.
