//! Generate the two curated Hatch drawings using independent native routes.
use std::{error::Error, path::Path};

fn ifccad(root: &Path) -> Result<(), Box<dyn Error>> {
    use ifccad::*;
    let mut solid = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-hatch-solid.ifcx"),
        Default::default(),
    )?
    .into_document();
    let pattern = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-hatch-pattern.ifcx"),
        Default::default(),
    )?
    .into_document();
    for mut entity in pattern.model.entities {
        let native = entity
            .as_native_mut()
            .ok_or("native Hatch example required")?;
        native.id += 2;
        match &mut native.kind {
            IfccadEntityKind::Hatch(hatch) => {
                hatch.placement.origin[0] += 8.;
                for contour in &mut hatch.loops {
                    for id in &mut contour.source_entity_ids {
                        *id += 2;
                    }
                }
            }
            IfccadEntityKind::Circle { placement, .. } => placement.origin[0] += 8.,
            _ => return Err("unexpected Hatch example geometry".into()),
        }
        solid.model.entities.push(entity);
    }
    solid.header.id = "explorer-ifccad-hatches".into();
    solid.id_counters.next_entity_id = 5;
    recompute_ifccad_document_bounds(&mut solid)?;
    let encoded = encode_ifccad_document(&solid)?;
    load_ifccad_bytes(encoded.bytes(), Default::default())?;
    encoded.write_file(root.join("ifccad/hatches.ifcx"))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("provide a new examples directory")?;
    let root = Path::new(&root);
    std::fs::create_dir_all(root.join("ifccad"))?;
    ifccad(root)
}
