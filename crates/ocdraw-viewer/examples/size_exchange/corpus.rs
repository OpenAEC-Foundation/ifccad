use super::Result;
use ocdraw::ifcx_cad::*;
use ocdraw_convert::opencadcodec::CadDocument;
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub family: String,
    pub count: usize,
}
pub fn cases(path: &Path) -> Result<Vec<Case>> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Corpus {
        version: u32,
        unit: String,
        cases: Vec<Case>,
    }
    let corpus: Corpus = serde_json::from_slice(&std::fs::read(path)?)?;
    if corpus.version != 1 || corpus.unit != "mm" || corpus.cases.is_empty() {
        return Err("unsupported/empty corpus".into());
    }
    let mut names = std::collections::BTreeSet::new();
    for case in &corpus.cases {
        check(case)?;
        if !names.insert(&case.id) {
            return Err("duplicate case ID".into());
        }
    }
    Ok(corpus.cases)
}
fn check(case: &Case) -> Result<()> {
    if case.id.is_empty()
        || !case
            .id
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || v == b'-')
        || case.count > 10000
        || !matches!(
            case.family.as_str(),
            "line" | "fractional" | "polyline" | "circle" | "blocks" | "patterns"
        )
    {
        return Err("invalid case".into());
    }
    if case.family == "blocks" && case.count != 3 {
        return Err("block recipe requires three model occurrences".into());
    }
    Ok(())
}
fn placement(origin: [f64; 3]) -> IfcxCadPlacement {
    IfcxCadPlacement {
        origin,
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn entity(id: u64, i: usize, kind: IfcxCadEntityKind) -> IfcxCadEntity {
    let appearance = if i.is_multiple_of(3) {
        IfcxCadEntityAppearance {
            color: IfcxCadMode::Explicit("#B4283C".into()),
            opacity: IfcxCadMode::Explicit(1.),
            line_pattern: IfcxCadMode::Explicit(IfcxCadLinePatternId(1)),
            line_weight: IfcxCadMode::Explicit(0.5),
        }
    } else {
        IfcxCadEntityAppearance {
            color: IfcxCadMode::ByLayer,
            opacity: IfcxCadMode::ByBlock,
            line_pattern: IfcxCadMode::ByLayer,
            line_weight: IfcxCadMode::ByLayer,
        }
    };
    IfcxCadEntity {
        id,
        layer_id: if i.is_multiple_of(2) { 1 } else { 2 },
        appearance,
        line_pattern_scale: 1.,
        kind,
    }
}
pub fn generate(case: &Case) -> Result<CadDocument> {
    check(case)?;
    let layer = |id, name: &str| IfcxCadLayer {
        id,
        name: name.into(),
        appearance: IfcxCadLayerAppearance {
            color: "#0A141E".into(),
            opacity: 1.,
            line_pattern: IfcxCadLinePatternId(1),
            line_weight: 0.25,
        },
    };
    let mut doc = IfcxCadDocument {
        header: super::projection::metadata().header,
        drawing_id: 1,
        id_counters: IfcxCadIdCounters {
            next_entity_id: 1,
            next_layer_id: 3,
            next_layout_id: 2,
            next_block_id: 1,
            next_line_pattern_id: 2,
        },
        length_unit: "mm".into(),
        line_patterns: vec![IfcxCadLinePattern {
            id: IfcxCadLinePatternId(1),
            name: "Continuous".into(),
            description: Some("Solid line".into()),
            pattern: vec![],
        }],
        line_pattern_scale: 1.,
        layers: vec![layer(1, "0"), layer(2, "Details")],
        model: IfcxCadLayout {
            id: 1,
            entities: vec![],
        },
        paper_layouts: vec![],
        blocks: vec![],
    };
    if case.family == "patterns" {
        for (id, name, pattern) in [
            (2, "Dashed", vec![0.5, -0.25]),
            (3, "Dotted", vec![0., -0.125]),
            (4, "Unused", vec![0.75, -0.25]),
        ] {
            doc.line_patterns.push(IfcxCadLinePattern {
                id: IfcxCadLinePatternId(id),
                name: name.into(),
                description: Some(format!("{name} test pattern")),
                pattern,
            });
        }
        doc.id_counters.next_line_pattern_id = 5;
        doc.line_pattern_scale = 2.;
        doc.layers[1].appearance.line_pattern = IfcxCadLinePatternId(2);
    }
    let mut next = 1;
    if case.family == "blocks" {
        let insert = |id, i, target, origin, scale| {
            entity(
                id,
                i,
                IfcxCadEntityKind::BlockInstance {
                    definition_id: target,
                    transform: IfcxCadBlockTransform {
                        placement: placement(origin),
                        rotation: 0.,
                        scale,
                    },
                },
            )
        };
        doc.blocks = vec![
            IfcxCadBlockDefinition {
                id: 1,
                name: "Core".into(),
                base_point: [1., 2., 0.],
                insertion_unit: "mm".into(),
                entities: vec![
                    entity(
                        1,
                        0,
                        IfcxCadEntityKind::LineSegment {
                            start: [1., 2., 0.],
                            end: [9., 6., 0.],
                        },
                    ),
                    entity(
                        2,
                        1,
                        IfcxCadEntityKind::Circle {
                            radius: 2.,
                            placement: placement([4., 4., 0.]),
                        },
                    ),
                ],
            },
            IfcxCadBlockDefinition {
                id: 2,
                name: "Assembly".into(),
                base_point: [2., 1., 0.],
                insertion_unit: "mm".into(),
                entities: vec![
                    insert(3, 0, 1, [10., 20., 0.], [2., 1., 1.]),
                    insert(4, 1, 1, [30., 40., 0.], [-1., 0.5, 1.]),
                ],
            },
            IfcxCadBlockDefinition {
                id: 3,
                name: "Unused".into(),
                base_point: [0.; 3],
                insertion_unit: "mm".into(),
                entities: vec![entity(
                    5,
                    0,
                    IfcxCadEntityKind::LineSegment {
                        start: [0.; 3],
                        end: [1., 1., 0.],
                    },
                )],
            },
        ];
        for i in 0..3 {
            doc.model.entities.push(insert(
                6 + i as u64,
                i,
                2,
                [i as f64 * 100., 0., 0.],
                [1. + i as f64, 1., 1.],
            ));
        }
        next = 9;
        doc.id_counters.next_block_id = 4;
    } else {
        for i in 0..case.count {
            let x = (i % 100) as f64 * 16.;
            let y = (i / 100) as f64 * 16.;
            let kind = match case.family.as_str() {
                "circle" => IfcxCadEntityKind::Circle {
                    radius: 2. + (i % 8) as f64 / 8.,
                    placement: placement([x, y, 0.]),
                },
                "polyline" | "patterns" if case.family == "polyline" || i % 2 == 1 => {
                    IfcxCadEntityKind::PlanarPolyline {
                        vertices: vec![[x, y], [x + 8., y], [x + 8., y + 4.], [x, y + 4.]],
                        closed: i.is_multiple_of(2),
                        placement: placement([0.; 3]),
                        line_pattern_generation: if i.is_multiple_of(3) {
                            IfcxCadLinePatternGeneration::Continuous
                        } else {
                            IfcxCadLinePatternGeneration::PerSegment
                        },
                    }
                }
                _ => {
                    let fraction = if case.family == "fractional" {
                        ((i * 37) % 1024) as f64 / 1024.
                    } else {
                        0.
                    };
                    IfcxCadEntityKind::LineSegment {
                        start: [x + fraction, y, 0.],
                        end: [x + fraction + 8.125, y + 4.0625, 0.],
                    }
                }
            };
            let mut e = entity(next, i, kind);
            if case.family == "patterns" {
                e.line_pattern_scale = 0.5;
                if i.is_multiple_of(3) {
                    e.appearance.line_pattern = IfcxCadMode::Explicit(IfcxCadLinePatternId(3));
                }
            }
            doc.model.entities.push(e);
            next += 1;
        }
    }
    doc.id_counters.next_entity_id = next;
    Ok(ifcx_cad_convert::ifcx_cad_document_to_cad_document(
        &doc,
        ifcx_cad_convert::IfcxCadToCadOptions {
            loss_policy: ifcx_cad_convert::IfcxCadLossPolicy::Reject,
        },
    )?
    .into_document())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_recipes_have_expected_inventories() {
        for (family, count, want) in [
            ("line", 3, 3),
            ("polyline", 4, 4),
            ("circle", 2, 2),
            ("blocks", 3, 3),
            ("patterns", 128, 128),
        ] {
            let cad = generate(&Case {
                id: "test".into(),
                family: family.into(),
                count,
            })
            .unwrap();
            let native = ifcx_cad_convert::cad_document_to_ifcx_cad_document(
                &cad,
                super::super::projection::metadata(),
                ifcx_cad_convert::CadToIfcxCadOptions {
                    loss_policy: ifcx_cad_convert::IfcxCadLossPolicy::Reject,
                },
            )
            .unwrap();
            assert_eq!(native.document().model.entities.len(), want);
        }
    }
    #[test]
    fn invalid_case_cannot_escape_output_directory() {
        assert!(generate(&Case {
            id: "../escape".into(),
            family: "line".into(),
            count: 1
        })
        .is_err());
    }
}
