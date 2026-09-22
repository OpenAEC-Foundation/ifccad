# Physical tolerance and symbolic units

IFCDR resource units and definition insertion units use the same 25 symbolic
values. Their exact physical definitions are in the
[logical contract](../../schemas/ifcdr/logical-contract-0.9.0.md#unit-meaning).
The converter maps CAD codes 0–24 explicitly; core enum discriminants have no
CAD meaning. Unknown CAD codes retain the existing unsupported-unit diagnostic.
Neither the unit mapping nor block insertion metadata rescales stored points.

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
