use super::DimensionFractionLayout;
use crate::text::TextRun;

/// Derived label content, never authoritative stored drawing data.
#[derive(Clone, Debug, PartialEq)]
pub enum DimensionFormattedFragment {
    Run(TextRun),
    /// A nonzero reduced fraction. The formatter uses only stacked layouts here.
    Fraction {
        numerator: String,
        denominator: String,
        layout: DimensionFractionLayout,
        text_scale: f64,
    },
}
