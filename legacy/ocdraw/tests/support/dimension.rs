use ocdraw::{dimension::*, ocdraw::*, text::*};
use std::collections::BTreeMap;

pub fn drawing() -> OcdrawDocument {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("dimensions", "mm")).unwrap();
    let pattern = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "Dimensions",
            RgbColor::new(1, 2, 3),
            pattern,
        ))
        .unwrap();
    let mut d = b.build_document().unwrap();
    d.text_styles.push(DrawingTextStyle {
        id: OcdrawTextStyleId(0),
        name: "Requested face".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested face")),
    });
    d.dimension_styles.push(DrawingDimensionStyle {
        id: OcdrawDimensionStyleId(0),
        name: "Building".into(),
        settings: DimensionStyleSettings::new(OcdrawTextStyleId(0)),
        type_overrides: BTreeMap::new(),
    });
    d.dimension_entities.push(DrawingDimensionEntity {
        id: 1,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        style_id: OcdrawDimensionStyleId(0),
        definition: DimensionDefinition::Aligned {
            first_point: [0.0, 0.0],
            second_point: [4000.0, 0.0],
            dimension_line_point: [0.0, 300.0],
            extension_direction_angle: None,
        },
        text_placement: Default::default(),
        label: Default::default(),
        overrides: Default::default(),
        associations: Vec::new(),
    });
    d.scopes[0].entities.push(1);
    d.next_entity_id = 2;
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    d
}
