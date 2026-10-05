use opencadcodec::{entities::ViewportStatusFlags, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

#[test]
fn codec_preserves_independent_clipping_activation() {
    for bits in [0, 0x8000, 0x10000, 0x4000 | 0x8000 | 0x10000] {
        assert_eq!(ViewportStatusFlags::from_bits(bits).to_bits(), bits);
    }
}

#[test]
fn literal_dxf_boundary_and_degree_angles_are_decoded() {
    let document = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/viewports/reference-clip-activation.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let viewport = document
        .entities()
        .find_map(|e| match e {
            EntityType::Viewport(v) => Some(v),
            _ => None,
        })
        .unwrap();
    assert_ne!(viewport.status.to_bits() & 0x10000, 0);
    assert_eq!(viewport.clip_boundary_handle.value(), 0x1002);
    assert!(matches!(
        document.get_entity(viewport.clip_boundary_handle),
        Some(EntityType::Circle(_))
    ));
    assert!((viewport.snap_angle - std::f64::consts::PI / 6.).abs() < 1e-12);
    assert!((viewport.twist_angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
}

#[test]
fn dxf_writer_emits_degree_angles_and_clip_reference() {
    let mut source = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/viewports/reference-clip-activation.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let (handle, boundary) = source
        .entities()
        .find_map(|e| match e {
            EntityType::Viewport(v) => Some((v.common.handle, v.clip_boundary_handle)),
            _ => None,
        })
        .unwrap();
    let EntityType::Viewport(viewport) = source.get_entity_mut(handle).unwrap() else {
        panic!()
    };
    viewport.snap_angle = std::f64::consts::PI / 6.;
    viewport.twist_angle = std::f64::consts::FRAC_PI_2;
    let text = String::from_utf8(DxfWriter::new(&source).write_to_vec().unwrap()).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    let pairs = lines.as_chunks::<2>().0;
    let start = pairs
        .iter()
        .position(|p| p[0].trim() == "5" && p[1].trim() == format!("{:X}", handle.value()))
        .unwrap();
    let end = start
        + 1
        + pairs[start + 1..]
            .iter()
            .position(|p| p[0].trim() == "0")
            .unwrap();
    let value = |code: &str| {
        pairs[start..end]
            .iter()
            .find(|p| p[0].trim() == code)
            .unwrap()[1]
            .trim()
    };
    assert!((value("50").parse::<f64>().unwrap() - 30.).abs() < 1e-12);
    assert!((value("51").parse::<f64>().unwrap() - 90.).abs() < 1e-12);
    assert_eq!(value("340"), format!("{:X}", boundary.value()));
    assert_ne!(value("90").parse::<i32>().unwrap() & 0x10000, 0);
}
