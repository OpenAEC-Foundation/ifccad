use crate::ocdraw::logical::DrawingColor;
use serde_json::{json, Value};

pub(crate) fn encode_color(color: &DrawingColor) -> Value {
    let mut value = json!({"rgb": color.rgb});
    if let Some((system, index)) = &color.indexed {
        value["indexedColor"] = json!({"system": system, "index": index});
    }
    if let Some((catalog, name)) = &color.named {
        value["namedColor"] = json!({"catalog": catalog, "name": name});
    }
    value
}
