use crate::ocdraw::logical::{AppearanceSelection, DrawingColor, EntityAppearance};
use serde_json::Value;

fn column<'a>(stream: &'a Value, field: &str, row: usize) -> Option<&'a Value> {
    stream.get(field)?.get(row)
}

pub(super) fn color(value: &Value) -> Option<DrawingColor> {
    let rgb = value.get("rgb")?.as_array()?;
    let indexed = match value.get("indexedColor") {
        Some(value) => Some((
            value.get("system")?.as_str()?.to_owned(),
            value.get("index")?.as_u64()?,
        )),
        None => None,
    };
    let named = match value.get("namedColor") {
        Some(value) => Some((
            value.get("catalog")?.as_str()?.to_owned(),
            value.get("name")?.as_str()?.to_owned(),
        )),
        None => None,
    };
    Some(DrawingColor {
        rgb: [
            u8::try_from(rgb.first()?.as_u64()?).ok()?,
            u8::try_from(rgb.get(1)?.as_u64()?).ok()?,
            u8::try_from(rgb.get(2)?.as_u64()?).ok()?,
        ],
        indexed,
        named,
    })
}

fn selection<T>(
    stream: &Value,
    row: usize,
    property: &str,
    parse: impl Fn(&Value) -> Option<T>,
) -> Option<AppearanceSelection<T>> {
    let mode_name = format!("{property}Mode");
    let mode = column(stream, &mode_name, row)
        .and_then(Value::as_str)
        .unwrap_or("ByLayer");
    Some(match mode {
        "ByLayer" => AppearanceSelection::ByLayer,
        "ByBlock" => AppearanceSelection::ByBlock,
        "Explicit" => AppearanceSelection::Explicit(parse(column(stream, property, row)?)?),
        _ => return None,
    })
}

pub(super) fn appearance(stream: &Value, row: usize) -> Option<EntityAppearance> {
    Some(EntityAppearance {
        color: selection(stream, row, "color", color)?,
        opacity: selection(stream, row, "opacity", Value::as_f64)?,
        line_pattern: selection(stream, row, "linePattern", |value| {
            value.as_str().map(str::to_owned)
        })?,
        line_weight: selection(stream, row, "lineWeight", Value::as_f64)?,
    })
}
