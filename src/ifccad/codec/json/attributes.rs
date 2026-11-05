//! Closed attribute payloads using the ordinary text value mapping.
use crate::ifccad::*;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Wire {
    tag: String,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    definition: Option<String>,
    #[serde(default)]
    flags: IfccadAttributeFlags,
    #[serde(default)]
    lock_position: bool,
    #[serde(default)]
    field_length: i32,
    text: Value,
    #[serde(default)]
    m_text: Option<Value>,
    #[serde(default)]
    annotation: Option<Value>,
    #[serde(default)]
    secondary_attributes: Vec<String>,
    #[serde(default)]
    fields: Option<Value>,
}

fn fail(path: &str, m: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-ATTRIBUTE-001", path, m.to_string())
}
fn no_null(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Object(o) => o.values().all(no_null),
        Value::Array(a) => a.iter().all(no_null),
        _ => true,
    }
}
fn id(v: &str, prefix: &str, path: &str) -> Result<u64, IfccadReport> {
    let n = v
        .strip_prefix(&format!("{prefix}/e"))
        .ok_or_else(|| fail(path, "foreign attribute reference"))?;
    let id: u64 = n
        .parse()
        .map_err(|_| fail(path, "invalid attribute reference"))?;
    if id.to_string() != n {
        return Err(fail(path, "noncanonical attribute reference"));
    }
    Ok(id)
}
fn field_refs(v: &mut Value, prefix: &str, path: &str, encoding: bool) -> Result<(), IfccadReport> {
    if v.get("kind").and_then(Value::as_str) == Some("Object") {
        v["value"] = if encoding {
            json!(format!(
                "{prefix}/e{}",
                v["value"]
                    .as_u64()
                    .ok_or_else(|| fail(path, "invalid field object"))?
            ))
        } else {
            json!(id(
                v["value"]
                    .as_str()
                    .ok_or_else(|| fail(path, "field object must be a path"))?,
                prefix,
                path
            )?)
        };
    } else {
        match v {
            Value::Object(o) => {
                for v in o.values_mut() {
                    field_refs(v, prefix, path, encoding)?;
                }
            }
            Value::Array(a) => {
                for v in a {
                    field_refs(v, prefix, path, encoding)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
fn embedded_scale(
    v: &mut Value,
    prefix: &str,
    path: &str,
    encoding: bool,
) -> Result<(), IfccadReport> {
    if let Some(contexts) = v.get_mut("contexts").and_then(Value::as_array_mut) {
        for c in contexts {
            if let Some(e) = c
                .get_mut("presentation")
                .and_then(|p| p.get_mut("embedded"))
            {
                if e.is_null() {
                    continue;
                }
                if encoding {
                    e["scaleId"] = json!(super::annotation_policy::scale_path(
                        IfccadAnnotationScaleId(
                            e["scaleId"]
                                .as_u64()
                                .ok_or_else(|| fail(path, "invalid embedded scale"))?
                        ),
                        prefix
                    ));
                } else {
                    e["scaleId"] = json!(
                        super::annotation_policy::scale_id(
                            e["scaleId"]
                                .as_str()
                                .ok_or_else(|| fail(path, "embedded scale must be a path"))?,
                            prefix
                        )?
                        .0
                    );
                }
            }
        }
    }
    Ok(())
}
pub(super) fn decode_value(
    v: &Value,
    placement: &Value,
    prefix: &str,
    path: &str,
    styles: &BTreeMap<String, IfccadTextStyleId>,
    patterns: &BTreeMap<String, IfccadLinePatternId>,
    definition: bool,
) -> Result<(Option<String>, IfccadAttributeValue), IfccadReport> {
    if !no_null(v) {
        return Err(fail(path, "omit absent attribute values instead of null"));
    }
    let w: Wire = serde_json::from_value(v.clone()).map_err(|e| fail(path, e))?;
    if definition && w.definition.is_some() || !definition && w.prompt.is_some() {
        return Err(fail(
            path,
            "attribute fields do not match definition/value role",
        ));
    }
    let IfccadEntityKind::Text(text) =
        super::text::decode_entity(&w.text, placement, "ifccad::text", path, styles)?
    else {
        unreachable!()
    };
    let m_text = w
        .m_text
        .map(|m| -> Result<_, IfccadReport> {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Embedded {
                #[serde(default = "yes")]
                visible: bool,
                layer: String,
                #[serde(default)]
                appearance: Option<Value>,
                #[serde(default = "one")]
                line_pattern_scale: f64,
                text: Value,
                placement: Value,
            }
            fn yes() -> bool {
                true
            }
            fn one() -> f64 {
                1.
            }
            let m: Embedded = serde_json::from_value(m).map_err(|e| fail(path, e))?;
            let layer = m
                .layer
                .strip_prefix(&format!("{prefix}/layer/"))
                .ok_or_else(|| fail(path, "foreign embedded layer"))?;
            let layer_id: u64 = layer.parse().map_err(|e| fail(path, e))?;
            if layer_id.to_string() != layer {
                return Err(fail(path, "noncanonical embedded layer"));
            }
            let IfccadEntityKind::MText(t) =
                super::text::decode_entity(&m.text, &m.placement, "ifccad::mText", path, styles)?
            else {
                unreachable!()
            };
            Ok(Box::new(IfccadAttributeMText {
                visible: m.visible,
                layer_id,
                appearance: super::wire::decode_native_appearance(m.appearance.as_ref(), patterns)?,
                line_pattern_scale: m.line_pattern_scale,
                text: *t,
            }))
        })
        .transpose()?;
    let annotation = w
        .annotation
        .map(|mut a| {
            embedded_scale(&mut a, prefix, path, false)?;
            super::annotation::decode(&a, prefix, path)
        })
        .transpose()?;
    let fields = w
        .fields
        .map(|mut v| {
            field_refs(&mut v, prefix, path, false)?;
            serde_json::from_value(v).map_err(|e| fail(path, e))
        })
        .transpose()?;
    Ok((
        w.prompt,
        IfccadAttributeValue {
            tag: w.tag,
            definition_id: w
                .definition
                .as_deref()
                .map(|v| id(v, prefix, path))
                .transpose()?,
            flags: w.flags,
            lock_position: w.lock_position,
            field_length: w.field_length,
            text,
            m_text,
            annotation,
            secondary_attributes: w
                .secondary_attributes
                .iter()
                .map(|v| id(v, prefix, path))
                .collect::<Result<_, _>>()?,
            fields,
        },
    ))
}
pub(super) fn definition(
    v: &Value,
    placement: &Value,
    prefix: &str,
    path: &str,
    styles: &BTreeMap<String, IfccadTextStyleId>,
    patterns: &BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadAttributeDefinition, IfccadReport> {
    let (prompt, a) = decode_value(v, placement, prefix, path, styles, patterns, true)?;
    Ok(IfccadAttributeDefinition {
        tag: a.tag,
        prompt: prompt.ok_or_else(|| fail(path, "definition prompt required"))?,
        flags: a.flags,
        lock_position: a.lock_position,
        field_length: a.field_length,
        text: a.text,
        m_text: a.m_text,
        annotation: a.annotation,
        secondary_attributes: a.secondary_attributes,
        fields: a.fields,
    })
}
pub(super) fn text_payload(kind: &IfccadEntityKind, prefix: &str) -> (Value, Value) {
    let mut attrs = Map::new();
    super::text::encode_entity(kind, prefix, &mut attrs);
    let key = if matches!(kind, IfccadEntityKind::Text(_)) {
        "ifccad::text"
    } else {
        "ifccad::mText"
    };
    (
        attrs.remove(key).unwrap(),
        attrs.remove("ifccad::geom::placement").unwrap(),
    )
}
pub(super) fn encode_value(a: &IfccadAttributeValue, prefix: &str) -> Value {
    let (text, _) = text_payload(&IfccadEntityKind::Text(a.text.clone()), prefix);
    let mut v = json!({"tag":a.tag,"flags":a.flags,"lockPosition":a.lock_position,"fieldLength":a.field_length,"text":text});
    if let Some(id) = a.definition_id {
        v["definition"] = json!(format!("{prefix}/e{id}"));
    }
    if !a.secondary_attributes.is_empty() {
        v["secondaryAttributes"] = json!(a
            .secondary_attributes
            .iter()
            .map(|id| format!("{prefix}/e{id}"))
            .collect::<Vec<_>>());
    }
    if let Some(m) = &a.m_text {
        let (text, placement) =
            text_payload(&IfccadEntityKind::MText(Box::new(m.text.clone())), prefix);
        let mut mv = json!({"visible":m.visible,"layer":format!("{prefix}/layer/{}",m.layer_id),"linePatternScale":m.line_pattern_scale,"text":text,"placement":placement});
        if let Some(appearance) = super::wire::native_entity(&m.appearance, prefix) {
            mv["appearance"] = appearance;
        }
        v["mText"] = mv;
    }
    if let Some(a) = &a.annotation {
        let mut av = super::annotation::encode(a, prefix);
        embedded_scale(&mut av, prefix, "/annotation", true).expect("validated scale");
        // None embedded presentation is omitted on wire, just like ordinary contexts.
        if let Some(cs) = av.get_mut("contexts").and_then(Value::as_array_mut) {
            for c in cs {
                if let Some(p) = c.get_mut("presentation").and_then(Value::as_object_mut) {
                    if p.get("embedded").is_some_and(Value::is_null) {
                        p.remove("embedded");
                    }
                }
            }
        }
        v["annotation"] = av;
    }
    if let Some(f) = &a.fields {
        let mut fv = serde_json::to_value(f).expect("validated fields");
        field_refs(&mut fv, prefix, "/fields", true).expect("validated references");
        v["fields"] = fv;
    }
    v
}
pub(super) fn encode_definition(a: &IfccadAttributeDefinition, prefix: &str) -> Value {
    let value = IfccadAttributeValue {
        tag: a.tag.clone(),
        definition_id: None,
        flags: a.flags,
        lock_position: a.lock_position,
        field_length: a.field_length,
        text: a.text.clone(),
        m_text: a.m_text.clone(),
        annotation: a.annotation.clone(),
        secondary_attributes: a.secondary_attributes.clone(),
        fields: a.fields.clone(),
    };
    let mut v = encode_value(&value, prefix);
    v["prompt"] = json!(a.prompt);
    v
}
