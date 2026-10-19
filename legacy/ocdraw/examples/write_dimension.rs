//! Author native ordinary dimensions, without a CAD runtime or font resolver.
use ocdraw::{dimension::*, ocdraw::*, text::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_dimension <new-output.ocdraw.json>")?;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("explorer-ocdraw-dimension", "mm"))?;
    let pattern = b.ensure_continuous_line_pattern()?;
    let layer = b.add_layer(LayerDefinition::new(
        "Dimensions",
        RgbColor::new(35, 90, 160),
        pattern,
    ))?;
    let paper = b.add_paper_layout("Sheet")?;
    let block = b.add_block_definition(BlockDefinition::new("Dimension example"))?;
    let mut font = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    font.last_used_height = Some(2.5);
    let text = b.add_text_style(TextStyleDefinition {
        name: "Dimension text".into(),
        properties: font,
    })?;
    let mut style = DimensionStyleDefinition::new("Building", text);
    style.settings.numbers.primary.suffix = " mm".into();
    let mut type_settings: OcdrawDimensionOverrides = Default::default();
    type_settings.text.height = Some(3.0);
    style
        .type_overrides
        .insert(DimensionKind::Aligned, type_settings);
    let style = b.add_dimension_style(style)?;
    // The next dimension receives ID 1; the following line receives ID 2.
    let source = 2;
    let associations = vec![
        OcdrawDimensionAssociation {
            role: DimensionRole::FirstPoint,
            source_entity_id: source,
            selector: DimensionPointSelector::Start,
        },
        OcdrawDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: source,
            selector: DimensionPointSelector::End,
        },
    ];
    let mut linear = DimensionEntityDefinition::new(
        layer,
        style,
        DimensionDefinition::Linear {
            first_point: [0.0, 0.0],
            second_point: [4000.0, 0.0],
            direction_angle: 0.0,
            dimension_line_point: [0.0, 300.0],
            extension_direction_angle: None,
        },
    );
    linear.associations = associations.clone();
    b.add_dimension(linear)?;
    let allocated = b.add_line(LineDefinition::new(
        layer,
        [0.0, 0.0, 0.0],
        [4000.0, 0.0, 0.0],
    ))?;
    if allocated != source {
        return Err("example forward-reference allocation changed".into());
    }
    let mut aligned = DimensionEntityDefinition::new(
        layer,
        style,
        DimensionDefinition::Aligned {
            first_point: [0.0, 0.0],
            second_point: [4000.0, 0.0],
            dimension_line_point: [0.0, 600.0],
            extension_direction_angle: None,
        },
    );
    aligned.associations = associations;
    aligned.overrides.text.height = Some(3.5);
    aligned.text_placement = DimensionTextPlacement::Explicit {
        position: [2000.0, 600.0],
        rotation: 0.0,
        attachment: MTextAttachment::MiddleCenter,
    };
    b.add_dimension(aligned)?;
    b.add_dimension(DimensionEntityDefinition::new(
        layer,
        style,
        DimensionDefinition::Linear {
            first_point: [0.0, 0.0],
            second_point: [0.0, 2500.0],
            direction_angle: std::f64::consts::FRAC_PI_2,
            dimension_line_point: [300.0, 0.0],
            extension_direction_angle: None,
        },
    ))?;
    for scope in [paper, block] {
        let source = b.add_geometric_entity(GeometricEntityDefinition::new(
            scope,
            layer,
            DrawingGeometry::Line {
                start: [0.0; 3],
                end: [100.0, 0.0, 0.0],
            },
        ))?;
        let mut d = DimensionEntityDefinition::new(
            layer,
            style,
            DimensionDefinition::Aligned {
                first_point: [0.0; 2],
                second_point: [100.0, 0.0],
                dimension_line_point: [0.0, 20.0],
                extension_direction_angle: None,
            },
        )
        .in_scope(scope);
        d.associations = vec![
            OcdrawDimensionAssociation {
                role: DimensionRole::FirstPoint,
                source_entity_id: source,
                selector: DimensionPointSelector::Start,
            },
            OcdrawDimensionAssociation {
                role: DimensionRole::SecondPoint,
                source_entity_id: source,
                selector: DimensionPointSelector::End,
            },
        ];
        b.add_dimension(d)?;
    }
    b.finish()?.write_file(path)?;
    Ok(())
}
