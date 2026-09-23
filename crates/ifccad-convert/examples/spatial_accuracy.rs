//! Reproducible production-path timings; run with --release -- OUTPUT_DIRECTORY.
use ifccad::ifcdr::{IfcdrEntityRef, IfcdrLengthUnit, PlanePlacement, Point2, Point3, Vector3};
use ifccad::package::*;
use ifccad::{PackageId, ResourceId};
use ifccad_convert::cadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use ifccad_convert::{cad_document_to_package, drawing_to_cad_document, ExportOptions};
use num_rational::BigRational;
use num_traits::ToPrimitive;
use serde_json::{json, Value};
use std::{fs, hint::black_box, path::Path, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("spatial-performance").unwrap(),
        data_version: "1".into(),
        author: "Spatial performance measurement".into(),
        timestamp: "2026-09-15T00:00:00Z".into(),
    }
}
fn create(name: &str) -> Result<EncodedPackage> {
    let half = 0.5_f64.sqrt();
    let plane = match name {
        "identity" => PlanePlacement::default(),
        "large" => PlanePlacement::try_new(
            Point3::new(1e12, -1e12, 1e12),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        )?,
        _ => PlanePlacement::try_new(
            Point3::new(0.1, 0.2, 0.3),
            Vector3::new(half, half, 0.),
            Vector3::new(0., 0., 1.),
        )?,
    };
    let mut package = PackageBuilder::new(options())?;
    let mut drawing = package.add_drawing(DrawingOptions {
        model_layout_name: "Model".into(),
        representation_resource_id: ResourceId::new("drawing")?,
        length_unit: IfcdrLengthUnit::Metre,
    })?;
    let appearance = drawing.appearances().add(AppearanceDefinition {
        name: "Default".into(),
        color: AppearanceColor::rgb(0, 0, 0),
        opacity: 1.,
        line_pattern: LinePatternDefinition::named("Continuous"),
        line_weight: 0.25,
    })?;
    let layer = drawing.layers().add(LayerDefinition {
        name: "0".into(),
        visible: true,
        frozen: false,
        locked: false,
        plottable: true,
        frozen_in_new_viewports: false,
        description: None,
        appearance,
    })?;
    for i in 0..10000 {
        let x = if name == "tight" { 0. } else { i as f64 };
        drawing.model_space().add_polyline(PolylineDefinition {
            points: vec![Point2::new(x, 0.), Point2::new(x + 1., 1.)],
            placement: plane,
            closed: false,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })?;
    }
    Ok(package.finish()?)
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
    use sha2::{Digest, Sha256};
    let path = root.join("resources/drawing.ifcdr.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let frame = &v["streams"]["polylineStream"]["placement"][0];
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
    v["scopeTable"][0]["bounds"] = bound;
    let bytes = serde_json::to_vec_pretty(&v)?;
    fs::write(path, &bytes)?;
    let path = root.join("package.ifcx.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&path)?)?;
    for node in v["data"].as_array_mut().unwrap() {
        if let Some(r) = node
            .get_mut("attributes")
            .and_then(|a| a.get_mut("resource"))
        {
            r["checksum"] = json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
        }
    }
    fs::write(path, serde_json::to_vec_pretty(&v)?)?;
    Ok(())
}
fn main() -> Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or("provide a fresh output directory")?;
    let root = Path::new(&arg);
    fs::create_dir(root)?;
    let mut report = json!({"entities_per_case":10000,"vertices_per_case":20000,"warmup_runs":2,"measured_runs":5,"profile":"release recommended","exact_fallback_counts":null,"counter_note":"Run the opt-in spatial_production_path_counters core test on these packages for validation and point-evaluation exact fallback counts; counters are excluded from timed production builds.","cases":{}});
    for name in ["identity", "oblique", "tight", "large"] {
        let path = root.join(name);
        create(name)?.write_directory(&path)?;
        if name == "tight" {
            tighten(&path)?;
        }
        let mut result = json!({});
        result["load_and_validate"] = measure(|| {
            let loaded = load_directory_package(&path)?;
            if loaded.validated_package().is_none() {
                return Err(format!("{:?}", loaded.report()).into());
            }
            black_box(loaded);
            Ok(())
        })?;
        let loaded = load_directory_package(&path)?;
        let drawing = loaded
            .validated_package()
            .ok_or("invalid measured drawing")?
            .drawings()
            .next()
            .unwrap();
        let layout = drawing.layouts().next().unwrap();
        result["scope_points"] = measure(|| {
            let mut count = 0;
            for e in layout
                .representation()
                .resource()
                .entities(layout.scope().id())
            {
                if let IfcdrEntityRef::Polyline(p) = e {
                    for point in p.scope_points() {
                        black_box(point?);
                        count += 1;
                    }
                }
            }
            assert_eq!(count, 20000);
            Ok(())
        })?;
        result["ifccad_to_cad"] = measure(|| {
            black_box(drawing_to_cad_document(drawing)?);
            Ok(())
        })?;
        let imported = drawing_to_cad_document(drawing)?;
        let assessment = imported.geometry_assessment();
        result["geometry"] = json!({"status":format!("{:?}",assessment.status()),"max_deviation_upper_bound":assessment.max_deviation_upper_bound(),"resolved_tolerance_upper":assessment.resolved_tolerance().upper(),"assessed_vertices":assessment.assessed_vertices(),"rounded_entities":assessment.rounded_entities()});
        result["cad_to_ifccad"] = measure(|| {
            black_box(cad_document_to_package(
                imported.document(),
                options(),
                ExportOptions::default(),
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
