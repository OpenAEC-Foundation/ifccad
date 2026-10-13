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
    pub(crate) color: AppearanceColor,
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
    pub(crate) losses: Vec<CadToOcdrawLossReason>,
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
        value @ (Color::Index(_) | Color::Rgb { .. }) => {
            match cad_presentation_convert::explicit_color_from_cad(value) {
                Ok(value) => (AppearanceMode::Explicit, Some(native_color(value))),
                Err(_) => {
                    losses.push(CadToOcdrawLossReason::EntityColorUnsupported {
                        color: common.color.to_string(),
                    });
                    (AppearanceMode::Explicit, None)
                }
            }
        }
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

    let mut metadata_losses = Vec::new();
    if matches!(explicit_color, Some(AppearanceColor::Foreground)) && named.is_some() {
        metadata_losses.push(CadToOcdrawLossReason::EntityNamedColorUnsupported {
            name: common.color_name.clone().expect("named source"),
        });
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
        let mut color = explicit_color.unwrap_or_else(|| AppearanceColor::rgb(0, 0, 0));
        if let AppearanceColor::Concrete {
            named: identity, ..
        } = &mut color
        {
            *identity = named;
        }
        (
            AppearanceSignature {
                color: color.clone(),
                opacity: opacity.to_bits(),
                line_weight: line_weight.to_bits(),
            },
            color,
        )
    });
    Ok(ConvertedEntityAppearance {
        losses: metadata_losses,
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
    let color_components = cad_presentation_convert::explicit_color_from_cad(layer.color)
        .ok()
        .map(native_color);
    if color_components.is_none() {
        required_losses.push(CadToOcdrawLossReason::LayerColorUnsupported {
            color: layer.color.to_string(),
        });
    }

    let opacity = match layer.transparency {
        Transparency::Explicit(alpha) => Some(1.0 - f64::from(alpha) / 255.0),
        // On a layer, the codec's ByLayer value denotes the opaque default;
        // entity inheritance is handled separately above.
        Transparency::ByLayer => Some(1.0),
        Transparency::ByBlock => {
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

    let mut color = color_components.expect("validated layer color components");
    let opacity = opacity.expect("validated layer opacity");
    let line_weight = line_weight.expect("validated layer line weight");

    if let Some((catalog, name)) = named {
        match &mut color {
            AppearanceColor::Concrete { named, .. } => *named = Some((catalog, name)),
            AppearanceColor::Foreground => {
                losses.push(CadToOcdrawLossReason::NamedColorOnForeground { catalog, name })
            }
        }
    }
    Ok(ConvertedAppearance {
        signature: AppearanceSignature {
            color: color.clone(),
            opacity: opacity.to_bits(),
            line_weight: line_weight.to_bits(),
        },
        color,
        losses,
    })
}

fn native_color(value: cad_presentation_convert::CadColorValue) -> AppearanceColor {
    match value {
        cad_presentation_convert::CadColorValue::Foreground => AppearanceColor::Foreground,
        cad_presentation_convert::CadColorValue::Concrete {
            rgb,
            indexed,
            named,
        } => AppearanceColor::Concrete {
            rgb,
            indexed,
            named,
        },
    }
}
