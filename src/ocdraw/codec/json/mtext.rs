//! Registered MTEXT row columns with closed nested authored values.
use super::{
    decode_appearance::appearance,
    encode_geometry::{appearance_columns, placement_frame},
    mtext_values::*,
};
use crate::ocdraw::*;
use serde_json::{json, Value};

pub(crate) fn encode_mtext(entities: &[DrawingMTextEntity]) -> Value {
    let mut stream = json!({"count":entities.len()});
    macro_rules! plain { ($($field:ident=>$key:literal),*) => { $(stream[$key]=json!(entities.iter().map(|t| t.$field).collect::<Vec<_>>());)* }; }
    plain!(id=>"entityId",layer_id=>"layerId",height=>"height",visible=>"visible",rotation=>"rotation",backward=>"backward",upside_down=>"upsideDown");
    stream["styleId"] = json!(entities.iter().map(|t| t.style_id.0).collect::<Vec<_>>());
    stream["placement"] = json!(entities
        .iter()
        .map(|t| placement_frame(t.placement))
        .collect::<Vec<_>>());
    stream["attachment"] = json!(entities
        .iter()
        .map(|t| attachment(t.attachment))
        .collect::<Vec<_>>());
    stream["flow"] = json!(entities.iter().map(|t| flow(t.flow)).collect::<Vec<_>>());
    stream["content"] = json!(entities
        .iter()
        .map(|t| content(&t.content))
        .collect::<Vec<_>>());
    stream["characterFormat"] = json!(entities
        .iter()
        .map(|t| character(&t.character_format))
        .collect::<Vec<_>>());
    stream["paragraphFormat"] = json!(entities
        .iter()
        .map(|t| paragraph(&t.paragraph_format))
        .collect::<Vec<_>>());
    if entities.iter().any(|t| t.wrap_width.is_some()) {
        stream["wrapWidth"] = json!(entities.iter().map(|t| t.wrap_width).collect::<Vec<_>>());
    }
    if entities.iter().any(|t| t.columns.is_some()) {
        stream["columns"] = json!(entities
            .iter()
            .map(|t| t.columns.as_ref().map(columns))
            .collect::<Vec<_>>());
    }
    if entities.iter().any(|t| t.background.is_some()) {
        stream["background"] = json!(entities
            .iter()
            .map(|t| t.background.as_ref().map(background))
            .collect::<Vec<_>>());
    }
    appearance_columns(
        stream.as_object_mut().expect("object stream"),
        &entities.iter().map(|t| &t.appearance).collect::<Vec<_>>(),
    );
    stream
}
pub(crate) fn decode_mtext(value: &Value) -> Result<Vec<DrawingMTextEntity>, OcdrawDiagnostic> {
    let Some(stream) = value["streams"].get("mTextStream") else {
        return Ok(Vec::new());
    };
    let failure = |row| OcdrawDiagnostic {
        code: "MTEXT_ENCODING",
        location: format!("/streams/mTextStream/{row}"),
        message: "invalid registered MTEXT row, exact ID backing or placement".into(),
    };
    let count = stream["count"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| failure(0))?;
    (0..count)
        .map(|row| decode_row(stream, row).ok_or_else(|| failure(row)))
        .collect()
}
fn decode_row(stream: &Value, row: usize) -> Option<DrawingMTextEntity> {
    let column = |name| stream.get(name)?.get(row);
    let present = |name| column(name).filter(|v| !v.is_null());
    Some(DrawingMTextEntity {
        id: column("entityId")?.as_u64()?,
        layer_id: u32::try_from(column("layerId")?.as_u64()?).ok()?,
        style_id: OcdrawTextStyleId(u32::try_from(column("styleId")?.as_u64()?).ok()?),
        appearance: appearance(stream, row)?,
        visible: column("visible").and_then(Value::as_bool).unwrap_or(true),
        placement: match present("placement") {
            None => CoordinateFrame3::default(),
            Some(v) => super::frame_from_json(v)?,
        },
        rotation: column("rotation").and_then(Value::as_f64).unwrap_or(0.0),
        backward: column("backward").and_then(Value::as_bool).unwrap_or(false),
        upside_down: column("upsideDown")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        height: column("height")?.as_f64()?,
        attachment: optional(column("attachment"), read_attachment)?.unwrap_or_default(),
        flow: optional(column("flow"), read_flow)?.unwrap_or_default(),
        wrap_width: optional(present("wrapWidth"), Value::as_f64)?,
        columns: optional(present("columns"), read_columns)?,
        background: optional(present("background"), read_background)?,
        character_format: optional(present("characterFormat"), read_character)?.unwrap_or_default(),
        paragraph_format: optional(present("paragraphFormat"), read_paragraph)?.unwrap_or_default(),
        content: read_content(column("content")?)?,
    })
}
