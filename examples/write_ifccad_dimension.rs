//! Author independent native IFCCAD ordinary dimensions and exact path references.
use ocdraw::{dimension::*, ifccad::*, text::*};
use std::collections::BTreeMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_ifccad_dimension <new-output.ifcx>")?;
    let mut d = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )?
    .into_document();
    d.header = IfccadHeader {
        id: "urn:example:ifccad:dimension-slice-1".into(),
        data_version: "0.1.0".into(),
        author: "OpenAEC".into(),
        timestamp: "2026-10-13T00:00:00Z".into(),
    };
    let mut entity = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .ok_or("example entity template")?
        .clone();
    d.paper_layouts.clear();
    d.blocks.clear();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let mut font = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    font.last_used_height = Some(2.5);
    d.text_styles = vec![IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Dimension text".into(),
        properties: font,
    }];
    d.id_counters.next_text_style_id = 1;
    let mut settings = DimensionStyleSettings::new(IfccadTextStyleId(0));
    settings.numbers.primary.suffix = " mm".into();
    d.dimension_styles = vec![IfccadDimensionStyle {
        id: IfccadDimensionStyleId(0),
        name: "Building".into(),
        settings,
        type_overrides: BTreeMap::new(),
    }];
    d.id_counters.next_dimension_style_id = 1;
    let mut type_settings: IfccadDimensionOverrides = Default::default();
    type_settings.text.height = Some(3.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Aligned, type_settings);
    const SOURCE: u64 = 2;
    entity.id = SOURCE;
    entity.kind = IfccadEntityKind::LineSegment {
        start: [0.0; 3],
        end: [4000.0, 0.0, 0.0],
    };
    let line = entity.clone();
    let relations = vec![
        IfccadDimensionAssociation {
            role: DimensionRole::FirstPoint,
            source_entity_id: SOURCE,
            selector: DimensionPointSelector::Start,
        },
        IfccadDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: SOURCE,
            selector: DimensionPointSelector::End,
        },
    ];
    let make = |definition: DimensionDefinition| IfccadDimension {
        placement: IfccadPlacement {
            origin: [0.0; 3],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
        },
        style_id: IfccadDimensionStyleId(0),
        definition,
        text_placement: Default::default(),
        label: Default::default(),
        overrides: Default::default(),
        associations: relations.clone(),
    };
    entity.id = SOURCE - 1;
    entity.kind = IfccadEntityKind::Dimension(Box::new(make(DimensionDefinition::Linear {
        first_point: [0.0, 0.0],
        second_point: [4000.0, 0.0],
        direction_angle: 0.0,
        dimension_line_point: [0.0, 300.0],
        extension_direction_angle: None,
    })));
    let linear = entity.clone();
    entity.id = SOURCE + 1;
    let mut aligned = make(DimensionDefinition::Aligned {
        first_point: [0.0, 0.0],
        second_point: [4000.0, 0.0],
        dimension_line_point: [0.0, 600.0],
        extension_direction_angle: None,
    });
    aligned.overrides.text.height = Some(3.5);
    entity.kind = IfccadEntityKind::Dimension(Box::new(aligned));
    let aligned = entity.clone();
    entity.id = SOURCE + 2;
    let mut vertical = make(DimensionDefinition::Linear {
        first_point: [0.0; 2],
        second_point: [0.0, 2500.0],
        direction_angle: std::f64::consts::FRAC_PI_2,
        dimension_line_point: [300.0, 0.0],
        extension_direction_angle: None,
    });
    vertical.associations.clear();
    entity.kind = IfccadEntityKind::Dimension(Box::new(vertical));
    let mut paper_dimension = aligned.clone();
    paper_dimension.id = SOURCE + 3;
    if let IfccadEntityKind::Dimension(d) = &mut paper_dimension.kind {
        d.associations = vec![IfccadDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: SOURCE + 4,
            selector: DimensionPointSelector::End,
        }];
    }
    let mut paper_line = line.clone();
    paper_line.id = SOURCE + 4;
    d.paper_layouts = vec![IfccadPaperLayout {
        id: 2,
        name: "Sheet".into(),
        tab_index: 1,
        settings: d.model.settings.clone(),
        entities: vec![
            IfccadEntity::Native(paper_dimension),
            IfccadEntity::Native(paper_line),
        ],
        canvas: None,
        bounds: None,
        bounds_quality: None,
    }];
    d.id_counters.next_layout_id = 3;
    let mut block_dimension = aligned.clone();
    block_dimension.id = SOURCE + 5;
    if let IfccadEntityKind::Dimension(d) = &mut block_dimension.kind {
        d.associations = vec![IfccadDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: SOURCE + 6,
            selector: DimensionPointSelector::End,
        }];
    }
    let mut block_line = line.clone();
    block_line.id = SOURCE + 6;
    d.blocks = vec![IfccadBlockDefinition {
        id: 1,
        name: "Dimension example".into(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        insertion_unit: "mm".into(),
        base_point: [0.0; 3],
        entities: vec![
            IfccadEntity::Native(block_dimension),
            IfccadEntity::Native(block_line),
        ],
        bounds: None,
        bounds_quality: None,
    }];
    d.id_counters.next_block_id = 2;
    d.model.entities = vec![
        IfccadEntity::Native(linear),
        IfccadEntity::Native(line),
        IfccadEntity::Native(aligned),
        IfccadEntity::Native(entity),
    ];
    d.id_counters.next_entity_id = SOURCE + 7;
    recompute_ifccad_document_bounds(&mut d)?;
    encode_ifccad_document(&d)?.write_file(path)?;
    Ok(())
}
