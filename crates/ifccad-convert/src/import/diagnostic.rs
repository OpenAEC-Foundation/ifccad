use ifccad::ifcdr::{AppearanceId, EntityId, LayerId};
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ImportDiagnostic {
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
}

impl fmt::Display for ImportDiagnostic {
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
        }
    }
}

#[derive(Default)]
pub(crate) struct DiagnosticAccumulator {
    unmodeled_entities: BTreeMap<String, usize>,
    line_pattern_fallbacks: BTreeMap<(String, String), usize>,
    line_weight_rounding: BTreeMap<(u64, u64), usize>,
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

    pub(crate) fn finish(self) -> Vec<ImportDiagnostic> {
        let mut diagnostics = self
            .unmodeled_entities
            .into_iter()
            .map(
                |(schema_id, count)| ImportDiagnostic::UnmodeledEntitiesSkipped {
                    schema_id,
                    count,
                },
            )
            .collect::<Vec<_>>();
        diagnostics.extend(self.line_pattern_fallbacks.into_iter().map(
            |((requested, applied), count)| ImportDiagnostic::LinePatternFallback {
                requested,
                applied,
                count,
            },
        ));
        diagnostics.extend(self.line_weight_rounding.into_iter().map(
            |((requested_bits, applied_bits), count)| ImportDiagnostic::LineWeightRounded {
                requested_mm: f64::from_bits(requested_bits),
                applied_mm: f64::from_bits(applied_bits),
                count,
            },
        ));
        diagnostics
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImportError {
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
    use super::DiagnosticAccumulator;
    use crate::ImportDiagnostic;

    #[test]
    fn accumulator_groups_occurrences_in_stable_category_order() {
        let mut diagnostics = DiagnosticAccumulator::default();
        diagnostics.record_unmodeled("ifccad:arc.v1");
        diagnostics.record_unmodeled("ifccad:arc.v1");
        diagnostics.record_line_pattern_fallback("center", "Continuous");
        diagnostics.record_line_weight_rounding(0.19, 0.18);

        assert_eq!(
            diagnostics.finish(),
            [
                ImportDiagnostic::UnmodeledEntitiesSkipped {
                    schema_id: "ifccad:arc.v1".to_owned(),
                    count: 2,
                },
                ImportDiagnostic::LinePatternFallback {
                    requested: "center".to_owned(),
                    applied: "Continuous".to_owned(),
                    count: 1,
                },
                ImportDiagnostic::LineWeightRounded {
                    requested_mm: 0.19,
                    applied_mm: 0.18,
                    count: 1,
                },
            ]
        );
    }

    #[test]
    fn diagnostic_display_includes_the_actionable_loss_context() {
        let diagnostic = ImportDiagnostic::LineWeightRounded {
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
    fn diagnostic_display_uses_the_singular_entity_noun() {
        let entity_message = ImportDiagnostic::UnmodeledEntitiesSkipped {
            schema_id: "ifccad:arc.v1".to_owned(),
            count: 1,
        }
        .to_string();
        assert!(entity_message.contains("1 unmodeled entity"));
        assert!(!entity_message.contains("1 unmodeled entities"));
    }
}
