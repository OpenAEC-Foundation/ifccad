mod common;
#[path = "support/dimension_graphics.rs"]
mod support;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::*;
use std::io::Cursor;
#[test]
fn paper_picture_uses_its_owner_domain_and_parent_placement_once() {
    let mut drawing = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/entities.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let index=drawing.model.entities.iter().position(|e|matches!(&e.as_native().unwrap().kind,IfccadEntityKind::Dimension(d) if d.graphics.is_some())).unwrap();
    let mut entity = drawing.model.entities.remove(index);
    let IfccadEntityKind::Dimension(d) = &mut entity.as_native_mut().unwrap().kind else {
        panic!()
    };
    d.placement.origin = [15., 25., 0.];
    let IfccadDimensionGraphicsKind::LineSegment { start, .. } =
        d.graphics.as_ref().unwrap().parts[0].kind
    else {
        panic!()
    };
    let expected = [start[0] + 15., start[1] + 25., start[2]];
    let paper = &mut drawing.paper_layouts[0];
    paper.entities.push(entity);
    paper.bounds = None;
    paper.bounds_quality = None;
    drawing.model.bounds = None;
    drawing.model.bounds_quality = None;
    let output = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    for route in 0..3 {
        let cad = reopened(output.document(), route);
        let restored =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let dim = restored
            .validated_source()
            .document()
            .paper_layouts
            .iter()
            .flat_map(|p| &p.entities)
            .find_map(|e| match &e.as_native()?.kind {
                IfccadEntityKind::Dimension(d) if d.graphics.is_some() => Some(d),
                _ => None,
            })
            .unwrap();
        let IfccadDimensionGraphicsKind::LineSegment { start, .. } =
            dim.graphics.as_ref().unwrap().parts[0].kind
        else {
            panic!()
        };
        let actual = dim
            .placement
            .coordinate_frame()
            .unwrap()
            .try_to_scope_spatial_point(geometry_kernel::Point3::new(start[0], start[1], start[2]))
            .unwrap()
            .components();
        assert_eq!(actual, expected);
    }
}
fn native() -> IfccadDocument {
    cad_document_to_ifccad_document(
        &support::picture(false),
        common::metadata(),
        Default::default(),
    )
    .unwrap()
    .document()
    .clone()
}
#[test]
fn refused_target_picture_does_not_change_the_definition_only_cache_anchor() {
    let cad = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/dimension-picture-text-reference.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let mut drawing = cad_document_to_ifccad_document(&cad, common::metadata(), Default::default())
        .unwrap()
        .document()
        .clone();
    let g = dim(&mut drawing).graphics.take().unwrap();
    let baseline = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    let anchor = baseline
        .document()
        .entities()
        .find_map(|e| match e {
            EntityType::Dimension(d) => Some(d.base().text_middle_point),
            _ => None,
        })
        .unwrap();
    let mut g = g;
    g.parts.push(IfccadDimensionGraphicsPart {
        layer_id: None,
        kind: IfccadDimensionGraphicsKind::Point {
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
        },
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        visible: true,
        line_pattern_scale: 1.,
        purpose: IfccadDimensionGraphicsPurpose::ReferencePoint,
    });
    dim(&mut drawing).graphics = Some(g);
    let target = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    let d = target
        .document()
        .entities()
        .find_map(|e| match e {
            EntityType::Dimension(d) => Some(d),
            _ => None,
        })
        .unwrap();
    assert!(d.base().block_name.is_empty());
    assert_eq!(d.base().text_middle_point, anchor);
}
#[test]
fn literal_ascii_cache_fraction_and_zero_wrap_survive_actual_writers() {
    let cad = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/dimension-picture-text-reference.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let imported =
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let mut drawing = imported.validated_source().document().clone();
    let g = dim(&mut drawing)
        .graphics
        .as_ref()
        .expect("literal linked cache");
    assert_eq!(g.parts.len(), 1);
    let IfccadDimensionGraphicsKind::MText(text) = &g.parts[0].kind else {
        panic!()
    };
    assert_eq!(text.wrap_width, None);
    assert_eq!(text.placement.origin, [20., 2., 0.]);
    let content = text.content.clone();
    let target = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    for route in 0..3 {
        let cad = reopened(target.document(), route);
        let raw = cad
            .entities()
            .find_map(|e| match e {
                EntityType::MText(t) => Some(t),
                _ => None,
            })
            .unwrap();
        assert_eq!(raw.rectangle_width, 0.);
        assert!(raw.value.contains(r"\S1/4;"));
        let restored =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let mut d = restored.validated_source().document().clone();
        let IfccadDimensionGraphicsKind::MText(t) =
            &dim(&mut d).graphics.as_ref().unwrap().parts[0].kind
        else {
            panic!()
        };
        assert_eq!(t.content, content);
    }
}
#[test]
fn every_allowed_picture_family_survives_actual_cad_readback() {
    let mut drawing = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/entities.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut seen = std::collections::HashSet::new();
    let parts:Vec<_>=drawing.model.entities.iter().filter_map(|e|e.as_native()).filter_map(|e|IfccadDimensionGraphicsKind::try_from(e.kind.clone()).ok()).filter(|kind|!matches!(kind,IfccadDimensionGraphicsKind::Hatch(h) if !matches!(h.fill,ifccad::geometry_kernel::hatch::HatchFill::Solid)||h.gradient.is_some()||h.association_enabled)).filter(|k|seen.insert(std::mem::discriminant(k))).map(|kind|IfccadDimensionGraphicsPart{layer_id:None,kind,appearance:IfccadEntityAppearance{color:IfccadMode::ByLayer,opacity:IfccadMode::ByLayer,line_pattern:IfccadMode::ByLayer,line_weight:IfccadMode::ByLayer},visible:true,line_pattern_scale:1.,purpose:IfccadDimensionGraphicsPurpose::Display}).collect();
    assert_eq!(parts.len(), 14);
    let d = drawing
        .model
        .entities
        .iter_mut()
        .find_map(|e| match &mut e.as_native_mut()?.kind {
            IfccadEntityKind::Dimension(d) => Some(d),
            _ => None,
        })
        .unwrap();
    d.graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts,
    });
    let output = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    for route in 0..3 {
        let cad = reopened(output.document(), route);
        let restored =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let d = restored
            .validated_source()
            .document()
            .model
            .entities
            .iter()
            .find_map(|e| match &e.as_native()?.kind {
                IfccadEntityKind::Dimension(d)
                    if d.graphics.as_ref().is_some_and(|g| g.parts.len() == 14) =>
                {
                    Some(d)
                }
                _ => None,
            })
            .expect("all families retained in one complete picture");
        let families: std::collections::HashSet<_> = d
            .graphics
            .as_ref()
            .unwrap()
            .parts
            .iter()
            .map(|p| std::mem::discriminant(&p.kind))
            .collect();
        assert_eq!(families.len(), 14);
    }
}
fn dim(d: &mut IfccadDocument) -> &mut IfccadDimension {
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    v
}
fn reopened(d: &CadDocument, route: usize) -> CadDocument {
    match route {
        0 => DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap(),
        1 => DxfReader::from_reader(Cursor::new(
            DxfWriter::new_binary(d).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap(),
        _ => DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap(),
    }
}
#[test]
fn complete_picture_is_linked_and_survives_ascii_binary_dxf_and_ac1032() {
    let n = native();
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    for route in 0..3 {
        let cad = reopened(out.document(), route);
        let v = cad
            .entities()
            .find_map(|e| match e {
                EntityType::Dimension(v) => Some(v),
                _ => None,
            })
            .unwrap();
        assert!(!v.base().block_name.is_empty());
        let b = cad
            .block_records
            .get(&v.base().block_name)
            .expect("linked picture exists");
        assert!(b.flags.anonymous);
        assert_eq!(b.base_point, Vector3::ZERO);
        assert_eq!(b.entity_handles.len(), 3);
        assert_eq!(
            cad.model_space_entities()
                .filter(|e| matches!(e, EntityType::Insert(_)))
                .count(),
            0
        );
        let returned =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let mut d = returned.validated_source().document().clone();
        let g = dim(&mut d).graphics.as_ref().unwrap();
        assert_eq!(g.parts.len(), 3);
        assert!(matches!(
            g.parts[2].kind,
            IfccadDimensionGraphicsKind::Solid(_)
        ));
        assert_eq!(
            g.parts[1].purpose,
            IfccadDimensionGraphicsPurpose::ReferencePoint
        );
    }
}
#[test]
fn source_snapshot_target_passes_reject_generated_origin_stays_a_located_loss() {
    let mut n = native();
    let out = ifccad_document_to_cad_document(
        &n,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!out
        .diagnostics()
        .iter()
        .any(|d| d.location.ends_with("/pictureBlock")));
    dim(&mut n).graphics.as_mut().unwrap().origin = IfccadDimensionGraphicsOrigin::Generated;
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "dimension-graphics-helper-state"
            && d.location.ends_with("/graphics/origin")));
    let v = out
        .document()
        .entities()
        .find_map(|e| match e {
            EntityType::Dimension(v) => Some(v),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        out.document()
            .block_records
            .get(&v.base().block_name)
            .unwrap()
            .entity_handles
            .len(),
        3
    );
    assert!(ifccad_document_to_cad_document(
        &n,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn absent_empty_nonempty_and_parent_placement_changes_remain_distinct() {
    let mut n = native();
    for parts in [
        None,
        Some(vec![]),
        Some(dim(&mut n).graphics.as_ref().unwrap().parts.clone()),
    ] {
        dim(&mut n).graphics = parts.map(|parts| IfccadDimensionGraphics {
            origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
            parts,
        });
        dim(&mut n).placement.origin[0] += 5.;
        let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
        let cad = out.document();
        let v = cad
            .entities()
            .find_map(|e| match e {
                EntityType::Dimension(v) => Some(v),
                _ => None,
            })
            .unwrap();
        match &dim(&mut n).graphics {
            None => assert!(v.base().block_name.is_empty()),
            Some(g) => {
                let b = cad.block_records.get(&v.base().block_name).unwrap();
                assert_eq!(b.entity_handles.len(), g.parts.len());
                if !g.parts.is_empty() {
                    let EntityType::Line(l) = cad.get_entity(b.entity_handles[0]).unwrap() else {
                        panic!()
                    };
                    assert_eq!(l.start.x, 115.);
                }
            }
        }
    }
}

#[test]
fn operation_evidence_separates_absent_empty_transferred_and_failed_pictures() {
    let mut n = native();
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    assert_eq!(
        out.dimension_assessment().entries()[0].graphics_coverage,
        IfccadDimensionGraphicsCoverage::Transferred { part_count: 3 }
    );
    dim(&mut n).graphics.as_mut().unwrap().parts.clear();
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    assert_eq!(
        out.dimension_assessment().entries()[0].graphics_coverage,
        IfccadDimensionGraphicsCoverage::ExplicitEmpty
    );
    dim(&mut n).graphics = None;
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    assert_eq!(
        out.dimension_assessment().entries()[0].graphics_coverage,
        IfccadDimensionGraphicsCoverage::Absent
    );
    let out = cad_document_to_ifccad_document(
        &support::picture(true),
        common::metadata(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        out.dimension_assessment().entries()[0].graphics_coverage,
        IfccadDimensionGraphicsCoverage::NotTransferred
    );
}

#[test]
fn graphics_filled_surface_evidence_has_part_identity_and_never_certifies_pixels() {
    let n = native();
    let out = ifccad_document_to_cad_document(&n, Default::default()).unwrap();
    let a = out.geometry_assessment().filled_surfaces();
    assert!(a.iter().any(|v| matches!(
        v.source(),
        IfccadGeometryEntitySource::DimensionGraphicsPart { part_index: 2, .. }
    )));
    assert_eq!(
        out.dimension_assessment().entries()[0].display_coverage,
        IfccadDimensionDisplayCoverage::Unassessed
    );
}
#[test]
fn curved_and_tilted_pictures_keep_literal_locations_after_real_exchange() {
    use opencadcodec::entities::{Dimension, DimensionAligned};
    for (normal, line, a, b, elevation, start, want) in [
        (
            Vector3::UNIT_X,
            [3., 8., 0.],
            [3., 0., 0.],
            [3., 4., 5.],
            3.,
            [2., 7., 0.],
            [3., 2., 7.],
        ),
        (
            Vector3::new(0., 0., -1.),
            [10., 30., -5.],
            [0., 0., -5.],
            [3., 4., -5.],
            5.,
            [2., 7., 0.],
            [-2., 7., -5.],
        ),
    ] {
        let v = |p: [f64; 3]| Vector3::new(p[0], p[1], p[2]);
        let mut cad = CadDocument::new();
        support::block(
            &mut cad,
            "*Picture",
            vec![
                EntityType::Line(Line::from_points(
                    v(start),
                    v([start[0] + 4., start[1], 0.]),
                )),
                EntityType::Arc(Arc::from_center_radius_angles(
                    Vector3::new(2., 7., 0.),
                    2.,
                    0.4,
                    1.1,
                )),
            ],
        );
        let mut d = DimensionAligned::new(v(a), v(b));
        d.definition_point = v(line);
        d.base.definition_point = v(line);
        d.base.normal = normal;
        d.base.text_middle_point = Vector3::new(2., 7., elevation);
        d.base.block_name = "*Picture".into();
        cad.add_entity(EntityType::Dimension(Box::new(Dimension::Aligned(d))))
            .unwrap();
        let n =
            cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
        let out = ifccad_document_to_cad_document(n.document(), Default::default()).unwrap();
        for route in 0..3 {
            let cad = reopened(out.document(), route);
            let c = cad
                .entities()
                .find_map(|e| match e {
                    EntityType::Dimension(v) => Some(v),
                    _ => None,
                })
                .unwrap();
            assert!(!c.base().text_user_positioned);
            let r = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default())
                .unwrap();
            let mut d = r.validated_source().document().clone();
            let dimension = dim(&mut d);
            let IfccadDimensionGraphicsKind::LineSegment { start, .. } =
                &dimension.graphics.as_ref().unwrap().parts[0].kind
            else {
                panic!()
            };
            let actual = dimension
                .placement
                .coordinate_frame()
                .unwrap()
                .try_to_scope_spatial_point(geometry_kernel::Point3::new(
                    start[0], start[1], start[2],
                ))
                .unwrap()
                .components();
            assert_eq!(actual, want);
        }
    }
}
