//! Add a small pattern palette to the two independently authored overviews.
use ifccad::{line_pattern::*, text::*, *};
use std::error::Error;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn properties() -> TextStyleProperties {
    let mut p = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    p.last_used_height = Some(2.5);
    p
}
fn palette<M: Clone>(
    text: M,
    standard: M,
    custom: M,
) -> Vec<(&'static str, Vec<LinePatternElement<M>>)> {
    let sequence = |marker| {
        vec![
            LinePatternElement::simple(10.),
            LinePatternElement {
                length: 0.,
                marker: Some(marker),
            },
            LinePatternElement::simple(-5.),
        ]
    };
    vec![
        ("GAS", sequence(text.clone())),
        ("Boxes", sequence(standard)),
        (
            "Mixed",
            vec![
                LinePatternElement::simple(10.),
                LinePatternElement {
                    length: 0.,
                    marker: Some(text),
                },
                LinePatternElement::simple(-4.),
                LinePatternElement {
                    length: 0.,
                    marker: Some(custom.clone()),
                },
                LinePatternElement::simple(-4.),
            ],
        ),
        (
            "Unused symbols",
            vec![LinePatternElement {
                length: 1.,
                marker: Some(custom),
            }],
        ),
    ]
}

pub fn ifccad(mut d: IfccadDocument) -> Result<IfccadDocument> {
    let style = d.id_counters.allocate_text_style_id()?;
    d.text_styles.push(IfccadTextStyle {
        id: style,
        name: "Pattern text".into(),
        properties: properties(),
    });
    let mut sources = Vec::new();
    for file_name in ["ltypeshp.shx", "custom-missing.shx", "custom-missing.shx"] {
        let id = d.id_counters.allocate_symbol_source_id()?;
        d.symbol_sources.push(IfccadSymbolSource {
            id,
            source: SymbolSource::Shx {
                file_name: file_name.into(),
            },
        });
        sources.push(id);
    }
    let text = IfccadLinePatternMarker::Text {
        text: "GAS".into(),
        text_style_id: style,
        transform: LinePatternTransform {
            scale: 2.,
            ..Default::default()
        },
    };
    let symbol = |source, shape| IfccadLinePatternMarker::Symbol {
        symbol_source_id: source,
        shape_number: shape,
        transform: LinePatternTransform {
            scale: 2.,
            ..Default::default()
        },
    };
    let mut ids = Vec::new();
    for (name, pattern) in palette(text, symbol(sources[0], 132), symbol(sources[1], 42)) {
        let id = d.id_counters.allocate_line_pattern_id()?;
        d.line_patterns.push(IfccadLinePattern {
            id,
            name: name.into(),
            description: Some("Text or external SHX glyph request".into()),
            pattern,
        });
        ids.push(id);
    }
    let mut layer = d.layers[0].clone();
    layer.id = d.id_counters.allocate_layer_id()?;
    layer.name = "Line patterns".into();
    layer.appearance.color = IfccadColor::Foreground;
    layer.appearance.line_pattern = ids[0];
    let layer_id = layer.id;
    d.layers.push(layer);
    let appearance = |pattern| IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByLayer,
        line_pattern: IfccadMode::Explicit(pattern),
        line_weight: IfccadMode::ByLayer,
    };
    for (i, pattern) in ids[..3].iter().enumerate() {
        let id = d.id_counters.allocate_entity_id()?;
        d.model
            .entities
            .push(IfccadEntity::Native(IfccadNativeEntity {
                id,
                layer_id,
                visible: true,
                appearance: appearance(*pattern),
                line_pattern_scale: 1.,
                kind: IfccadEntityKind::LineSegment {
                    start: [10., 70. + i as f64 * 10., 0.],
                    end: [70., 70. + i as f64 * 10., 0.],
                },
            }));
    }
    if let Some(paper) = d.paper_layouts.first_mut() {
        let id = d.id_counters.allocate_entity_id()?;
        paper
            .entities
            .push(IfccadEntity::Native(IfccadNativeEntity {
                id,
                layer_id,
                visible: true,
                appearance: appearance(ids[1]),
                line_pattern_scale: 1.,
                kind: IfccadEntityKind::LineSegment {
                    start: [20., 20., 0.],
                    end: [80., 20., 0.],
                },
            }));
    }
    if let Some(block) = d.blocks.first_mut() {
        let id = block.id;
        if let Some(child) = block
            .entities
            .first_mut()
            .and_then(IfccadEntity::as_native_mut)
        {
            child.appearance.line_pattern = IfccadMode::ByBlock;
        }
        if let Some(instance)=d.model.entities.iter_mut().filter_map(IfccadEntity::as_native_mut).find(|e|matches!(e.kind,IfccadEntityKind::BlockInstance{definition_id,..} if definition_id==id)) {instance.appearance.line_pattern=IfccadMode::Explicit(ids[2]);}
    }
    recompute_ifccad_document_bounds(&mut d)?;
    validate_ifccad_document(&d)?;
    Ok(d)
}
