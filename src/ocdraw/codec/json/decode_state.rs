use crate::ocdraw::logical::{DrawingBlockDefinition, DrawingUcsDefinition, DrawingWorkspaceState};
use crate::ocdraw::{
    CoordinateFrame3, Point3, PointDisplay, PointGlyph, PointSize, UcsDefinition, Vector3,
};
use serde_json::Value;

pub(crate) fn frame_from_json(value: &Value) -> Option<CoordinateFrame3> {
    let point = value.get("origin")?;
    let x = value.get("X")?;
    let y = value.get("Y")?;
    CoordinateFrame3::try_new(
        Point3::new(
            point.get("x")?.as_f64()?,
            point.get("y")?.as_f64()?,
            point.get("z")?.as_f64()?,
        ),
        Vector3::new(
            x.get("x")?.as_f64()?,
            x.get("y")?.as_f64()?,
            x.get("z")?.as_f64()?,
        ),
        Vector3::new(
            y.get("x")?.as_f64()?,
            y.get("y")?.as_f64()?,
            y.get("z")?.as_f64()?,
        ),
    )
    .ok()
}

pub(crate) fn decode_ucs_definitions(value: &Value) -> Option<Vec<DrawingUcsDefinition>> {
    value
        .get("ucsDefinitions")
        .map(Value::as_array)
        .transpose_option()?
        .map(|definitions| {
            definitions
                .iter()
                .map(|row| {
                    Some(DrawingUcsDefinition {
                        id: u32::try_from(row.get("ucsId")?.as_u64()?).ok()?,
                        definition: UcsDefinition::new(
                            row.get("name")?.as_str()?,
                            frame_from_json(row.get("frame")?)?,
                            row.get("elevation")?.as_f64()?,
                        ),
                    })
                })
                .collect::<Option<Vec<_>>>()
        })
        .unwrap_or_else(|| Some(Vec::new()))
}

pub(crate) fn decode_workspace_state(value: &Value) -> Option<Option<DrawingWorkspaceState>> {
    let Some(row) = value.get("drawingWorkspaceState") else {
        return Some(None);
    };
    Some(Some(DrawingWorkspaceState {
        current_layer_id: row
            .get("currentLayerId")
            .map(|value| u32::try_from(value.as_u64()?).ok())
            .transpose_option()?,
        active_layout_id: row
            .get("activeLayoutId")
            .map(|value| u32::try_from(value.as_u64()?).ok())
            .transpose_option()?,
    }))
}

trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}

impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            Some(Some(value)) => Some(Some(value)),
            Some(None) => None,
            None => Some(None),
        }
    }
}

pub(crate) fn decode_point_display(value: &Value) -> Option<Option<PointDisplay>> {
    let Some(row) = value.get("pointDisplay") else {
        return Some(None);
    };
    let form = row.get("form")?;
    let size = row.get("size")?;
    let glyph = match form.get("glyph")?.as_str()? {
        "dot" => PointGlyph::Dot,
        "hidden" => PointGlyph::Hidden,
        "plus" => PointGlyph::Plus,
        "cross" => PointGlyph::Cross,
        "shortLine" => PointGlyph::ShortLine,
        _ => return None,
    };
    let size = match size.get("kind")?.as_str()? {
        "defaultFivePercent" => PointSize::DefaultFivePercent,
        "absolute" => PointSize::Absolute(size.get("value")?.as_f64()?),
        "viewportPercent" => PointSize::ViewportPercent(size.get("value")?.as_f64()?),
        _ => return None,
    };
    Some(Some(PointDisplay {
        glyph,
        circle: form.get("circle")?.as_bool()?,
        square: form.get("square")?.as_bool()?,
        size,
    }))
}

pub(crate) fn decode_block_definitions(value: &Value) -> Option<Vec<DrawingBlockDefinition>> {
    let Some(rows) = value.get("blockDefinitions") else {
        return Some(Vec::new());
    };
    rows.as_array()?
        .iter()
        .map(|row| {
            let base_point = row.get("basePoint");
            Some(DrawingBlockDefinition {
                scope_id: u32::try_from(row.get("scopeId")?.as_u64()?).ok()?,
                name: row.get("name")?.as_str()?.to_owned(),
                base_point: match base_point {
                    Some(point) => [
                        point.get("x")?.as_f64()?,
                        point.get("y")?.as_f64()?,
                        point.get("z")?.as_f64()?,
                    ],
                    None => [0.0; 3],
                },
                description: row
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                anonymous: row
                    .get("anonymous")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                insertion_unit: row
                    .get("insertionUnit")
                    .and_then(Value::as_str)
                    .unwrap_or("unitless")
                    .to_owned(),
                explodable: row
                    .get("explodable")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                uniform_scaling: row.get("scaling").and_then(Value::as_str) == Some("Uniform"),
            })
        })
        .collect()
}
