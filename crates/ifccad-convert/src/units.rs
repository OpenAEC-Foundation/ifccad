use ifccad::ifcdr::IfcdrLengthUnit;
use num_rational::BigRational;

// CAD's integer codes belong to this adapter, not to core enum discriminants.
const CAD_UNITS: [IfcdrLengthUnit; 25] = [
    IfcdrLengthUnit::Unitless,
    IfcdrLengthUnit::Inch,
    IfcdrLengthUnit::Foot,
    IfcdrLengthUnit::Mile,
    IfcdrLengthUnit::Millimetre,
    IfcdrLengthUnit::Centimetre,
    IfcdrLengthUnit::Metre,
    IfcdrLengthUnit::Kilometre,
    IfcdrLengthUnit::Microinch,
    IfcdrLengthUnit::Mil,
    IfcdrLengthUnit::Yard,
    IfcdrLengthUnit::Angstrom,
    IfcdrLengthUnit::Nanometre,
    IfcdrLengthUnit::Micrometre,
    IfcdrLengthUnit::Decimetre,
    IfcdrLengthUnit::Decametre,
    IfcdrLengthUnit::Hectometre,
    IfcdrLengthUnit::Gigametre,
    IfcdrLengthUnit::AstronomicalUnit,
    IfcdrLengthUnit::LightYear,
    IfcdrLengthUnit::Parsec,
    IfcdrLengthUnit::UsSurveyFoot,
    IfcdrLengthUnit::UsSurveyInch,
    IfcdrLengthUnit::UsSurveyYard,
    IfcdrLengthUnit::UsSurveyMile,
];
pub(crate) fn from_cad_code(code: i16) -> Option<IfcdrLengthUnit> {
    usize::try_from(code)
        .ok()
        .and_then(|code| CAD_UNITS.get(code).copied())
}
pub(crate) fn cad_code(unit: IfcdrLengthUnit) -> i16 {
    CAD_UNITS
        .iter()
        .position(|u| *u == unit)
        .expect("complete unit domain") as i16
}
pub(crate) fn measurement(unit: IfcdrLengthUnit) -> Option<i16> {
    use IfcdrLengthUnit::*;
    match unit {
        Unitless | AstronomicalUnit | LightYear | Parsec => None,
        Inch | Foot | Mile | Microinch | Mil | Yard | UsSurveyFoot | UsSurveyInch
        | UsSurveyYard | UsSurveyMile => Some(0),
        Millimetre | Centimetre | Metre | Kilometre | Angstrom | Nanometre | Micrometre
        | Decimetre | Decametre | Hectometre | Gigametre => Some(1),
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedTolerance {
    pub lower: BigRational,
    pub upper: BigRational,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToleranceVerdict {
    Within,
    Exceeds,
    Unresolved,
}
pub(crate) fn q(n: i64, d: i64) -> BigRational {
    BigRational::new(n.into(), d.into())
}
impl ResolvedTolerance {
    pub(crate) fn exact(value: BigRational) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }
    pub(crate) fn check_squared(&self, d2: &BigRational) -> ToleranceVerdict {
        if d2 <= &(&self.lower * &self.lower) {
            ToleranceVerdict::Within
        } else if d2 > &(&self.upper * &self.upper) {
            ToleranceVerdict::Exceeds
        } else {
            ToleranceVerdict::Unresolved
        }
    }
    pub(crate) fn from_metres(t: BigRational, unit: IfcdrLengthUnit) -> Option<Self> {
        use IfcdrLengthUnit::*;
        if unit == Parsec {
            let k = q(648000, 1) * q(149597870700, 1);
            let lower = BigRational::from_float(f64::from_bits(0x400921fb54442d18)).unwrap();
            let upper = BigRational::from_float(f64::from_bits(0x400921fb54442d19)).unwrap();
            return Some(Self {
                lower: &t * lower / &k,
                upper: t * upper / k,
            });
        }
        let factor = match unit {
            Unitless => return None,
            Millimetre => q(1, 1000),
            Centimetre => q(1, 100),
            Metre => q(1, 1),
            Kilometre => q(1000, 1),
            Inch => q(127, 5000),
            Foot => q(381, 1250),
            Mile => q(201168, 125),
            Yard => q(1143, 1250),
            Microinch => q(127, 5000000000),
            Mil => q(127, 5000000),
            Angstrom => q(1, 10000000000),
            Nanometre => q(1, 1000000000),
            Micrometre => q(1, 1000000),
            Decimetre => q(1, 10),
            Decametre => q(10, 1),
            Hectometre => q(100, 1),
            Gigametre => q(1000000000, 1),
            AstronomicalUnit => q(149597870700, 1),
            LightYear => q(9460730472580800, 1),
            UsSurveyFoot => q(1200, 3937),
            UsSurveyInch => q(100, 3937),
            UsSurveyYard => q(3600, 3937),
            UsSurveyMile => q(6336000, 3937),
            Parsec => unreachable!(),
        };
        Some(Self::exact(t / factor))
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
            IfcdrLengthUnit::Parsec,
        )
        .unwrap();
        assert!(interval.lower < pi_lower && pi_lower < pi_upper && pi_upper < interval.upper);
    }

    #[test]
    fn exact_factors_and_cad_codes_cover_all_units() {
        use IfcdrLengthUnit::*;
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
        use crate::ConversionGeometryTolerance as T;
        for unit in CAD_UNITS {
            assert_eq!(
                T::exact().resolve(unit).unwrap(),
                ResolvedTolerance::exact(q(0, 1))
            );
            assert_eq!(
                T::drawing_units(2.).unwrap().resolve(unit).unwrap(),
                ResolvedTolerance::exact(q(2, 1))
            );
            if unit == IfcdrLengthUnit::Unitless {
                assert!(T::metres(0.).unwrap().resolve(unit).is_err());
                assert!(T::millimetres(1.).unwrap().resolve(unit).is_err());
                assert_eq!(
                    T::default().resolve(unit).unwrap(),
                    ResolvedTolerance::exact(q(0, 1))
                );
            } else {
                assert_eq!(
                    T::default().resolve(unit).unwrap(),
                    ResolvedTolerance::from_metres(q(1, 1000000), unit).unwrap()
                );
                assert_eq!(
                    T::millimetres(1000.).unwrap().resolve(unit).unwrap(),
                    T::metres(1.).unwrap().resolve(unit).unwrap()
                );
            }
        }
    }
}
