use opencadcodec::entities::hatch::*;
use opencadcodec::{CadDocument, Circle, EntityType, Handle, Line, Point, Vector2, Vector3};

pub fn source(enabled: bool, unsupported: bool) -> (CadDocument, Handle, Vec<Handle>) {
    let mut d = CadDocument::new();
    let a = d
        .add_entity(EntityType::Line(Line::from_points(
            Vector3::ZERO,
            Vector3::new(10., 0., 0.),
        )))
        .unwrap();
    let b = if unsupported {
        d.add_entity(EntityType::Point(Point::new())).unwrap()
    } else {
        d.add_entity(EntityType::Line(Line::from_points(
            Vector3::ZERO,
            Vector3::new(0., 10., 0.),
        )))
        .unwrap()
    };
    let c = d.add_entity(EntityType::Circle(Circle::new())).unwrap();
    let mut h = Hatch::solid();
    h.is_associative = enabled;
    let mut p = BoundaryPath::new();
    p.flags = BoundaryPathFlags::from_bits(136);
    p.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    p.boundary_handles = if unsupported {
        vec![a, b, c]
    } else {
        vec![c, a, c, b]
    };
    h.paths.push(p.clone());
    p.boundary_handles = vec![c];
    h.paths.push(p);
    let hh = d.add_entity(EntityType::Hatch(h)).unwrap();
    (d, hh, vec![a, b, c])
}

pub fn exchange(d: &CadDocument, dwg: bool) -> CadDocument {
    use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
    use std::io::Cursor;
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
