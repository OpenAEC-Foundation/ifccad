//! Matching strict reader for the unpublished attribute review contract.
use ifccad::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1) {
        let b = std::fs::read(&path)?;
        let d = if path.ends_with(".cbor") {
            load_ifccad_cbor_bytes(&b, Default::default())?.into_document()
        } else {
            load_ifccad_bytes(&b, Default::default())?.into_document()
        };
        let mut defs = 0;
        let mut values = 0;
        for e in d
            .model
            .entities
            .iter()
            .chain(d.paper_layouts.iter().flat_map(|l| &l.entities))
            .chain(d.blocks.iter().flat_map(|b| &b.entities))
            .filter_map(|e| e.as_native())
        {
            match &e.kind {
                IfccadEntityKind::AttributeDefinition(_) => defs += 1,
                IfccadEntityKind::BlockInstance { attributes, .. } => values += attributes.len(),
                _ => {}
            }
        }
        println!("{path}: strict OK; {defs} definitions, {values} values");
    }
    Ok(())
}
