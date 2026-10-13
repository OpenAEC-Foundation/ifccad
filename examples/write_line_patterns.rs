//! Complex native patterns without external font or SHX loading.
use ocdraw::{line_pattern::*, ocdraw::*, text::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_line_patterns <output.ocdraw.json>")?;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("complex-line-patterns", "mm"))?;
    let continuous = b.ensure_continuous_line_pattern()?;
    let mut properties = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    properties.last_used_height = Some(2.5);
    let style = b.add_text_style(TextStyleDefinition {
        name: "Pattern text".into(),
        properties,
    })?;
    let standard = b.add_symbol_source(SymbolSource::Shx {
        file_name: "ltypeshp.shx".into(),
    })?;
    let custom = b.add_symbol_source(SymbolSource::Shx {
        file_name: "custom-missing.shx".into(),
    })?;
    b.add_symbol_source(SymbolSource::Shx {
        file_name: "custom-missing.shx".into(),
    })?;
    let text = DrawingLinePatternMarker::Text {
        text: "GAS".into(),
        text_style_id: style,
        transform: LinePatternTransform {
            scale: 2.,
            ..Default::default()
        },
    };
    let symbol = |source, shape| DrawingLinePatternMarker::Symbol {
        symbol_source_id: source,
        shape_number: shape,
        transform: LinePatternTransform {
            scale: 2.,
            ..Default::default()
        },
    };
    let mut add = |name: &str, marker: DrawingLinePatternMarker| {
        b.add_line_pattern(LinePatternDefinition {
            name: name.into(),
            description: Some("External glyph availability is not checked".into()),
            pattern: vec![
                LinePatternElement::simple(10.),
                LinePatternElement {
                    length: 0.,
                    marker: Some(marker),
                },
                LinePatternElement::simple(-5.),
            ],
        })
    };
    let gas = add("GAS", text.clone())?;
    let boxes = add("Boxes", symbol(standard, 132))?;
    let mixed = b.add_line_pattern(LinePatternDefinition {
        name: "Mixed".into(),
        description: Some("Text and custom SHX".into()),
        pattern: vec![
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
    })?;
    b.add_line_pattern(LinePatternDefinition {
        name: "Unused".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 1.,
            marker: Some(symbol(custom, 42)),
        }],
    })?;
    b.add_line_pattern(LinePatternDefinition {
        name: "Named empty".into(),
        description: None,
        pattern: vec![],
    })?;
    let zero = b.add_layer(LayerDefinition::new(
        "0",
        RgbColor::new(25, 80, 150),
        continuous,
    ))?;
    let gas_layer = b.add_layer(LayerDefinition::new("Gas", RgbColor::new(25, 80, 150), gas))?;
    let mixed_layer = b.add_layer(LayerDefinition::new(
        "Mixed",
        RgbColor::new(150, 60, 25),
        mixed,
    ))?;
    b.add_line(LineDefinition::new(gas_layer, [0.; 3], [60., 0., 0.]))?;
    b.add_line(LineDefinition::new(
        mixed_layer,
        [0., 10., 0.],
        [60., 10., 0.],
    ))?;
    let mut library_line = LineDefinition::new(zero, [0., 30., 0.], [60., 30., 0.]);
    library_line.appearance.line_pattern = AppearanceSelection::Explicit(boxes);
    b.add_line(library_line)?;
    let paper = b.add_paper_layout("Patterns")?;
    let block = b.add_block_definition(BlockDefinition::new("Inherited pattern"))?;
    let mut child = LineDefinition::new(zero, [0.; 3], [60., 0., 0.]).in_scope(block);
    child.appearance.line_pattern = AppearanceSelection::ByBlock;
    b.add_line(child)?;
    b.add_block_instance(BlockInstanceDefinition::new(
        mixed_layer,
        block,
        [0., 20., 0.],
    ))?;
    let mut line = LineDefinition::new(zero, [0.; 3], [60., 0., 0.]).in_scope(paper);
    line.appearance.line_pattern = AppearanceSelection::Explicit(boxes);
    b.add_line(line)?;
    let mut doc = b.build_document()?;
    doc.scopes.iter_mut().find(|s| s.id == 0).unwrap().bounds = Some(Bounds3d::new(
        Point3::new(0., 0., 0.),
        Point3::new(65., 35., 1.),
    ));
    doc.scopes
        .iter_mut()
        .find(|s| s.id == 0)
        .unwrap()
        .bounds_quality = Some(OcdrawBoundsQuality::Enclosing);
    let encoded = encode_ocdraw_document(&doc)?;
    load_ocdraw_bytes(encoded.bytes())?;
    encoded.write_file(path)?;
    Ok(())
}
