# Native block transform foundation

`BlockTransform` retains a validated `PlanePlacement`, a separate finite angle
in radians and three finite, nonzero signed scale factors. It does not retain an
affine matrix or replace its inputs with a normalized representation.
`Scale3::is_uniform` compares the three signed values exactly; a mixed-sign
reflection is not uniform. Subnormal nonzero scale factors are valid.

For a definition point `p`, definition base point `B`, insertion origin `O`,
stored placement directions `U,V`, angle `theta` and scale `sx,sy,sz`:

```text
d = p - B
N = U cross V
Xr = cos(theta)*U + sin(theta)*V
Yr = -sin(theta)*U + cos(theta)*V
owningScopePoint = O + dx*sx*Xr + dy*sy*Yr + dz*sz*N
```

The cross product is not renormalized. A local polyline placement is applied
before this calculation. Nested inserts evaluate inner-to-outer, retaining
shear created by composition instead of decomposing it back into one placement.
Neither insertion units nor paper/plot settings change this calculation.

## Construction versus validity

`BlockTransform::try_new` accepts any valid plane placement unchanged.
`BlockTransform::from_normal` is an optional authoring convenience: robustly
normalize the finite nonzero normal, then use world-Y cross normal when both
normalized X and Y components have absolute value strictly less than `1/64`,
otherwise world-Z cross normal. Normalize the resulting X direction; Y is normal
cross X. Rotation remains a separate supplied value. This neutral arbitrary-axis
construction is not a universal file-format requirement.

The public constructor and raw backing components use shared validity predicates.
Invalid placement, nonfinite rotation, nonfinite scale and zero scale are distinct
errors. The normal-based constructor also distinguishes an invalid normal.

## Evaluation and implementation status

The private `PreparedBlockTransform` caches outward interval columns and applies
them to points or enclosed points. It is a helper for shared reader/writer
bounds assessment, not a second public model or a stored property. Exactly zero
rotation, identity and equal-point base subtraction retain exact shortcuts.
Other finite angles use the qualified [fixed trigonometric
enclosures](block-trigonometry.md). Arithmetic rounds outward; an intermediate
overflow or an unavailable finite enclosure returns no proof, not a conclusion
that the geometry or transform is invalid. No positional epsilon is introduced.

The active drawing contract is IFCDR 0.9 with IFCX overlay 0.11. Typed scopes,
definition rows, instances, iterative graph validation and evaluated leaf bounds
share the reader/writer logical contract. Definitions stay resource-local;
IFCX model/paper layouts select scopes but do not own block definitions.
Ordinary CAD block conversion and its dependency limits are described in the
[codec boundary qualification](block-cad-boundary.md). Presentation, viewports,
plot settings, arrays and external references remain separate future work.
