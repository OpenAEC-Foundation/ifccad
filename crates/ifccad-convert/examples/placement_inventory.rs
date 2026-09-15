//! Inspect one CAD file: placement_inventory INPUT OUTPUT_JSON.
//! Counts stored LWPOLYLINE definitions, without expanding block instances.
use ifccad_convert::cadcodec::{DwgReader, DxfReader, EntityType};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    let path = Path::new(args.get(1).ok_or("provide INPUT OUTPUT_JSON")?);
    let output = args.get(2).ok_or("provide OUTPUT_JSON")?;
    let doc = if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("dwg"))
    {
        DwgReader::from_file(path)?.read()?
    } else {
        DxfReader::from_file(path)?.read()?
    };
    let mut kinds = BTreeMap::new();
    let mut frames = BTreeMap::<[u64; 4], usize>::new();
    let mut vertices = 0;
    let mut identity = 0;
    let mut non_z_normal = 0;
    let mut model_polylines = 0;
    for e in doc.model_space_entities() {
        if matches!(e, EntityType::LwPolyline(_)) {
            model_polylines += 1;
        }
    }
    for e in doc.entities() {
        *kinds.entry(e.as_entity().entity_type()).or_insert(0usize) += 1;
        if let EntityType::LwPolyline(p) = e {
            let values = [p.normal.x, p.normal.y, p.normal.z, p.elevation];
            // Exact equality with signed zeros treated as equal. No tolerance merging.
            let key = values.map(|n| if n == 0. { 0 } else { n.to_bits() });
            *frames.entry(key).or_default() += 1;
            vertices += p.vertices.len();
            identity += usize::from(values == [0., 0., 1., 0.]);
            non_z_normal += usize::from(values[..3] != [0., 0., 1.]);
        }
    }
    let groups: Vec<_> = frames
        .into_iter()
        .map(|(key, count)| {
            json!({
                "normal_elevation":key.map(f64::from_bits), "count":count
            })
        })
        .collect();
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "entity_kinds":kinds, "vertices":vertices, "identity":identity,
            "non_z_normal":non_z_normal, "model_polylines":model_polylines,
            "placement_groups":groups
        }))?,
    )?;
    Ok(())
}
