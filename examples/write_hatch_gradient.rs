//! Write and strictly reopen independent active/inactive gradient examples.
use ocdraw::{geometry_kernel::hatch::*, ifccad::*, ocdraw::*};
#[path = "support/ifccad_consistency.rs"]
mod example_consistency;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let route = args.get(1).map(String::as_str).unwrap_or("ocdraw");
    let (path, bytes) = match route {
        "ocdraw" => {
            let mut d = load_ocdraw_bytes(include_bytes!(
                "../conformance/next/ocdraw/valid/hatch-gradient.ocdraw.json"
            ))?
            .into_document();
            d.drawing_id = "hatch-gradient-example".into();
            let mut cold = d.hatch_entities[0].clone();
            cold.id = d.next_entity_id;
            d.next_entity_id += 1;
            cold.fill = HatchFill::Solid;
            let g = cold.gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
            g.rotation = 0.5;
            g.shift = 0.;
            g.tint = 0.65;
            d.scopes
                .iter_mut()
                .find(|s| s.entities.contains(&d.hatch_entities[0].id))
                .unwrap()
                .entities
                .push(cold.id);
            d.hatch_entities.push(cold);
            let encoded = encode_ocdraw_document(&d)?;
            load_ocdraw_bytes(encoded.bytes())?;
            (
                "tests/fixtures/ocdraw/hello-hatch-gradient.ocdraw.json",
                encoded.bytes().to_vec(),
            )
        }
        "ifccad" => {
            let mut d = load_ifccad_bytes(
                include_bytes!("../conformance/next/ifccad/valid/hatch-gradient.ifcx"),
                Default::default(),
            )?
            .into_document();
            d.header.id = "ifccad-hatch-gradient-example".into();
            d.header.timestamp = "2026-10-13T00:00:00Z".into();
            d.layers[0].appearance.line_weight = 0.09;
            let mut cold = d
                .model
                .entities
                .iter()
                .find(|e| {
                    e.as_native()
                        .is_some_and(|e| matches!(e.kind, IfccadEntityKind::Hatch(_)))
                })
                .unwrap()
                .clone();
            let e = cold.as_native_mut().unwrap();
            e.id = d.id_counters.next_entity_id;
            d.id_counters.next_entity_id += 1;
            let IfccadEntityKind::Hatch(h) = &mut e.kind else {
                unreachable!()
            };
            h.fill = HatchFill::Solid;
            let g = h.gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
            g.rotation = 0.5;
            g.shift = 0.;
            g.tint = 0.65;
            d.model.entities.push(cold);
            let encoded = encode_ifccad_document(&d)?;
            let normalized =
                example_consistency::normalize(&serde_json::from_slice(encoded.bytes())?)?;
            load_ifccad_bytes(&normalized, Default::default())?;
            (
                "tests/fixtures/ifccad/hello-hatch-gradient.ifcx",
                normalized,
            )
        }
        _ => return Err("Usage: write_hatch_gradient [ocdraw|ifccad] [output path]".into()),
    };
    let path = args.get(2).map(String::as_str).unwrap_or(path);
    std::fs::write(path, bytes)?;
    Ok(())
}
