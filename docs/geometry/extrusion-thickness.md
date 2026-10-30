# Primitive extrusion thickness (development profile 0.3.0)

Point, LineSegment, Circle, Arc and PlanarPolyline store finite signed `thickness`
in coordinate units, in both ordinary entities and saved Dimension graphics.
SOLID/TRACE and Text retain their existing thickness contracts. Positive zero is
the intrinsic default; native JSON/CBOR omit only exact +0 and retain explicit -0.
Null, nonfinite values and unknown fields are rejected.

For a placed primitive the sweep is `B + s*(X cross Y)*thickness`, for s in [0,1].
Stored axes are not renormalized by native IO. LineSegment retains scope-space
XYZ endpoints and has `extrusionDirection`, default [0,0,1], with finite components
and squared length within the existing exact binary64 1e-12 unit tolerance. The
direction is validated even for zero thickness and need not be perpendicular to
the chord. Native IO does not normalize, clamp or approximate the authored values.

Polyline full width remains an independent in-plane cross-section; lineweight
remains a presentation stroke in millimetres. Thickness does not replace either.
Native Constant, PerVertex and Defaulted width modes retain all existing authored
presence, inheritance, explicit-zero and dormant outgoing-segment values.

`geometry_centerline_bounds` continues to supply the base curves for selectors,
Hatch sources and Paper clips. `geometry_bounds` and scope display assessments
include the selected primitive sweeps. Base and top bounds use outward arithmetic,
including complete curves and proven wide bands. Existing incomplete join/pattern/
glyph evidence stays incomplete. Filled quadrilaterals already include their
intrinsic thickness and are never swept twice. Nested/mirrored/nonuniform block
transforms act on complete local enclosures; transformed normals are not normalized.
Bounds recomputation remains explicit and atomic, including overflow failure.

The format transition is profile 0.2.0 -> 0.3.0 and CBOR mapping 2 -> 3. Zstd
transport remains 1 with inner mapping 3. Published release 2026.44.1 and its
profile 0.2.0/CBOR2 artifacts are unchanged. This development source is not a new
published release; no compatibility reader or migration is promised.

## Active conversion development pair

The isolated Document changes are based on
`e6135a72ec46046cd23a84df1b13c8b4fe9541f2`, with codec
`75dd8f5cd3b5dd2c56d9c9d4e8c14b816d31d68d` unchanged. The local patch and dependency
selection must be recorded with review results; do not describe it as unmodified
upstream. The core has no CAD runtime dependency. The published Explorer still
uses its recorded profile 0.2.0 reader until separately authorized integration.

POINT locations and LINE endpoints remain owning-scope/WCS coordinates. POINT's
frame carries its normal and marker angle; its origin is not transformed as OCS.
Circle/Arc/Polyline retain their existing OCS frame handling. Thickness and normal
are set on CAD output. Saved LINE graphics rotate the extrusion direction as a
vector, never translate it as a point. Native exactness and CAD normal/parameter
rounding are distinct; the review regression's numerical budget is not a runtime
error certificate. CAD signed zero may normalize to +0.

Ordinary unfitted classic 2D POLYLINE maps to PlanarPolyline, with elevation,
bulges, closure, pattern generation, thickness and **effective vertex widths**.
DXF reader widths may already have been resolved from defaults/overrides. DWG
numeric vertex widths are inspected separately; numeric zero is never guessed to
mean inheritance. The converter reports that original default/override presence
is unavailable or unassessed in the codec model, and that return uses equivalent
LWPOLYLINE. Native Defaulted export reports materialization while retaining the
effective pairs. Neither report certifies source dependency preservation. Fitted,
mesh and unqualified vertex details remain diagnosed omissions. Source classic
members inside saved picture graphs are not separately qualified by this slice.

Ellipse/EllipseArc, MText and Hatch plane normals do not imply a thickness field.
The shared DXF POLYLINE header lists group 39; that alone does not establish
effective extrusion semantics for the 3D variant. No new spatial-path extrusion,
annotation scales, automatic updates or renderer/undo repair is introduced.

## Review boundary

The compact gallery exercises positive/negative and oblique sweeps, width modes,
existing SOLID/Text controls and stored Dimension graphics. One literal classic
DXF case distinguishes inherited widths and explicit zero; physical classic DWG
is tested independently. Both return formats use actual AC1032 production
writers/readers and strict native JSON/CBOR readback, comparing semantics instead
of handles or counts alone. These are correctness checks, not size/timing research.

No practice drawing with a distinct nonzero-thickness case has been selected.
The supplied files qualify this synthetic gallery and classic control only, not
the practice corpus or every CAD version. The user has accepted these control
files and the local Explorer integration with matching browser, host and parser
reader. The existing catalogue samples use 15 mm thickness with base Z=0; the
separate gallery remains local review material. A bounded LINE endpoint/grip and
undo/redo smoke checked dependent extrusion graphics, without claiming general
host rendering or editing fidelity. Circle/Arc display remains tracked in
[OCS #2361](https://github.com/HakanSeven12/OpenCADStudio/issues/2361).
Fixed dependency revisions, the Document PR and publication remain pending.
