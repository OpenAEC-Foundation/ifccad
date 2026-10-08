use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ViewportWire {
    model: String,
    frame: IfccadViewportFrame,
    view: IfccadViewportView,
    render_mode: IfccadViewportRenderMode,
    view_enabled: bool,
    view_locked: bool,
    paper_clip: PaperClipWire,
    layer_overrides: Vec<LayerOverrideWire>,
    #[serde(
        default,
        deserialize_with = "super::presentation::present",
        skip_serializing_if = "Option::is_none"
    )]
    plot_shading_override: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LayerOverrideWire {
    layer: String,
    #[serde(default)]
    frozen: bool,
    #[serde(
        default,
        deserialize_with = "super::presentation::present",
        skip_serializing_if = "Option::is_none"
    )]
    color: Option<IfccadColor>,
    #[serde(
        default,
        deserialize_with = "super::presentation::present",
        skip_serializing_if = "Option::is_none"
    )]
    opacity: Option<f64>,
    #[serde(
        default,
        deserialize_with = "super::presentation::present",
        skip_serializing_if = "Option::is_none"
    )]
    line_pattern: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::presentation::present",
        skip_serializing_if = "Option::is_none"
    )]
    line_weight: Option<f64>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PaperClipWire {
    enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boundary: Option<String>,
}
fn id(path: &str, prefix: &str) -> Result<u64, IfccadReport> {
    let tail = path.strip_prefix(prefix).ok_or_else(|| {
        crate::ifccad::diagnostics::failure("IFCCAD-ID-001", path, "invalid viewport reference")
    })?;
    let id: u64 = tail.parse().map_err(|_| {
        crate::ifccad::diagnostics::failure("IFCCAD-ID-001", path, "invalid viewport ID path")
    })?;
    if tail != id.to_string() {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-ID-001",
            path,
            format!("noncanonical viewport ID path {path}"),
        ));
    }
    Ok(id)
}
pub(super) fn encode_viewport(viewport: &IfccadViewport, prefix: &str) -> Value {
    let mut layers: Vec<_> = viewport.layer_overrides.iter().collect();
    layers.sort_by_key(|r| r.layer_id);
    serde_json::to_value(ViewportWire {
        model: format!("{prefix}/layout/{}", viewport.model_id),
        frame: viewport.frame.clone(),
        view: viewport.view.clone(),
        render_mode: viewport.render_mode,
        view_enabled: viewport.view_enabled,
        view_locked: viewport.view_locked,
        paper_clip: PaperClipWire {
            enabled: viewport.paper_clip.enabled,
            boundary: viewport
                .paper_clip
                .boundary_entity_id
                .map(|id| format!("{prefix}/e{id}")),
        },
        layer_overrides: layers
            .iter()
            .map(|row| LayerOverrideWire {
                layer: format!("{prefix}/layer/{}", row.layer_id),
                frozen: row.frozen,
                color: row.color.clone(),
                opacity: row.opacity,
                line_weight: row.line_weight,
                line_pattern: row
                    .line_pattern_id
                    .map(|id| format!("{prefix}/linePattern/{}", id.0)),
            })
            .collect(),
        plot_shading_override: viewport.plot_shading_override.map(|mode| {
            use crate::plot_kernel::ShadedPlotMode::*;
            match mode {
                AsDisplayed => "AsDisplayed",
                Wireframe => "Wireframe",
                Hidden => "Hidden",
                Rendered => "Rendered",
            }
            .to_owned()
        }),
    })
    .expect("validated viewport serializes")
}
pub(super) fn decode_viewport(
    value: &Value,
    prefix: &str,
    path: &str,
) -> Result<IfccadViewport, IfccadReport> {
    for pointer in [
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/view/backClip/distance",
        "/paperClip/boundary",
    ] {
        if value.pointer(pointer).is_some_and(Value::is_null) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-VIEWPORT-001",
                &format!("{path}/ifccad::viewport{pointer}"),
                format!("viewport {pointer} must be omitted rather than null"),
            ));
        }
    }
    let wire: ViewportWire = serde_json::from_value(value.clone()).map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-VIEWPORT-001",
            &format!("{path}/ifccad::viewport"),
            format!("invalid viewport payload: {e}"),
        )
    })?;
    let mut layer_overrides = wire
        .layer_overrides
        .iter()
        .map(|row| {
            Ok(IfccadViewportLayerOverride {
                layer_id: id(&row.layer, &format!("{prefix}/layer/"))?,
                frozen: row.frozen,
                color: row.color.clone(),
                opacity: row.opacity,
                line_weight: row.line_weight,
                line_pattern_id: row
                    .line_pattern
                    .as_ref()
                    .map(|p| id(p, &format!("{prefix}/linePattern/")).map(IfccadLinePatternId))
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    layer_overrides.sort_by_key(|r| r.layer_id);
    let plot_shading_override = wire
        .plot_shading_override
        .as_deref()
        .map(|value| {
            use crate::plot_kernel::ShadedPlotMode::*;
            Ok(match value {
                "AsDisplayed" => AsDisplayed,
                "Wireframe" => Wireframe,
                "Hidden" => Hidden,
                "Rendered" => Rendered,
                _ => return Err(IfccadReport::one("invalid viewport plot shading mode")),
            })
        })
        .transpose()?;
    Ok(IfccadViewport {
        workspace: None,

        model_id: id(&wire.model, &format!("{prefix}/layout/"))?,
        frame: wire.frame,
        view: wire.view,
        render_mode: wire.render_mode,
        view_enabled: wire.view_enabled,
        view_locked: wire.view_locked,
        paper_clip: IfccadViewportPaperClip {
            enabled: wire.paper_clip.enabled,
            boundary_entity_id: wire
                .paper_clip
                .boundary
                .as_ref()
                .map(|p| id(p, &format!("{prefix}/e")))
                .transpose()?,
        },
        layer_overrides,
        plot_shading_override,
    })
}
