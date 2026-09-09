use super::mapping::canonical_registry;
use crate::ifcdr::logical::*;
use crate::ifcdr::IfcdrLengthUnit;
use crate::ResourceId;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub(crate) struct EncodedIfcdrResource {
    pub resource_id: ResourceId,
    pub bytes: Vec<u8>,
    pub value: Value,
    pub checksum: String,
}
#[derive(Debug, thiserror::Error)]
pub(crate) enum IfcdrEncodeError {
    #[error("{kind} count exceeds JSON packing range")]
    RangeExhausted { kind: &'static str },
    #[error("IFCDR serialization failed: {message}")]
    Serialization { message: String },
}
fn count(value: usize) -> Result<u32, IfcdrEncodeError> {
    u32::try_from(value).map_err(|_| IfcdrEncodeError::RangeExhausted {
        kind: "stream or pool",
    })
}
fn push(columns: &mut Map<String, Value>, name: &str, value: impl Into<Value>) {
    columns
        .entry(name)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap()
        .push(value.into());
}
fn common(columns: &mut Map<String, Value>, e: IfcdrEntityRow) {
    for (n, v) in [
        ("entityId", e.entity_id),
        ("scopeId", u64::from(e.scope_id)),
        ("layerId", u64::from(e.layer_id)),
        ("appearanceId", u64::from(e.appearance_id)),
    ] {
        push(columns, n, v);
    }
    push(columns, "visible", e.visible);
}
pub(crate) fn encode_json<R: IfcdrResourceAccess>(
    proof: &ValidatedIfcdr<R>,
) -> Result<EncodedIfcdrResource, IfcdrEncodeError> {
    let r = proof.loaded().resource();
    let registry = canonical_registry();
    let mut streams = Map::new();
    let mut directory = Vec::new();
    for schema in registry.streams() {
        let mut columns: Map<String, Value> = schema
            .columns
            .iter()
            .map(|c| (c.name.clone(), json!([])))
            .collect();
        let row_count;
        match schema.name() {
            "line" => {
                let lines = r.lines();
                row_count = count(lines.len())?;
                for i in 0..lines.len() {
                    let l = lines.get(i).expect("validated line row");
                    common(&mut columns, l.entity);
                    for (n, v) in [
                        ("x1", l.start.x()),
                        ("y1", l.start.y()),
                        ("x2", l.end.x()),
                        ("y2", l.end.y()),
                    ] {
                        push(&mut columns, n, v);
                    }
                }
            }
            "polyline" => {
                let polylines = r.polylines();
                row_count = count(polylines.len())?;
                let mut offset = 0usize;
                for i in 0..polylines.len() {
                    let p = polylines.get(i).expect("validated polyline row");
                    common(&mut columns, p.entity());
                    push(&mut columns, "vertexOffset", count(offset)?);
                    push(&mut columns, "vertexCount", count(p.vertex_count())?);
                    push(&mut columns, "closed", p.closed());
                    offset = offset.checked_add(p.vertex_count()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "vertex pool",
                        },
                    )?;
                    count(offset)?;
                    for j in 0..p.vertex_count() {
                        let point = p.vertex(j).expect("validated vertex");
                        push(&mut columns, "x", point.x());
                        push(&mut columns, "y", point.y());
                    }
                }
            }
            "entityOrder" => {
                row_count = count(r.orders().len())?;
                let mut offset = 0usize;
                for order in r.orders() {
                    push(&mut columns, "scopeId", order.scope_id);
                    push(&mut columns, "entryOffset", count(offset)?);
                    push(&mut columns, "entryCount", count(order.entities.len())?);
                    offset = offset.checked_add(order.entities.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "order entries",
                        },
                    )?;
                    count(offset)?;
                }
            }
            "entityOrderEntry" => {
                let mut length = 0usize;
                for order in r.orders() {
                    length = length.checked_add(order.entities.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "order entries",
                        },
                    )?;
                    count(length)?;
                    for id in &order.entities {
                        push(&mut columns, "entityId", *id);
                    }
                }
                row_count = count(length)?;
            }
            _ => unreachable!("registered base-profile stream"),
        }
        if columns
            .get("visible")
            .is_some_and(|v| v.as_array().unwrap().iter().all(|v| v == true))
        {
            columns.remove("visible");
        }
        let names: Vec<_> = schema
            .columns
            .iter()
            .filter(|c| columns.contains_key(&c.name))
            .map(|c| c.name.as_str())
            .collect();
        let mut entry = json!({"name":schema.name(),"schema":schema.schema_id(),"role":match schema.role() { super::mapping::StreamRole::Object=>"object",super::mapping::StreamRole::Order=>"order",super::mapping::StreamRole::Child=>"child" },"count":row_count,"columns":names});
        if let Some(parent) = &schema.parent {
            entry["parent"] = json!(parent);
        }
        if !schema.children.is_empty() {
            entry["children"] = json!(schema.children);
        }
        directory.push(entry);
        columns.insert("count".into(), json!(row_count));
        streams.insert(schema.payload_key().into(), Value::Object(columns));
    }
    let bounds = r.bounds().map(
        |b| json!({"minX":b.min().x(),"minY":b.min().y(),"maxX":b.max().x(),"maxY":b.max().y()}),
    );
    let root = json!({
        "header":{"format":"openaec.ifcdr","version":"0.7.0","resourceId":r.resource_id(),"unit":unit_name(r.unit()),"nextEntityId":r.next_entity_id()},
        "bounds":bounds,
        "scopeTable":r.scopes().iter().map(|s| json!({"id":s.id,"kind":s.kind,"name":s.name,"baseX":s.base.x(),"baseY":s.base.y(),"flags":s.flags})).collect::<Vec<_>>(),
        "layerBindings":r.layers().iter().map(|l| json!({"id":l.id,"ifcxLayer":l.ifcx_layer})).collect::<Vec<_>>(),
        "appearanceBindings":r.appearances().iter().map(|a| json!({"id":a.id,"ifcxAppearance":a.ifcx_appearance,"colorMode":a.modes[0],"opacityMode":a.modes[1],"linePatternMode":a.modes[2],"lineWeightMode":a.modes[3],"overrideId":a.override_id})).collect::<Vec<_>>(),
        "appearanceOverrides":r.overrides().iter().map(|o| json!({"id":o.id,"color":o.color.as_ref().map(color_json),"opacity":o.opacity,"lineWeight":o.line_weight,"ifcxLinePattern":o.ifcx_line_pattern})).collect::<Vec<_>>(),
        "streamDirectory":{"version":"ifccad.ifcdr.streamDirectory.v1","streams":directory},"streams":streams
    });
    let bytes = serde_json::to_vec_pretty(&root).map_err(|e| IfcdrEncodeError::Serialization {
        message: e.to_string(),
    })?;
    let checksum = format!("sha256:{:x}", Sha256::digest(&bytes));
    Ok(EncodedIfcdrResource {
        resource_id: r.resource_id().clone(),
        bytes,
        checksum,
        value: root,
    })
}
pub(crate) fn color_json(c: &IfcdrColor) -> Value {
    let mut value = json!({"rgb":c.rgb});
    if let Some(i) = &c.indexed {
        value["indexedColor"] = json!({"system":i.system,"index":i.index});
    }
    if let Some(n) = &c.named {
        value["namedColor"] = json!({"catalog":n.catalog,"name":n.name});
    }
    value
}
pub(crate) fn unit_name(unit: IfcdrLengthUnit) -> &'static str {
    match unit {
        IfcdrLengthUnit::Unitless => "unitless",
        IfcdrLengthUnit::Millimetre => "mm",
        IfcdrLengthUnit::Centimetre => "cm",
        IfcdrLengthUnit::Metre => "m",
        IfcdrLengthUnit::Kilometre => "km",
        IfcdrLengthUnit::Inch => "in",
        IfcdrLengthUnit::Foot => "ft",
    }
}
