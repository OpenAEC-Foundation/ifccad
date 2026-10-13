use crate::ocdraw::logical::DrawingColor;
use serde_json::{json, Value};

pub(crate) fn encode_color(color: &DrawingColor) -> Value {
    let DrawingColor::Concrete {
        rgb,
        indexed,
        named,
    } = color
    else {
        return json!({"kind":"Foreground"});
    };
    let mut value = json!({"rgb": rgb});
    if let Some((system, index)) = indexed {
        value["indexedColor"] = json!({"system": system, "index": index});
    }
    if let Some((catalog, name)) = named {
        value["namedColor"] = json!({"catalog": catalog, "name": name});
    }
    value
}
