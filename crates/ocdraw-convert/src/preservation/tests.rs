use super::*;
use opencadcodec::entities::Spline;
use opencadcodec::xdata::{ExtendedDataRecord, XDataValue};
use opencadcodec::{Color, Handle, LineWeight, Transparency, Vector3};

#[test]
fn snapshot_retains_every_parameter_and_skipped_common_carrier() {
    let mut source = Spline::new();
    source.degree = -4;
    source.flags.closed = true;
    source.flags.periodic = true;
    source.flags.rational = true;
    source.flags.planar = true;
    source.flags.linear = true;
    source.knots = vec![f64::from_bits(0x7ff8000000000042), -0., f64::INFINITY];
    source.control_points = vec![Vector3::new(1., 2., 3.)];
    source.weights = vec![4.];
    source.fit_points = vec![Vector3::new(5., 6., 7.)];
    source.normal = Vector3::new(8., 9., 10.);
    source.knot_tolerance = 11.;
    source.control_tolerance = 12.;
    source.fit_tolerance = 13.;
    source.begin_tangent = Vector3::new(14., 15., 16.);
    source.end_tangent = Vector3::new(17., 18., 19.);
    source.knot_parameterization = 15;
    source.cv_frame_visible = true;
    source.dwg_flags1 = -123;
    source.dxf_flags = -456;
    source.common.handle = Handle::new(0xab);
    source.common.owner_handle = Handle::new(0xcd);
    source.common.layer = "source layer".into();
    source.common.color = Color::None;
    source.common.line_weight = LineWeight::Default;
    source.common.transparency = Transparency::Explicit(217);
    source.common.linetype = "source pattern".into();
    source.common.linetype_handle = Some(Handle::new(0x101));
    source.common.linetype_scale = -0.;
    source.common.color_name = Some("book$name".into());
    source.common.invisible = true;
    source.common.graphic_data = Some(vec![0xff, 0, 7]);
    source.common.reactors = vec![Handle::new(0x102), Handle::new(0x103)];
    source.common.xdictionary_handle = Some(Handle::new(0x104));
    source.common.color_book_handle = Some(Handle::new(0x105));
    source.common.full_visual_style_handle = Some(Handle::new(0x106));
    source.common.face_visual_style_handle = Some(Handle::new(0x107));
    source.common.edge_visual_style_handle = Some(Handle::new(0x108));
    source.common.material_flags = 3;
    source.common.material_handle = Some(Handle::new(0x109));
    source.common.shadow_flags = 7;
    source.common.plotstyle_flags = 3;
    source.common.plotstyle_handle = Some(Handle::new(0x10a));
    source.common.entity_mode = Some(0);
    source.common.has_ds_data = true;
    let mut xdata = ExtendedDataRecord::new("APP");
    xdata.values = vec![
        XDataValue::String("text".into()),
        XDataValue::ControlString("{".into()),
        XDataValue::LayerName("layer".into()),
        XDataValue::BinaryData(vec![0xff, 0]),
        XDataValue::Handle(Handle::new(0xab)),
        XDataValue::Point3D(Vector3::new(1., 2., 3.)),
        XDataValue::Position3D(Vector3::new(4., 5., 6.)),
        XDataValue::Displacement3D(Vector3::new(7., 8., 9.)),
        XDataValue::Direction3D(Vector3::new(10., 11., 12.)),
        XDataValue::Real(13.),
        XDataValue::Distance(14.),
        XDataValue::ScaleFactor(15.),
        XDataValue::Integer16(-16),
        XDataValue::Integer32(-17),
    ];
    source.common.extended_data.add_record(xdata);
    source.common.extended_data.raw_dwg_eed = vec![(0x200, vec![0xff, 0, 3])];
    let bytes = capture_spline(&source, 3, CODEC_REVISION).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["sourceHandle"], "ab");
    assert_eq!(value["sourceOwnerHandle"], "cd");
    assert_eq!(value["sourceOrderIndex"], 3);
    assert_eq!(
        value["knots"],
        serde_json::json!(["7ff8000000000042", "8000000000000000", "7ff0000000000000"])
    );
    assert_eq!(value["common"]["graphicData"], "/wAH");
    assert_eq!(value["common"]["lineWeight"], "Default");
    assert_eq!(value["common"]["linetypeScale"], "8000000000000000");
    assert_eq!(
        value["common"]["transparency"],
        serde_json::json!({"Explicit":217})
    );
    let snapshot = decode_spline_snapshot(&bytes).unwrap();
    let restored = snapshot.to_source();
    assert_eq!(restored.degree, -4);
    assert_eq!(
        restored
            .knots
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        vec![0x7ff8000000000042, 0x8000000000000000, 0x7ff0000000000000]
    );
    assert_eq!(restored.flags, source.flags);
    assert_eq!(restored.control_points, source.control_points);
    assert_eq!(restored.weights, source.weights);
    assert_eq!(restored.fit_points, source.fit_points);
    assert_eq!(restored.normal, source.normal);
    assert_eq!(
        (
            restored.knot_tolerance,
            restored.control_tolerance,
            restored.fit_tolerance
        ),
        (11., 12., 13.)
    );
    assert_eq!(restored.begin_tangent, source.begin_tangent);
    assert_eq!(restored.end_tangent, source.end_tangent);
    assert_eq!(
        (
            restored.knot_parameterization,
            restored.cv_frame_visible,
            restored.dwg_flags1,
            restored.dxf_flags
        ),
        (15, true, -123, -456)
    );
    assert_eq!(restored.common, source.common);
}

#[test]
fn closed_payload_rejects_shapes_but_not_unusual_geometry() {
    let mut s = Spline::new();
    s.degree = i32::MIN;
    s.knots = vec![f64::NEG_INFINITY];
    let bytes = capture_spline(&s, 0, CODEC_REVISION).unwrap();
    decode_spline_snapshot(&bytes).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (field, invalid) in [
        ("knots", serde_json::json!(["7FF0000000000000"])),
        ("degree", serde_json::json!(2147483648_i64)),
        (
            "normal",
            serde_json::json!(["0000000000000000", "0000000000000000"]),
        ),
        ("codecRevision", serde_json::json!("unaudited")),
    ] {
        let mut changed = value.clone();
        changed[field] = invalid;
        assert!(
            decode_spline_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut changed = value;
    changed["invented"] = true.into();
    assert!(decode_spline_snapshot(&serde_json::to_vec(&changed).unwrap()).is_err());
}

#[test]
fn missing_optional_source_fields_are_not_inferred_from_defaults() {
    let bytes = capture_spline(&Spline::new(), 0, CODEC_REVISION).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for field in [
        "linetypeHandle",
        "colorName",
        "graphicData",
        "xdictionaryHandle",
        "entityMode",
    ] {
        let mut value = original.clone();
        value["common"].as_object_mut().unwrap().remove(field);
        assert!(
            decode_spline_snapshot(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field}"
        );
    }
}
