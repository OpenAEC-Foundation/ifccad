//! Dimension presentations reuse the ordinary owned-picture projection.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Presentation {
    text_location: [f64; 2],
    user_text_location: bool,
    text_rotation: f64,
    layout: IfccadDimensionContextLayout,
    geometry: IfccadDimensionContextGeometry,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    graphics: Option<Value>,
}
pub(super) fn decode(
    value: &Value,
    prefix: &str,
    path: &str,
) -> Result<Box<IfccadAnnotation<IfccadDimensionAnnotationContext>>, IfccadReport> {
    let a = super::annotation::decode::<Presentation>(value, prefix, path)?;
    Ok(Box::new(IfccadAnnotation{orientation:a.orientation,mode:a.mode,reference:a.reference,contexts:a.contexts.into_iter().enumerate().map(|(i,c)|Ok(IfccadAnnotationContext{key:c.key,presentation:c.presentation.map(|p|Ok(IfccadDimensionAnnotationContext{text_location:p.text_location,user_text_location:p.user_text_location,text_rotation:p.text_rotation,layout:p.layout,geometry:p.geometry,graphics:p.graphics.map(|g|super::dimension_graphics::decode_inline(&g,prefix,&format!("{path}/ifccad::annotation/contexts/{i}/presentation/graphics"))).transpose()?})).transpose()?})).collect::<Result<_,IfccadReport>>()?}))
}
pub(super) fn encode(
    a: &IfccadAnnotation<IfccadDimensionAnnotationContext>,
    prefix: &str,
) -> Value {
    let wire = IfccadAnnotation {
        orientation: a.orientation,
        mode: a.mode,
        reference: a.reference,
        contexts: a
            .contexts
            .iter()
            .map(|c| IfccadAnnotationContext {
                key: c.key,
                presentation: c.presentation.as_ref().map(|p| Presentation {
                    text_location: p.text_location,
                    user_text_location: p.user_text_location,
                    text_rotation: p.text_rotation,
                    layout: p.layout,
                    geometry: p.geometry.clone(),
                    graphics: p
                        .graphics
                        .as_ref()
                        .map(|g| super::dimension_graphics::encode_inline(g, prefix)),
                }),
            })
            .collect(),
    };
    super::annotation::encode(&wire, prefix)
}
