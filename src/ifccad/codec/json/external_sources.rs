use super::*;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceValue {
    pub name: String,
    pub location: IfccadExternalSourceLocation,
    pub source: IfccadExternalSourceContent,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct XrefValue {
    source: String,
    scale: f64,
}

pub(super) fn encode_source(source: &IfccadExternalSource) -> Value {
    json!({"name":source.name,"location":source.location,"source":source.source})
}

pub(super) fn decode_xref(
    attrs: &serde_json::Map<String, Value>,
    prefix: &str,
    path: &str,
) -> Result<IfccadEntityKind, IfccadReport> {
    let value: XrefValue = serde_json::from_value(attrs["ifccad::xref"].clone())
        .map_err(|e| super::validate::problem(format!("{path}: {e}")))?;
    let source_id = IfccadExternalSourceId(super::validate::numbered(
        &value.source,
        &format!("{prefix}/externalSource/"),
    )?);
    let placement = attrs
        .get("ifccad::geom::placement")
        .ok_or_else(|| super::validate::problem(format!("{path} requires XREF placement")))?;
    let placement = super::geometry::decode(placement, path)?;
    Ok(IfccadEntityKind::Xref(IfccadXref {
        source_id,
        placement,
        scale: value.scale,
    }))
}
