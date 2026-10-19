//! Read the opt-in matrix experiment and optionally write ordinary native IFCCAD.
use ifccad::{encode_ifccad_document, load_ifccad_xformop_bytes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: read_ifccad_xformop INPUT [NEW_NATIVE_OUTPUT]".into());
    }
    let source = load_ifccad_xformop_bytes(&std::fs::read(&args[0])?, Default::default())?;
    let document = source.document();
    println!(
        "Validated drawing {}: {} model entities, {} block definitions",
        document.drawing_id,
        document.model.entities.len(),
        document.blocks.len()
    );
    if let Some(output) = args.get(1) {
        // Ordinary native encoding validates and reads its own generated output.
        encode_ifccad_document(document)?.write_file(output)?;
    }
    Ok(())
}
