#[path = "support/ifccad_dimension.rs"]
mod support;
use ocdraw::{dimension::*, ifccad::*};
#[test]
fn independent_dimension_style_domain_has_full_width_ids_and_monotonic_counters() {
    let mut d = support::drawing();
    validate_ifccad_document(&d).unwrap();
    let id = d.id_counters.allocate_dimension_style_id().unwrap();
    assert_eq!(id, IfccadDimensionStyleId(1));
    d.id_counters.next_dimension_style_id = u64::MAX;
    assert!(d.id_counters.allocate_dimension_style_id().is_err());
    assert_eq!(d.id_counters.next_dimension_style_id, u64::MAX);
    d.id_counters.next_dimension_style_id = 0;
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn forward_source_edit_does_not_recompute_stored_definition() {
    let mut d = support::drawing();
    let mut source = d.model.entities[0].as_native().unwrap().clone();
    source.id = support::DIMENSION + 1;
    source.kind = IfccadEntityKind::LineSegment {
        start: [0.0, 0.0, 0.0],
        end: [4500.0, 0.0, 0.0],
    };
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.associations = vec![IfccadDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: source.id,
            selector: DimensionPointSelector::End,
        }];
    }
    d.model.entities.push(IfccadEntity::Native(source));
    d.id_counters.next_entity_id = support::DIMENSION + 2;
    validate_ifccad_document(&d).unwrap();
    let IfccadEntityKind::Dimension(v) = &d.model.entities[0].as_native().unwrap().kind else {
        panic!("dimension")
    };
    assert_eq!(
        evaluate_dimension(&v.definition).unwrap(),
        DimensionMeasurement::Distance(4000.0)
    );
    d.model.entities.pop();
    assert!(validate_ifccad_document(&d).is_err());
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.associations.clear();
    }
    validate_ifccad_document(&d).unwrap();
}

#[test]
fn unused_marker_block_cycles_and_duplicate_style_names_are_invalid() {
    let mut d = support::drawing();
    let mut block = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .document()
    .blocks[0]
        .clone();
    block.entities = d.model.entities.clone();
    let id = block.id;
    d.model.entities.clear();
    d.blocks = vec![block];
    d.id_counters.next_block_id = id + 1;
    d.dimension_styles[0].settings.arrows.first = DimensionMarker::LocalBlock { block: id };
    assert!(validate_ifccad_document(&d).is_err());
    let mut d = support::drawing();
    d.dimension_styles.push(d.dimension_styles[0].clone());
    assert!(validate_ifccad_document(&d).is_err());
}
