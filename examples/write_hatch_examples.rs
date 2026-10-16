//! Generate the two curated Hatch drawings using independent native routes.
use std::{error::Error, path::Path};

fn ifccad(root: &Path) -> Result<(), Box<dyn Error>> {
    use ocdraw::ifccad::*;
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

fn ocdraw(root: &Path) -> Result<(), Box<dyn Error>> {
    use ocdraw::ocdraw::*;
    let mut solid = load_ocdraw_bytes(include_bytes!(
        "../tests/fixtures/ocdraw/hello-hatch-solid.ocdraw.json"
    ))?
    .into_document();
    let pattern = load_ocdraw_bytes(include_bytes!(
        "../tests/fixtures/ocdraw/hello-hatch-pattern.ocdraw.json"
    ))?
    .into_document();
    let placement = CoordinateFrame3::try_new(
        Point3::new(8., 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )?;
    for mut hatch in pattern.hatch_entities {
        hatch.id += 2;
        hatch.placement = placement;
        for contour in &mut hatch.loops {
            for id in &mut contour.source_entity_ids {
                *id += 2;
            }
        }
        solid.hatch_entities.push(hatch);
    }
    for mut entity in pattern.geometric_entities {
        entity.id += 2;
        let DrawingGeometry::Circle {
            placement: frame, ..
        } = &mut entity.geometry
        else {
            return Err("circle source required".into());
        };
        *frame = placement;
        solid.geometric_entities.push(entity);
    }
    solid.scopes[0].entities.extend([4, 3]);
    solid.next_entity_id = 5;
    solid.drawing_id = "explorer-ocdraw-hatches".into();
    recompute_ocdraw_document_bounds(&mut solid)?;
    let encoded = encode_ocdraw_document(&solid)?;
    load_ocdraw_bytes(encoded.bytes())?;
    encoded.write_file(root.join("ocdraw/hatches.ocdraw.json"))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("provide a new examples directory")?;
    let root = Path::new(&root);
    std::fs::create_dir_all(root.join("ifccad"))?;
    std::fs::create_dir_all(root.join("ocdraw"))?;
    ifccad(root)?;
    ocdraw(root)
}
