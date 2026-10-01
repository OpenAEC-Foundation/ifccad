//! Convert one validated standalone drawing to DXF.
use ocdraw::ocdraw::load_drawing_file;
use ocdraw_convert::cadcodec::DxfWriter;
use ocdraw_convert::{ocdraw_to_cad_document, ImportOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let input = args.next().ok_or("provide INPUT.ocdraw.json OUTPUT.dxf")?;
    let output = args.next().ok_or("provide OUTPUT.dxf")?;
    let loaded = load_drawing_file(input)?;
    let drawing = loaded
        .validated_drawing()
        .ok_or_else(|| format!("invalid drawing: {:?}", loaded.diagnostics()))?;
    let converted = ocdraw_to_cad_document(drawing, ImportOptions::default())?;
    for d in converted.diagnostics() {
        eprintln!("{}: {}", d.code, d.message);
    }
    DxfWriter::new(converted.document()).write_to_file(output)?;
    Ok(())
}
