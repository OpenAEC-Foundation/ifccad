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
    },
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
            Tolerance::Custom { value, unit } => match unit.as_str() {
                "mm" => OcdrawGeometryTolerance::millimetres(*value),
                "m" => OcdrawGeometryTolerance::metres(*value),
                "drawing" => OcdrawGeometryTolerance::drawing_units(*value),
                _ => return Err("Unknown tolerance unit".into()),
            }
            .map_err(|e| e.to_string()),
        }
    }
    pub(crate) fn value(&self) -> Value {
        json!({"tolerance":match &self.tolerance{Tolerance::Default=>json!({"mode":"default"}),Tolerance::Exact=>json!({"mode":"exact"}),Tolerance::Custom{value,unit}=>json!({"mode":"custom","value":value,"unit":unit})}})
    }
}
pub(crate) fn geometry(value: &OcdrawGeometryAssessment) -> Value {
    let interval = value.resolved_tolerance();
    json!({"status":format!("{:?}",value.status()),"unit":format!("{:?}",value.drawing_unit()),"resolvedTolerance":{"lower":interval.lower(),"upper":interval.upper()},"maxDeviationUpperBound":value.max_deviation_upper_bound(),"assessedEntities":value.assessed_entities(),"assessedVertices":value.assessed_vertices(),"roundedEntities":value.rounded_entities(),"worstEntity":value.worst_entity().map(|e|format!("{e:?}"))})
}
pub(crate) fn invalid(name: &str, format: &str, code: &str, message: impl ToString) -> Value {
    let mut output = crate::result(std::path::Path::new(name), format);
    crate::fail(&mut output, "converting", code, message);
    output
}
pub(crate) fn geometry_failure(f: &ocdraw_convert::OcdrawGeometryFailure) -> Value {
    let interval =
        |i: ocdraw_convert::OcdrawDistanceInterval| json!({"lower":i.lower(),"upper":i.upper()});
    json!({"source":format!("{:?}",f.source),"vertexIndex":f.vertex_index,"stage":format!("{:?}",f.stage),"requestedTolerance":format!("{:?}",f.requested_tolerance),"resolvedTolerance":f.resolved_tolerance.map(interval),"deviation":f.deviation.map(interval),"reason":format!("{:?}",f.reason)})
}
