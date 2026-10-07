#[test]
fn representative_native_examples_strict_read_and_cover_entity_families() {
    use ocdraw::ocdraw::{load_ocdraw_bytes, DrawingGeometry};
    for bytes in [
        include_bytes!("../examples/ocdraw/overview.ocdraw.json").as_slice(),
        include_bytes!("../examples/ocdraw/layouts-viewports.ocdraw.json").as_slice(),
        include_bytes!("../examples/ocdraw/state-and-storage.ocdraw.json").as_slice(),
    ] {
        load_ocdraw_bytes(bytes).unwrap();
    }
    let drawing =
        load_ocdraw_bytes(include_bytes!("../examples/ocdraw/overview.ocdraw.json")).unwrap();
    let kinds: std::collections::BTreeSet<_> = drawing
        .geometric_entities()
        .iter()
        .map(|e| match e.geometry {
            DrawingGeometry::Line { .. } => "line",
            DrawingGeometry::Point { .. } => "point",
            DrawingGeometry::Circle { .. } => "circle",
            DrawingGeometry::Arc { .. } => "arc",
            DrawingGeometry::Ellipse { .. } => "ellipse",
            DrawingGeometry::PlanarPolyline { .. } => "planar",
            DrawingGeometry::SpatialPolyline { .. } => "spatial",
            DrawingGeometry::BlockInstance { .. } => "block",
        })
        .collect();
    assert_eq!(kinds.len(), 8);
    for bytes in [
        include_bytes!("../examples/ifccad/overview.ifcx").as_slice(),
        include_bytes!("../examples/ifccad/layouts-viewports.ifcx").as_slice(),
        include_bytes!("../examples/ifccad/blocks-and-fragments.ifcx").as_slice(),
    ] {
        ocdraw::ifccad::load_ifccad_bytes(bytes, Default::default()).unwrap();
    }
}
