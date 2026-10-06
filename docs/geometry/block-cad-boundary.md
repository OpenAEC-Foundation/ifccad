# Block codec boundary qualification

The converter currently selects the unmodified opencadcodec revision
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`, without a local codec patch.
These observations characterize the dependency separately from the converter's
local-definition and ordinary-instance support.

`block_codec_boundary` exercises independent DXF and DWG roundtrips of a
definition named `Door`, base point `(2,0,0)`, line `(2,0,0)..(3,0,0)`,
all insertion-unit codes 0 through 24, and an anonymous flag independent of
the name. Inserts use position `(3,4,5)`, normal `(1,2,3)`, angle `0.7`, and
scales `(2,3,-4)` or `(-2,-2,-2)`.

## Resolved reader/writer defects and remaining limitations

The current unmodified opencadcodec pin resolves three earlier defects:
DWG BLOCK markers retain their record's nonzero base point (#52), anonymous
names retain BLOCK_CONTROL ordinals including unresolved slots (#55), and DXF
BLOCKS retains description group 4 (#49). The former narrow anonymous rename
on an export copy has been removed. Present marker name/owner/base conflicts
are structural errors under both loss policies.

The synthetic AutoCAD 2025 fixture in
`crates/ocdraw-convert/tests/fixtures/anonymous-names.dwg` checks named/anonymous
alternation, marker agreement, INSERT names and strict OCDraw readback without
repair. Its attributed INSERTs remain unsupported and diagnosed; the test does
not claim attribute preservation.

Two limitations still apply:

- An extra paper-space DWG BLOCK marker now has the correct name, but can still
  carry the primary paper record's owner instead of its own record's handle.
  The multi-paper exchange test characterizes this remaining inconsistency;
  conversion rejects it rather than guessing an owner.
  [Upstream #78](https://github.com/HakanSeven12/opencadcodec/issues/78)
  isolates it with a fresh `CadDocument` and one `add_layout` call at this pin.
- Public Insert scale setters replace every magnitude below `1e-12` with
  positive `1e-12`. Both `+1e-13` and `-1e-13` lose their exact value before
  serialization, even for an empty definition. The DWG document builder also
  calls these setters when materializing decoded scale values. Native drawing
  semantics must not inherit this constraint or treat the change as exact.

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
whole-domain numerical proof. Marker tests check nonzero DWG base consistency and DXF marker absence;
explodable and uniform-scaling flags roundtrip.

## Converter policy and exchange evidence

The converter rejects contradictory present marker name/owner/base-point data
as `InvalidSourceStructure` under both loss policies. Marker absence is allowed;
a zero marker is not guessed to mean absent. Native-to-CAD conversion constructs
consistent records and markers. `ocdraw_block_exchange` and the IFCCAD
`exchange` suite use production native readback and actual codecs: local
nonzero-base blocks now return successfully through both DXF and DWG. The
standalone route also retains descriptions in both codecs. IFCCAD's narrower
block model still diagnoses descriptions as unsupported source metadata.

Scale setters are checked after construction. Changed values give
`BlockTargetLimitation`, also for empty definitions. Non-neutral native frames
produce an explicit `BlockParameterizationChanged` diagnostic; actual target
geometry is assessed, not assumed equivalent from the extracted angle alone.
Occurrence-space assessment carries correlated source-minus-target intervals
through each nested transform and reports the instance path and leaf. An
export regression shows locally acceptable rounding failing after 1e9 outer
scaling. These checks do not certify arbitrary downstream CAD applications or
recover source fields already lost before the CadDocument boundary.

These correlated evaluation kernels now live in the shared `cad-geometry-convert`
companion and serve both converters. IFCCAD retains independent uint64 owner
identities and applies each retained Paper occurrence's own coordinate-unit
budget. Core transforms and conservative bounds live in `ocdraw::geometry_kernel`.
See [shared geometry](shared-geometry.md); this extraction does not change
the documented codec pin, marker conventions or qualification assumptions.
