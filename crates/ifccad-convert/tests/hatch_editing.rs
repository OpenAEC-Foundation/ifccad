use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{entities::hatch::*, EntityType, Vector2};
#[path = "../../../tests/support/cad_hatch_associations.rs"]
mod fixture;
fn metadata() -> IfccadTargetMetadata {
    let d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    IfccadTargetMetadata {
        header: d.header,
        drawing_id: d.drawing_id,
    }
}
#[test]
fn inactive_origin_is_consumed_as_hatch_state_and_survives_common_assignment() {
    let (mut d, id, _) = fixture::source(false, false);
    let EntityType::Hatch(h) = d.get_entity_mut(id).unwrap() else {
        panic!()
    };
    h.record_pattern_origin(Vector2::ZERO);
    h.pattern.description = "authored".into();
    let native = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    assert!(
        !native
            .diagnostics()
            .iter()
            .any(|d| d.location.contains("extended_data") || d.location.contains("xdata")),
        "{:?}",
        native.diagnostics()
    );
    let out = ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
    let handle = out
        .mappings()
        .entities
        .cad_handle(native.mappings().entities.ifccad_id(id).unwrap())
        .unwrap();
    let EntityType::Hatch(h) = out.document().get_entity(handle).unwrap() else {
        panic!()
    };
    assert_eq!(h.stored_pattern_origin(), Some(Vector2::ZERO));
    assert_eq!(h.pattern.description, "authored");
    for dwg in [false, true] {
        let back = fixture::exchange(out.document(), dwg);
        let EntityType::Hatch(h) = back.get_entity(handle).unwrap() else {
            panic!()
        };
        assert_eq!(h.stored_pattern_origin(), Some(Vector2::ZERO));
        assert!(
            h.pattern.description.is_empty(),
            "description loss is physical, not a native/model omission"
        );
    }
}
#[test]
fn full_user_defined_double_recipe_roundtrips_and_stale_record_is_a_located_model_loss() {
    let (mut d, id, _) = fixture::source(false, false);
    let EntityType::Hatch(h) = d.get_entity_mut(id).unwrap() else {
        panic!()
    };
    h.is_solid = false;
    h.pattern_type = HatchPatternType::UserDefined;
    h.is_double = true;
    h.pattern = HatchPattern::new("U");
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 1.),
        dash_lengths: vec![],
    });
    let native = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    let mut doc = native.document().clone();
    let entity = native.mappings().entities.ifccad_id(id).unwrap();
    let output = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    let EntityType::Hatch(h) = output
        .document()
        .get_entity(output.mappings().entities.cad_handle(entity).unwrap())
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(h.pattern_type, HatchPatternType::UserDefined);
    assert!(h.is_double);
    assert_eq!(h.pattern.lines.len(), 1);
    let e = doc
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find(|e| e.id == entity)
        .unwrap();
    let IfccadEntityKind::Hatch(h) = &mut e.kind else {
        panic!()
    };
    h.classic_settings.as_mut().unwrap().scale = 2.;
    let out = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(
        out.diagnostics()
            .iter()
            .any(|d| d.code == "hatch-classic-settings-loss"
                && d.location.contains("classicSettings"))
    );
    let EntityType::Hatch(h) = out
        .document()
        .get_entity(out.mappings().entities.cad_handle(entity).unwrap())
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(h.pattern_type, HatchPatternType::Custom);
    assert_eq!(h.pattern.lines.len(), 2);
    assert!(ifccad_document_to_cad_document(
        &doc,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
