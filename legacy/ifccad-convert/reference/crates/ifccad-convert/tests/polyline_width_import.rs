mod common;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{CadDocument, DxfReader, DxfVersion, EntityType, LwPolyline, Vector2};
use std::io::Cursor;
fn classic() -> CadDocument {
    let read = DxfReader::from_reader(Cursor::new(
        include_bytes!("fixtures/polyline-width-defaults.dxf").to_vec(),
    ))
    .unwrap()
    .read()
    .unwrap();
    let mut d = common::cad();
    let mut e = read
        .entities()
        .find(|e| matches!(e, EntityType::Polyline2D(_)))
        .unwrap()
        .clone();
    e.common_mut().owner_handle = Default::default();
    d.add_entity(e).unwrap();
    d
}
fn polyline(d: &IfccadDocument) -> &IfccadPlanarPolylineWidths {
    d.model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::PlanarPolyline { widths, .. } = &e.kind {
                Some(widths)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn classic_import_retains_defaults_and_zero_overrides() {
    let cad = classic();
    let native = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let IfccadPlanarPolylineWidths::Defaulted {
        default_start_width,
        default_end_width,
        overrides,
    } = polyline(native.document())
    else {
        panic!("classic mode lost")
    };
    assert_eq!([*default_start_width, *default_end_width], [0.2, 0.4]);
    assert_eq!(
        overrides
            .iter()
            .map(|w| (w.start, w.end))
            .collect::<Vec<_>>(),
        vec![
            (None, None),
            (Some(0.), None),
            (None, Some(0.)),
            (Some(0.2), Some(0.4))
        ]
    );
    assert!(!native.geometry_assessment().polyline_widths().is_empty());
}
#[test]
fn defaulted_exports_classic_without_baking() {
    let source =
        cad_document_to_ifccad_document(&classic(), common::metadata(), Default::default())
            .unwrap();
    let out = ifccad_document_to_cad_document(
        source.document(),
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let p = out
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Polyline2D(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!([p.start_width, p.end_width], [0.2, 0.4]);
    assert_eq!(p.vertices[1].start_width, Some(0.));
    assert!(p.vertices[1].end_width.is_none());
}
#[test]
fn constant_lw_import_remains_constant_and_band_proven() {
    let mut cad = common::cad();
    let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(10., 0.)]);
    p.constant_width = 0.15;
    cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    let out = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        polyline(out.document()),
        &IfccadPlanarPolylineWidths::Constant { width: 0.15 }
    );
    let assessment = out.geometry_assessment().polyline_widths();
    assert_eq!(assessment.len(), 1);
    assert_eq!(
        assessment[0].segments[0].band_coverage,
        IfccadPolylineWidthBandCoverage::Proven
    );
}
#[test]
fn dual_nonzero_controls_are_located_loss() {
    let mut cad = common::cad();
    let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(10., 0.)]);
    p.constant_width = 1.;
    p.vertices[0].start_width = 2.;
    cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "polyline-width-unsupported"));
    assert!(out.document().model.entities.is_empty());
    assert!(cad_document_to_ifccad_document(
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
fn target_dwg_reports_inherited_leaf_loss() {
    let source =
        cad_document_to_ifccad_document(&classic(), common::metadata(), Default::default())
            .unwrap();
    let target = IfccadPolylineWidthCadTarget {
        format: IfccadPolylineWidthCadFormat::Dwg,
        version: DxfVersion::AC1032,
    };
    let issues =
        assess_ifccad_polyline_width_target(source.document(), target, IfccadLossPolicy::Allow)
            .unwrap();
    assert!(issues.iter().any(|d| d.code == "polyline-width-dependency"));
    assert!(assess_ifccad_polyline_width_target(
        source.document(),
        target,
        IfccadLossPolicy::Reject
    )
    .is_err());
    let target = IfccadPolylineWidthCadTarget {
        format: IfccadPolylineWidthCadFormat::AsciiDxf,
        version: DxfVersion::AC1032,
    };
    assert!(assess_ifccad_polyline_width_target(
        source.document(),
        target,
        IfccadLossPolicy::Reject
    )
    .unwrap()
    .is_empty());
}

#[test]
fn absent_override_leaves_are_not_expanded_by_source_audit() {
    let source =
        cad_document_to_encoded_ifccad(&classic(), common::metadata(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(
        source.validated_source(),
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!out
        .diagnostics()
        .iter()
        .any(|d| matches!(d.code, "foreign-ifcx" | "precision")));
    let mut json: serde_json::Value = serde_json::from_slice(source.encoded().bytes()).unwrap();
    let node = json["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| {
            n["attributes"]
                .get("ifccad::geom::planarPolyline")
                .is_some()
        })
        .unwrap();
    node["attributes"]["ifccad::geom::planarPolyline"]["widths"]["overrides"][3]["start"] =
        serde_json::json!(9007199254740993u64);
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&json).unwrap(), Default::default()).unwrap();
    let Err(IfccadConversionError::Unsupported(issues)) =
        ifccad_source_to_cad_document(&loaded, Default::default())
    else {
        panic!("raw width precision must remain visible")
    };
    assert!(issues
        .iter()
        .any(|d| d.code == "precision" && d.location.contains("widths")));
}
#[test]
fn unused_negative_defaults_and_dormant_nan_are_source_errors() {
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let mut cad = classic();
        let p = cad
            .entities_mut()
            .find_map(|e| {
                if let EntityType::Polyline2D(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        for v in &mut p.vertices {
            v.start_width = Some(1.)
        }
        p.start_width = -1.;
        assert!(cad_document_to_ifccad_document(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy: policy,
                ..Default::default()
            }
        )
        .is_err());
        let mut cad = classic();
        let p = cad
            .entities_mut()
            .find_map(|e| {
                if let EntityType::Polyline2D(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        p.vertices.last_mut().unwrap().end_width = Some(f64::NAN);
        assert!(cad_document_to_ifccad_document(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy: policy,
                ..Default::default()
            }
        )
        .is_err());
    }
}
#[test]
fn coincident_wide_band_is_unassessed_without_a_semantic_rejection() {
    let mut cad = common::cad();
    let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(0., 0.)]);
    p.constant_width = 1.;
    cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    let out = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let a = out.geometry_assessment().polyline_widths();
    assert_eq!(
        a[0].segments[0].band_coverage,
        IfccadPolylineWidthBandCoverage::Unassessed
    );
    assert!(!out.geometry_assessment().is_complete());
    assert_eq!(
        polyline(out.document()),
        &IfccadPlanarPolylineWidths::Constant { width: 1. }
    );
}
#[test]
fn classic_pattern_generation_and_mode_loss_are_independent() {
    let mut cad = classic();
    let p = cad
        .entities_mut()
        .find_map(|e| {
            if let EntityType::Polyline2D(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    p.flags = opencadcodec::entities::PolylineFlags::from_bits(128);
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    let back = ifccad_document_to_cad_document(out.document(), Default::default()).unwrap();
    let p = back
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Polyline2D(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(p.flags.bits() & 128, 128);
    let mut d = out.document().clone();
    let e = d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap();
    if let IfccadEntityKind::PlanarPolyline { widths, .. } = &mut e.kind {
        *widths = IfccadPlanarPolylineWidths::PerVertex {
            widths: vec![[0.; 2]; 4],
        };
    }
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "polyline-width-target"));
    assert!(ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
