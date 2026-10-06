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
