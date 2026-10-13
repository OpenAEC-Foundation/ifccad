//! Independently author IFCCAD marker/source records and occurrence contexts.
use ocdraw::{ifccad::*, line_pattern::*, text::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_ifccad_line_patterns <output.ifcx>")?;
    let mut d =
        load_ifccad_bytes(include_bytes!("ifccad/text.ifcx"), Default::default())?.into_document();
    d.model.entities.clear();
    for p in &mut d.paper_layouts {
        p.entities.clear();
    }
    d.blocks.clear();
    // This is a fresh authored drawing; only layout/style scaffolding is reused.
    d.id_counters.next_entity_id = 1;
    d.id_counters.next_block_id = 1;
    let style = d.text_styles[0].id;
    let mut properties = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    properties.last_used_height = Some(2.5);
    d.text_styles[0].properties = properties;
    let standard = d.id_counters.allocate_symbol_source_id()?;
    let custom = d.id_counters.allocate_symbol_source_id()?;
    let duplicate = d.id_counters.allocate_symbol_source_id()?;
    d.symbol_sources = vec![
        IfccadSymbolSource {
            id: standard,
            source: SymbolSource::Shx {
                file_name: "ltypeshp.shx".into(),
            },
        },
        IfccadSymbolSource {
            id: custom,
            source: SymbolSource::Shx {
                file_name: "custom-missing.shx".into(),
            },
        },
        IfccadSymbolSource {
            id: duplicate,
            source: SymbolSource::Shx {
                file_name: "custom-missing.shx".into(),
            },
        },
    ];
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
    let mut add = |name: &str,
                   pattern: Vec<IfccadLinePatternElement>|
     -> Result<IfccadLinePatternId, IfccadIdAllocationError> {
        let id = d.id_counters.allocate_line_pattern_id()?;
        d.line_patterns.push(IfccadLinePattern {
            id,
            name: name.into(),
            description: None,
            pattern,
        });
        Ok(id)
    };
    let gas = add(
        "GAS",
        vec![
            LinePatternElement::simple(10.),
            LinePatternElement {
                length: 0.,
                marker: Some(text.clone()),
            },
            LinePatternElement::simple(-5.),
        ],
    )?;
    let boxes = add(
        "Boxes",
        vec![
            LinePatternElement::simple(10.),
            LinePatternElement {
                length: 0.,
                marker: Some(symbol(standard, 132)),
            },
            LinePatternElement::simple(-5.),
        ],
    )?;
    let mixed = add(
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
                marker: Some(symbol(custom, 42)),
            },
            LinePatternElement::simple(-4.),
        ],
    )?;
    add(
        "Unused",
        vec![LinePatternElement {
            length: 1.,
            marker: Some(symbol(custom, 42)),
        }],
    )?;
    add("Named empty", vec![])?;
    d.layers[0].name = "0".into();
    d.layers[0].appearance.line_pattern = gas;
    let layer = d.layers[0].id;
    let appearance = |pattern| IfccadEntityAppearance {
        color: IfccadMode::ByLayer,
        opacity: IfccadMode::ByLayer,
        line_pattern: pattern,
        line_weight: IfccadMode::ByLayer,
    };
    let line = |id, pattern, y| {
        IfccadEntity::Native(IfccadNativeEntity {
            id,
            layer_id: layer,
            visible: true,
            appearance: appearance(pattern),
            line_pattern_scale: 1.,
            kind: IfccadEntityKind::LineSegment {
                start: [0., y, 0.],
                end: [60., y, 0.],
            },
        })
    };
    d.model.entities.push(line(
        d.id_counters.allocate_entity_id()?,
        IfccadMode::Explicit(gas),
        0.,
    ));
    d.model.entities.push(line(
        d.id_counters.allocate_entity_id()?,
        IfccadMode::Explicit(mixed),
        10.,
    ));
    d.model.entities.push(line(
        d.id_counters.allocate_entity_id()?,
        IfccadMode::Explicit(boxes),
        30.,
    ));
    let instance_id = d.id_counters.allocate_entity_id()?;
    if let Some(p) = d.paper_layouts.first_mut() {
        p.entities.push(line(
            d.id_counters.allocate_entity_id()?,
            IfccadMode::Explicit(boxes),
            0.,
        ));
    }
    let block = d.id_counters.allocate_block_id()?;
    let child = line(d.id_counters.allocate_entity_id()?, IfccadMode::ByBlock, 0.);
    d.blocks.push(IfccadBlockDefinition {
        id: block,
        name: "Inherited pattern".into(),
        base_point: [0.; 3],
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        insertion_unit: "mm".into(),
        bounds: None,
        bounds_quality: None,
        entities: vec![child],
    });
    d.model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id: instance_id,
            layer_id: layer,
            visible: true,
            appearance: appearance(IfccadMode::Explicit(mixed)),
            line_pattern_scale: 1.,
            kind: IfccadEntityKind::BlockInstance {
                definition_id: block,
                transform: IfccadBlockTransform {
                    placement: IfccadPlacement {
                        origin: [0., 20., 0.],
                        x_axis: [1., 0., 0.],
                        y_axis: [0., 1., 0.],
                    },
                    rotation: 0.,
                    scale: [1.; 3],
                },
            },
        }));
    recompute_ifccad_document_bounds(&mut d)?;
    d.model.bounds = Some(IfccadBounds3d {
        min: [0.; 3],
        max: [65., 35., 1.],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Enclosing);
    let encoded = encode_ifccad_document(&d)?;
    load_ifccad_bytes(encoded.bytes(), Default::default())?;
    std::fs::write(path, encoded.bytes())?;
    Ok(())
}
