use num_rational::BigRational;
use ocdraw::geometry_kernel::CoordinateLengthUnit as DrawingLengthUnit;

pub const UNIT_TOKENS: [&str; 25] = [
    "unitless",
    "in",
    "ft",
    "mi",
    "mm",
    "cm",
    "m",
    "km",
    "microin",
    "mil",
    "yd",
    "angstrom",
    "nm",
    "um",
    "dm",
    "dam",
    "hm",
    "Gm",
    "au",
    "ly",
    "pc",
    "usSurveyFoot",
    "usSurveyInch",
    "usSurveyYard",
    "usSurveyMile",
];

// CAD's integer codes belong to this adapter, not to core enum discriminants.
const CAD_UNITS: [DrawingLengthUnit; 25] = [
    DrawingLengthUnit::Unitless,
    DrawingLengthUnit::Inch,
    DrawingLengthUnit::Foot,
    DrawingLengthUnit::Mile,
    DrawingLengthUnit::Millimetre,
    DrawingLengthUnit::Centimetre,
    DrawingLengthUnit::Metre,
    DrawingLengthUnit::Kilometre,
    DrawingLengthUnit::Microinch,
    DrawingLengthUnit::Mil,
    DrawingLengthUnit::Yard,
    DrawingLengthUnit::Angstrom,
    DrawingLengthUnit::Nanometre,
    DrawingLengthUnit::Micrometre,
    DrawingLengthUnit::Decimetre,
    DrawingLengthUnit::Decametre,
    DrawingLengthUnit::Hectometre,
    DrawingLengthUnit::Gigametre,
    DrawingLengthUnit::AstronomicalUnit,
    DrawingLengthUnit::LightYear,
    DrawingLengthUnit::Parsec,
    DrawingLengthUnit::UsSurveyFoot,
    DrawingLengthUnit::UsSurveyInch,
    DrawingLengthUnit::UsSurveyYard,
    DrawingLengthUnit::UsSurveyMile,
];
pub fn from_cad_code(code: i16) -> Option<DrawingLengthUnit> {
    usize::try_from(code)
        .ok()
        .and_then(|code| CAD_UNITS.get(code).copied())
}
pub fn cad_code(unit: DrawingLengthUnit) -> i16 {
    CAD_UNITS
        .iter()
        .position(|u| *u == unit)
        .expect("complete unit domain") as i16
}
pub fn measurement(unit: DrawingLengthUnit) -> Option<i16> {
    use DrawingLengthUnit::*;
    match unit {
        Unitless | AstronomicalUnit | LightYear | Parsec => None,
        Inch | Foot | Mile | Microinch | Mil | Yard | UsSurveyFoot | UsSurveyInch
        | UsSurveyYard | UsSurveyMile => Some(0),
        Millimetre | Centimetre | Metre | Kilometre | Angstrom | Nanometre | Micrometre
        | Decimetre | Decametre | Hectometre | Gigametre => Some(1),
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTolerance {
    pub lower: BigRational,
    pub upper: BigRational,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToleranceVerdict {
    Within,
    Exceeds,
    Unresolved,
}
pub fn q(n: i64, d: i64) -> BigRational {
    BigRational::new(n.into(), d.into())
}
impl ResolvedTolerance {
    pub fn exact(value: BigRational) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }
    pub fn check_squared(&self, d2: &BigRational) -> ToleranceVerdict {
        if d2 <= &(&self.lower * &self.lower) {
            ToleranceVerdict::Within
        } else if d2 > &(&self.upper * &self.upper) {
            ToleranceVerdict::Exceeds
        } else {
            ToleranceVerdict::Unresolved
        }
    }
    pub fn from_metres(t: BigRational, unit: DrawingLengthUnit) -> Option<Self> {
        let (lower, upper) = unit.coordinates_per_metre()?;
        Some(Self {
            lower: &t * lower,
            upper: t * upper,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_rational::BigRational;
    #[test]
    fn uncertainty_is_not_proven_exceedance() {
        let tolerance = ResolvedTolerance {
            lower: BigRational::from_integer(2.into()),
            upper: BigRational::from_integer(3.into()),
        };
        for (square, expected) in [
            (4, ToleranceVerdict::Within),
            (5, ToleranceVerdict::Unresolved),
            (9, ToleranceVerdict::Unresolved),
            (10, ToleranceVerdict::Exceeds),
        ] {
            assert_eq!(
                tolerance.check_squared(&BigRational::from_integer(square.into())),
                expected
            );
        }
    }

    #[test]
    fn fixed_parsec_pi_interval_is_independently_certified_by_machin() {
        use num_bigint::BigInt;
        fn atan_interval(inverse: u32) -> (BigRational, BigRational) {
            let base = BigInt::from(inverse);
            let mut sum = q(0, 1);
            // 32 terms end with a negative term; the positive next term bounds
            // the entire alternating-series remainder. No floating pi is used.
            for k in 0..32_u32 {
                let term =
                    BigRational::new(1.into(), base.pow(2 * k + 1) * BigInt::from(2 * k + 1));
                if k % 2 == 0 {
                    sum += term;
                } else {
                    sum -= term;
                }
            }
            let upper = &sum + BigRational::new(1.into(), base.pow(65) * BigInt::from(65));
            (sum, upper)
        }
        let (a, b) = atan_interval(5);
        let (c, d) = atan_interval(239);
        let pi_lower = q(16, 1) * a - q(4, 1) * d;
        let pi_upper = q(16, 1) * b - q(4, 1) * c;
        let interval = ResolvedTolerance::from_metres(
            q(648000, 1) * q(149597870700, 1),
            DrawingLengthUnit::Parsec,
        )
        .unwrap();
        assert!(interval.lower < pi_lower && pi_lower < pi_upper && pi_upper < interval.upper);
    }

    #[test]
    fn exact_factors_and_cad_codes_cover_all_units() {
        use DrawingLengthUnit::*;
        let cases = [
            (Inch, 1, 127, 5000),
            (Foot, 2, 381, 1250),
            (Mile, 3, 201168, 125),
            (Millimetre, 4, 1, 1000),
            (Centimetre, 5, 1, 100),
            (Metre, 6, 1, 1),
            (Kilometre, 7, 1000, 1),
            (Microinch, 8, 127, 5000000000),
            (Mil, 9, 127, 5000000),
            (Yard, 10, 1143, 1250),
            (Angstrom, 11, 1, 10000000000),
            (Nanometre, 12, 1, 1000000000),
            (Micrometre, 13, 1, 1000000),
            (Decimetre, 14, 1, 10),
            (Decametre, 15, 10, 1),
            (Hectometre, 16, 100, 1),
            (Gigametre, 17, 1000000000, 1),
            (AstronomicalUnit, 18, 149597870700, 1),
            (LightYear, 19, 9460730472580800, 1),
            (UsSurveyFoot, 21, 1200, 3937),
            (UsSurveyInch, 22, 100, 3937),
            (UsSurveyYard, 23, 3600, 3937),
            (UsSurveyMile, 24, 6336000, 3937),
        ];
        for (unit, code, n, d) in cases {
            assert_eq!(from_cad_code(code), Some(unit));
            assert_eq!(cad_code(unit), code);
            assert_eq!(
                ResolvedTolerance::from_metres(q(n, d), unit),
                Some(ResolvedTolerance::exact(q(1, 1)))
            );
        }
        assert_eq!(from_cad_code(20), Some(Parsec));
        assert_eq!(cad_code(Parsec), 20);
        assert_eq!(from_cad_code(0), Some(Unitless));
        assert_eq!(cad_code(Unitless), 0);
        assert!(ResolvedTolerance::from_metres(q(0, 1), Unitless).is_none());
        for code in [-1, 25, i16::MAX] {
            assert!(from_cad_code(code).is_none());
        }
    }

    #[test]
    fn default_and_explicit_tolerances_keep_unitless_and_physical_policies_distinct() {
        use crate::GeometryTolerance as T;
        for unit in CAD_UNITS {
            assert_eq!(
                T::exact().resolve(unit).unwrap(),
                ResolvedTolerance::exact(q(0, 1))
            );
            assert_eq!(
                T::drawing_units(2.).unwrap().resolve(unit).unwrap(),
                ResolvedTolerance::exact(q(2, 1))
            );
            if unit == DrawingLengthUnit::Unitless {
                assert!(T::metres(0.).unwrap().resolve(unit).is_err());
                assert!(T::millimetres(1.).unwrap().resolve(unit).is_err());
                assert_eq!(
                    T::default().resolve(unit).unwrap(),
                    ResolvedTolerance::exact(q(1, 1_000_000_000))
                );
            } else {
                assert_eq!(
                    T::default().resolve(unit).unwrap(),
                    ResolvedTolerance::exact(q(1, 1_000_000_000))
                );
                assert_eq!(
                    T::millimetres(1000.).unwrap().resolve(unit).unwrap(),
                    T::metres(1.).unwrap().resolve(unit).unwrap()
                );
            }
        }
    }
}
