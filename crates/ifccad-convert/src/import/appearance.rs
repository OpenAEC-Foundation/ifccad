use super::diagnostic::DiagnosticAccumulator;
use cadcodec::{Color, LineWeight, Transparency};
use ifccad::package::{AppearanceColorRef, AppearanceProperty, LinePatternRef};

const LINE_WEIGHTS: [i16; 24] = [
    0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158, 200,
    211,
];

pub(crate) struct MappedColor {
    pub(crate) color: Color,
    pub(crate) name: Option<String>,
}

pub(crate) fn map_explicit_color(value: AppearanceColorRef<'_>) -> MappedColor {
    let [red, green, blue] = value.rgb().components();
    let color = value
        .indexed()
        .filter(|indexed| indexed.system().eq_ignore_ascii_case("ACI"))
        .and_then(|indexed| u8::try_from(indexed.index()).ok())
        .filter(|index| (1..=255).contains(index))
        .map(Color::Index)
        .unwrap_or_else(|| Color::from_rgb(red, green, blue));
    let name = value
        .named()
        .map(|named| format!("{}${}", named.catalog(), named.name()));
    MappedColor { color, name }
}

pub(crate) fn map_line_pattern(
    property: AppearanceProperty<LinePatternRef<'_>>,
    diagnostics: &mut DiagnosticAccumulator,
) -> String {
    use AppearanceProperty::{ByBlock, ByLayer, Explicit};

    match property {
        ByLayer => String::new(),
        ByBlock => "ByBlock".to_owned(),
        Explicit(pattern) => {
            let requested = match pattern {
                LinePatternRef::Name(name) | LinePatternRef::IfcxIdentity(name) => name,
            };
            if requested.eq_ignore_ascii_case("continuous") {
                "Continuous".to_owned()
            } else if requested.eq_ignore_ascii_case("dashed") {
                "Dashed".to_owned()
            } else {
                diagnostics.record_line_pattern_fallback(requested, "Continuous");
                "Continuous".to_owned()
            }
        }
    }
}

pub(crate) fn map_line_weight(
    property: AppearanceProperty<f64>,
    diagnostics: &mut DiagnosticAccumulator,
) -> LineWeight {
    use AppearanceProperty::{ByBlock, ByLayer, Explicit};

    match property {
        ByLayer => LineWeight::ByLayer,
        ByBlock => LineWeight::ByBlock,
        Explicit(requested_mm) => {
            let requested_hundredths = requested_mm * 100.0;
            let mut applied = LINE_WEIGHTS[0];
            let mut distance = (requested_hundredths - f64::from(applied)).abs();
            for candidate in LINE_WEIGHTS.into_iter().skip(1) {
                let candidate_distance = (requested_hundredths - f64::from(candidate)).abs();
                if candidate_distance < distance {
                    applied = candidate;
                    distance = candidate_distance;
                }
            }
            let applied_mm = f64::from(applied) / 100.0;
            if applied_mm != requested_mm {
                diagnostics.record_line_weight_rounding(requested_mm, applied_mm);
            }
            LineWeight::from_value(applied)
        }
    }
}

pub(crate) fn map_entity_opacity(property: AppearanceProperty<f64>) -> Transparency {
    use AppearanceProperty::{ByBlock, ByLayer, Explicit};

    match property {
        ByLayer => Transparency::BY_LAYER,
        ByBlock => Transparency::BY_BLOCK,
        Explicit(opacity) => Transparency::from_percent(1.0 - opacity),
    }
}

pub(crate) fn map_layer_opacity(opacity: f64) -> Transparency {
    Transparency::from_percent(1.0 - opacity)
}

#[cfg(test)]
mod tests {
    use super::super::diagnostic::DiagnosticAccumulator;
    use super::{map_entity_opacity, map_line_pattern, map_line_weight};
    use crate::ImportDiagnostic;
    use cadcodec::{LineWeight, Transparency};
    use ifccad::package::{AppearanceProperty, LinePatternRef};

    #[test]
    fn line_pattern_preserves_known_modes_and_groups_fallbacks() {
        use AppearanceProperty::{ByBlock, ByLayer, Explicit};

        let mut diagnostics = DiagnosticAccumulator::default();
        assert_eq!(map_line_pattern(ByLayer, &mut diagnostics), "");
        assert_eq!(map_line_pattern(ByBlock, &mut diagnostics), "ByBlock");
        assert_eq!(
            map_line_pattern(
                Explicit(LinePatternRef::Name("continuous")),
                &mut diagnostics
            ),
            "Continuous"
        );
        assert_eq!(
            map_line_pattern(Explicit(LinePatternRef::Name("dashed")), &mut diagnostics),
            "Dashed"
        );
        assert_eq!(
            map_line_pattern(Explicit(LinePatternRef::Name("center")), &mut diagnostics),
            "Continuous"
        );
        assert_eq!(
            map_line_pattern(Explicit(LinePatternRef::Name("center")), &mut diagnostics),
            "Continuous"
        );

        assert_eq!(
            diagnostics.finish(),
            [ImportDiagnostic::LinePatternFallback {
                requested: "center".to_owned(),
                applied: "Continuous".to_owned(),
                count: 2,
            }]
        );
    }

    #[test]
    fn line_weight_preserves_modes_and_rounds_ties_to_the_lighter_weight() {
        use AppearanceProperty::{ByBlock, ByLayer, Explicit};

        let mut diagnostics = DiagnosticAccumulator::default();
        assert_eq!(
            map_line_weight(ByLayer, &mut diagnostics),
            LineWeight::ByLayer
        );
        assert_eq!(
            map_line_weight(ByBlock, &mut diagnostics),
            LineWeight::ByBlock
        );
        assert_eq!(
            map_line_weight(Explicit(0.18), &mut diagnostics),
            LineWeight::W0_18
        );
        assert_eq!(
            map_line_weight(Explicit(0.19), &mut diagnostics),
            LineWeight::W0_18
        );
        assert_eq!(
            map_line_weight(Explicit(0.19), &mut diagnostics),
            LineWeight::W0_18
        );

        assert_eq!(
            diagnostics.finish(),
            [ImportDiagnostic::LineWeightRounded {
                requested_mm: 0.19,
                applied_mm: 0.18,
                count: 2,
            }]
        );
    }

    #[test]
    fn opacity_preserves_inheritance_and_explicit_values() {
        use AppearanceProperty::{ByBlock, ByLayer, Explicit};

        assert_eq!(map_entity_opacity(ByLayer), Transparency::BY_LAYER);
        assert_eq!(map_entity_opacity(ByBlock), Transparency::BY_BLOCK);
        assert_eq!(map_entity_opacity(Explicit(1.0)), Transparency::OPAQUE);
        assert_eq!(
            map_entity_opacity(Explicit(0.5)),
            Transparency::Explicit(127)
        );
    }
}
