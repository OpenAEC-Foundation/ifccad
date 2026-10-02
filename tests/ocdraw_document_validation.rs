use ocdraw::ocdraw::*;

fn drawing() -> OcdrawDocument {
    load_drawing_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json"
    ))
    .into_validated_drawing()
    .unwrap()
    .into_document()
}

#[test]
fn authored_documents_share_reference_and_watermark_rules() {
    assert!(validate_document(&drawing()).is_ok());
    for mutation in 0..8 {
        let mut doc = drawing();
        match mutation {
            0 => {
                let id = doc.scopes[0].entities[0];
                doc.scopes[0].entities.push(id);
            }
            1 => {
                doc.scopes[0].entities.remove(0);
            }
            2 => doc.geometric_entities[0].layer_id = 999,
            3 => doc.next_entity_id = doc.geometric_entities.iter().map(|e| e.id).max().unwrap(),
            4 => doc.next_layer_id = doc.layers.iter().map(|e| e.id).max().unwrap(),
            5 => doc.next_layout_id = doc.layouts.iter().map(|e| e.id).max().unwrap(),
            6 => doc.layers[0].line_pattern_id = LinePatternId(999),
            _ => doc.geometric_entities[1].id = doc.geometric_entities[0].id,
        }
        assert!(validate_document(&doc).is_err(), "mutation {mutation}");
    }
}

#[test]
fn authored_values_cannot_bypass_schema_field_constraints() {
    for mutation in 0..11 {
        let mut doc = drawing();
        match mutation {
            0 => doc.drawing_id.clear(),
            1 => doc.unit = "invalid".into(),
            2 => doc.layers[0].opacity = -0.1,
            3 => doc.layers[0].opacity = 1.1,
            4 => doc.layers[0].line_weight = -1.0,
            5 => doc.line_pattern_scale = 0.,
            6 => doc.geometric_entities[0].appearance.line_pattern_scale = f64::INFINITY,
            7 => {
                doc.geometric_entities[0].geometry = DrawingGeometry::Line {
                    start: [f64::NAN, 0., 0.],
                    end: [0., 0., 0.],
                }
            }
            8 => {
                doc.geometric_entities[0].geometry = DrawingGeometry::Circle {
                    placement: CoordinateFrame3::default(),
                    radius: 0.,
                }
            }
            9 => {
                doc.geometric_entities[0].appearance.color =
                    AppearanceSelection::Explicit(DrawingColor::rgb(0, 0, 0).with_indexed("", 0))
            }
            _ => doc.next_entity_id = 0,
        }
        assert!(validate_document(&doc).is_err(), "mutation {mutation}");
    }
}

#[test]
fn dormant_bulges_are_validated_even_when_not_evaluated() {
    let mut doc = drawing();
    doc.geometric_entities[0].geometry = DrawingGeometry::PlanarPolyline {
        placement: CoordinateFrame3::default(),
        vertices: vec![[0., 0., 0.], [1., 1., 8.]],
        closed: false,
        line_pattern_generation: LinePatternGeneration::PerSegment,
    };
    // Existing generous bounds enclose this replacement segment.
    for scope in &mut doc.scopes {
        if !scope.entities.is_empty() {
            scope.bounds = Some(Bounds3d::new(
                Point3::new(-100., -100., -100.),
                Point3::new(100., 100., 100.),
            ));
        }
    }
    assert!(validate_document(&doc).is_ok());
    let after = load_drawing_bytes(encode_document(&doc).unwrap().bytes())
        .into_validated_drawing()
        .unwrap()
        .into_document();
    assert_eq!(
        Some(&doc.geometric_entities[0]),
        after
            .geometric_entities
            .iter()
            .find(|e| e.id == doc.geometric_entities[0].id)
    );
    if let DrawingGeometry::PlanarPolyline { vertices, .. } =
        &mut doc.geometric_entities[0].geometry
    {
        vertices[1][2] = f64::NAN;
    }
    assert!(validate_document(&doc).is_err());
}

#[test]
fn authored_blocks_and_saved_state_share_cross_record_rules() {
    let mut cycle = drawing();
    let block_scope = cycle.block_definitions[0].scope_id;
    let id = cycle
        .scopes
        .iter()
        .find(|s| s.id == block_scope)
        .unwrap()
        .entities[0];
    cycle
        .geometric_entities
        .iter_mut()
        .find(|e| e.id == id)
        .unwrap()
        .geometry = DrawingGeometry::BlockInstance {
        definition_scope_id: block_scope,
        transform: BlockTransform::try_new(CoordinateFrame3::default(), 0., Scale3::default())
            .unwrap(),
    };
    assert!(validate_document(&cycle)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|d| d.code == "BLOCK_CYCLE"));
    let mut d = load_drawing_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .into_validated_drawing()
    .unwrap()
    .into_document();
    d.viewports[0].id = 0;
    assert!(validate_document(&d).is_err());
    d.viewports[0].id = 1;
    d.viewports[0].view.height = f64::NAN;
    assert!(validate_document(&d).is_err());
}
