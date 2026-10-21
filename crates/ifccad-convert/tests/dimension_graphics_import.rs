mod common;
#[path = "support/dimension_graphics.rs"]
mod support;
use ifccad::*;
use ifccad_convert::*;
fn dimensions(d: &IfccadDocument) -> Vec<&IfccadDimension> {
    d.model
        .entities
        .iter()
        .filter_map(|e| match &e.as_native()?.kind {
            IfccadEntityKind::Dimension(v) => Some(v.as_ref()),
            _ => None,
        })
        .collect()
}
#[test]
fn picture_referenced_by_an_unused_dimension_style_remains_an_ordinary_resource() {
    for legacy_name in [false, true] {
        let mut cad = support::picture(false);
        let root = cad.block_records.get("*DimensionPicture").unwrap().handle;
        let mut style = opencadcodec::DimStyle::new("Unused picture marker");
        style.handle = cad.allocate_handle();
        if legacy_name {
            style.dimblk_name = "*DimensionPicture".into();
        } else {
            style.dimblk = root;
        }
        cad.dim_styles.add(style).unwrap();
        let out =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        assert!(
            out.mappings().blocks.ifccad_id(root).is_some(),
            "style resource must not be consumed: legacy={legacy_name}"
        );
        assert_eq!(
            dimensions(out.validated_source().document())[0]
                .graphics
                .as_ref()
                .unwrap()
                .parts
                .len(),
            3
        );
    }
}

#[test]
fn capture_picture_is_atomic() {
    let cad = support::picture(false);
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    let g = dimensions(d)[0]
        .graphics
        .as_ref()
        .expect("complete picture");
    assert_eq!(g.parts.len(), 3);
    assert_eq!(g.origin, IfccadDimensionGraphicsOrigin::SourceSnapshot);
    assert!(d.blocks.is_empty());
    assert_eq!(d.model.entities.len(), 1);
    assert_eq!(out.mappings().entities.iter().count(), 1);
    assert!(!out
        .diagnostics()
        .iter()
        .any(|v| v.location.ends_with("/pictureBlock")));
}
#[test]
fn malformed_hidden_picture_part_obeys_independent_loss_and_recovery_policies() {
    use opencadcodec::{EntityType, Line};
    for losses in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let mut cad = support::picture(false);
        let handle = cad
            .block_records
            .get("*DimensionPicture")
            .unwrap()
            .entity_handles[0];
        if let EntityType::Line(line) = cad.get_entity_mut(handle).unwrap() {
            line.end.x = f64::NAN;
            line.common.invisible = true;
        } else {
            panic!()
        }
        let good = cad
            .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 0., 0.)))
            .unwrap();
        assert!(cad_document_to_encoded_ifccad(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy: losses,
                ..Default::default()
            }
        )
        .is_err());
        let result = cad_document_to_encoded_ifccad(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy: losses,
                error_policy: CadImportErrorPolicy::SkipInvalidEntities,
                ..Default::default()
            },
        );
        if losses == IfccadLossPolicy::Reject {
            assert!(result.is_err());
            continue;
        }
        let out = result.unwrap();
        assert!(dimensions(out.validated_source().document()).is_empty());
        assert!(out.mappings().entities.ifccad_id(good).is_some());
        let report = out.partial_open_report().unwrap();
        assert_eq!(report.errors.len(), 2);
        assert!(report.errors.iter().any(|e| e
            .source
            .as_ref()
            .is_some_and(|s| s.handle == handle.to_string())));
        assert!(report
            .entities
            .iter()
            .all(|e| e.status == CadAuditStatus::Omitted));
        assert!(out.mappings().entities.ifccad_id(handle).is_none());
        assert!(matches!(cad.get_entity(handle).unwrap(),EntityType::Line(l) if l.end.x.is_nan()));
    }
}
#[test]
fn shared_picture_uses_are_accounted() {
    let mut cad = support::picture(false);
    support::dimension(&mut cad, "*DimensionPicture");
    support::ordinary_insert(&mut cad, "*DimensionPicture");
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert_eq!(dimensions(d).len(), 2);
    assert!(dimensions(d)
        .iter()
        .all(|v| v.graphics.as_ref().unwrap().parts.len() == 3));
    assert_eq!(d.blocks.len(), 1);
    assert_eq!(d.blocks[0].entities.len(), 3);
    assert_eq!(d.model.entities.len(), 3);
}
#[test]
fn helpers_and_layer_context_are_retained() {
    let cad = support::picture(false);
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    let g = dimensions(d)[0].graphics.as_ref().unwrap();
    assert_eq!(g.parts[0].layer_id, None);
    assert_eq!(
        g.parts[1].purpose,
        IfccadDimensionGraphicsPurpose::ReferencePoint
    );
    let id = g.parts[1].layer_id.unwrap();
    assert_eq!(
        d.layers.iter().find(|l| l.id == id).unwrap().name,
        "Defpoints"
    );
    let IfccadDimensionGraphicsKind::LineSegment { start, end } = &g.parts[0].kind else {
        panic!()
    };
    assert_eq!(*start, [0., 0., 0.]);
    assert_eq!(*end, [40., 0., 0.]);
}
#[test]
fn red_wide_marker_does_not_become_partial_graphics() {
    let cad = support::picture(true);
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert!(dimensions(out.validated_source().document())[0]
        .graphics
        .is_none());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "dimension-graphics-unsupported"));
    assert!(cad_document_to_encoded_ifccad(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn complete_pictures_in_empty_unused_and_nested_marker_contexts_are_distinct() {
    use opencadcodec::{
        entities::{Insert, Solid},
        *,
    };
    let mut cad = CadDocument::new();
    support::block(&mut cad, "*EmptyPicture", vec![]);
    support::dimension(&mut cad, "*EmptyPicture");
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert!(dimensions(out.validated_source().document())[0]
        .graphics
        .as_ref()
        .unwrap()
        .parts
        .is_empty());
    let q = [
        Vector3::new(0., 0., 0.),
        Vector3::new(3., 0., 0.),
        Vector3::new(0., 2., 0.),
        Vector3::new(3., 2., 0.),
    ];
    let mut solid = Solid::new(q[0], q[1], q[2], q[3]);
    solid.thickness = -3.;
    let mut trace = solid.clone();
    trace.is_trace = true;
    let mut cad = CadDocument::new();
    support::block(
        &mut cad,
        "Arrow",
        vec![EntityType::Solid(solid), EntityType::Solid(trace)],
    );
    let mut nested = Insert::new("Arrow", Vector3::new(1., 2., 0.));
    nested.set_x_scale(-2.);
    nested.set_y_scale(3.);
    support::block(&mut cad, "Joint", vec![EntityType::Insert(nested)]);
    support::block(
        &mut cad,
        "*NestedPicture",
        vec![EntityType::Insert(Insert::new(
            "Joint",
            Vector3::new(110., 10., 0.),
        ))],
    );
    support::dimension(&mut cad, "*NestedPicture");
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert_eq!(d.blocks.len(), 2);
    assert_eq!(dimensions(d)[0].graphics.as_ref().unwrap().parts.len(), 1);
    assert!(out.geometry_assessment().domains()[0].assessed_vertices() >= 48);
    // An unsupported sibling in the nested marker removes the whole picture.
    let mut wide = LwPolyline::new();
    wide.constant_width = 0.15;
    wide.add_vertex(entities::LwVertex::new(Vector2::new(0., 0.)));
    wide.add_vertex(entities::LwVertex::new(Vector2::new(1., 1.)));
    wide.common.owner_handle = cad.block_records.get("Arrow").unwrap().handle;
    cad.add_entity(EntityType::LwPolyline(wide)).unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert!(dimensions(out.validated_source().document())[0]
        .graphics
        .is_none());
}
#[test]
fn tilted_negative_z_and_elevated_picture_points_match_literal_owner_positions() {
    use opencadcodec::{
        entities::{Dimension, DimensionAligned},
        *,
    };
    for (normal, elevation, line, first, second, start, want) in [
        (
            Vector3::UNIT_Z,
            4.,
            [100., 10., 4.],
            [100., 0., 4.],
            [140., 0., 4.],
            [100., 10., 4.],
            [100., 10., 4.],
        ),
        (
            Vector3::UNIT_X,
            3.,
            [3., 8., 0.],
            [3., 0., 0.],
            [3., 4., 5.],
            [2., 7., 0.],
            [3., 2., 7.],
        ),
        (
            Vector3::new(0., 0., -1.),
            5.,
            [10., 30., -5.],
            [0., 0., -5.],
            [3., 4., -5.],
            [2., 7., 0.],
            [-2., 7., -5.],
        ),
    ] {
        let v = |p: [f64; 3]| Vector3::new(p[0], p[1], p[2]);
        let mut cad = CadDocument::new();
        support::block(
            &mut cad,
            "*Picture",
            vec![EntityType::Line(Line::from_points(
                v(start),
                v([start[0] + 4., start[1], start[2]]),
            ))],
        );
        let mut dim = DimensionAligned::new(v(first), v(second));
        dim.definition_point = v(line);
        dim.base.definition_point = v(line);
        dim.base.normal = normal;
        dim.base.block_name = "*Picture".into();
        dim.base.text_middle_point = Vector3::new(0., 0., elevation);
        cad.add_entity(EntityType::Dimension(Box::new(Dimension::Aligned(dim))))
            .unwrap();
        let out =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let d = dimensions(out.validated_source().document())[0];
        let IfccadDimensionGraphicsKind::LineSegment { start, .. } =
            &d.graphics.as_ref().unwrap().parts[0].kind
        else {
            panic!()
        };
        let p = d
            .placement
            .coordinate_frame()
            .unwrap()
            .try_to_scope_spatial_point(geometry_kernel::Point3::new(start[0], start[1], start[2]))
            .unwrap()
            .components();
        assert_eq!(p, want);
    }
}
#[test]
fn literal_spaces_wrap_zero_and_stacked_cache_text_survive_independently_of_definition() {
    use opencadcodec::*;
    for text in [" ", r#"4'-0\S1/4;""#] {
        let mut cad = CadDocument::new();
        let mut m = MText::new();
        m.value = text.into();
        m.insertion_point = Vector3::new(120., 12., 0.);
        m.rectangle_width = 0.;
        m.height = 2.5;
        support::block(
            &mut cad,
            "*TextPicture",
            vec![EntityType::MText(Box::new(m))],
        );
        support::dimension(&mut cad, "*TextPicture");
        let out =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let d = dimensions(out.validated_source().document())[0];
        let IfccadDimensionGraphicsKind::MText(m) = &d.graphics.as_ref().unwrap().parts[0].kind
        else {
            panic!()
        };
        assert_eq!(m.wrap_width, None);
        assert_eq!(m.placement.origin, [20., 2., 0.]);
        assert!(!m.content.is_empty());
    }
}

#[test]
fn complete_source_snapshot_keeps_unrelated_style_losses_under_reject() {
    let cad = support::picture(false);
    let Err(IfccadConversionError::Unsupported(losses)) = cad_document_to_encoded_ifccad(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    ) else {
        panic!("existing style loss must remain visible")
    };
    assert!(losses.iter().any(|d| d.location.ends_with("/DIMDLI")));
    assert!(!losses.iter().any(
        |d| d.code.starts_with("dimension-graphics-") || d.location.ends_with("/pictureBlock")
    ));
}

#[test]
fn stored_picture_curves_are_proved_in_their_own_parameter_frames() {
    use opencadcodec::*;
    let mut cad = CadDocument::new();
    let mut ellipse =
        Ellipse::from_center_axes(Vector3::new(120., 10., 0.), Vector3::new(5., 0., 0.), 0.5);
    ellipse.start_parameter = 0.25;
    ellipse.end_parameter = 1.25;
    support::block(
        &mut cad,
        "*Curves",
        vec![
            EntityType::Circle(Circle::from_center_radius(Vector3::new(110., 10., 0.), 3.)),
            EntityType::Arc(Arc::from_center_radius_angles(
                Vector3::new(115., 10., 0.),
                3.,
                0.4,
                1.1,
            )),
            EntityType::Ellipse(ellipse),
        ],
    );
    support::dimension(&mut cad, "*Curves");
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert_eq!(
        dimensions(out.validated_source().document())[0]
            .graphics
            .as_ref()
            .unwrap()
            .parts
            .len(),
        3
    );
}
