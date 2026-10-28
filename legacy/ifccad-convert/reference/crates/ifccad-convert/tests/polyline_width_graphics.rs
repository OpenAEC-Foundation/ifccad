mod common;
#[path = "support/dimension_graphics.rs"]
mod support;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{entities::Insert, types::Color, *};
fn picture() -> CadDocument {
    let mut d = support::picture(false);
    let mut p = LwPolyline::from_points(vec![Vector2::new(-0.5, -0.5), Vector2::new(0.5, 0.5)]);
    p.constant_width = 0.15;
    p.common.color = Color::ByBlock;
    p.common.linetype = "ByBlock".into();
    support::block(&mut d, "WideArrow", vec![EntityType::LwPolyline(p)]);
    let owner = d.block_records.get("*DimensionPicture").unwrap().handle;
    for (x, rotation) in [
        (100., std::f64::consts::FRAC_PI_4),
        (140., 5. * std::f64::consts::FRAC_PI_4),
    ] {
        let mut i = Insert::new("WideArrow", Vector3::new(x, 10., 0.));
        i.rotation = rotation;
        i.set_x_scale(2.5);
        i.set_y_scale(2.5);
        i.set_z_scale(2.5);
        i.common.owner_handle = owner;
        d.add_entity(EntityType::Insert(i)).unwrap();
    }
    d
}
fn dimensions(d: &IfccadDocument) -> Vec<&IfccadDimension> {
    d.model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .filter_map(|e| {
            if let IfccadEntityKind::Dimension(d) = &e.kind {
                Some(d.as_ref())
            } else {
                None
            }
        })
        .collect()
}
fn graphics_source(s: &IfccadGeometryEntitySource) -> bool {
    match s {
        IfccadGeometryEntitySource::DimensionGraphicsPart { .. } => true,
        IfccadGeometryEntitySource::BlockOccurrence { path, leaf } => {
            path.iter().any(graphics_source) || graphics_source(leaf)
        }
        _ => false,
    }
}
#[test]
fn wide_picture_keeps_complete_marker_closure() {
    let cad = picture();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert_eq!(dimensions(d)[0].graphics.as_ref().unwrap().parts.len(), 5);
    let arrow = d.blocks.iter().find(|b| b.name == "WideArrow").unwrap();
    assert!(
        matches!(&arrow.entities[0].as_native().unwrap().kind,IfccadEntityKind::PlanarPolyline{widths:IfccadPlanarPolylineWidths::Constant{width},..} if *width==0.15)
    );
    let records = out.geometry_assessment().polyline_widths();
    assert_eq!(
        records
            .iter()
            .filter(|r| graphics_source(&r.source))
            .count(),
        2
    );
    assert!(records
        .iter()
        .all(|r| r.segments[0].band_coverage == IfccadPolylineWidthBandCoverage::Proven));
    let back = ifccad_document_to_cad_document(d, Default::default()).unwrap();
    assert_eq!(
        back.geometry_assessment()
            .polyline_widths()
            .iter()
            .filter(|r| graphics_source(&r.source))
            .count(),
        2
    );
    let dimension = back
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        back.document()
            .block_records
            .get(&dimension.base().block_name)
            .unwrap()
            .entity_handles
            .len(),
        5
    );
}
#[test]
fn ordinary_and_picture_insert_share_wide_marker() {
    let mut cad = picture();
    support::ordinary_insert(&mut cad, "WideArrow");
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert_eq!(d.blocks.iter().filter(|b| b.name == "WideArrow").count(), 1);
    assert!(out.geometry_assessment().polyline_widths().len() >= 4);
    ifccad_document_to_cad_document(d, Default::default()).unwrap();
}
#[test]
fn classic_defaulted_part_keeps_zero_overrides() {
    let read = DxfReader::from_reader(std::io::Cursor::new(
        include_bytes!("fixtures/polyline-width-defaults.dxf").to_vec(),
    ))
    .unwrap()
    .read()
    .unwrap();
    let mut p = read
        .entities()
        .find(|e| matches!(e, EntityType::Polyline2D(_)))
        .unwrap()
        .clone();
    let mut cad = support::picture(false);
    p.common_mut().handle = Default::default();
    p.common_mut().owner_handle = cad.block_records.get("*DimensionPicture").unwrap().handle;
    cad.add_entity(p).unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    let part = &dimensions(d)[0].graphics.as_ref().unwrap().parts[3];
    let IfccadDimensionGraphicsKind::PlanarPolyline {
        widths:
            IfccadPlanarPolylineWidths::Defaulted {
                default_start_width,
                default_end_width,
                overrides,
            },
        ..
    } = &part.kind
    else {
        panic!("classic part lost")
    };
    assert_eq!([*default_start_width, *default_end_width], [0.2, 0.4]);
    assert_eq!(overrides[1].start, Some(0.));
    assert!(overrides[1].end.is_none());
    let back = ifccad_document_to_cad_document(d, Default::default()).unwrap();
    assert!(back.document().entities().any(|e|matches!(e,EntityType::Polyline2D(p) if p.vertices[1].start_width==Some(0.)&&p.vertices[1].end_width.is_none())));
}
#[test]
fn failed_picture_discards_resources_and_bands_but_preserves_ordinary_user() {
    let mut cad = picture();
    support::ordinary_insert(&mut cad, "WideArrow");
    let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(1., 0.)]);
    p.constant_width = 1.;
    p.vertices[0].start_width = 2.;
    p.common.owner_handle = cad.block_records.get("*DimensionPicture").unwrap().handle;
    cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert!(dimensions(d)[0].graphics.is_none());
    assert!(d.blocks.iter().any(|b| b.name == "WideArrow"));
    assert!(!out
        .geometry_assessment()
        .polyline_widths()
        .iter()
        .any(|r| graphics_source(&r.source)));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "polyline-width-unsupported"));
    let back = ifccad_document_to_cad_document(d, Default::default()).unwrap();
    assert!(!back
        .document()
        .block_records
        .iter()
        .any(|b| b.name.starts_with("*D") && !b.entity_handles.is_empty()));
}

#[test]
fn outer_insert_amplifies_width_error_with_an_exact_centerline() {
    let mut cad = CadDocument::new();
    let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(10., 0.)]);
    p.constant_width = 1.;
    cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    let mut d = out.document().clone();
    let mut e = d.model.entities[0].clone();
    if let IfccadEntityKind::PlanarPolyline { placement, .. } = &mut e.as_native_mut().unwrap().kind
    {
        placement.y_axis[1] = 1.0f64.next_up();
    }
    d.blocks = vec![IfccadBlockDefinition {
        id: 1,
        name: "Wide".into(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds: None,
        bounds_quality: None,
        base_point: [0.; 3],
        insertion_unit: "unitless".into(),
        entities: vec![e],
    }];
    d.id_counters.next_block_id = 2;
    let mut instance = d.model.entities[0].clone();
    instance.as_native_mut().unwrap().id = 2;
    instance.as_native_mut().unwrap().kind = IfccadEntityKind::BlockInstance {
        definition_id: 1,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1., 1e10, 1.],
        },
    };
    d.model.entities = vec![instance];
    d.id_counters.next_entity_id = 3;
    d.model.bounds = None;
    d.model.bounds_quality = None;
    validate_ifccad_document(&d).unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let Err(IfccadConversionError::Geometry(error)) = ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                loss_policy,
                ..Default::default()
            },
        ) else {
            panic!("required amplified band failure must be hard")
        };
        assert_eq!(error.reason, IfccadGeometryFailureReason::ProvenExceedance);
        assert_eq!(error.vertex_index, Some(0));
    }
}

#[test]
fn hidden_invalid_dormant_classic_width_is_not_waived_by_loss_policy() {
    let mut d = picture();
    let mut p = opencadcodec::entities::Polyline2D::new();
    p.add_vertex(opencadcodec::entities::Vertex2D::new(Vector3::new(
        0., 0., 0.,
    )));
    p.add_vertex(opencadcodec::entities::Vertex2D::new(Vector3::new(
        1., 0., 0.,
    )));
    p.vertices[1].end_width = Some(-1.);
    p.common.invisible = true;
    p.common.owner_handle = d.block_records.get("*DimensionPicture").unwrap().handle;
    d.add_entity(EntityType::Polyline2D(p)).unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(cad_document_to_encoded_ifccad(
            &d,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy,
                ..Default::default()
            }
        )
        .is_err());
    }
    let out = cad_document_to_encoded_ifccad(
        &d,
        common::metadata(),
        CadToIfccadOptions {
            error_policy: CadImportErrorPolicy::SkipInvalidEntities,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(dimensions(out.validated_source().document()).is_empty());
    assert!(!out
        .geometry_assessment()
        .polyline_widths()
        .iter()
        .any(|r| graphics_source(&r.source)));
}
#[test]
fn failed_picture_does_not_poison_later_shared_marker_picture() {
    let mut d = picture();
    let owner = d.block_records.get("*DimensionPicture").unwrap().handle;
    let mut bad = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(1., 0.)]);
    bad.thickness = 1.;
    bad.common.owner_handle = owner;
    d.add_entity(EntityType::LwPolyline(bad)).unwrap();
    support::block(
        &mut d,
        "*LaterPicture",
        vec![EntityType::Insert(Insert::new("WideArrow", Vector3::ZERO))],
    );
    support::dimension(&mut d, "*LaterPicture");
    let out = cad_document_to_encoded_ifccad(&d, common::metadata(), Default::default()).unwrap();
    let dims = dimensions(out.validated_source().document());
    assert_eq!(dims.len(), 2);
    assert_eq!(dims.iter().filter(|d| d.graphics.is_some()).count(), 1);
    assert_eq!(
        out.geometry_assessment()
            .polyline_widths()
            .iter()
            .filter(|r| graphics_source(&r.source))
            .count(),
        1
    );
}
#[test]
fn tilted_and_negative_z_wide_pictures_keep_actual_bands() {
    for normal in [Vector3::UNIT_X, -Vector3::UNIT_Z] {
        let mut d = picture();
        for e in d.entities_mut() {
            if let EntityType::Dimension(v) = e {
                v.base_mut().normal = normal;
                v.base_mut().text_middle_point.z = 3.;
            }
        }
        let out =
            cad_document_to_encoded_ifccad(&d, common::metadata(), Default::default()).unwrap();
        assert!(dimensions(out.validated_source().document())[0]
            .graphics
            .is_some());
        let back =
            ifccad_document_to_cad_document(out.validated_source().document(), Default::default())
                .unwrap();
        assert!(back
            .geometry_assessment()
            .polyline_widths()
            .iter()
            .filter(|r| graphics_source(&r.source))
            .all(|r| r
                .segments
                .iter()
                .all(|s| s.band_coverage == IfccadPolylineWidthBandCoverage::Proven)));
    }
}
