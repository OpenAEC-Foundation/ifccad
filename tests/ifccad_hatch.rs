use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ifccad::*;
#[path = "support/ifccad_hatch.rs"]
mod fixture;
use fixture::{drawing, plane, SOURCE};
#[test]
fn native_hatch_keeps_large_ids_complete_source_paths_and_optional_bounds() {
    let d = drawing();
    let out = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(out.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(read.model.entities, d.model.entities);
    assert_eq!(read.model.bounds, None);
    assert_eq!(read.id_counters, d.id_counters);
    let text = std::str::from_utf8(out.bytes()).unwrap();
    assert!(text.contains(&format!("/cad/d{}/e{SOURCE}", d.drawing_id)));
}
#[test]
fn missing_source_fails_and_source_edit_does_not_update_stored_contour() {
    let mut d = drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.loops[1].source_entity_id = Some(123);
    assert!(validate_ifccad_document(&d).is_err());
    let mut d = drawing();
    d.model.entities[1].as_native_mut().unwrap().kind = IfccadEntityKind::Circle {
        radius: 2.0,
        placement: plane(),
    };
    let expected = d.model.entities[0].clone();
    let out = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(out.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(read.model.entities[0], expected);
}
#[test]
fn supplied_hatch_bounds_require_enclosure_and_recompute_is_atomic() {
    let mut d = drawing();
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1.0, -1.0, 0.0],
        max: [1.0, 1.0, 0.0],
    });
    assert!(validate_ifccad_document(&d).is_err());
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let box_before = d.model.bounds;
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.loops[0].boundary = HatchBoundary2::Edges(vec![HatchEdge2::Line {
        start: [0.0, 0.0],
        end: [1.0, 0.0],
    }]);
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d.model.bounds, box_before);
}

#[test]
fn native_ifccad_creation_policy_uses_known_units_or_explicit_fallback() {
    let mut d = drawing();
    d.length_unit = "mm".into();
    let request = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: None,
    };
    let owner = IfccadScopeId::Layout(d.model.id);
    assert_eq!(
        resolve_ifccad_hatch_join_tolerance(&d, owner, request).unwrap(),
        1.0
    );
    d.length_unit = "unitless".into();
    assert!(resolve_ifccad_hatch_join_tolerance(&d, owner, request).is_err());
    let fallback = HatchJoinToleranceRequest::Metres {
        value: 1.0,
        coordinate_fallback: Some(0.125),
    };
    assert_eq!(
        resolve_ifccad_hatch_join_tolerance(&d, owner, fallback).unwrap(),
        0.125
    );
}
