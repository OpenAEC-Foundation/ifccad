use ocdraw::ocdraw::*;
use std::collections::BTreeSet;

fn doc(bytes: &[u8]) -> OcdrawDocument {
    load_ocdraw_bytes(bytes).ok().unwrap().into_document()
}
#[test]
fn all_current_content_survives_document_roundtrip() {
    for path in std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/conformance/next/ocdraw/valid"
    ))
    .unwrap()
    {
        let original = doc(&std::fs::read(path.unwrap().path()).unwrap());
        let encoded = encode_ocdraw_document(&original).unwrap();
        let after = doc(encoded.bytes());
        assert_eq!(original.drawing_id, after.drawing_id);
        assert_eq!(original.unit, after.unit);
        assert_eq!(original.plot_style_mode, after.plot_style_mode);
        assert_eq!(original.point_display, after.point_display);
        assert_eq!(original.line_pattern_scale, after.line_pattern_scale);
        assert_eq!(
            (
                original.next_entity_id,
                original.next_layer_id,
                original.next_layout_id,
                original.next_line_pattern_id
            ),
            (
                after.next_entity_id,
                after.next_layer_id,
                after.next_layout_id,
                after.next_line_pattern_id
            )
        );
        assert_eq!(original.scopes, after.scopes);
        assert_eq!(original.layers, after.layers);
        assert_eq!(original.layouts, after.layouts);
        assert_eq!(original.line_patterns, after.line_patterns);
        assert_eq!(original.block_definitions, after.block_definitions);
        assert_eq!(original.ucs_definitions, after.ucs_definitions);
        assert_eq!(original.workspace_state, after.workspace_state);
        assert_eq!(original.view_state, after.view_state);
        assert_eq!(original.model_windows, after.model_windows);
        assert_eq!(original.paper_canvases, after.paper_canvases);
        assert_eq!(original.viewport_workspaces, after.viewport_workspaces);
        assert_eq!(original.viewports, after.viewports);
        for e in &original.geometric_entities {
            assert_eq!(
                Some(e),
                after.geometric_entities.iter().find(|x| x.id == e.id)
            );
        }
        assert_eq!(
            encode_ocdraw_document(&original).unwrap().bytes(),
            encoded.bytes()
        );
    }
}
#[test]
fn document_encoding_preserves_sparse_ids_and_advanced_watermarks() {
    let mut d = doc(include_bytes!(
        "../conformance/next/ocdraw/valid/placed-geometry.ocdraw.json"
    ));
    for s in &mut d.scopes {
        s.id += 10;
    }
    for l in &mut d.layouts {
        l.id += 17;
        l.scope_id += 10;
    }
    d.next_layout_id = 24;
    for b in &mut d.block_definitions {
        b.scope_id += 10;
    }
    for e in &mut d.geometric_entities {
        if let DrawingGeometry::BlockInstance {
            definition_scope_id,
            ..
        } = &mut e.geometry
        {
            *definition_scope_id += 10;
        }
    }
    let old = d.geometric_entities[0].id;
    let new = 9_007_199_254_740_993;
    d.geometric_entities[0].id = new;
    for s in &mut d.scopes {
        for id in &mut s.entities {
            if *id == old {
                *id = new;
            }
        }
    }
    d.next_entity_id = u64::MAX;
    d.next_layer_id = u32::MAX;
    d.next_line_pattern_id = u32::MAX;
    d.scopes.reverse();
    let encoded = encode_ocdraw_document(&d).unwrap();
    let after = doc(encoded.bytes());
    assert_eq!(after.next_entity_id, u64::MAX);
    assert_eq!(after.next_layer_id, u32::MAX);
    assert_eq!(after.next_line_pattern_id, u32::MAX);
    assert_eq!(after.next_layout_id, 24);
    assert_eq!(after.scopes, d.scopes);
    assert!(after.geometric_entities.iter().any(|e| e.id == new));
}
#[test]
fn encoding_preserves_valid_enlarged_bounds_without_preparation() {
    let mut d = doc(include_bytes!(
        "../conformance/next/ocdraw/valid/placed-geometry.ocdraw.json"
    ));
    for s in &mut d.scopes {
        if !s.entities.is_empty() {
            s.bounds = Some(Bounds3d::new(
                Point3::new(-100., -100., -100.),
                Point3::new(100., 100., 100.),
            ));
        }
    }
    let after = doc(encode_ocdraw_document(&d).unwrap().bytes());
    assert_eq!(after.scopes, d.scopes);
    d.geometric_entities[0].geometry = DrawingGeometry::Line {
        start: [500., 0., 0.],
        end: [501., 0., 0.],
    };
    assert!(matches!(
        encode_ocdraw_document(&d),
        Err(OcdrawEncodeError::InvalidDocument(_))
    ));
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    assert!(encode_ocdraw_document(&d).is_ok());
}
#[test]
fn delete_rewrite_and_new_entity_do_not_reuse_ids() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("history", "mm")).unwrap();
    b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            LinePatternId(0),
        ))
        .unwrap();
    for _ in 0..4 {
        b.add_line(LineDefinition::new(layer, [0., 0., 0.], [1., 1., 0.]))
            .unwrap();
    }
    let mut d = b.build_document().unwrap();
    d.geometric_entities.retain(|e| e.id != 2);
    d.scopes[0].entities.retain(|id| *id != 2);
    d.next_entity_id = 6;
    d = doc(encode_ocdraw_document(&d).unwrap().bytes());
    assert_eq!(
        d.geometric_entities
            .iter()
            .map(|e| e.id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([1, 3, 4])
    );
    d.geometric_entities.retain(|e| e.id != 4);
    d.scopes[0].entities.retain(|id| *id != 4);
    d = doc(encode_ocdraw_document(&d).unwrap().bytes());
    assert_eq!(d.next_entity_id, 6);
    let mut e = d.geometric_entities[0].clone();
    e.id = 6;
    d.geometric_entities.push(e);
    d.scopes[0].entities.push(6);
    d.next_entity_id = 7;
    d.geometric_entities.retain(|e| e.id != 6);
    d.scopes[0].entities.retain(|id| *id != 6);
    assert_eq!(
        doc(encode_ocdraw_document(&d).unwrap().bytes()).next_entity_id,
        7
    );
}
#[test]
fn finish_uses_the_document_encoder() {
    let a = OcdrawBuilder::new(OcdrawBuildOptions::new("fresh", "mm"))
        .unwrap()
        .finish()
        .unwrap();
    let d = OcdrawBuilder::new(OcdrawBuildOptions::new("fresh", "mm"))
        .unwrap()
        .build_document()
        .unwrap();
    assert_eq!(a.bytes(), encode_ocdraw_document(&d).unwrap().bytes());
}
