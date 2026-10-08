use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::DrawingGeometry as EntityGeometry;
use ocdraw::ocdraw::*;

const SOURCE: u64 = 9007199254740993;
const HATCH: u64 = SOURCE + 1;

fn empty_document() -> OcdrawDocument {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("hatch-native", "mm")).unwrap();
    let p = b
        .add_line_pattern(LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
        .unwrap();
    b.add_layer(LayerDefinition::new("0", RgbColor::new(255, 255, 255), p))
        .unwrap();
    b.build_document().unwrap()
}

fn drawing() -> OcdrawDocument {
    let mut d = empty_document();
    let layer = d.layers[0].id;
    d.geometric_entities.push(DrawingGeometricEntity {
        id: SOURCE,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        geometry: EntityGeometry::Circle {
            placement: Default::default(),
            radius: 1.0,
        },
    });
    d.hatch_entities.push(DrawingHatchEntity {
        id: HATCH,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        area_rule: HatchAreaRule::Normal,
        join_tolerance: DEFAULT_HATCH_JOIN_TOLERANCE,
        fill: HatchFill::Solid,
        loops: vec![
            OcdrawHatchLoop {
                boundary: HatchBoundary2::Polyline {
                    vertices: vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]],
                    bulges: vec![0.0; 4],
                },
                source_entity_id: None,
            },
            OcdrawHatchLoop {
                boundary: HatchBoundary2::Circle {
                    center: [0.0, 0.0],
                    radius: 1.0,
                },
                source_entity_id: Some(SOURCE),
            },
        ],
    });
    d.next_entity_id = HATCH + 1;
    d.scopes[0].entities = vec![HATCH, SOURCE];
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    d
}

#[test]
fn solid_hatch_roundtrips_with_holes_sources_and_large_ids() {
    let d = drawing();
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    let h = &loaded.hatch_entities()[0];
    assert_eq!(h.id, HATCH);
    assert_eq!(h.loops, d.hatch_entities[0].loops);
    assert_eq!(h.fill, HatchFill::Solid);
    assert_eq!(h.join_tolerance, 1e-9);
    assert_eq!(loaded.document().next_entity_id, HATCH + 1);
    assert_eq!(loaded.document().scopes[0].entities, [HATCH, SOURCE]);
}

#[test]
fn missing_wrong_kind_and_wrong_owner_sources_are_not_repaired() {
    let mut d = drawing();
    d.hatch_entities[0].loops[1].source_entity_id = Some(123);
    assert!(validate_ocdraw_document(&d)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|e| e.code == "HATCH_SOURCE"));
    d.hatch_entities[0].loops[1].source_entity_id = Some(SOURCE);
    d.geometric_entities[0].geometry = EntityGeometry::Line {
        start: [0.0, 0.0, 0.0],
        end: [1.0, 0.0, 0.0],
    };
    assert!(validate_ocdraw_document(&d)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|e| e.code == "HATCH_SOURCE"));
    let mut d = drawing();
    let mut paper = d.scopes[0].clone();
    paper.id = 1;
    paper.kind = DrawingScopeKind::Paper;
    paper.entities = vec![SOURCE];
    let mut layout = d.layouts[0].clone();
    layout.id = 1;
    layout.scope_id = 1;
    layout.kind = DrawingLayoutKind::Paper;
    layout.name = "Paper".into();
    layout.tab_index = 1;
    d.scopes[0].entities.retain(|id| *id != SOURCE);
    d.scopes.push(paper);
    d.layouts.push(layout);
    d.next_layout_id = 2;
    assert!(validate_ocdraw_document(&d)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|e| e.code == "HATCH_SOURCE"));
}

#[test]
fn changing_a_valid_source_preserves_authored_boundary_and_detach_is_explicit() {
    let mut d = drawing();
    let boundary = d.hatch_entities[0].loops[1].boundary.clone();
    d.geometric_entities[0].geometry = EntityGeometry::Circle {
        placement: Default::default(),
        radius: 2.0,
    };
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.hatch_entities()[0].loops[1].boundary, boundary);
    d.hatch_entities[0].loops[1].source_entity_id = None;
    d.geometric_entities.clear();
    d.scopes[0].entities = vec![HATCH];
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.hatch_entities()[0].loops[1].boundary, boundary);
    assert_eq!(loaded.hatch_entities()[0].loops[1].source_entity_id, None);
}

#[test]
fn invalid_join_does_not_mutate_prepared_bounds() {
    let mut d = drawing();
    let before = d.scopes[0].bounds;
    d.hatch_entities[0].loops[0].boundary = HatchBoundary2::Edges(vec![
        HatchEdge2::Line {
            start: [0.0, 0.0],
            end: [1.0, 0.0],
        },
        HatchEdge2::Line {
            start: [2.0, 0.0],
            end: [0.0, 0.0],
        },
    ]);
    assert!(recompute_ocdraw_document_bounds(&mut d).is_err());
    assert_eq!(d.scopes[0].bounds, before);
    assert_eq!(d.hatch_entities[0].id, HATCH);
}

#[test]
fn unknown_nested_fields_null_refs_and_unavailable_fill_are_rejected() {
    let encoded = encode_ocdraw_document(&drawing()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let mut bad = value.clone();
    bad["streams"]["hatchStream"]["loops"][0][0]["boundary"]["secret"] = serde_json::json!(1);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = value.clone();
    bad["streams"]["hatchStream"]["loops"][0][1]["sourceEntityId"] = serde_json::Value::Null;
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = value;
    bad["streams"]["hatchStream"]["fill"][0] = serde_json::json!({"kind":"linePattern"});
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
}

#[test]
fn creation_tolerance_respects_known_units_and_explicit_unknown_unit_fallback() {
    let mut d = empty_document();
    let one_mm = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: None,
    };
    assert_eq!(
        resolve_ocdraw_hatch_join_tolerance(&d, 0, one_mm).unwrap(),
        1.0
    );
    d.unit = "unitless".into();
    assert!(resolve_ocdraw_hatch_join_tolerance(&d, 0, one_mm).is_err());
    let fallback = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: Some(0.125),
    };
    assert_eq!(
        resolve_ocdraw_hatch_join_tolerance(&d, 0, fallback).unwrap(),
        0.125
    );
    assert_eq!(
        resolve_ocdraw_hatch_join_tolerance(&d, 0, HatchJoinToleranceRequest::Default).unwrap(),
        1e-9
    );
}
