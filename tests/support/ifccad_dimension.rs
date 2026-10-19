use ifccad::{dimension::*, text::*, *};
use std::collections::BTreeMap;
pub const DIMENSION: u64 = 9007199254740993;
pub fn drawing() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("../fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut e = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .unwrap()
        .clone();
    d.paper_layouts.clear();
    d.blocks.clear();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    d.text_styles = vec![IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Text".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested face")),
    }];
    d.id_counters.next_text_style_id = 1;
    d.dimension_styles = vec![IfccadDimensionStyle {
        id: IfccadDimensionStyleId(0),
        name: "Building".into(),
        settings: DimensionStyleSettings::new(IfccadTextStyleId(0)),
        type_overrides: BTreeMap::new(),
    }];
    d.id_counters.next_dimension_style_id = 1;
    e.id = DIMENSION;
    e.kind = IfccadEntityKind::Dimension(Box::new(IfccadDimension {
        placement: IfccadPlacement {
            origin: [0.0; 3],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
        },
        style_id: IfccadDimensionStyleId(0),
        definition: DimensionDefinition::Aligned {
            first_point: [0.0, 0.0],
            second_point: [4000.0, 0.0],
            dimension_line_point: [0.0, 300.0],
            extension_direction_angle: None,
        },
        text_placement: Default::default(),
        label: Default::default(),
        overrides: Default::default(),
        associations: vec![],
    }));
    d.model.entities = vec![IfccadEntity::Native(e)];
    d.id_counters.next_entity_id = DIMENSION + 1;
    recompute_ifccad_document_bounds(&mut d).unwrap();
    d
}
