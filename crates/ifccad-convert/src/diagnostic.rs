use ifccad::ifcdr::{AppearanceId, EntityId, LayerId};
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConversionDiagnostic {
    UnmodeledEntitiesSkipped {
        schema_id: String,
        count: usize,
    },
    LinePatternFallback {
        requested: String,
        applied: String,
        count: usize,
    },
    LineWeightRounded {
        requested_mm: f64,
        applied_mm: f64,
        count: usize,
    },
    TransparencySemanticsLost {
        source: LostTransparencyMode,
        count: usize,
    },
    NamedLayerColorIdentityLost {
        layer: String,
        catalog: String,
        name: String,
        count: usize,
    },
}

impl fmt::Display for ConversionDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmodeledEntitiesSkipped { schema_id, count } => {
                let noun = if *count == 1 { "entity" } else { "entities" };
                write!(
                    formatter,
                    "skipped {count} unmodeled {noun} with schema {schema_id}"
                )
            }
            Self::LinePatternFallback {
                requested,
                applied,
                count,
            } => {
                let noun = if *count == 1 { "entity" } else { "entities" };
                write!(
                    formatter,
                    "replaced line pattern {requested} with {applied} on {count} {noun}"
                )
            }
            Self::LineWeightRounded {
                requested_mm,
                applied_mm,
                count,
            } => {
                let noun = if *count == 1 { "entity" } else { "entities" };
                write!(
                    formatter,
                    "rounded line weight {requested_mm} mm to {applied_mm} mm on {count} {noun}"
                )
            }
            Self::TransparencySemanticsLost { source, count } => {
                let noun = if *count == 1 { "entity" } else { "entities" };
                write!(
                    formatter,
                    "lost {source:?} transparency semantics on {count} {noun}"
                )
            }
            Self::NamedLayerColorIdentityLost {
                layer,
                catalog,
                name,
                count,
            } => {
                let noun = if *count == 1 {
                    "occurrence"
                } else {
                    "occurrences"
                };
                write!(
                    formatter,
                    "preserved RGB but omitted named color {catalog}/{name} from layer {layer} ({count} {noun})"
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LostTransparencyMode {
    ByBlock,
    ExplicitOpaque,
}

#[derive(Default)]
pub(crate) struct DiagnosticAccumulator {
    unmodeled_entities: BTreeMap<String, usize>,
    line_pattern_fallbacks: BTreeMap<(String, String), usize>,
    line_weight_rounding: BTreeMap<(u64, u64), usize>,
    transparency_losses: BTreeMap<LostTransparencyMode, usize>,
    named_layer_colors: BTreeMap<(String, String, String), usize>,
}

impl DiagnosticAccumulator {
    pub(crate) fn record_unmodeled(&mut self, schema_id: &str) {
        *self
            .unmodeled_entities
            .entry(schema_id.to_owned())
            .or_default() += 1;
    }

    pub(crate) fn record_line_pattern_fallback(&mut self, requested: &str, applied: &str) {
        *self
            .line_pattern_fallbacks
            .entry((requested.to_owned(), applied.to_owned()))
            .or_default() += 1;
    }

    pub(crate) fn record_line_weight_rounding(&mut self, requested_mm: f64, applied_mm: f64) {
        *self
            .line_weight_rounding
            .entry((requested_mm.to_bits(), applied_mm.to_bits()))
            .or_default() += 1;
    }

    pub(crate) fn record_transparency_loss(&mut self, source: LostTransparencyMode) {
        *self.transparency_losses.entry(source).or_default() += 1;
    }

    pub(crate) fn record_named_layer_color(&mut self, layer: &str, catalog: &str, name: &str) {
        *self
            .named_layer_colors
            .entry((layer.to_owned(), catalog.to_owned(), name.to_owned()))
            .or_default() += 1;
    }

    pub(crate) fn finish(self) -> Vec<ConversionDiagnostic> {
        let mut diagnostics = self
            .unmodeled_entities
            .into_iter()
            .map(
                |(schema_id, count)| ConversionDiagnostic::UnmodeledEntitiesSkipped {
                    schema_id,
                    count,
                },
            )
            .collect::<Vec<_>>();
        diagnostics.extend(self.line_pattern_fallbacks.into_iter().map(
            |((requested, applied), count)| ConversionDiagnostic::LinePatternFallback {
                requested,
                applied,
                count,
            },
        ));
        diagnostics.extend(self.line_weight_rounding.into_iter().map(
            |((requested_bits, applied_bits), count)| ConversionDiagnostic::LineWeightRounded {
                requested_mm: f64::from_bits(requested_bits),
                applied_mm: f64::from_bits(applied_bits),
                count,
            },
        ));
        diagnostics.extend(self.transparency_losses.into_iter().map(|(source, count)| {
            ConversionDiagnostic::TransparencySemanticsLost { source, count }
        }));
        diagnostics.extend(self.named_layer_colors.into_iter().map(
            |((layer, catalog, name), count)| ConversionDiagnostic::NamedLayerColorIdentityLost {
                layer,
                catalog,
                name,
                count,
            },
        ));
        diagnostics
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConversionError {
    #[error(
        "drawing must contain exactly one model layout (found {total_layouts} layouts, {model_layouts} model layouts)"
    )]
    UnsupportedDrawingStructure {
        total_layouts: usize,
        model_layouts: usize,
    },
    #[error("entity {entity_id:?} refers to missing layer {layer_id:?}")]
    MissingEntityLayer {
        entity_id: EntityId,
        layer_id: LayerId,
    },
    #[error("entity {entity_id:?} refers to missing appearance {appearance_id:?}")]
    MissingEntityAppearance {
        entity_id: EntityId,
        appearance_id: AppearanceId,
    },
    #[error("cadcodec could not insert converted entity {entity_id:?}")]
    CadcodecEntityInsertion {
        entity_id: EntityId,
        #[source]
        source: cadcodec::DxfError,
    },
    #[error("cadcodec could not insert layer {layer}: {reason}")]
    LayerInsertion { layer: String, reason: String },
    #[error("internal conversion invariant failed: {message}")]
    InternalInvariant { message: String },
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticAccumulator, LostTransparencyMode};
    use crate::ConversionDiagnostic;

    #[test]
    fn accumulator_groups_occurrences_in_stable_category_order() {
        let mut diagnostics = DiagnosticAccumulator::default();
        diagnostics.record_unmodeled("ifccad:arc.v1");
        diagnostics.record_unmodeled("ifccad:arc.v1");
        diagnostics.record_line_pattern_fallback("center", "Continuous");
        diagnostics.record_line_weight_rounding(0.19, 0.18);
        diagnostics.record_transparency_loss(LostTransparencyMode::ByBlock);
        diagnostics.record_named_layer_color("A-WALL", "RAL", "Traffic red");
        diagnostics.record_named_layer_color("A-WALL", "RAL", "Traffic red");

        assert_eq!(
            diagnostics.finish(),
            [
                ConversionDiagnostic::UnmodeledEntitiesSkipped {
                    schema_id: "ifccad:arc.v1".to_owned(),
                    count: 2,
                },
                ConversionDiagnostic::LinePatternFallback {
                    requested: "center".to_owned(),
                    applied: "Continuous".to_owned(),
                    count: 1,
                },
                ConversionDiagnostic::LineWeightRounded {
                    requested_mm: 0.19,
                    applied_mm: 0.18,
                    count: 1,
                },
                ConversionDiagnostic::TransparencySemanticsLost {
                    source: LostTransparencyMode::ByBlock,
                    count: 1,
                },
                ConversionDiagnostic::NamedLayerColorIdentityLost {
                    layer: "A-WALL".to_owned(),
                    catalog: "RAL".to_owned(),
                    name: "Traffic red".to_owned(),
                    count: 2,
                },
            ]
        );
    }

    #[test]
    fn diagnostic_display_includes_the_actionable_loss_context() {
        let diagnostic = ConversionDiagnostic::LineWeightRounded {
            requested_mm: 0.19,
            applied_mm: 0.18,
            count: 2,
        };

        let message = diagnostic.to_string();
        assert!(message.contains("0.19 mm"));
        assert!(message.contains("0.18 mm"));
        assert!(message.contains("2 entities"));
    }

    #[test]
    fn diagnostic_display_uses_singular_nouns_for_one_occurrence() {
        let entity_message = ConversionDiagnostic::TransparencySemanticsLost {
            source: LostTransparencyMode::ExplicitOpaque,
            count: 1,
        }
        .to_string();
        assert!(entity_message.contains("1 entity"));
        assert!(!entity_message.contains("1 entities"));

        let layer_message = ConversionDiagnostic::NamedLayerColorIdentityLost {
            layer: "A-WALL".to_owned(),
            catalog: "RAL".to_owned(),
            name: "Traffic red".to_owned(),
            count: 1,
        }
        .to_string();
        assert!(layer_message.contains("1 occurrence"));
        assert!(!layer_message.contains("1 occurrences"));
    }
}
