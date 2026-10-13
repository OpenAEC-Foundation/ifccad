use ocdraw::{dimension::*, ocdraw::*};
use ocdraw_convert::*;
use opencadcodec::EntityType;
#[test]
fn native_dimensions_are_materialized_with_style_and_literal_template() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("dims", "mm")).unwrap();
    let pattern = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0), pattern))
        .unwrap();
    let text = b
        .add_text_style(TextStyleDefinition {
            name: "Text".into(),
            properties: ocdraw::text::TextStyleProperties::new(ocdraw::text::FontRequest {
                cad_font_name: Some("txt.shx".into()),
                ..Default::default()
            }),
        })
        .unwrap();
    let style = b
        .add_dimension_style(DimensionStyleDefinition::new("Building", text))
        .unwrap();
    let mut d = DimensionEntityDefinition::new(
        layer,
        style,
        DimensionDefinition::Aligned {
            first_point: [0.0, 0.0],
            second_point: [3.0, 4.0],
            dimension_line_point: [0.0, 8.0],
            extension_direction_angle: None,
        },
    );
    d.overrides.dimension_line.suppress_first = Some(false);
    d.label = DimensionLabel::Suppressed;
    b.add_dimension(d).unwrap();
    let doc = b.build_document().unwrap();
    let result = ocdraw_document_to_cad_document(&doc, Default::default()).unwrap();
    let d = result
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(d.base().actual_measurement, 5.0);
    assert_eq!(d.base().text_override(), Some(" "));
    assert!(result
        .document()
        .dim_styles
        .get(&d.base().style_name)
        .is_some());
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.message.contains("picture")));
}

#[test]
fn signed_nonuniform_nested_occurrence_amplifies_dimension_rounding_to_hard_failure() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("nested-dims", "mm")).unwrap();
    let p = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0), p))
        .unwrap();
    let mut block = BlockDefinition::new("Inner");
    block.insertion_unit = "mm".into();
    let inner = b.add_block_definition(block).unwrap();
    let mut block = BlockDefinition::new("Outer");
    block.insertion_unit = "mm".into();
    let outer = b.add_block_definition(block).unwrap();
    let text = b
        .add_text_style(TextStyleDefinition {
            name: "Text".into(),
            properties: ocdraw::text::TextStyleProperties::new(ocdraw::text::FontRequest {
                cad_font_name: Some("txt.shx".into()),
                ..Default::default()
            }),
        })
        .unwrap();
    let style = b
        .add_dimension_style(DimensionStyleDefinition::new("Building", text))
        .unwrap();
    let mut d = DimensionEntityDefinition::new(
        layer,
        style,
        DimensionDefinition::Aligned {
            first_point: [0.0; 2],
            second_point: [3e-12, 0.0],
            dimension_line_point: [0.0, 1e-12],
            extension_direction_angle: None,
        },
    )
    .in_scope(inner);
    d.placement = CoordinateFrame3::try_new(
        Point3::new(1e8, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    b.add_dimension(d).unwrap();
    for (owner, definition, scale) in [
        (outer, inner, Scale3::new(-1e4, 2.0, 3.0)),
        (0, outer, Scale3::new(1e4, -2.0, 3.0)),
    ] {
        b.add_geometric_entity(GeometricEntityDefinition::new(
            owner,
            layer,
            DrawingGeometry::BlockInstance {
                definition_scope_id: definition,
                transform: BlockTransform::try_new(CoordinateFrame3::default(), 0.0, scale)
                    .unwrap(),
            },
        ))
        .unwrap();
    }
    let doc = b.build_document().unwrap();
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            ocdraw_document_to_cad_document(
                &doc,
                OcdrawToCadOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(OcdrawToCadError::Geometry(_))
        ));
    }
}
