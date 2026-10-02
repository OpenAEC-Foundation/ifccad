pub(crate) const UNITS: [&str; 25] = [
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
pub(crate) fn unit_code(unit: &str) -> i16 {
    UNITS
        .iter()
        .position(|u| *u == unit)
        .expect("validated unit") as i16
}
