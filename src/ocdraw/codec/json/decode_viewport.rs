use super::decode_appearance::{appearance, color};
use super::decode_view_state::{point2, render_mode, view};
use crate::ocdraw::logical::{
    DrawingPaperClip, DrawingViewport, DrawingViewportFrame, DrawingViewportLayerOverride,
};
use crate::ocdraw::ShadedPlotMode;
use serde_json::Value;

fn column<'a>(stream: &'a Value, key: &str, row: usize) -> Option<&'a Value> {
    stream.get(key)?.get(row)
}

fn override_row(stream: &Value, row: usize) -> Option<DrawingViewportLayerOverride> {
    let optional = |key: &str| column(stream, key, row).filter(|value| !value.is_null());
    Some(DrawingViewportLayerOverride {
        layer_id: u32::try_from(column(stream, "layerId", row)?.as_u64()?).ok()?,
        frozen: column(stream, "frozen", row)?.as_bool()?,
        color: optional("color").map(color).transpose_option()?,
        opacity: optional("opacity").map(Value::as_f64).transpose_option()?,
        line_pattern_id: optional("linePatternId")
            .map(|value| {
                value
                    .as_u64()
                    .map(|id| crate::ocdraw::LinePatternId(id as u32))
            })
            .transpose_option()?,
        line_weight: optional("lineWeight")
            .map(Value::as_f64)
            .transpose_option()?,
    })
}

fn shaded(value: &Value) -> Option<ShadedPlotMode> {
    Some(match value.as_str()? {
        "AsDisplayed" => ShadedPlotMode::AsDisplayed,
        "Wireframe" => ShadedPlotMode::Wireframe,
        "Hidden" => ShadedPlotMode::Hidden,
        "Rendered" => ShadedPlotMode::Rendered,
        _ => return None,
    })
}
trait OptionOptionExt<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}
impl<T> OptionOptionExt<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            Some(Some(value)) => Some(Some(value)),
            Some(None) => None,
            None => Some(None),
        }
    }
}

pub(crate) fn decode_viewports(root: &Value) -> Option<Vec<DrawingViewport>> {
    let Some(stream) = root.get("streams")?.get("viewportStream") else {
        return Some(Vec::new());
    };
    let count = usize::try_from(stream.get("count")?.as_u64()?).ok()?;
    let overrides = root["streams"].get("viewportLayerOverrideStream");
    let mut decoded = Vec::with_capacity(count);
    for row in 0..count {
        let frame = column(stream, "frame", row)?;
        let paper_clip = column(stream, "paperClip", row)?;
        let offset = column(stream, "layerOverrideOffset", row)
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let override_count = column(stream, "layerOverrideCount", row)
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let offset = usize::try_from(offset).ok()?;
        let override_count = usize::try_from(override_count).ok()?;
        let layer_overrides = if override_count == 0 {
            Vec::new()
        } else {
            let overrides = overrides?;
            (offset..offset.checked_add(override_count)?)
                .map(|at| override_row(overrides, at))
                .collect::<Option<Vec<_>>>()?
        };
        let shading = column(stream, "plotShadingOverride", row)?;
        decoded.push(DrawingViewport {
            id: column(stream, "entityId", row)?.as_u64()?,
            view_scope_id: u32::try_from(column(stream, "viewScopeId", row)?.as_u64()?).ok()?,
            layer_id: u32::try_from(column(stream, "layerId", row)?.as_u64()?).ok()?,
            frame: DrawingViewportFrame {
                center: point2(frame.get("center")?)?,
                width: frame.get("width")?.as_f64()?,
                height: frame.get("height")?.as_f64()?,
            },
            view: view(column(stream, "view", row)?)?,
            render_mode: render_mode(column(stream, "renderMode", row)?)?,
            view_enabled: column(stream, "viewEnabled", row)?.as_bool()?,
            view_locked: column(stream, "viewLocked", row)?.as_bool()?,
            paper_clip: DrawingPaperClip {
                enabled: paper_clip.get("enabled")?.as_bool()?,
                boundary_entity_id: paper_clip.get("boundaryEntityId").and_then(Value::as_u64),
            },
            plot_shading_override: if shading.is_null() {
                None
            } else {
                Some(shaded(shading)?)
            },
            appearance: appearance(stream, row)?,
            visible: column(stream, "visible", row)
                .and_then(Value::as_bool)
                .unwrap_or(true),
            layer_overrides,
        });
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn viewport_columns_become_a_typed_record() {
        let source = json!({"streams":{"viewportStream":{
            "count":1,"entityId":[7],"scopeId":[1],"viewScopeId":[0],"layerId":[2],
            "frame":[{"center":{"x":3.0,"y":4.0},"width":8.0,"height":6.0}],
            "view":[{"center":{"x":0.0,"y":0.0},
                "target":{"x":0.0,"y":0.0,"z":0.0},
                "direction":{"x":0.0,"y":0.0,"z":1.0},
                "height":10.0,"twist":0.0,"projection":"Orthographic",
                "frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}}],
            "renderMode":["Wireframe"],"viewEnabled":[true],"viewLocked":[false],
            "paperClip":[{"enabled":false}],"plotShadingOverride":[null],
            "layerOverrideOffset":[0],"layerOverrideCount":[0]
        }}});
        let decoded = decode_viewports(&source).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].id, 7);
        assert_eq!(decoded[0].frame.center.x(), 3.0);
        assert_eq!(
            decoded[0].render_mode,
            crate::ocdraw::DrawingRenderMode::Wireframe
        );
    }
}
