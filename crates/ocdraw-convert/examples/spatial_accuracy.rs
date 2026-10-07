//! Reproducible production-path timings; run with --release -- OUTPUT_DIRECTORY.
use num_rational::BigRational;
use num_traits::ToPrimitive;
use ocdraw::ocdraw::*;
use ocdraw_convert::opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, ocdraw_source_to_cad_document, CadToOcdrawOptions,
    OcdrawToCadOptions,
};
use serde_json::{json, Value};
use std::{fs, hint::black_box, path::Path, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn create(name: &str) -> Result<EncodedOcdraw> {
    let half = 0.5_f64.sqrt();
    let plane = match name {
        "identity" => CoordinateFrame3::default(),
        "large" => CoordinateFrame3::try_new(
            Point3::new(1e12, -1e12, 1e12),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        )?,
        _ => CoordinateFrame3::try_new(
            Point3::new(0.1, 0.2, 0.3),
            Vector3::new(half, half, 0.),
            Vector3::new(0., 0., 1.),
        )?,
    };
    let mut drawing = OcdrawBuilder::new(OcdrawBuildOptions::new("spatial-performance", "m"))?;
    drawing.ensure_continuous_line_pattern().unwrap();
    let layer = drawing.add_layer(LayerDefinition::new(
        "0",
        DrawingColor::rgb(0, 0, 0),
        ocdraw::ocdraw::LinePatternId(0),
    ))?;
    for i in 0..10000 {
        let x = if name == "tight" { 0. } else { i as f64 };
        drawing.add_geometric_entity(GeometricEntityDefinition::new(
            0,
            layer,
            DrawingGeometry::PlanarPolyline {
                line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

                placement: plane,
                vertices: vec![[x, 0., 0.], [x + 1., 1., 0.]],
                closed: false,
            },
        ))?;
    }
    Ok(drawing.finish()?)
}
fn measure(mut operation: impl FnMut() -> Result<()>) -> Result<Value> {
    for _ in 0..2 {
        operation()?;
    }
    let mut times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        operation()?;
        times.push(start.elapsed().as_secs_f64() * 1000.);
    }
    times.sort_by(f64::total_cmp);
    Ok(json!({"median_ms":times[2],"min_ms":times[0],"max_ms":times[4]}))
}
// The tight case consists of identical segments. Construct directed bounds
// independently from the exact affine values, outside the timed validation.
fn tighten(root: &Path) -> Result<()> {
    let path = root.to_path_buf();
    let mut v: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let frame = &v["streams"]["planarPolylineStream"]["placement"][0];
    let exact = |v: f64| BigRational::from_float(v).unwrap();
    let mut bound = json!({});
    for axis in ["x", "y", "z"] {
        let o = exact(frame["origin"][axis].as_f64().unwrap());
        let end = &o
            + exact(frame["X"][axis].as_f64().unwrap())
            + exact(frame["Y"][axis].as_f64().unwrap());
        for (prefix, q, up) in [
            ("min", o.clone().min(end.clone()), false),
            ("max", o.max(end), true),
        ] {
            let mut n = q.to_f64().ok_or("tight bound not representable")?;
            if up {
                while exact(n) < q {
                    n = n.next_up();
                }
            } else {
                while exact(n) > q {
                    n = n.next_down();
                }
            }
            bound[format!("{prefix}{}", axis.to_uppercase())] = json!(n);
        }
    }
    v["scopes"][0]["bounds"] = bound;
    let bytes = serde_json::to_vec_pretty(&v)?;
    fs::write(path, &bytes)?;
    Ok(())
}
fn main() -> Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or("provide a fresh output directory")?;
    let root = Path::new(&arg);
    fs::create_dir(root)?;
    let mut report = json!({"entities_per_case":10000,"vertices_per_case":20000,"warmup_runs":2,"measured_runs":5,"profile":"release recommended","exact_fallback_counts":null,"counter_note":"This standalone production-path measurement does not instrument exact fallback counts.","cases":{}});
    for name in ["identity", "oblique", "tight", "large"] {
        let path = root.join(format!("{name}.ocdraw.json"));
        create(name)?.write_file(&path)?;
        if name == "tight" {
            tighten(&path)?;
        }
        let mut result = json!({});
        result["load_and_validate"] = measure(|| {
            let loaded = load_ocdraw_file(&path)?;
            black_box(loaded);
            Ok(())
        })?;
        let loaded = load_ocdraw_file(&path)?;
        let drawing = &loaded;
        result["scope_points"] = measure(|| {
            let mut count = 0;
            for e in drawing.geometric_entities() {
                if let DrawingGeometry::PlanarPolyline {
                    placement,
                    vertices,
                    ..
                } = e.geometry()
                {
                    for p in vertices {
                        black_box(placement.try_to_scope_point(Point2::new(p[0], p[1]))?);
                        count += 1;
                    }
                }
            }
            assert_eq!(count, 20000);
            Ok(())
        })?;
        result["ocdraw_to_cad"] = measure(|| {
            black_box(ocdraw_source_to_cad_document(
                drawing,
                OcdrawToCadOptions::default(),
            )?);
            Ok(())
        })?;
        let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default())?;
        let assessment = imported.geometry_assessment();
        result["geometry"] = json!({"status":format!("{:?}",assessment.status()),"max_deviation_upper_bound":assessment.domains().iter().find(|d|d.domain()==ocdraw_convert::OcdrawGeometryDomain::Drawing).unwrap().max_deviation_upper_bound(),"resolved_tolerance_upper":assessment.domains().iter().find(|d|d.domain()==ocdraw_convert::OcdrawGeometryDomain::Drawing).unwrap().resolved_tolerance().upper(),"assessed_vertices":assessment.assessed_vertices(),"rounded_entities":assessment.rounded_entities()});
        result["cad_to_ocdraw"] = measure(|| {
            black_box(cad_document_to_encoded_ocdraw(
                imported.document(),
                CadToOcdrawOptions::default(),
            )?);
            Ok(())
        })?;
        let dxf = root.join(format!("{name}.dxf"));
        let dwg = root.join(format!("{name}.dwg"));
        result["dxf_write"] = measure(|| {
            DxfWriter::new(imported.document()).write_to_file(&dxf)?;
            Ok(())
        })?;
        result["dxf_read"] = measure(|| {
            black_box(DxfReader::from_file(&dxf)?.read()?);
            Ok(())
        })?;
        result["dwg_write"] = measure(|| {
            fs::write(&dwg, DwgWriter::write_to_vec(imported.document())?)?;
            Ok(())
        })?;
        result["dwg_read"] = measure(|| {
            black_box(DwgReader::from_file(&dwg)?.read()?);
            Ok(())
        })?;
        report["cases"][name] = result;
        fs::write(
            root.join("results.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("{name}: {}", report["cases"][name]);
    }
    Ok(())
}
