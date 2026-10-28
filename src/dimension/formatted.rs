use super::{DimensionArcSymbol, DimensionFractionLayout};
use crate::text::TextRun;

/// Derived label content, never authoritative stored drawing data.
#[derive(Clone, Debug, PartialEq)]
pub enum DimensionFormattedFragment {
    Run(TextRun),
    /// Symbol attached to the following generated measurement, with authored
    /// placement retained. Plain text projections use a preceding arc glyph.
    ArcSymbol {
        placement: DimensionArcSymbol,
    },
    /// A nonzero reduced fraction. The formatter uses only stacked layouts here.
    Fraction {
        numerator: String,
        denominator: String,
        layout: DimensionFractionLayout,
        text_scale: f64,
    },
}
