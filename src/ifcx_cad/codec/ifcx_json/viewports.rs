use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ViewportWire {
    model: String,
    frame: IfcxCadViewportFrame,
    view: IfcxCadViewportView,
    render_mode: IfcxCadViewportRenderMode,
    view_enabled: bool,
    view_locked: bool,
    visible: bool,
    paper_clip: PaperClipWire,
    frozen_layers: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PaperClipWire {
    enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    boundary: Option<String>,
}
fn id(path: &str, prefix: &str) -> Result<u64, IfcxCadReport> {
    let tail = path
        .strip_prefix(prefix)
        .ok_or_else(|| IfcxCadReport::one(format!("invalid viewport reference {path}")))?;
    let id: u64 = tail
        .parse()
        .map_err(|_| IfcxCadReport::one(format!("invalid viewport ID path {path}")))?;
    if tail != id.to_string() {
        return Err(IfcxCadReport::one(format!(
            "noncanonical viewport ID path {path}"
        )));
    }
    Ok(id)
}
pub(super) fn encode_viewport(viewport: &IfcxCadViewport, prefix: &str) -> Value {
    let mut layers = viewport.frozen_layers.clone();
    layers.sort_unstable();
    serde_json::to_value(ViewportWire {
        model: format!("{prefix}/layout/{}", viewport.model_id),
        frame: viewport.frame.clone(),
        view: viewport.view.clone(),
        render_mode: viewport.render_mode,
        view_enabled: viewport.view_enabled,
        view_locked: viewport.view_locked,
        visible: viewport.visible,
        paper_clip: PaperClipWire {
            enabled: viewport.paper_clip.enabled,
            boundary: viewport
                .paper_clip
                .boundary_entity_id
                .map(|id| format!("{prefix}/e{id}")),
        },
        frozen_layers: layers
            .iter()
            .map(|id| format!("{prefix}/layer/{id}"))
            .collect(),
    })
    .expect("validated viewport serializes")
}
pub(super) fn decode_viewport(
    value: &Value,
    prefix: &str,
) -> Result<IfcxCadViewport, IfcxCadReport> {
    for pointer in [
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/view/backClip/distance",
        "/paperClip/boundary",
    ] {
        if value.pointer(pointer).is_some_and(Value::is_null) {
            return Err(IfcxCadReport::one(format!(
                "viewport {pointer} must be omitted rather than null"
            )));
        }
    }
    let wire: ViewportWire = serde_json::from_value(value.clone())
        .map_err(|e| IfcxCadReport::one(format!("invalid viewport payload: {e}")))?;
    let mut frozen_layers = wire
        .frozen_layers
        .iter()
        .map(|p| id(p, &format!("{prefix}/layer/")))
        .collect::<Result<Vec<_>, _>>()?;
    frozen_layers.sort_unstable();
    Ok(IfcxCadViewport {
        model_id: id(&wire.model, &format!("{prefix}/layout/"))?,
        frame: wire.frame,
        view: wire.view,
        render_mode: wire.render_mode,
        view_enabled: wire.view_enabled,
        view_locked: wire.view_locked,
        visible: wire.visible,
        paper_clip: IfcxCadViewportPaperClip {
            enabled: wire.paper_clip.enabled,
            boundary_entity_id: wire
                .paper_clip
                .boundary
                .as_ref()
                .map(|p| id(p, &format!("{prefix}/e")))
                .transpose()?,
        },
        frozen_layers,
    })
}
