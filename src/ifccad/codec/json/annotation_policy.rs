//! Typed path projection of saved annotation selections.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub(super) fn scale_path(id: IfccadAnnotationScaleId, prefix: &str) -> String {
    format!("{prefix}/annotationScale/{}", id.0)
}
pub(super) fn scale_id(path: &str, prefix: &str) -> Result<IfccadAnnotationScaleId, IfccadReport> {
    let root = format!("{prefix}/annotationScale/");
    let id = path
        .strip_prefix(&root)
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|id| path == format!("{root}{id}"));
    id.map(IfccadAnnotationScaleId).ok_or_else(|| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-ANNOTATION-008",
            path,
            "invalid drawing-local scale reference",
        )
    })
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct State {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    current_scale: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    visibility: Option<IfccadAnnotationVisibility>,
}
impl State {
    pub(super) fn decode(self, prefix: &str) -> Result<IfccadLayoutAnnotationState, IfccadReport> {
        Ok(IfccadLayoutAnnotationState {
            current_scale_id: self
                .current_scale
                .map(|p| scale_id(&p, prefix))
                .transpose()?,
            visibility: self.visibility,
        })
    }
}
fn state(value: IfccadLayoutAnnotationState, prefix: &str) -> Value {
    serde_json::to_value(State {
        current_scale: value.current_scale_id.map(|id| scale_path(id, prefix)),
        visibility: value.visibility,
    })
    .expect("validated state")
}
pub(super) fn encode_document(
    d: &IfccadDocument,
    prefix: &str,
    nodes: &mut [super::encode::NodeOut],
) {
    for node in nodes {
        if node.path == prefix {
            if let Some(settings) = d.annotation_settings {
                node.attributes["ifccad::drawing"]["annotationSettings"] = json!(settings);
            }
        }
        if let Some(value) = node.attributes.get_mut("ifccad::layout") {
            let value_state = if node.path == format!("{prefix}/layout/{}", d.model.id) {
                d.model.annotation_state
            } else {
                d.paper_layouts
                    .iter()
                    .find(|p| node.path == format!("{prefix}/layout/{}", p.id))
                    .and_then(|p| p.annotation_state)
            };
            if let Some(s) = value_state {
                value["annotationState"] = state(s, prefix);
            }
        }
    }
}
