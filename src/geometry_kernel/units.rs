/// Coordinate length unit declared by a drawing coordinate domain.
///
/// Coordinates use this declared coordinate length unit; no conversion is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateLengthUnit {
    Unitless,
    Millimetre,
    Centimetre,
    Metre,
    Kilometre,
    Inch,
    Foot,
    Mile,
    Microinch,
    Mil,
    Yard,
    Angstrom,
    Nanometre,
    Micrometre,
    Decimetre,
    Decametre,
    Hectometre,
    Gigametre,
    AstronomicalUnit,
    LightYear,
    Parsec,
    UsSurveyFoot,
    UsSurveyInch,
    UsSurveyYard,
    UsSurveyMile,
}

impl CoordinateLengthUnit {
    /// Inclusive rational scale interval for one metre in this coordinate unit.
    /// Unitless coordinates have no physical scale. Parsec retains the existing
    /// outward interval for pi rather than substituting a rounded exact factor.
    pub fn coordinates_per_metre(
        self,
    ) -> Option<(num_rational::BigRational, num_rational::BigRational)> {
        use num_rational::BigRational;
        use CoordinateLengthUnit::*;
        let q = |n: i64, d: i64| BigRational::new(n.into(), d.into());
        if self == Parsec {
            let k = q(648000, 1) * q(149597870700, 1);
            let lower = BigRational::from_float(f64::from_bits(0x400921fb54442d18))
                .expect("finite pi bound");
            let upper = BigRational::from_float(f64::from_bits(0x400921fb54442d19))
                .expect("finite pi bound");
            return Some((&lower / &k, upper / k));
        }
        let factor = match self {
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
        let scale = factor.recip();
        Some((scale.clone(), scale))
    }
    /// Language-neutral coordinate-unit token used by drawing contracts.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unitless => "unitless",
            Self::Millimetre => "mm",
            Self::Centimetre => "cm",
            Self::Metre => "m",
            Self::Kilometre => "km",
            Self::Inch => "in",
            Self::Foot => "ft",
            Self::Mile => "mi",
            Self::Microinch => "microin",
            Self::Mil => "mil",
            Self::Yard => "yd",
            Self::Angstrom => "angstrom",
            Self::Nanometre => "nm",
            Self::Micrometre => "um",
            Self::Decimetre => "dm",
            Self::Decametre => "dam",
            Self::Hectometre => "hm",
            Self::Gigametre => "Gm",
            Self::AstronomicalUnit => "au",
            Self::LightYear => "ly",
            Self::Parsec => "pc",
            Self::UsSurveyFoot => "usSurveyFoot",
            Self::UsSurveyInch => "usSurveyInch",
            Self::UsSurveyYard => "usSurveyYard",
            Self::UsSurveyMile => "usSurveyMile",
        }
    }
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "unitless" => Self::Unitless,
            "mm" => Self::Millimetre,
            "cm" => Self::Centimetre,
            "m" => Self::Metre,
            "km" => Self::Kilometre,
            "in" => Self::Inch,
            "ft" => Self::Foot,
            "mi" => Self::Mile,
            "microin" => Self::Microinch,
            "mil" => Self::Mil,
            "yd" => Self::Yard,
            "angstrom" => Self::Angstrom,
            "nm" => Self::Nanometre,
            "um" => Self::Micrometre,
            "dm" => Self::Decimetre,
            "dam" => Self::Decametre,
            "hm" => Self::Hectometre,
            "Gm" => Self::Gigametre,
            "au" => Self::AstronomicalUnit,
            "ly" => Self::LightYear,
            "pc" => Self::Parsec,
            "usSurveyFoot" => Self::UsSurveyFoot,
            "usSurveyInch" => Self::UsSurveyInch,
            "usSurveyYard" => Self::UsSurveyYard,
            "usSurveyMile" => Self::UsSurveyMile,
            _ => return None,
        })
    }
}
