use ifccad::dimension::*;
use serde_json::{json, Value};
pub(crate) fn label(value: DimensionFormattedLabel) -> Value {
    let fragments: Vec<_> = value.fragments.into_iter().map(|part| match part {
        DimensionFormattedFragment::Run(run) => json!({"kind":"run","run":run}),
        DimensionFormattedFragment::ArcSymbol { placement } => json!({"kind":"arcSymbol","placement":placement}),
        DimensionFormattedFragment::Fraction { numerator, denominator, layout, text_scale } => json!({"kind":"fraction","numerator":numerator,"denominator":denominator,"layout":layout,"textScale":text_scale}),
    }).collect();
    json!({"state":match value.state {DimensionFormattedLabelState::Visible=>"visible",DimensionFormattedLabelState::Suppressed=>"suppressed"},"runs":value.runs,"primary":value.primary,"alternate":value.alternate,"fragments":fragments})
}
