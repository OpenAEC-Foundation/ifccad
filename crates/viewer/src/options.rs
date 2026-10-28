use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversionOptions {
    #[serde(default, rename = "partialOpen")]
    pub partial_open: bool,
    #[serde(default, rename = "lossPolicy")]
    pub loss_policy: ExportLossPolicy,
    #[serde(default, rename = "preserveSplines")]
    pub preserve_splines: bool,
    #[serde(default)]
    pub tolerance: Tolerance,
}
#[derive(Clone, Copy, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ExportLossPolicy {
    #[default]
    Allow,
    Reject,
}
#[derive(Default, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Tolerance {
    #[default]
    Default,
    Exact,
    Custom {
        value: f64,
        unit: String,
        #[serde(
            default,
            rename = "coordinateFallback",
            deserialize_with = "coordinate_fallback"
        )]
        coordinate_fallback: Option<f64>,
    },
}
fn coordinate_fallback<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    f64::deserialize(deserializer).map(Some)
}
impl ConversionOptions {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if let Tolerance::Custom {
            value,
            unit,
            coordinate_fallback,
        } = &value.tolerance
        {
            if !value.is_finite() || *value < 0.0 {
                return Err("Tolerance must be finite and nonnegative".into());
            }
            if !matches!(unit.as_str(), "mm" | "m" | "drawing") {
                return Err("Unknown tolerance unit".into());
            }
            if let Some(fallback) = coordinate_fallback {
                if !fallback.is_finite() || *fallback < 0.0 {
                    return Err("Coordinate fallback must be finite and nonnegative".into());
                }
                if unit == "drawing" {
                    return Err("A coordinate fallback requires a physical tolerance".into());
                }
            }
        }
        Ok(value)
    }
    pub(crate) fn value(&self) -> Value {
        let tolerance = match &self.tolerance {
            Tolerance::Default => json!({"mode":"default"}),
            Tolerance::Exact => json!({"mode":"exact"}),
            Tolerance::Custom {
                value,
                unit,
                coordinate_fallback,
            } => {
                let mut value = json!({"mode":"custom","value":value,"unit":unit});
                if let Some(fallback) = coordinate_fallback {
                    value["coordinateFallback"] = json!(fallback);
                }
                value
            }
        };
        json!({"partialOpen":self.partial_open,"preserveSplines":self.preserve_splines,"tolerance":tolerance,"lossPolicy":self.loss_policy})
    }
}

pub(crate) fn invalid(name: &str, format: &str, code: &str, message: impl ToString) -> Value {
    let mut output = crate::result(std::path::Path::new(name), format);
    crate::fail(&mut output, "converting", code, message);
    output
}
