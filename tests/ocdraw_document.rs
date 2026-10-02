use ocdraw::ocdraw::{load_drawing_bytes, Bounds3d, DrawingLoadStatus, Point3};
use serde_json::{json, Value};

#[test]
fn reader_exposes_and_moves_complete_document() {
    let mut value: Value = serde_json::from_str(include_str!(
        "../conformance/next/ocdraw/valid/empty.ocdraw.json"
    ))
    .unwrap();
    for entity in [6, 9_007_199_254_740_993, u64::MAX] {
        value["header"]["nextEntityId"] = json!(entity);
        value["header"]["nextLayerId"] = json!(17);
        value["header"]["nextLayoutId"] = json!(24);
        value["header"]["nextLinePatternId"] = json!(19);
        let read = load_drawing_bytes(&serde_json::to_vec(&value).unwrap());
        assert_eq!(
            read.status(),
            DrawingLoadStatus::Valid,
            "{:?}",
            read.diagnostics()
        );
        let borrowed = read.validated_drawing().unwrap().document();
        assert_eq!(borrowed.next_entity_id, entity);
        assert_eq!(borrowed.next_layer_id, 17);
        assert_eq!(borrowed.next_layout_id, 24);
        assert_eq!(borrowed.next_line_pattern_id, 19);
        let doc = read.into_validated_drawing().unwrap().into_document();
        assert_eq!(doc.next_entity_id, entity);
        assert_eq!(doc.drawing_id, "empty-drawing");
        assert_eq!(doc.unit, "unitless");
        assert!(doc.geometric_entities.is_empty());
        assert_eq!(doc.layouts[0].scope_id, doc.scopes[0].id);
    }
    assert!(load_drawing_bytes(b"{}").into_validated_drawing().is_none());
}

#[test]
fn public_geometry_can_be_updated_without_mutating_original_snapshot() {
    let read = load_drawing_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json"
    ));
    let original = read.validated_drawing().unwrap().document();
    let mut edited = original.clone();
    edited.geometric_entities[0].visible = !original.geometric_entities[0].visible;
    edited.geometric_entities[0].id = 100;
    assert_ne!(
        edited.geometric_entities[0].visible,
        original.geometric_entities[0].visible
    );
    assert_ne!(
        edited.geometric_entities[0].id,
        original.geometric_entities[0].id
    );
}

#[test]
fn bounds_can_be_authored_without_recomputation() {
    let bounds = Bounds3d::new(
        Point3::new(-100., -100., -100.),
        Point3::new(100., 100., 100.),
    );
    assert_eq!(bounds.min(), Point3::new(-100., -100., -100.));
    assert_eq!(bounds.max(), Point3::new(100., 100., 100.));
}

#[test]
fn builder_builds_valid_document_without_serializing() {
    use ocdraw::ocdraw::*;
    let doc = DrawingBuilder::new(DrawingOptions::new("fresh", "mm"))
        .unwrap()
        .build_document()
        .unwrap();
    validate_document(&doc).unwrap();
    assert_eq!(doc.next_entity_id, 1);
    assert_eq!(doc.next_layout_id, 1);
    assert_eq!(doc.scopes[0].bounds, None);
    let mut b = DrawingBuilder::new(DrawingOptions::new("filled", "mm")).unwrap();
    b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            LinePatternId(0),
        ))
        .unwrap();
    let paper = b.add_paper_layout("Sheet").unwrap();
    let first = b
        .add_line(LineDefinition::new(layer, [0., 0., 0.], [2., 3., 0.]))
        .unwrap();
    let second = b
        .add_line(LineDefinition::new(layer, [4., 5., 0.], [6., 7., 0.]).in_scope(paper))
        .unwrap();
    let doc = b.build_document().unwrap();
    validate_document(&doc).unwrap();
    assert_eq!(doc.scopes[0].entities, vec![first]);
    assert_eq!(doc.scopes[1].entities, vec![second]);
    assert_eq!(doc.next_entity_id, 3);
    assert_eq!(doc.next_layer_id, 1);
}

#[test]
fn builder_document_preserves_admission_errors() {
    use ocdraw::ocdraw::*;
    let mut b = DrawingBuilder::new(DrawingOptions::new("bad", "mm")).unwrap();
    b.add_line(LineDefinition::new(99, [0., 0., 0.], [1., 1., 0.]))
        .unwrap();
    assert!(matches!(
        b.build_document(),
        Err(DrawingBuildError::Invalid(_))
    ));
}

#[test]
fn watermark_decoding_rejects_noninteger_backings_without_panicking() {
    let value: Value = serde_json::from_str(include_str!(
        "../conformance/next/ocdraw/valid/empty.ocdraw.json"
    ))
    .unwrap();
    for field in [
        "nextEntityId",
        "nextLayerId",
        "nextLayoutId",
        "nextLinePatternId",
    ] {
        let mut d = value.clone();
        d["header"][field] = json!(6.0);
        assert_eq!(
            load_drawing_bytes(&serde_json::to_vec(&d).unwrap()).status(),
            DrawingLoadStatus::Invalid
        );
    }
    let bytes = serde_json::to_string(&value).unwrap().replace(
        "\"nextEntityId\":1",
        "\"nextEntityId\":18446744073709551616",
    );
    assert_eq!(
        load_drawing_bytes(bytes.as_bytes()).status(),
        DrawingLoadStatus::Invalid
    );
}
