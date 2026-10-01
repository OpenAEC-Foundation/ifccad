use super::decode_plot::{plot_settings, rectangle};
use crate::ocdraw::logical::{DrawingLayout, DrawingLayoutKind};
use crate::ocdraw::LayoutSettings;
use serde_json::Value;

pub(crate) fn decode_layouts(value: &Value) -> Option<Vec<DrawingLayout>> {
    value
        .get("layouts")?
        .as_array()?
        .iter()
        .map(|row| {
            Some(DrawingLayout {
                id: u32::try_from(row.get("id")?.as_u64()?).ok()?,
                scope_id: u32::try_from(row.get("scopeId")?.as_u64()?).ok()?,
                kind: match row.get("kind")?.as_str()? {
                    "model" => DrawingLayoutKind::Model,
                    "paper" => DrawingLayoutKind::Paper,
                    _ => return None,
                },
                name: row.get("name")?.as_str()?.to_owned(),
                tab_index: u32::try_from(row.get("tabIndex")?.as_u64()?).ok()?,
                settings: LayoutSettings {
                    limits: row.get("limits").map(rectangle).transpose_option()?,
                    limits_checking: row
                        .get("limitsChecking")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    paper_space_linetype_scaling: row
                        .get("paperSpaceLinetypeScaling")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                    plot_settings: row
                        .get("plotSettings")
                        .map(plot_settings)
                        .transpose_option()?,
                },
            })
        })
        .collect()
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
