use ocdraw_convert::{OcdrawGeometryAssessment, OcdrawGeometryTolerance};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversionOptions {
    #[serde(default, rename = "preserveSplines")]
    pub preserve_splines: bool,
    #[serde(default)]
    pub tolerance: Tolerance,
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
        value.ocdraw_tolerance()?;
        Ok(value)
    }
    pub(crate) fn ocdraw_tolerance(&self) -> Result<OcdrawGeometryTolerance, String> {
        match &self.tolerance {
            Tolerance::Default => Ok(OcdrawGeometryTolerance::default()),
            Tolerance::Exact => Ok(OcdrawGeometryTolerance::exact()),
            Tolerance::Custom {
                value,
                unit,
                coordinate_fallback,
            } => {
                let tolerance = match unit.as_str() {
                    "mm" => OcdrawGeometryTolerance::millimetres(*value),
                    "m" => OcdrawGeometryTolerance::metres(*value),
                    "drawing" => OcdrawGeometryTolerance::drawing_units(*value),
                    _ => return Err("Unknown tolerance unit".into()),
                }
                .map_err(|e| e.to_string())?;
                match coordinate_fallback {
                    Some(value) => tolerance
                        .with_coordinate_fallback(*value)
                        .map_err(|e| e.to_string()),
                    None => Ok(tolerance),
                }
            }
        }
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
        json!({"preserveSplines":self.preserve_splines,"tolerance":tolerance})
    }
}
pub(crate) fn geometry(value: &OcdrawGeometryAssessment) -> Value {
    crate::ocdraw::geometry_report(value)
}
pub(crate) fn invalid(name: &str, format: &str, code: &str, message: impl ToString) -> Value {
    let mut output = crate::result(std::path::Path::new(name), format);
    crate::fail(&mut output, "converting", code, message);
    output
}
pub(crate) fn geometry_failure(f: &ocdraw_convert::OcdrawGeometryFailure) -> Value {
    let interval =
        |i: ocdraw_convert::OcdrawDistanceInterval| json!({"lower":i.lower(),"upper":i.upper()});
    json!({"domain":format!("{:?}",f.domain),"coordinateMeaning":crate::geometry::meaning(&f.coordinate_meaning),"source":format!("{:?}",f.source),"vertexIndex":f.vertex_index,"stage":format!("{:?}",f.stage),"requestedTolerance":format!("{:?}",f.requested_tolerance),"resolvedTolerance":f.resolved_tolerance.map(interval),"deviation":f.deviation.map(interval),"reason":format!("{:?}",f.reason)})
}

pub(crate) fn paper_tolerance_failure(error: &ocdraw_convert::OcdrawPaperToleranceError) -> Value {
    json!({"domain":{"kind":"PaperLayout","layoutId":error.layout_id.to_string()},"reason":format!("{:?}",error.reason)})
}
