# Fixed binary64 trigonometric enclosures

## Scope and qualification

The scope/block implementation uses a fixed binary64 sine/cosine backend with
an explicit error contract. It does not redefine geometric sine/cosine as a
platform's rounded answer, and it does not progressively increase runtime
precision. This document records the qualification boundary, not completed
block support or a formal verification of a third-party math library.

Selected backend: **fpmath 0.1.1**, pinned exactly. Its published crate archive
has SHA-256 `f617488244e3bd5aa86da93efaf4072579a301832ca5a9c7e3493563b509a43d`;
its VCS metadata identifies revision
`45208b9e92b80e479a2ca5483552238394f0e203` of
[rust-fpmath](https://github.com/eduardosm/rust-fpmath/tree/45208b9e92b80e479a2ca5483552238394f0e203).
The package is MIT OR Apache-2.0, MSRV 1.70. Its large argument reducer retains
the original Sun Microsystems permissive notice. The crate source is unmodified.

The [public API](https://docs.rs/fpmath/0.1.1/fpmath/) specifies an error below
one ULP for both sin and cos; sin_cos explicitly inherits those accuracy and
special-case contracts. Qualification relies on that documented dependency
contract, source inspection and independent regression evidence. Sampling does
not prove the contract for all binary64 inputs. If the dependency contract is
withdrawn or a counterexample is found, the enclosure guarantee must be revisited;
silently increasing an empirical margin is not a remedy.

The inspected implementation has these relevant properties:

- Zero/subnormal inputs return `(x, 1)`; only zero is an exact mathematical
  shortcut. Small nonzero inputs must still receive an enclosure.
- Arguments up to the stored pi/4 threshold skip reduction. Medium arguments
  use split pi/2 constants with extra cancellation-sensitive terms, explicitly
  assuming round-to-nearest.
- Larger finite arguments use an integer/fixed-point, 24-bit-chunk reduction
  based on the musl/Sun reducer, followed by a high/low residual. The table has
  66 chunks of 2/pi. The f64 reduction initially uses five terms and can extend
  on cancellation. The wrapper passes the original angle unchanged; a prior
  modulo operation using rounded 2*pi would destroy the contract.
- Sine and cosine use different polynomial kernels on the reduced interval,
  including low-part corrections. Their approximation/reduction error is covered
  by the published whole-function guarantee, not independently re-proven here.
- Native f64 evaluation has an x87 excess-precision workaround. The reference
  assumptions are IEEE binary64 arithmetic, round-to-nearest/ties-to-even,
  gradual underflow and no unsafe reassociation/fast-math transformations.
  Altering the floating-point environment is outside these assumptions, just as
  it is for the existing outward-arithmetic kernel.

The inspected upstream reference metric uses the true result's binary exponent
with a subnormal floor, not merely the spacing immediately above the computed
answer. Its MPFR-based tests require errors below 0.9 of that unit. The published
one-ULP bound, not the observed maximum or a stronger 0.9 assumption, is used here.

## From the backend guarantee to an interval

Let r be the mathematical result and y the computed binary64 value. For normal
results in a binade with spacing d, the contract gives `abs(y-r) < d`. Across a
power-of-two boundary the spacing doubles. A one-neighbor envelope around y is
therefore not a generally sufficient translation: y can be immediately below
the boundary while r is just above it, within one ULP measured on r's side.

Use the second representable neighbor below and above y. Away from a binade
boundary these cover two spacings. When crossing a boundary they include both
the smaller-side gap and the larger-side gap. The strict one-ULP error cannot
span more than those two adjacent gaps. Negative values follow by reflection;
at zero/subnormal values the constant minimum spacing is no smaller than the
error unit. This is a derived envelope, not an empirically chosen padding count.

Intersect the resulting interval with `[-1,1]`, the exact sine/cosine range.
Return exact `(0,1)` intervals for either signed zero angle. Reject nonfinite
angles as unassessable by this helper; semantic rotation validation diagnoses
them separately. An unexpected nonfinite/out-of-range backend result must not
produce a fabricated certificate.

No writer adds another ULP or tolerance to these enclosures. Later geometry
operations use the shared outward interval arithmetic and store their resulting
endpoints. Existing exact rational decisions remain available where no
transcendental operation is involved.

## Independent reference generator

`scripts/block_trig_reference.py` uses only Python's standard library and
arbitrary-sized integers. It is **test tooling**, not a runtime evaluator or a
new production precision-escalation path. The checked-in result is
`tests/data/block-trig-reference.json`; all input and endpoint values are exact
binary64 bit strings, not ambiguous decimal text.

The oracle uses fixed-point intervals `[L/Q,U/Q]`, `Q=2^4096`:

1. Integer addition and negation are exact. Multiplication evaluates all endpoint
   products and floors/ceils after division by Q. Division by a positive integer
   also floors/ceils. These operations enclose their real counterparts, including
   negative values and underflow far below binary64.
2. Machin's identity `pi=16*atan(1/5)-4*atan(1/239)` supplies pi. Each atan uses
   1024 terms of its convergent alternating power series; each term is enclosed
   by directed integer division and the omitted tail by the next term.
3. Every finite binary64 input is exactly representable on this grid. An integer
   multiple of pi/2 is selected using the midpoint only as a candidate. Subtract
   that multiple of the **entire pi interval** from the exact input. The resulting
   interval must pass `-1 <= r <= 1`; this explicit check, not the midpoint,
   justifies the subsequent Taylor domain and holds even at f64::MAX.
4. Evaluate 320 terms of sine and cosine via outward interval recurrence.
   Their polynomial degrees are 639 and 638. On this domain all real derivatives
   have magnitude at most one, so Lagrange remainders are at most `1/640!` and
   `1/639!`. Both are smaller than `1/Q`, checked by exact integer comparison.
   Add one fixed-point quantum on either side before the quadrant permutation.
5. Convert endpoint rationals to binary64 candidates and adjust outward with
   nextafter until **exact rational comparisons** certify enclosure. No platform
   sin/cos or platform pi constant is used to calculate a reference result.

The 202-case corpus covers signed zero, minimum subnormals, the normal/subnormal
boundary, small angles, power-of-two cases, neighbors of quadrant/reduction
thresholds, huge finite angles and f64::MAX, and seeded bit-pattern samples with
both signs. Input selection may use rounded constants; reference evaluation
always interprets the selected bits exactly.

Reproduction from the repository root (Python 3.10+ and IEEE binary64 floats):

```text
python scripts/test_block_trig_reference.py
python scripts/block_trig_reference.py --check
cargo test -p ifccad geometry::trig
cargo test --release -p ifccad geometry::trig
```

Omit `--check` only to regenerate the fixture deliberately. The generator tests
exercise signed directed arithmetic against Fraction, certify pi between two
adjacent known floats, distinguish tiny nonzero sine/cosine from identity, and
check signed symmetry, quadrants and large-angle residual bounds.

Initial qualification used Python 3.14 and Rust 1.98.0 on
`x86_64-pc-windows-msvc`. A standalone local probe of the checksum-verified source
enclosed all 202 oracle cases before adding the dependency to the core crate.
Native arithmetic on other targets must satisfy the assumptions above and run
the same regression corpus; this record is not evidence of executed tests on
every platform. Normal production builds require neither Python nor MPFR.
