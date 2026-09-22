"""Generate independent, certified test enclosures; never used at runtime.

Python standard library only. All proof arithmetic is directed integer fixed
point with 4096 fractional bits. Machin's identity supplies an interval for pi;
Taylor polynomials with explicit remainders supply sin/cos after reduction.
No platform sine/cosine, fpmath, or rounded pi is used to compute references.
See docs/geometry/block-trigonometry.md for the derivation and limits.
"""

import argparse
import functools
import json
import math
import struct
import sys
from fractions import Fraction
from pathlib import Path

BITS = 4096
Q = 1 << BITS
ATAN_TERMS = 1024
TRIG_TERMS = 320


def ceil_div(a, b):
    assert b > 0
    return -((-a) // b)


def enclose(value):
    scaled = value * Q
    return (scaled.numerator // scaled.denominator,
            ceil_div(scaled.numerator, scaled.denominator))


def add(a, b):
    return a[0] + b[0], a[1] + b[1]


def neg(a):
    return -a[1], -a[0]


def mul(a, b):
    products = [x * y for x in a for y in b]
    return min(products) // Q, ceil_div(max(products), Q)


def divide(a, denominator):
    return a[0] // denominator, ceil_div(a[1], denominator)


def scale(a, factor):
    products = (a[0] * factor, a[1] * factor)
    return min(products), max(products)


def atan_inverse(denominator):
    """Alternating atan(1/d) series; bound remainder by the next term."""
    assert denominator > 1 and ATAN_TERMS % 2 == 0
    total = (0, 0)
    power = denominator
    for k in range(ATAN_TERMS):
        divisor = (2 * k + 1) * power
        term = (Q // divisor, ceil_div(Q, divisor))
        total = add(total, term if k % 2 == 0 else neg(term))
        power *= denominator * denominator
    # Even term count: the next term is positive, as is the remaining tail.
    return total[0], total[1] + ceil_div(Q, (2 * ATAN_TERMS + 1) * power)


@functools.cache
def pi_interval():
    return add(scale(atan_inverse(5), 16), scale(atan_inverse(239), -4))


def sin_cos(x):
    assert math.isfinite(x)
    if x == 0:
        return (0, 0), (Q, Q)
    if x < 0:
        sine, cosine = sin_cos(-x)
        return neg(sine), cosine
    point = enclose(Fraction.from_float(x))
    assert point[0] == point[1]  # Every binary64 input is exact on this grid.
    pi = pi_interval()
    pi_mid = (pi[0] + pi[1]) // 2
    quadrant = (4 * point[0] + pi_mid) // (2 * pi_mid)
    r = add(point, neg(divide(scale(pi, quadrant), 2)))
    assert -Q <= r[0] <= r[1] <= Q
    r2 = mul(r, r)
    sine_term = sine = r
    cosine_term = cosine = (Q, Q)
    for k in range(1, TRIG_TERMS):
        sine_term = neg(divide(mul(sine_term, r2), (2 * k) * (2 * k + 1)))
        cosine_term = neg(divide(mul(cosine_term, r2), (2 * k - 1) * (2 * k)))
        sine = add(sine, sine_term)
        cosine = add(cosine, cosine_term)
    # Lagrange remainders for degrees 639 and 638 on |r| <= 1:
    # 1/640! and 1/639! are both smaller than one fixed-point quantum.
    assert math.factorial(2 * TRIG_TERMS - 1) > Q
    sine = (max(-Q, sine[0] - 1), min(Q, sine[1] + 1))
    cosine = (max(-Q, cosine[0] - 1), min(Q, cosine[1] + 1))
    return [(sine, cosine), (cosine, neg(sine)),
            (neg(sine), neg(cosine)), (neg(cosine), sine)][quadrant % 4]


def to_bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def from_bits(value):
    return struct.unpack('>d', struct.pack('>Q', value))[0]


def outward_bits(interval):
    exact_low, exact_high = (Fraction(n, Q) for n in interval)
    lower, upper = float(exact_low), float(exact_high)
    # Float conversion supplies candidates only; rational comparisons certify.
    while Fraction.from_float(lower) > exact_low:
        lower = math.nextafter(lower, -math.inf)
    while Fraction.from_float(upper) < exact_high:
        upper = math.nextafter(upper, math.inf)
    assert Fraction.from_float(lower) <= exact_low <= exact_high <= Fraction.from_float(upper)
    return [f'{to_bits(lower):016x}', f'{to_bits(upper):016x}']


def cases():
    values = {0, 1, 2, 3, 0x000fffffffffffff, 0x0010000000000000,
              0x0010000000000001, 0x7fefffffffffffff, 0x7feffffffffffffe}
    # Neighbors of small-angle, quadrant and backend reduction boundaries.
    anchors = [0.5, 0.7, 1., float.fromhex('0x1.921fb54442d18p-1'),
               float.fromhex('0x1.921fb54442d18p+0'),
               float.fromhex('0x1.921fb54442d18p+1'),
               ((1 << 20) - 1) * float.fromhex('0x1.921fb54442d18p+0')]
    for anchor in anchors:
        values.update(to_bits(anchor) + delta for delta in range(-2, 3))
    for exponent in [-1022, -512, -26, -1, 0, 1, 20, 48, 100, 512, 1000, 1023]:
        values.add(to_bits(math.ldexp(1.0, exponent)))
    seed = 0x985a9231a0046a3d
    for _ in range(48):
        seed = (seed * 6364136223846793005 + 1) & ((1 << 64) - 1)
        values.add(seed & 0x7fefffffffffffff)
    values |= {value | (1 << 63) for value in values}
    for bits in sorted(values):
        sine, cosine = sin_cos(from_bits(bits))
        yield {'angle_bits': f'{bits:016x}',
               'sin': outward_bits(sine), 'cos': outward_bits(cosine)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path,
                        default=Path('tests/data/block-trig-reference.json'))
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    assert sys.float_info.mant_dig == 53 and sys.float_info.max_exp == 1024
    document = {'oracle': 'integer-interval-machin-taylor-v1',
                'fractional_bits': BITS, 'atan_terms': ATAN_TERMS,
                'trig_terms': TRIG_TERMS, 'cases': list(cases())}
    encoded = json.dumps(document, indent=2) + '\n'
    if args.check:
        assert args.output.read_text(encoding='utf-8') == encoded, 'Reference data differ'
    else:
        args.output.write_text(encoded, encoding='utf-8', newline='\n')
    print(f"{'Verified' if args.check else 'Generated'} {len(document['cases'])} certified cases")


if __name__ == '__main__':
    main()
