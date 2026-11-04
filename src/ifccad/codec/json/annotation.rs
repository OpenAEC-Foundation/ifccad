use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScaleValue {
    name: String,
    paper_units: f64,
    drawing_units: f64,
    #[serde(default)]
    unit_scale: bool,
}

fn resources<'a>(
    raw: &'a Value,
    drawing_id: u64,
    attribute: &str,
    domain: &str,
) -> Result<Vec<(u64, &'a Value, &'a str)>, IfccadReport> {
    let prefix = format!("/cad/d{drawing_id}");
    let nodes = raw["data"].as_array().expect("checked nodes");
    let drawing = nodes
        .iter()
        .find(|n| n["path"].as_str() == Some(&prefix))
        .expect("drawing");
    let declared: BTreeSet<_> = drawing["children"]
        .as_object()
        .expect("checked children")
        .values()
        .filter_map(Value::as_str)
        .collect();
    let path_prefix = format!("{prefix}/{domain}/");
    let mut out = vec![];
    for node in nodes {
        let Some(value) = node["attributes"].get(attribute) else {
            continue;
        };
        let path = node["path"].as_str().expect("checked path");
        let invalid = |detail: &str| {
            crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-001", path, detail)
        };
        if !declared.contains(path) {
            return Err(invalid("drawing must reference annotation resource"));
        }
        let attrs = node["attributes"].as_object().expect("checked attributes");
        if attrs
            .keys()
            .any(|name| name.starts_with("ifccad::") && name != attribute)
        {
            return Err(invalid("annotation resource has incompatible CAD role"));
        }
        let suffix = path
            .strip_prefix(&path_prefix)
            .ok_or_else(|| invalid("misplaced annotation resource"))?;
        let id = suffix
            .parse::<u64>()
            .map_err(|_| invalid("invalid annotation resource ID"))?;
        if suffix != id.to_string() {
            return Err(invalid("noncanonical annotation resource path"));
        }
        super::supplemental::validate_value(attribute, value, path)?;
        out.push((id, value, path));
    }
    Ok(out)
}
pub(super) fn decode_scales(
    raw: &Value,
    document: &mut IfccadDocument,
) -> Result<(), IfccadReport> {
    for (id, value, path) in resources(
        raw,
        document.drawing_id,
        "ifccad::annotationScale",
        "annotationScale",
    )? {
        let v: ScaleValue = serde_json::from_value(value.clone()).map_err(|e| {
            crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-001", path, e.to_string())
        })?;
        document.annotation_scales.push(IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(id),
            name: v.name,
            paper_units: v.paper_units,
            drawing_units: v.drawing_units,
            unit_scale: v.unit_scale,
        });
    }
    document.annotation_scales.sort_by_key(|v| v.id);
    Ok(())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredView {
    #[serde(default, deserialize_with = "crate::dimension::present_option")]
    annotation_scale: Option<String>,
    name: String,
    space: IfccadAnnotationViewSpace,
    width: f64,
    view: IfccadViewportView,
}
pub(super) fn decode_views(raw: &Value, document: &mut IfccadDocument) -> Result<(), IfccadReport> {
    for (id, value, path) in resources(
        raw,
        document.drawing_id,
        "ifccad::annotationView",
        "annotationView",
    )? {
        let v: StoredView = serde_json::from_value(value.clone()).map_err(|e| {
            crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-007", path, e.to_string())
        })?;
        document.annotation_views.push(IfccadAnnotationStoredView {
            annotation_scale_id: v
                .annotation_scale
                .map(|p| {
                    super::annotation_policy::scale_id(
                        &p,
                        &format!("/cad/d{}", document.drawing_id),
                    )
                })
                .transpose()?,
            id: IfccadAnnotationViewId(id),
            name: v.name,
            space: v.space,
            width: v.width,
            view: v.view,
        });
    }
    document.annotation_views.sort_by_key(|v| v.id);
    Ok(())
}
pub(super) fn view_value(v: &IfccadAnnotationStoredView, prefix: &str) -> Value {
    let mut value = json!({"name":v.name,"space":v.space,"width":v.width,"view":v.view});
    if let Some(id) = v.annotation_scale_id {
        value["annotationScale"] = json!(super::annotation_policy::scale_path(id, prefix));
    }
    value
}
pub(super) fn scale_value(scale: &IfccadAnnotationScale) -> Value {
    let mut value = json!({"name":scale.name,"paperUnits":scale.paper_units,"drawingUnits":scale.drawing_units});
    if scale.unit_scale {
        value["unitScale"] = json!(true);
    }
    value
}

#[derive(serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Key {
    scale_id: String,
}
#[derive(serde::Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(deserialize = "P: serde::de::DeserializeOwned")
)]
struct Context<P> {
    key: Key,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    presentation: Option<P>,
}
#[derive(serde::Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(deserialize = "P: serde::de::DeserializeOwned")
)]
struct Annotation<P> {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    orientation: Option<IfccadAnnotationOrientation>,
    mode: IfccadAnnotationMode,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    reference: Option<Key>,
    contexts: Vec<Context<P>>,
}
pub(super) fn decode<P: serde::de::DeserializeOwned>(
    v: &Value,
    prefix: &str,
    path: &str,
) -> Result<Box<IfccadAnnotation<P>>, IfccadReport> {
    let invalid = |s: String| crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-004", path, s);
    let wire: Annotation<P> =
        serde_json::from_value(v.clone()).map_err(|e| invalid(e.to_string()))?;
    let key = |k: Key| -> Result<IfccadAnnotationContextKey, IfccadReport> {
        let p = format!("{prefix}/annotationScale/");
        let id = k
            .scale_id
            .strip_prefix(&p)
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|id| k.scale_id == format!("{p}{id}"))
            .ok_or_else(|| invalid("invalid drawing-local annotation scale reference".into()))?;
        Ok(IfccadAnnotationContextKey {
            scale_id: IfccadAnnotationScaleId(id),
        })
    };
    Ok(Box::new(IfccadAnnotation {
        orientation: wire.orientation,
        mode: wire.mode,
        reference: wire.reference.map(&key).transpose()?,
        contexts: wire
            .contexts
            .into_iter()
            .map(|c| {
                Ok(IfccadAnnotationContext {
                    key: key(c.key)?,
                    presentation: c.presentation,
                })
            })
            .collect::<Result<_, IfccadReport>>()?,
    }))
}
pub(super) fn encode<P: serde::Serialize>(a: &IfccadAnnotation<P>, prefix: &str) -> Value {
    let key = |k: IfccadAnnotationContextKey| Key {
        scale_id: format!("{prefix}/annotationScale/{}", k.scale_id.0),
    };
    serde_json::to_value(Annotation {
        orientation: a.orientation,
        mode: a.mode,
        reference: a.reference.map(&key),
        contexts: a
            .contexts
            .iter()
            .map(|c| Context {
                key: key(c.key),
                presentation: c.presentation.as_ref(),
            })
            .collect(),
    })
    .expect("validated annotation")
}
pub(super) fn attach(
    kind: &mut IfccadEntityKind,
    value: &Value,
    prefix: &str,
    path: &str,
) -> Result<(), IfccadReport> {
    match kind {
        IfccadEntityKind::Dimension(d) => {
            d.annotation = Some(super::annotation_dimension::decode(value, prefix, path)?)
        }
        IfccadEntityKind::BlockInstance { annotation, .. } => {
            *annotation = Some(decode(value, prefix, path)?)
        }
        IfccadEntityKind::Text(t) => t.annotation = Some(decode(value, prefix, path)?),
        IfccadEntityKind::MText(t) => t.annotation = Some(decode(value, prefix, path)?),
        IfccadEntityKind::Hatch(h) => {
            h.annotation = Some(super::annotation_hatch::decode_annotation(
                value, prefix, path,
            )?)
        }
        _ => {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-004",
                path,
                "annotation is unsupported on this owner",
            ))
        }
    }
    Ok(())
}
pub(super) fn encode_entity(
    kind: &IfccadEntityKind,
    prefix: &str,
    attrs: &mut serde_json::Map<String, Value>,
) {
    let value = match kind {
        IfccadEntityKind::Dimension(d) => d
            .annotation
            .as_ref()
            .map(|a| super::annotation_dimension::encode(a, prefix)),
        IfccadEntityKind::BlockInstance { annotation, .. } => {
            annotation.as_ref().map(|a| encode(a, prefix))
        }
        IfccadEntityKind::Text(t) => t.annotation.as_ref().map(|a| encode(a, prefix)),
        IfccadEntityKind::MText(t) => t.annotation.as_ref().map(|a| encode(a, prefix)),
        _ => None,
    };
    if let Some(value) = value {
        attrs.insert("ifccad::annotation".into(), value);
    }
}
