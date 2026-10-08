use crate::{CadLineweightMapping, CadOpacityMapping, PresentationValueError};
use opencadcodec::{LineWeight, Transparency};

const STANDARD_WEIGHTS: [i16; 24] = [
    0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158, 200,
    211,
];

/// Retain byte-derived values exactly before applying CAD's upward transparency rounding.
pub fn opacity_to_cad(value: f64) -> Result<CadOpacityMapping, PresentationValueError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(PresentationValueError::InvalidOpacity);
    }
    let transparency = (0u8..=255)
        .find(|byte| 1.0 - f64::from(*byte) / 255.0 == value)
        .map(Transparency::Explicit)
        .unwrap_or_else(|| Transparency::from_percent(1.0 - value));
    let roundtrip = 1.0 - f64::from(transparency.alpha()) / 255.0;
    Ok(CadOpacityMapping {
        transparency,
        roundtrip,
        changed: roundtrip != value,
    })
}

/// Select a standard DWG table weight, with the lower weight winning interior ties.
pub fn lineweight_to_cad(value: f64) -> Result<CadLineweightMapping, PresentationValueError> {
    if !value.is_finite() || value < 0.0 {
        return Err(PresentationValueError::InvalidLineweight);
    }
    // Outside the table interval, the endpoint is unambiguously nearest even
    // when subtraction from an enormous finite value would collapse distances.
    let hundredths = if value >= 2.11 {
        211
    } else {
        STANDARD_WEIGHTS
            .into_iter()
            .min_by(|a, b| {
                (f64::from(*a) / 100.0 - value)
                    .abs()
                    .total_cmp(&(f64::from(*b) / 100.0 - value).abs())
                    .then_with(|| a.cmp(b))
            })
            .expect("standard weight table is nonempty")
    };
    let roundtrip_mm = f64::from(hundredths) / 100.0;
    Ok(CadLineweightMapping {
        weight: LineWeight::Value(hundredths),
        roundtrip_mm,
        changed: roundtrip_mm != value,
    })
}
