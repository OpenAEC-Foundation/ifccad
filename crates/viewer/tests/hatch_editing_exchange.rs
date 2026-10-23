use ifccad::{geometry_kernel::hatch::*, *};
use viewer::export_ifccad_bytes_with_options;
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;

#[test]
fn direct_original_definition_survives_native_storage_and_all_twelve_physical_targets() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use ifccad_convert::opencadcodec::*;
    use std::io::Cursor;
    let mut d = fixture::drawing();
    for layer in &mut d.layers {
        layer.appearance.line_weight = 0.09;
    }
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = HatchFill::LinePattern(HatchLinePattern {
        name: Some("Original".into()),
        description: Some(String::new()),
        origin: [0., 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![HatchLineFamily {
            angle: 0.,
            base_point: [89.80256121069161, 0.],
            offset: [0., 2.],
            dashes: vec![],
        }],
    });
    h.classic_settings = Some(HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::LinePattern),
        creation_type: Some(HatchCreationType::Custom),
        name: Some("Original".into()),
        description: Some(String::new()),
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: 0.,
            base_point: [89.80256121069161, 2f64.powi(-40)],
            offset_vector: [0., 2.],
            dash_lengths: vec![],
        }]),
        ..Default::default()
    });
    for cbor in [false, true] {
        let bytes = if cbor {
            encode_ifccad_cbor_document(&d).unwrap().bytes().to_vec()
        } else {
            encode_ifccad_document(&d).unwrap().bytes().to_vec()
        };
        for format in ["dxf", "dwg"] {
            for version in ["AC1015", "AC1018", "AC1021", "AC1024", "AC1027", "AC1032"] {
                let out = export_ifccad_bytes_with_options(
                    if cbor {
                        "direct.ifccad.cbor"
                    } else {
                        "direct.ifcx"
                    },
                    &bytes,
                    format,
                    version,
                    r#"{"lossPolicy":"reject"}"#,
                );
                assert!(
                    out["failure"].is_null(),
                    "{format}/{version}: {}",
                    out["failure"]
                );
                assert_eq!(out["export"]["fileCheck"]["cadReadback"], true);
                assert_eq!(out["export"]["fileCheck"]["ifccadStrictReadback"], true);
                assert!(!out["export"]["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["code"] == "hatch-classic-settings-loss"));
                let bytes = STANDARD
                    .decode(out["export"]["download"]["base64"].as_str().unwrap())
                    .unwrap();
                let read = if format == "dwg" {
                    DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
                } else {
                    DxfReader::from_reader(Cursor::new(bytes))
                        .unwrap()
                        .read()
                        .unwrap()
                };
                assert_eq!(
                    read.dwg_source_version.unwrap_or(read.version).as_str(),
                    version
                );
                let EntityType::Hatch(h) = read
                    .entities()
                    .find(|e| matches!(e, EntityType::Hatch(_)))
                    .unwrap()
                else {
                    panic!()
                };
                assert_eq!(
                    h.pattern.lines[0].base_point.x.to_bits(),
                    89.80256121069161f64.to_bits()
                );
                assert_eq!(
                    h.pattern.lines[0].base_point.y.to_bits(),
                    2f64.powi(-40).to_bits()
                );
                assert!(h.stored_pattern_origin().is_none());
            }
        }
    }
}

#[test]
fn codec_hatch_angles_are_retained_in_all_offered_dxf_versions() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use ifccad_convert::{opencadcodec::*, *};
    use opencadcodec::entities::hatch::*;
    let mut cad = CadDocument::new();
    let mut h = Hatch::with_pattern(HatchPattern::new("Original"));
    h.pattern_angle = 0.7853981633974487;
    h.pattern.lines.push(HatchPatternLine {
        angle: 5.4977871437821335,
        base_point: Vector2::ZERO,
        offset: Vector2::new(2., 2.),
        dash_lengths: vec![],
    });
    let mut boundary = BoundaryPath::new();
    boundary
        .edges
        .push(BoundaryEdge::CircularArc(CircularArcEdge {
            center: Vector2::ZERO,
            radius: 10.,
            start_angle: 0.,
            end_angle: std::f64::consts::TAU,
            counter_clockwise: true,
        }));
    h.paths.push(boundary);
    cad.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
    let template = fixture::drawing();
    let native = cad_document_to_encoded_ifccad(
        &cad,
        IfccadTargetMetadata {
            header: template.header,
            drawing_id: template.drawing_id,
        },
        Default::default(),
    )
    .unwrap();
    for version in ["AC1015", "AC1018", "AC1021", "AC1024", "AC1027", "AC1032"] {
        let bytes = encode_ifccad_document(native.validated_source().document()).unwrap();
        let out = export_ifccad_bytes_with_options(
            "angle.ifcx",
            bytes.bytes(),
            "dxf",
            version,
            r#"{"lossPolicy":"reject"}"#,
        );
        assert!(out["failure"].is_null(), "{out}");
        let diagnostics = out["export"]["diagnostics"].as_array().unwrap();
        assert!(!diagnostics
            .iter()
            .any(|d| d["code"] == "hatch-classic-settings-loss"));
        assert!(
            !diagnostics
                .iter()
                .any(|d| d["code"] == "TARGET_CODEC_HATCH_CLASSIC_SETTINGS_LOSS"),
            "{version}: {diagnostics:?}"
        );
        let data = STANDARD
            .decode(out["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        let read = DxfReader::from_reader(std::io::Cursor::new(data))
            .unwrap()
            .read()
            .unwrap();
        let returned = read
            .entities()
            .find_map(|e| match e {
                EntityType::Hatch(h) => Some(h),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            returned.pattern_angle.to_bits(),
            0.7853981633974487_f64.to_bits(),
            "{version} code 52"
        );
        assert_eq!(
            returned.pattern.lines[0].angle.to_bits(),
            5.4977871437821335_f64.to_bits(),
            "{version} code 53"
        );
        assert_eq!(out["export"]["fileCheck"]["ifccadStrictReadback"], true);
    }
}
#[test]
fn pixel_without_derived_is_a_located_loss_and_reject_has_no_download() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-solid.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.layers[0].appearance.line_weight = 0.09;
    for e in &mut d.model.entities {
        if let Some(e) = e.as_native_mut() {
            if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                for l in &mut h.loops {
                    l.provenance = None;
                }
                h.boundary_construction = Some(HatchBoundaryConstruction {
                    seed_points: vec![[20., -2.], [20., -2.]],
                    pixel_size: 0.25,
                });
            }
        }
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    for format in ["dxf", "dwg"] {
        let out = export_ifccad_bytes_with_options("h.ifcx", bytes.bytes(), format, "AC1032", "{}");
        assert!(out["failure"].is_null(), "{}", out["failure"]);
        assert!(
            out["export"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "TARGET_CODEC_HATCH_BOUNDARY_CONSTRUCTION_LOSS"),
            "{}",
            out["export"]["diagnostics"]
        );
        let out = export_ifccad_bytes_with_options(
            "h.ifcx",
            bytes.bytes(),
            format,
            "AC1032",
            r#"{"lossPolicy":"reject"}"#,
        );
        assert_eq!(
            out["failure"]["code"], "CAD_EXCHANGE_LOSS_REJECTED",
            "{}",
            out["failure"]
        );
        assert!(out["export"]["download"].is_null());
    }
}
