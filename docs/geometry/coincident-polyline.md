# Exact coincident planar polyline segments

The experimental IFCCAD planar-polyline contract retains ordered XY vertices
and each finite signed outgoing bulge. Exactly coincident segment endpoints
describe point-shaped geometry, even with a nonzero bulge. Both vertices and
their original bulges remain editable source data. The dormant final bulge of
an open path is retained as before. No point removal, bulge reset/reassignment,
invented radius/tangent or proximity epsilon is permitted. This rule cannot
recover an author's possible intended nonzero arc from coincident endpoints.

This is a semantic extension of the mutable draft, with no new fields or CBOR
version increase. The core has no CAD runtime dependency. Shared validation,
bounds and containment use the same rules for the packed and separate vertex
backings. Bounds of a coincident segment are its placed point enclosure. Existing
point, full-curve and nested-occurrence accuracy proofs remain in force.

CAD conversion distinguishes existing source degeneracy from newly introduced
degeneracy. A nonzero source bulged chord whose target endpoints become equal
is rejected even under Allow and a generous tolerance. The existing coordinate
domain limits, finite/range checks, exact requests and signed nonuniform block
transform proofs are unchanged. A source-degenerate segment still has to meet
the same placed-point and occurrence accuracy limits.

Hatch and viewport contours retain their independent eligibility requirements.
A stored bulge on a coincident segment supplies no geometric curvature and
cannot make an otherwise invalid two-distinct-point straight contour valid.
Other Hatch join/parameter and Paper containment requirements remain unchanged.
Width, thickness and unsupported metadata remain governed by their existing
conversion loss contracts; this support does not imply general lossless CAD IO.

## Practice-file cause and verification

In the original test.dxf, 154 of 15,054 LWPOLYLINE records contain 156 active
coincident segments with nonzero bulges. All endpoint pairs are already exactly
equal in the decimal source and arrive unchanged as binary64 in the selected
codec. The previous native rule rejected them as InvalidGeometry, exposed by
the converter as the overly general CadAxisEvaluationFailed. This is an
intrinsic representation refusal, not a tolerance exceedance or normal failure.
150 records are window frames, two roof-window records and two gasket records.
152 paths are closed and two open. Four retain nonzero elevation near -544;
all normals are +Z and no record has nonzero width or thickness.

The complete original test.dxf has been checked: all 154 formerly failing
LWPOLYLINE records now pass, with all 156 coincident segments retained. All
54,598 entity handles are accounted for. The previous 267 Dimension repairs
still pass. The remaining 89 failure records are unchanged: 78 Hatch accuracy
exceedances and 11 Hatch gaps. The full drawing therefore remains blocked.

An independent check against the original source proves bit-exact ordered
coordinates, bulges, elevation and normal through geometry preparation both
ways for all 154 records, with zero point residual. Production IFCX/CBOR readback,
ASCII/binary DXF and AC1032 DWG exchange, Model/Paper/blocks and negative/tilted
planes are covered by focused regressions. Newly collapsed nonzero bulged chords
and amplified nested point errors still fail under both semantic loss policies.
The full workspace gate passes (974 tests, one existing ignored performance test),
as do seven optional Zstd tests. Current tolerances and dependency configuration
are unchanged.

The complete report, 154-handle comparison, source parameter check, source/config
snapshots and gate logs are retained locally under the task-specific
`.superpowers/coincident-polyline/` directory. Source SHA256:
`74d8d7b861cda0e3859879c8149df2344a457f5c9cefce6a144a7cf52a4fbe9a`;
33,288,016 bytes, unchanged before and after the run.
No entity exclusions or controlled size/exchange measurement is part of this task.

Relevant design context: the tighter nondegenerate-arc proof in issue #12 and
placement research in #11 remain independent; neither is needed to retain a
source-coincident segment. The existing tolerance decision in #10 is unchanged.
