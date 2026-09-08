use super::physical::validate_physical;
use crate::diagnostic::{PackageDiagnostic, PackageDiagnosticSeverity};
use crate::ifcdr::logical::*;
use crate::ifcdr::read::decoded::*;
use crate::ifcdr::{Bounds2d, Point2};
use crate::ResourceId;
use serde_json::Value;

fn error(uri: &str, location: String, code: &str, message: &str) -> PackageDiagnostic {
    PackageDiagnostic {
        code: code.into(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(uri.into()),
        location: Some(location),
        context: Default::default(),
        message: message.into(),
    }
}
pub(crate) fn decode_json(
    uri: &str,
    value: &Value,
) -> Result<DecodedIfcdrResource, Vec<PackageDiagnostic>> {
    let errors = validate_physical(uri, value);
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut errors = Vec::new();
    let id = ResourceId::new(
        value["header"]["resourceId"]
            .as_str()
            .expect("physical string"),
    );
    if id.is_err() {
        errors.push(error(
            uri,
            "/header/resourceId".into(),
            IFCCAD_IFCDR_STRUCTURE_INVALID,
            "resource identity must not be empty",
        ));
    }
    let unit = length_unit(value["header"]["unit"].as_str().expect("physical string"));
    if unit.is_none() {
        errors.push(error(
            uri,
            "/header/unit".into(),
            "IFCCAD_IFCDR_UNIT_UNSUPPORTED",
            "unsupported length unit",
        ));
    }
    let overrides = rows(value, "appearanceOverrides")
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let color = if v["color"].is_null() {
                None
            } else {
                let c = color(&v["color"]);
                if c.is_none() {
                    errors.push(error(
                        uri,
                        format!("/appearanceOverrides/{i}/color"),
                        IFCCAD_IFCDR_APPEARANCE_INVALID,
                        "color requires three RGB bytes and typed metadata",
                    ));
                }
                c
            };
            IfcdrAppearanceOverride {
                id: u32v(&v["id"]),
                color,
                opacity: v["opacity"].as_f64(),
                ifcx_line_pattern: v["ifcxLinePattern"].as_str().map(String::from),
                line_weight: v["lineWeight"].as_f64(),
            }
        })
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    let l = &value["streams"]["lineStream"];
    let p = &value["streams"]["polylineStream"];
    let o = &value["streams"]["entityOrderStream"];
    let entries: Vec<u64> = values(
        &value["streams"]["entityOrderEntryStream"]["entityId"],
        |v| v.as_u64().unwrap(),
    );
    let orders = (0..o["count"].as_u64().unwrap_or(0) as usize)
        .map(|i| {
            let start = u32v(&o["entryOffset"][i]) as usize;
            let count = u32v(&o["entryCount"][i]) as usize;
            IfcdrScopeOrder {
                scope_id: u32v(&o["scopeId"][i]),
                entities: entries[start..start + count].to_vec(),
            }
        })
        .collect();
    Ok(DecodedIfcdrResource {
        id: id.unwrap(),
        unit: unit.unwrap(),
        next: value["header"]["nextEntityId"].as_u64().unwrap(),
        bounds: (!value["bounds"].is_null()).then(|| Bounds2d {
            min: Point2::new(num(&value["bounds"]["minX"]), num(&value["bounds"]["minY"])),
            max: Point2::new(num(&value["bounds"]["maxX"]), num(&value["bounds"]["maxY"])),
        }),
        scopes: rows(value, "scopeTable")
            .iter()
            .map(|v| IfcdrScope {
                id: u32v(&v["id"]),
                kind: u32v(&v["kind"]),
                name: v["name"].as_str().unwrap().into(),
                base: Point2::new(num(&v["baseX"]), num(&v["baseY"])),
                flags: u32v(&v["flags"]),
            })
            .collect(),
        layers: rows(value, "layerBindings")
            .iter()
            .map(|v| IfcdrLayerBinding {
                id: u32v(&v["id"]),
                ifcx_layer: v["ifcxLayer"].as_str().unwrap().into(),
            })
            .collect(),
        appearances: rows(value, "appearanceBindings")
            .iter()
            .map(|v| IfcdrAppearanceBinding {
                id: u32v(&v["id"]),
                ifcx_appearance: v["ifcxAppearance"].as_str().map(String::from),
                modes: [
                    "colorMode",
                    "opacityMode",
                    "linePatternMode",
                    "lineWeightMode",
                ]
                .map(|key| u32v(&v[key])),
                override_id: v["overrideId"].as_u64().map(|v| v as u32),
            })
            .collect(),
        overrides,
        orders,
        lines: LineColumns {
            entity: entity(l),
            x1: values(&l["x1"], num),
            y1: values(&l["y1"], num),
            x2: values(&l["x2"], num),
            y2: values(&l["y2"], num),
        },
        polylines: PolylineColumns {
            entity: entity(p),
            offsets: values(&p["vertexOffset"], |v| u32v(v) as usize),
            counts: values(&p["vertexCount"], |v| u32v(v) as usize),
            closed: values(&p["closed"], |v| v.as_bool().unwrap()),
            x: values(&p["x"], num),
            y: values(&p["y"], num),
        },
    })
}
fn rows<'a>(root: &'a Value, key: &str) -> &'a [Value] {
    root[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn values<T>(value: &Value, convert: impl Fn(&Value) -> T) -> Vec<T> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(convert)
        .collect()
}
fn num(value: &Value) -> f64 {
    value.as_f64().expect("physical number")
}
fn u32v(value: &Value) -> u32 {
    u32::try_from(value.as_u64().expect("physical unsigned integer")).expect("physical u32")
}
fn entity(value: &Value) -> EntityColumns {
    EntityColumns {
        ids: values(&value["entityId"], |v| v.as_u64().unwrap()),
        scopes: values(&value["scopeId"], u32v),
        layers: values(&value["layerId"], u32v),
        appearances: values(&value["appearanceId"], u32v),
        visible: value
            .get("visible")
            .map(|v| values(v, |v| v.as_bool().unwrap()))
            .unwrap_or_else(|| vec![true; value["count"].as_u64().unwrap_or(0) as usize]),
    }
}
pub(crate) fn color(value: &Value) -> Option<IfcdrColor> {
    let rgb = value["rgb"].as_array()?;
    if rgb.len() != 3 {
        return None;
    }
    let indexed = if let Some(v) = value.get("indexedColor") {
        Some(IfcdrIndexedColor {
            system: v["system"].as_str()?.into(),
            index: v["index"].as_u64()?,
        })
    } else {
        None
    };
    let named = if let Some(v) = value.get("namedColor") {
        Some(IfcdrNamedColor {
            catalog: v["catalog"].as_str()?.into(),
            name: v["name"].as_str()?.into(),
        })
    } else {
        None
    };
    Some(IfcdrColor {
        rgb: [
            rgb_channel(rgb[0].as_u64()?)?,
            rgb_channel(rgb[1].as_u64()?)?,
            rgb_channel(rgb[2].as_u64()?)?,
        ],
        indexed,
        named,
    })
}
