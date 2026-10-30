#[path = "support/extrusion_gallery.rs"]
mod gallery;
use ifccad::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--explorer")) || args[1] != "--output" {
        return Err("usage: extrusion_gallery --output FRESH_DIRECTORY [--explorer]".into());
    }
    let dir = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(dir.parent().unwrap())?;
    std::fs::create_dir(dir)?;
    let d = if args.len() == 4 {
        gallery::explorer_document()
    } else {
        gallery::document()
    };
    let j = encode_ifccad_document(&d)?;
    let b = encode_ifccad_cbor_document(&d)?;
    load_ifccad_bytes(j.bytes(), Default::default())?;
    load_ifccad_cbor_bytes(b.bytes(), Default::default())?;
    std::fs::write(dir.join("extrusion-gallery.ifcx"), j.bytes())?;
    std::fs::write(dir.join("extrusion-gallery.ifccad.cbor"), b.bytes())?;
    Ok(())
}
