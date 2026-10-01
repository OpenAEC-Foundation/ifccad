use crate::source::AppearanceMode;
use crate::source::ExportLossReason;
use crate::source::{
    convert_entity_appearance, convert_layer_appearance, EntityAppearanceError,
    LayerAppearanceError,
};
use cadcodec::entities::EntityCommon;
use cadcodec::Layer;
use ocdraw::ocdraw::DrawingColor as AppearanceColor;
use ocdraw::ocdraw::{
    AppearanceSelection, DrawingColor, EntityAppearance as DirectEntityAppearance,
    LayerDefinition as DirectLayerDefinition,
};

fn direct_color(source: &AppearanceColor) -> DrawingColor {
    source.clone()
}

pub(crate) fn direct_layer(
    layer: &Layer,
) -> Result<(DirectLayerDefinition, Vec<ExportLossReason>), Vec<ExportLossReason>> {
    let converted = match convert_layer_appearance(layer) {
        Ok(converted) => converted,
        Err(LayerAppearanceError::Loss(reasons)) => return Err(reasons),
    };
    if !converted
        .signature
        .line_pattern
        .eq_ignore_ascii_case("Continuous")
    {
        return Err(vec![ExportLossReason::UnsupportedSemantic {
            name: format!("layer linetype {}", converted.signature.line_pattern),
        }]);
    }
    let mut target = DirectLayerDefinition::new(&layer.name, direct_color(&converted.color));
    target.visible = !layer.flags.off;
    target.frozen = layer.flags.frozen;
    target.locked = layer.flags.locked;
    target.plottable = layer.is_plottable;
    target.frozen_in_new_viewports = layer.flags.frozen_in_new_viewport;
    target.description = (!layer.description.is_empty()).then(|| layer.description.clone());
    target.opacity = f64::from_bits(converted.signature.opacity);
    target.line_pattern = "Continuous".into();
    target.line_weight = f64::from_bits(converted.signature.line_weight);
    Ok((target, converted.losses))
}

fn direct_selection<T>(mode: AppearanceMode, value: T) -> AppearanceSelection<T> {
    match mode {
        AppearanceMode::ByLayer => AppearanceSelection::ByLayer,
        AppearanceMode::ByBlock => AppearanceSelection::ByBlock,
        AppearanceMode::Explicit => AppearanceSelection::Explicit(value),
    }
}

pub(crate) fn direct_entity(
    common: &EntityCommon,
) -> Result<DirectEntityAppearance, Vec<ExportLossReason>> {
    let converted = match convert_entity_appearance(common) {
        Ok(converted) => converted,
        Err(EntityAppearanceError::Loss(reasons)) => return Err(reasons),
    };
    let (color, opacity, line_pattern, line_weight) =
        if let Some((signature, color)) = converted.definition {
            (
                direct_color(&color),
                f64::from_bits(signature.opacity),
                signature.line_pattern,
                f64::from_bits(signature.line_weight),
            )
        } else {
            (DrawingColor::rgb(0, 0, 0), 1.0, "Continuous".into(), 0.25)
        };
    if converted.line_pattern_mode == AppearanceMode::Explicit
        && !line_pattern.eq_ignore_ascii_case("Continuous")
    {
        return Err(vec![ExportLossReason::UnsupportedSemantic {
            name: format!("entity linetype {line_pattern}"),
        }]);
    }
    Ok(DirectEntityAppearance {
        color: direct_selection(converted.color_mode, color),
        opacity: direct_selection(converted.opacity_mode, opacity),
        line_pattern: direct_selection(converted.line_pattern_mode, "Continuous".into()),
        line_weight: direct_selection(converted.line_weight_mode, line_weight),
    })
}
