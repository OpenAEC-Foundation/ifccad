use super::CadToOcdrawLossReason;
use ocdraw::ocdraw::DrawingColor as AppearanceColor;
use opencadcodec::entities::EntityCommon;
use opencadcodec::{Color, Layer, LineWeight, Transparency};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppearanceMode {
    ByLayer,
    ByBlock,
    Explicit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppearanceSignature {
    pub(crate) rgb: [u8; 3],
    pub(crate) indexed: Option<u32>,
    pub(crate) named: Option<(String, String)>,
    pub(crate) opacity: u64,
    pub(crate) line_weight: u64,
}

pub(crate) struct ConvertedAppearance {
    pub(crate) signature: AppearanceSignature,
    pub(crate) color: AppearanceColor,
    pub(crate) losses: Vec<CadToOcdrawLossReason>,
}

pub(crate) enum LayerAppearanceError {
    Loss(Vec<CadToOcdrawLossReason>),
}

pub(crate) enum EntityAppearanceError {
    Loss(Vec<CadToOcdrawLossReason>),
}

pub(crate) struct ConvertedEntityAppearance {
    pub(crate) definition: Option<(AppearanceSignature, AppearanceColor)>,
    pub(crate) color_mode: AppearanceMode,
    pub(crate) opacity_mode: AppearanceMode,
    pub(crate) line_weight_mode: AppearanceMode,
}

pub(crate) fn convert_entity_appearance(
    common: &EntityCommon,
) -> Result<ConvertedEntityAppearance, EntityAppearanceError> {
    let mut losses = Vec::new();
    let (color_mode, explicit_color) = match common.color {
        Color::ByLayer => (AppearanceMode::ByLayer, None),
        Color::ByBlock => (AppearanceMode::ByBlock, None),
        Color::Index(index) => match common.color.rgb() {
            Some((r, g, b)) => (
                AppearanceMode::Explicit,
                Some(([r, g, b], Some(u32::from(index)))),
            ),
            None => {
                losses.push(CadToOcdrawLossReason::EntityColorUnsupported {
                    color: common.color.to_string(),
                });
                (AppearanceMode::Explicit, None)
            }
        },
        Color::Rgb { r, g, b } => (AppearanceMode::Explicit, Some(([r, g, b], None))),
        _ => {
            losses.push(CadToOcdrawLossReason::EntityColorUnsupported {
                color: common.color.to_string(),
            });
            (AppearanceMode::Explicit, None)
        }
    };

    let named = match &common.color_name {
        None => None,
        Some(value) => match value.split_once('$') {
            Some((catalog, name)) if !catalog.is_empty() && !name.is_empty() => {
                Some((catalog.to_owned(), name.to_owned()))
            }
            _ => {
                losses.push(CadToOcdrawLossReason::EntityNamedColorUnsupported {
                    name: value.clone(),
                });
                None
            }
        },
    };
    if named.is_some() && explicit_color.is_none() {
        losses.push(CadToOcdrawLossReason::EntityNamedColorWithoutExplicitColor);
    }

    let (opacity_mode, opacity) = match common.transparency {
        Transparency::ByLayer => (AppearanceMode::ByLayer, 1.0),
        Transparency::ByBlock => (AppearanceMode::ByBlock, 1.0),
        Transparency::Explicit(alpha) => (AppearanceMode::Explicit, 1.0 - f64::from(alpha) / 255.0),
    };
    let (line_weight_mode, line_weight) = match common.line_weight {
        LineWeight::ByLayer => (AppearanceMode::ByLayer, 0.25),
        LineWeight::ByBlock => (AppearanceMode::ByBlock, 0.25),
        LineWeight::Default => (AppearanceMode::Explicit, 0.25),
        LineWeight::Value(value) if value >= 0 => {
            (AppearanceMode::Explicit, f64::from(value) / 100.0)
        }
        _ => {
            losses.push(CadToOcdrawLossReason::EntityLineWeightUnsupported {
                value: common.line_weight.value(),
            });
            (AppearanceMode::Explicit, 0.25)
        }
    };
    if !losses.is_empty() {
        return Err(EntityAppearanceError::Loss(losses));
    }

    let needs_definition =
        [color_mode, opacity_mode, line_weight_mode].contains(&AppearanceMode::Explicit);
    let definition = needs_definition.then(|| {
        let (rgb, indexed) = explicit_color.unwrap_or(([0, 0, 0], None));
        let mut color = AppearanceColor::rgb(rgb[0], rgb[1], rgb[2]);
        if let Some(index) = indexed {
            color = color.with_indexed("ACI", u64::from(index));
        }
        if let Some((catalog, name)) = &named {
            color = color.with_named(catalog, name);
        }
        (
            AppearanceSignature {
                rgb,
                indexed,
                named,
                opacity: opacity.to_bits(),
                line_weight: line_weight.to_bits(),
            },
            color,
        )
    });
    Ok(ConvertedEntityAppearance {
        definition,
        color_mode,
        opacity_mode,
        line_weight_mode,
    })
}

pub(crate) fn convert_layer_appearance(
    layer: &Layer,
) -> Result<ConvertedAppearance, LayerAppearanceError> {
    let mut required_losses = Vec::new();
    let color_components = match layer.color {
        Color::Index(index) => layer
            .color
            .rgb()
            .map(|(r, g, b)| ([r, g, b], Some(u32::from(index)))),
        Color::Rgb { r, g, b } => Some(([r, g, b], None)),
        _ => None,
    };
    if color_components.is_none() {
        required_losses.push(CadToOcdrawLossReason::LayerColorUnsupported {
            color: layer.color.to_string(),
        });
    }

    let opacity = match layer.transparency {
        Transparency::Explicit(alpha) => Some(1.0 - f64::from(alpha) / 255.0),
        _ => {
            required_losses.push(CadToOcdrawLossReason::LayerTransparencyUnsupported);
            None
        }
    };
    if layer.line_type.is_empty() {
        required_losses.push(CadToOcdrawLossReason::LayerLinePatternMissing);
    }
    let line_weight = match layer.line_weight {
        LineWeight::Value(value) if value >= 0 => Some(f64::from(value) / 100.0),
        LineWeight::Default => Some(0.25),
        _ => {
            required_losses.push(CadToOcdrawLossReason::LayerLineWeightUnsupported {
                value: layer.line_weight.value(),
            });
            None
        }
    };

    let mut losses = Vec::new();
    let named = match (&layer.book_name, &layer.color_name) {
        (None, None) => None,
        (Some(catalog), Some(name)) if !catalog.is_empty() && !name.is_empty() => {
            Some((catalog.clone(), name.clone()))
        }
        _ => {
            losses.push(CadToOcdrawLossReason::NamedColorIdentityIncomplete {
                color_name: layer.color_name.clone(),
                book_name: layer.book_name.clone(),
            });
            None
        }
    };
    if !required_losses.is_empty() {
        required_losses.extend(losses);
        return Err(LayerAppearanceError::Loss(required_losses));
    }

    let (rgb, indexed) = color_components.expect("validated layer color components");
    let opacity = opacity.expect("validated layer opacity");
    let line_weight = line_weight.expect("validated layer line weight");

    let mut color = AppearanceColor::rgb(rgb[0], rgb[1], rgb[2]);
    if let Some(index) = indexed {
        color = color.with_indexed("ACI", u64::from(index));
    }
    if let Some((catalog, name)) = &named {
        color = color.with_named(catalog, name);
    }
    Ok(ConvertedAppearance {
        signature: AppearanceSignature {
            rgb,
            indexed,
            named,
            opacity: opacity.to_bits(),
            line_weight: line_weight.to_bits(),
        },
        color,
        losses,
    })
}
