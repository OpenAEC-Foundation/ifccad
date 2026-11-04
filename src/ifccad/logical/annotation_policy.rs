//! Saved annotation policy and scoped selections, without application updates.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadModelLinetypeAnnotationMode {
    Unscaled,
    ByAnnotationScale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadAnnotationVisibility {
    CurrentScaleOnly,
    All,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadDrawingAnnotationSettings {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    pub insert_as_annotative: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    pub model_linetype_scaling: Option<IfccadModelLinetypeAnnotationMode>,
}
impl IfccadDrawingAnnotationSettings {
    pub fn into_option(self) -> Option<Self> {
        (self.insert_as_annotative.is_some() || self.model_linetype_scaling.is_some())
            .then_some(self)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfccadLayoutAnnotationState {
    pub current_scale_id: Option<IfccadAnnotationScaleId>,
    pub visibility: Option<IfccadAnnotationVisibility>,
}
pub(super) fn validate(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let fail = |path: &str, message: &str| {
        crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-008", path, message)
    };
    if d.annotation_settings
        .is_some_and(|s| s.into_option().is_none())
    {
        return Err(fail(
            "/ifccad::drawing/annotationSettings",
            "empty annotation settings must be absent",
        ));
    }
    let known = d
        .annotation_scales
        .iter()
        .map(|s| s.id)
        .collect::<std::collections::BTreeSet<_>>();
    let check = |id: Option<IfccadAnnotationScaleId>, path: String| {
        if id.is_some_and(|id| !known.contains(&id)) {
            Err(fail(&path, "unresolved saved annotation scale selection"))
        } else {
            Ok(())
        }
    };
    check(
        d.model.annotation_state.and_then(|s| s.current_scale_id),
        format!("/layout/{}/annotationState/currentScale", d.model.id),
    )?;
    for p in &d.paper_layouts {
        check(
            p.annotation_state.and_then(|s| s.current_scale_id),
            format!("/layout/{}/annotationState/currentScale", p.id),
        )?;
    }
    for v in &d.annotation_views {
        check(
            v.annotation_scale_id,
            format!("/annotationView/{}/annotationScale", v.id.0),
        )?;
    }
    for e in d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
        .filter_map(IfccadEntity::as_native)
    {
        if let IfccadEntityKind::Viewport(v) = &e.kind {
            check(v.annotation_scale_id, format!("/e{}/annotationScale", e.id))?;
        }
    }
    Ok(())
}
