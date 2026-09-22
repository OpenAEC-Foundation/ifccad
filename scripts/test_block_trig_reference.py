"""Tests for the independent integer-interval reference generator."""

import math
import unittest
from fractions import Fraction

import block_trig_reference as oracle


class ReferenceTests(unittest.TestCase):
    def test_signed_operations_enclose_exact_rationals(self):
        for a, b in [(Fraction(-7, 3), Fraction(5, 11)),
                     (Fraction(1, 1 << 1074), Fraction(1, 3))]:
            x, y = oracle.enclose(a), oracle.enclose(b)
            for interval, exact in [(oracle.add(x, y), a + b),
                                    (oracle.mul(x, y), a * b),
                                    (oracle.divide(x, 7), a / 7)]:
                self.assertLessEqual(Fraction(interval[0], oracle.Q), exact)
                self.assertGreaterEqual(Fraction(interval[1], oracle.Q), exact)

    def test_pi_is_certified_between_adjacent_binary64_constants(self):
        lower, upper = oracle.pi_interval()
        self.assertLess(Fraction.from_float(float.fromhex('0x1.921fb54442d18p+1')),
                        Fraction(lower, oracle.Q))
        self.assertGreater(Fraction.from_float(float.fromhex('0x1.921fb54442d19p+1')),
                           Fraction(upper, oracle.Q))

    def test_zero_is_exact_but_small_nonzero_angles_are_not(self):
        self.assertEqual(oracle.sin_cos(0.0), ((0, 0), (oracle.Q, oracle.Q)))
        x = math.ulp(0.0)
        sine, cosine = oracle.sin_cos(x)
        self.assertGreater(sine[0], 0)
        self.assertLess(Fraction(sine[1], oracle.Q), Fraction.from_float(x))
        self.assertLess(cosine[1], oracle.Q)
        self.assertGreater(cosine[0], 0)

    def test_quadrants_and_large_arguments_have_certified_finite_results(self):
        for x in [0.7, 2., 4., 6., float.fromhex('0x1.fffffffffffffp+1023')]:
            sine, cosine = oracle.sin_cos(x)
            negative_sine, negative_cosine = oracle.sin_cos(-x)
            self.assertEqual(negative_sine, oracle.neg(sine))
            self.assertEqual(negative_cosine, cosine)
            for value in [sine, cosine]:
                self.assertLessEqual(value[0], value[1])
                self.assertLess(value[1] - value[0], 1 << 1100)
                self.assertGreaterEqual(value[0], -oracle.Q)
                self.assertLessEqual(value[1], oracle.Q)


if __name__ == '__main__':
    unittest.main()
