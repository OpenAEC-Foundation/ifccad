#[path = "../../../examples/support/extrusion_gallery.rs"]
mod gallery;
use ifccad::*;
use opencaddocument::{Dwg, Dxf, DxfVersion, Format, Ifcx};
use serde_json::json;

fn signature(k: &IfccadEntityKind) -> Option<(&'static str, Vec<f64>)> {
    let at = |p: &IfccadPlacement, v: [f64; 2]| {
        std::array::from_fn::<_, 3, _>(|i| p.origin[i] + p.x_axis[i] * v[0] + p.y_axis[i] * v[1])
    };
    let normal = |p: &IfccadPlacement| {
        std::array::from_fn::<_, 3, _>(|i| {
            p.x_axis[(i + 1) % 3] * p.y_axis[(i + 2) % 3]
                - p.x_axis[(i + 2) % 3] * p.y_axis[(i + 1) % 3]
        })
    };
    let (name, h, n, mut values) = match k {
        IfccadEntityKind::Point {
            placement,
            thickness,
        } => (
            "point",
            *thickness,
            normal(placement),
            placement.origin.to_vec(),
        ),
        IfccadEntityKind::LineSegment {
            start,
            end,
            thickness,
            extrusion_direction,
        } => (
            "line",
            *thickness,
            *extrusion_direction,
            start.iter().chain(end).copied().collect(),
        ),
        IfccadEntityKind::Circle {
            radius,
            placement,
            thickness,
        } => {
            let mut v = placement.origin.to_vec();
            v.push(*radius);
            ("circle", *thickness, normal(placement), v)
        }
        IfccadEntityKind::Arc {
            radius,
            placement,
            start_parameter,
            sweep_parameter,
            thickness,
        } => {
            let mut v = placement.origin.to_vec();
            for a in [*start_parameter, *start_parameter + *sweep_parameter] {
                v.extend(at(placement, [radius * a.cos(), radius * a.sin()]));
            }
            v.push(*radius);
            ("arc", *thickness, normal(placement), v)
        }
        IfccadEntityKind::PlanarPolyline {
            vertices,
            bulges,
            closed,
            widths,
            placement,
            thickness,
            ..
        } => {
            let mut v = vec![f64::from(*closed)];
            for p in vertices {
                v.extend(at(placement, *p));
            }
            v.extend(bulges);
            v.extend(
                widths
                    .effective_widths(vertices.len())
                    .unwrap()
                    .into_iter()
                    .flatten(),
            );
            ("polyline", *thickness, normal(placement), v)
        }
        IfccadEntityKind::Solid(v) | IfccadEntityKind::Trace(v) => {
            let mut a = vec![];
            for p in v.corners {
                a.extend(at(&v.placement, p));
            }
            ("filled", v.thickness, normal(&v.placement), a)
        }
        IfccadEntityKind::Text(t) => (
            "text",
            t.thickness,
            normal(&t.placement),
            t.placement.origin.to_vec(),
        ),
        _ => return None,
    };
    values.push(h);
    values.extend(n.map(|v| v * h));
    Some((name, values))
}
fn compare(a: &IfccadDocument, b: &IfccadDocument) {
    let collect = |d: &IfccadDocument| {
        d.model
            .entities
            .iter()
            .filter_map(|e| signature(&e.as_native().unwrap().kind))
            .collect::<Vec<_>>()
    };
    let widths = |d: &IfccadDocument| {
        d.model
            .entities
            .iter()
            .filter_map(|e| match &e.as_native().unwrap().kind {
                IfccadEntityKind::PlanarPolyline {
                    vertices, widths, ..
                } => Some(widths.effective_widths(vertices.len()).unwrap()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(widths(a), widths(b), "effective widths changed");
    let aa = collect(a);
    let bb = collect(b);
    assert_eq!(aa.len(), bb.len());
    for ((ka, va), (kb, vb)) in aa.iter().zip(&bb) {
        assert_eq!(ka, kb);
        assert_eq!(va.len(), vb.len());
        assert_eq!(
            va[va.len() - 4],
            vb[vb.len() - 4],
            "signed thickness changed"
        );
        for (a, b) in va.iter().zip(vb) {
            assert!((a - b).abs() < 1e-9, "{ka}: {a} != {b}");
        }
    }
    let graphics = |d: &IfccadDocument| {
        d.model
            .entities
            .iter()
            .filter_map(|e| {
                if let IfccadEntityKind::Dimension(v) = &e.as_native().unwrap().kind {
                    Some(v)
                } else {
                    None
                }
            })
            .flat_map(|v| {
                v.graphics.iter().flat_map(move |g| {
                    g.parts.iter().map(move |part| {
                        let mut k: IfccadEntityKind = part.kind.clone().into();
                        if let IfccadEntityKind::LineSegment {
                            start,
                            end,
                            extrusion_direction,
                            ..
                        } = &mut k
                        {
                            let p = &v.placement;
                            let n = std::array::from_fn::<_, 3, _>(|i| {
                                p.x_axis[(i + 1) % 3] * p.y_axis[(i + 2) % 3]
                                    - p.x_axis[(i + 2) % 3] * p.y_axis[(i + 1) % 3]
                            });
                            let direction = |q: [f64; 3]| {
                                std::array::from_fn(|i| {
                                    p.x_axis[i] * q[0] + p.y_axis[i] * q[1] + n[i] * q[2]
                                })
                            };
                            for q in [start, end] {
                                let rotated = direction(*q);
                                *q = std::array::from_fn(|i| p.origin[i] + rotated[i]);
                            }
                            *extrusion_direction = direction(*extrusion_direction);
                        }
                        signature(&k)
                    })
                })
            })
            .flatten()
            .collect::<Vec<_>>()
    };
    let ga = graphics(a);
    let gb = graphics(b);
    assert_eq!(ga.len(), gb.len(), "saved graphics lost");
    for ((ka, va), (kb, vb)) in ga.iter().zip(&gb) {
        assert_eq!(ka, kb);
        for (a, b) in va.iter().zip(vb) {
            assert!((a - b).abs() < 1e-9, "picture {ka}: {a} != {b}")
        }
    }
}
fn strict(d: &IfccadDocument) -> IfccadDocument {
    let j = encode_ifccad_document(d).unwrap();
    let d = load_ifccad_bytes(j.bytes(), Default::default()).unwrap();
    let b = encode_ifccad_cbor_document(d.document()).unwrap();
    load_ifccad_cbor_bytes(b.bytes(), Default::default())
        .unwrap()
        .into_document()
}
#[test]
fn grouped_gallery_survives_physical_dwg_and_dxf() {
    let native = strict(&gallery::document());
    let mapped = Ifcx.import_ifccad(&native).unwrap();
    assert!(mapped
        .losses
        .iter()
        .any(|l| l.message.contains("inheritance materialized")));
    let mut cad = mapped.document;
    cad.version = DxfVersion::AC1032;
    let mut report = json!({"profile":"0.3.0","cborMapping":3,"cadVersion":"AC1032","nativeBase":"07e1ef1c4e975afcab0a07012bed8a82a56202cb","documentBase":"28f1ad1214816e6b5c21162b732df0604bdf38bd","codec":"0a5542baa6052a2b07fc700b7ba041549650a997","nativeToCadDiagnostics":mapped.losses.iter().map(|l|&l.message).collect::<Vec<_>>(),"roundtrips":[]});
    let output = std::env::var_os("IFCCAD_EXTRUSION_REVIEW_DIR").map(std::path::PathBuf::from);
    if let Some(p) = &output {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::create_dir(p).expect("review directory must be fresh");
        std::fs::write(
            p.join("extrusion-gallery.ifcx"),
            encode_ifccad_document(&native).unwrap().bytes(),
        )
        .unwrap();
        std::fs::write(
            p.join("extrusion-gallery.ifccad.cbor"),
            encode_ifccad_cbor_document(&native).unwrap().bytes(),
        )
        .unwrap();
    }
    for (ext, format) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let (bytes, write_losses) = format.write(&cad).unwrap();
        let (read, read_losses) = format.read(&bytes).unwrap();
        assert_eq!(read.version, DxfVersion::AC1032, "actual CAD version");
        assert!(write_losses.is_empty() && read_losses.is_empty());
        let out = Ifcx.export_ifccad(&read).unwrap();
        let roundtrip = strict(&out.drawing);
        compare(&native, &roundtrip);
        if let Some(p) = &output {
            std::fs::write(p.join(format!("extrusion-gallery.{ext}")), &bytes).unwrap();
            std::fs::write(
                p.join(format!("extrusion-gallery-from-{ext}.ifcx")),
                encode_ifccad_document(&roundtrip).unwrap().bytes(),
            )
            .unwrap();
        }
        report["roundtrips"].as_array_mut().unwrap().push(json!({"format":ext,"semanticReadback":"passed","strictNativeJsonCbor":"passed","diagnostics":out.losses.iter().map(|l|&l.message).collect::<Vec<_>>() }));
    }
    // Independent classic source, not emitted by the new native writer.
    let source = include_bytes!("../../../tests/fixtures/ifccad/extrusion-classic.dxf");
    let (classic, _) = Dxf.read(source).unwrap();
    let classic_native = Ifcx.export_ifccad(&classic).unwrap();
    let reference = strict(&classic_native.drawing);
    if let Some(p) = &output {
        std::fs::write(p.join("classic-source.dxf"), source).unwrap();
        std::fs::write(p.join("classic-source.dwg"), Dwg.write(&classic).unwrap().0).unwrap();
        std::fs::write(
            p.join("classic.ifcx"),
            encode_ifccad_document(&reference).unwrap().bytes(),
        )
        .unwrap();
    }
    for (ext, format) in [("dxf", &Dxf as &dyn Format), ("dwg", &Dwg as &dyn Format)] {
        let return_cad = Ifcx.import_ifccad(&reference).unwrap().document;
        let bytes = format.write(&return_cad).unwrap().0;
        let read = format.read(&bytes).unwrap().0;
        let out = Ifcx.export_ifccad(&read).unwrap();
        compare(&reference, &strict(&out.drawing));
        if let Some(p) = &output {
            std::fs::write(p.join(format!("classic-roundtrip.{ext}")), bytes).unwrap();
        }
    }
    report["classicDiagnostics"] = json!(classic_native
        .losses
        .iter()
        .map(|l| &l.message)
        .collect::<Vec<_>>());
    if let Some(p) = &output {
        std::fs::write(
            p.join("readback-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
