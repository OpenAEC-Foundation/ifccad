use super::ExportLossReason;
use cadcodec::entities::EntityCommon;
use cadcodec::{Color, Layer, LineWeight, Transparency};
use ifccad::package::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, AppearanceMode, DrawingBuilder,
    EntityAppearance, LinePatternDefinition, PackageBuildError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct AppearanceSignature {
    rgb: [u8; 3],
    indexed: Option<u32>,
    named: Option<(String, String)>,
    opacity: u64,
    line_pattern: String,
    line_weight: u64,
}

#[derive(Default)]
pub(crate) struct AppearanceRegistry {
    entries: Vec<(AppearanceSignature, AppearanceKey)>,
}

impl AppearanceRegistry {
    pub(crate) fn add_layer_appearance(
        &mut self,
        drawing: &mut DrawingBuilder<'_>,
        layer: &Layer,
    ) -> Result<(AppearanceKey, Vec<ExportLossReason>), LayerAppearanceError> {
        let converted = convert_layer_appearance(layer)?;
        if let Some((_, key)) = self
            .entries
            .iter()
            .find(|(signature, _)| signature == &converted.signature)
        {
            return Ok((*key, converted.losses));
        }

        let key = drawing
            .appearances()
            .add(AppearanceDefinition {
                name: format!("CAD appearance {}", self.entries.len()),
                color: converted.color,
                opacity: f64::from_bits(converted.signature.opacity),
                line_pattern: LinePatternDefinition::named(
                    converted.signature.line_pattern.clone(),
                ),
                line_weight: f64::from_bits(converted.signature.line_weight),
            })
            .map_err(LayerAppearanceError::Build)?;
        self.entries.push((converted.signature, key));
        Ok((key, converted.losses))
    }

    pub(crate) fn add_entity_appearance(
        &mut self,
        drawing: &mut DrawingBuilder<'_>,
        common: &EntityCommon,
    ) -> Result<EntityAppearance, EntityAppearanceError> {
        let converted = convert_entity_appearance(common)?;
        let Some((signature, color)) = converted.definition else {
            return Ok(EntityAppearance {
                appearance: None,
                color_mode: converted.color_mode,
                opacity_mode: converted.opacity_mode,
                line_pattern_mode: converted.line_pattern_mode,
                line_weight_mode: converted.line_weight_mode,
            });
        };
        let key = if let Some((_, key)) = self
            .entries
            .iter()
            .find(|(existing, _)| existing == &signature)
        {
            *key
        } else {
            let key = drawing
                .appearances()
                .add(AppearanceDefinition {
                    name: format!("CAD appearance {}", self.entries.len()),
                    color,
                    opacity: f64::from_bits(signature.opacity),
                    line_pattern: LinePatternDefinition::named(signature.line_pattern.clone()),
                    line_weight: f64::from_bits(signature.line_weight),
                })
                .map_err(EntityAppearanceError::Build)?;
            self.entries.push((signature, key));
            key
        };
        Ok(EntityAppearance {
            appearance: Some(key),
            color_mode: converted.color_mode,
            opacity_mode: converted.opacity_mode,
            line_pattern_mode: converted.line_pattern_mode,
            line_weight_mode: converted.line_weight_mode,
        })
    }
}

struct ConvertedAppearance {
    signature: AppearanceSignature,
    color: AppearanceColor,
    losses: Vec<ExportLossReason>,
}

pub(crate) enum LayerAppearanceError {
    Loss(Vec<ExportLossReason>),
    Build(PackageBuildError),
}

pub(crate) enum EntityAppearanceError {
    Loss(Vec<ExportLossReason>),
    Build(PackageBuildError),
}

struct ConvertedEntityAppearance {
    definition: Option<(AppearanceSignature, AppearanceColor)>,
    color_mode: AppearanceMode,
    opacity_mode: AppearanceMode,
    line_pattern_mode: AppearanceMode,
    line_weight_mode: AppearanceMode,
}

fn convert_entity_appearance(
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
                losses.push(ExportLossReason::EntityColorUnsupported {
                    color: common.color.to_string(),
                });
                (AppearanceMode::Explicit, None)
            }
        },
        Color::Rgb { r, g, b } => (AppearanceMode::Explicit, Some(([r, g, b], None))),
        _ => {
            losses.push(ExportLossReason::EntityColorUnsupported {
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
                losses.push(ExportLossReason::EntityNamedColorUnsupported {
                    name: value.clone(),
                });
                None
            }
        },
    };
    if named.is_some() && explicit_color.is_none() {
        losses.push(ExportLossReason::EntityNamedColorWithoutExplicitColor);
    }

    let (opacity_mode, opacity) = match common.transparency {
        Transparency::ByLayer => (AppearanceMode::ByLayer, 1.0),
        Transparency::ByBlock => (AppearanceMode::ByBlock, 1.0),
        Transparency::Explicit(alpha) => (AppearanceMode::Explicit, 1.0 - f64::from(alpha) / 255.0),
    };
    let (line_pattern_mode, line_pattern) =
        if common.linetype.is_empty() || common.linetype.eq_ignore_ascii_case("ByLayer") {
            (AppearanceMode::ByLayer, "Continuous".to_owned())
        } else if common.linetype.eq_ignore_ascii_case("ByBlock") {
            (AppearanceMode::ByBlock, "Continuous".to_owned())
        } else {
            (AppearanceMode::Explicit, common.linetype.clone())
        };
    let (line_weight_mode, line_weight) = match common.line_weight {
        LineWeight::ByLayer => (AppearanceMode::ByLayer, 0.25),
        LineWeight::ByBlock => (AppearanceMode::ByBlock, 0.25),
        LineWeight::Default => (AppearanceMode::Explicit, 0.25),
        LineWeight::Value(value) if value >= 0 => {
            (AppearanceMode::Explicit, f64::from(value) / 100.0)
        }
        _ => {
            losses.push(ExportLossReason::EntityLineWeightUnsupported {
                value: common.line_weight.value(),
            });
            (AppearanceMode::Explicit, 0.25)
        }
    };
    if !losses.is_empty() {
        return Err(EntityAppearanceError::Loss(losses));
    }

    let needs_definition = [
        color_mode,
        opacity_mode,
        line_pattern_mode,
        line_weight_mode,
    ]
    .contains(&AppearanceMode::Explicit);
    let definition = needs_definition.then(|| {
        let (rgb, indexed) = explicit_color.unwrap_or(([0, 0, 0], None));
        let mut color = AppearanceColor::rgb(rgb[0], rgb[1], rgb[2]);
        if let Some(index) = indexed {
            color = color.with_indexed("ACI", index);
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
                line_pattern,
                line_weight: line_weight.to_bits(),
            },
            color,
        )
    });
    Ok(ConvertedEntityAppearance {
        definition,
        color_mode,
        opacity_mode,
        line_pattern_mode,
        line_weight_mode,
    })
}

fn convert_layer_appearance(layer: &Layer) -> Result<ConvertedAppearance, LayerAppearanceError> {
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
        required_losses.push(ExportLossReason::LayerColorUnsupported {
            color: layer.color.to_string(),
        });
    }

    let opacity = match layer.transparency {
        Transparency::Explicit(alpha) => Some(1.0 - f64::from(alpha) / 255.0),
        _ => {
            required_losses.push(ExportLossReason::LayerTransparencyUnsupported);
            None
        }
    };
    if layer.line_type.is_empty() {
        required_losses.push(ExportLossReason::LayerLinePatternMissing);
    }
    let line_weight = match layer.line_weight {
        LineWeight::Value(value) if value >= 0 => Some(f64::from(value) / 100.0),
        LineWeight::Default => Some(0.25),
        _ => {
            required_losses.push(ExportLossReason::LayerLineWeightUnsupported {
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
            losses.push(ExportLossReason::NamedColorIdentityIncomplete {
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
        color = color.with_indexed("ACI", index);
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
            line_pattern: layer.line_type.clone(),
            line_weight: line_weight.to_bits(),
        },
        color,
        losses,
    })
}
