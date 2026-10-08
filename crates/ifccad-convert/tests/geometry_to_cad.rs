mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
#[test]
fn native_families_emit_supported_cad_geometry() {
    let mut d = empty();
    let mut template = primitives().model.entities[0].clone();
    let frame = IfccadPlacement {
        origin: [2., 3., 4.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    for kind in [
        IfccadEntityKind::Point {
            placement: frame.clone(),
        },
        IfccadEntityKind::Arc {
            placement: frame.clone(),
            radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: -1.,
        },
        IfccadEntityKind::Ellipse {
            placement: frame.clone(),
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
        },
        IfccadEntityKind::EllipseArc {
            placement: frame,
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: 1.,
        },
        IfccadEntityKind::SpatialPolyline {
            vertices: vec![[0.; 3], [1., 2., 3.]],
            closed: false,
            line_pattern_generation: IfccadLinePatternGeneration::Continuous,
        },
    ] {
        template.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
        template.as_native_mut().unwrap().kind = kind;
        d.model.entities.push(template.clone());
    }
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert_eq!(out.mappings().entities.iter().count(), 5);
    let back =
        cad_document_to_encoded_ifccad(out.document(), metadata(), Default::default()).unwrap();
    assert_eq!(back.validated_source().document().model.entities.len(), 5);
}
#[test]
fn signed_nested_scale_cannot_hide_an_amplified_residual() {
    let mut d = nested([0.; 3]);
    let inner = d.blocks.iter_mut().find(|b| b.name == "Inner").unwrap();
    inner.entities[0].as_native_mut().unwrap().kind = IfccadEntityKind::PlanarPolyline {
        placement: IfccadPlacement {
            origin: [1e20, 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        vertices: vec![[1., 0.], [2., 0.]],
        bulges: vec![0., 0.],
        closed: false,
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    let options = IfccadToCadOptions {
        geometry_tolerance: IfccadGeometryTolerance::drawing_units(10.).unwrap(),
        ..Default::default()
    };
    assert!(ifccad_document_to_cad_document(&d, options).is_ok());
    for e in &mut d.model.entities {
        if let IfccadEntityKind::BlockInstance { transform, .. } =
            &mut e.as_native_mut().unwrap().kind
        {
            transform.scale = [-100.; 3];
        }
    }
    assert!(matches!(
        ifccad_document_to_cad_document(&d, options),
        Err(IfccadConversionError::Geometry(_))
    ));
}

#[test]
fn paper_occurrences_compare_error_in_their_declared_coordinate_unit() {
    let options = IfccadToCadOptions {
        geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001)
            .unwrap()
            .with_coordinate_fallback(0.)
            .unwrap(),
        ..Default::default()
    };
    let mut d = nested([0.; 3]);
    d.model.entities.clear();
    let inner = d.blocks.iter_mut().find(|b| b.name == "Inner").unwrap();
    inner.entities[0].as_native_mut().unwrap().kind = IfccadEntityKind::PlanarPolyline {
        placement: IfccadPlacement {
            origin: [4398046511104., 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        vertices: vec![[0.0005, 0.], [0.001, 0.]],
        bulges: vec![0., 0.],
        closed: false,
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    assert!(ifccad_document_to_cad_document(&d, options).is_ok());
    let mut e = primitives().model.entities[0].clone();
    e.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
    e.as_native_mut().unwrap().kind = IfccadEntityKind::BlockInstance {
        definition_id: 9,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    let id = d.id_counters.allocate_layout_id().unwrap();
    d.paper_layouts.push(IfccadPaperLayout {
        settings: ocdraw::ifccad::IfccadLayoutSettings {
            media: None,
            ..Default::default()
        },
        id,
        name: "Unknown-scale paper".into(),
        tab_index: 1,
        bounds: None,
        entities: vec![e],
    });
    let Err(IfccadConversionError::Geometry(error)) = ifccad_document_to_cad_document(&d, options)
    else {
        panic!("paper error must use its coordinate fallback")
    };
    assert_eq!(error.domain, IfccadGeometryDomain::PaperLayout(id));
    assert!(
        matches!(&error.failure.source,IfccadGeometryEntitySource::BlockOccurrence{path,..} if path.len()>=2)
    );
}

#[test]
fn source_defaults_do_not_create_foreign_loss_but_exact_bounds_projection_stays_hard() {
    use serde_json::{json, Value};
    let d = primitives();
    let mut raw: Value =
        serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(p) = node["attributes"].get_mut("ifccad::geom::planarPolyline") {
            let n = p["vertices"].as_array().unwrap().len();
            p["bulges"] = json!(vec![0.; n]);
        }
    }
    let source = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(
        &source,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!out
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx" || d.code == "precision"));
    for node in raw["data"].as_array_mut().unwrap() {
        if node["attributes"]["ifccad::layout"]["kind"] == "Model" {
            node["attributes"]["ifccad::layout"]["bounds"] = json!({"min":[-9_007_199_254_740_993_i64,-1e6,-1e6],"max":[9_007_199_254_740_993_u64,1e6,1e6]});
        }
    }
    let source = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let error = ifccad_source_to_cad_document(
            &source,
            IfccadToCadOptions {
                preservation: Default::default(),
                loss_policy,
                geometry_tolerance: IfccadGeometryTolerance::drawing_units(1e10).unwrap(),
            },
        )
        .err()
        .expect("projection precision");
        assert!(
            matches!(error,IfccadConversionError::Unsupported(ref diagnostics) if diagnostics.iter().any(|d|d.code=="precision"&&d.location.ends_with("bounds")))
        );
    }
}
