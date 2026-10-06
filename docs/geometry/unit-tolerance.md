# Physical tolerance and symbolic units

OCDraw and IFCCAD coordinate units use the same 25 symbolic values. Their
physical definitions are in the
[OCDraw logical contract](../../schemas/ocdraw/logical-contract-0.1.0.md).
The converter maps CAD codes 0–24 explicitly; core enum discriminants have no
CAD meaning. Unknown CAD codes retain the existing unsupported-unit diagnostic.
Neither the unit mapping nor block insertion metadata rescales stored points.

Both converters use the shared resolver in `cad-geometry-convert`. IFCCAD Model and
definition-local checks use drawing units; each retained Paper layout resolves
its own coordinate unit. Physical media does not infer a missing unit and
reports never compare a global numeric maximum across unlike units. The exact
registry/parsec assumptions below are unchanged by this extraction. See
[shared geometry](shared-geometry.md) for public options and evidence.

Physical tolerance defaults to exactly one micrometre, except unitless drawings
default to exact geometry. An explicitly requested physical tolerance on unitless
data is rejected, including zero. Exact and drawing-coordinate tolerances do not
need a physical unit. Finite supplied tolerances retain their exact binary64
values; all rational unit factors are constructed as integer fractions.

Parsec uses the exact definition `K/pi` metres, `K=648000*149597870700`. A metre
tolerance `t` therefore becomes `[t*piLower/K, t*piUpper/K]` coordinate units.
The fixed pi endpoints are binary64 bit patterns `400921fb54442d18` and
`400921fb54442d19`. The unit test certifies them independently with Machin's
identity, `pi=16*atan(1/5)-4*atan(1/239)`: 32 exact rational alternating-series
terms plus each next term produce strict bounds inside these endpoints. There
is no runtime precision refinement or rounded decimal parsec conversion factor.

Compare exact squared deviation with the squared lower and upper tolerance:
at or below lower is accepted; strictly above upper proves exceedance; the
remaining uncertainty is NumericalProofIncomplete. Both conversion directions
block this last case as an accuracy failure, regardless of semantic loss policy.
The outward floating values in reports never replace these rational decisions.

CAD MEASUREMENT is separate pattern/linetype metadata, not another unit code.
When constructing a CAD document the adapter selects metric for SI/submetric
length units, imperial for international inch/foot-derived and US survey units.
For unitless and astronomical units it leaves the document's existing setting
unchanged. This is a construction convention, not an implicit physical factor.
