use cad_geometry_convert::hatch::*;
use ocdraw::geometry_kernel::hatch::HatchBoundary2;
use opencadcodec::{entities::hatch::*, Vector2};
use serde_json::Value;
fn point(v: &Value) -> Vector2 {
    Vector2::new(v["x"].as_f64().unwrap(), v["y"].as_f64().unwrap())
}
fn sources() -> Vec<Hatch> {
    serde_json::from_str::<Value>(include_str!("fixtures/hatch-9aa.json"))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let mut h = Hatch::solid();
            h.paths = v["paths"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    let mut path = BoundaryPath::new();
                    for e in p["edges"].as_array().unwrap() {
                        if let Some(e) = e.get("Line") {
                            path.add_edge(BoundaryEdge::Line(LineEdge {
                                start: point(&e["start"]),
                                end: point(&e["end"]),
                            }));
                        } else {
                            let e = &e["CircularArc"];
                            path.add_edge(BoundaryEdge::CircularArc(CircularArcEdge {
                                center: point(&e["center"]),
                                radius: e["radius"].as_f64().unwrap(),
                                start_angle: e["start_angle"].as_f64().unwrap(),
                                end_angle: e["end_angle"].as_f64().unwrap(),
                                counter_clockwise: e["counter_clockwise"].as_bool().unwrap(),
                            }));
                        }
                    }
                    path
                })
                .collect();
            h
        })
        .collect()
}
#[test]
fn original_dxf_9aa_and_dwg_counterpart_close_at_the_existing_default_limit() {
    for h in sources() {
        let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
        assert_eq!(p.boundaries.len(), 2);
        let HatchBoundary2::Edges(edges) = &p.boundaries[0] else {
            panic!()
        };
        assert_eq!(edges.len(), 5);
        let q = prepare_hatch_to_cad(
            p.placement,
            &p.boundaries,
            p.area_rule,
            p.join_tolerance,
            &ocdraw::geometry_kernel::hatch::HatchFill::Solid,
        )
        .unwrap();
        assert_eq!(
            prepare_hatch_from_cad(&q.hatch, 1e-9).unwrap().boundaries,
            p.boundaries
        );
        for pair in p.pairs {
            for curve in pair.curves {
                assert!(
                    curve.squared_deviation().unwrap().1
                        < cad_geometry_convert::geometry::numeric::exact(1e-18)
                );
            }
        }
    }
}

#[test]
fn emitted_dxf_and_dwg_keep_the_original_clockwise_wire_orientation() {
    use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
    use std::io::Cursor;
    for source in sources() {
        let p = prepare_hatch_from_cad(&source, 1e-9).unwrap();
        let q = prepare_hatch_to_cad(
            p.placement,
            &p.boundaries,
            p.area_rule,
            p.join_tolerance,
            &p.fill,
        )
        .unwrap();
        let mut document = CadDocument::new();
        document.add_entity(EntityType::Hatch(q.hatch)).unwrap();
        for dwg in [false, true] {
            let returned = if dwg {
                DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&document).unwrap()))
                    .read()
                    .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(
                    DxfWriter::new(&document).write_to_vec().unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap()
            };
            let h = returned
                .entities()
                .find_map(|e| {
                    if let EntityType::Hatch(h) = e {
                        Some(h)
                    } else {
                        None
                    }
                })
                .unwrap();
            for (actual, expected) in h
                .paths
                .iter()
                .flat_map(|p| &p.edges)
                .zip(source.paths.iter().flat_map(|p| &p.edges))
            {
                if let (BoundaryEdge::CircularArc(a), BoundaryEdge::CircularArc(b)) =
                    (actual, expected)
                {
                    assert_eq!(a.counter_clockwise, b.counter_clockwise);
                    assert!((a.start_angle - b.start_angle).abs() < 2e-14);
                    assert!((a.end_angle - b.end_angle).abs() < 2e-14);
                }
            }
        }
    }
}
