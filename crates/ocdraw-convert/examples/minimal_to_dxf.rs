//! Convert one validated standalone drawing to DXF.
use ocdraw::ocdraw::load_ocdraw_file;
use ocdraw_convert::opencadcodec::DxfWriter;
use ocdraw_convert::{ocdraw_source_to_cad_document, OcdrawToCadOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let input = args.next().ok_or("provide INPUT.ocdraw.json OUTPUT.dxf")?;
    let output = args.next().ok_or("provide OUTPUT.dxf")?;
    let loaded = load_ocdraw_file(input)?;
    let drawing = &loaded;
    let converted = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default())?;
    for d in converted.diagnostics() {
        eprintln!("{}: {}", d.code, d.message);
    }
    DxfWriter::new(converted.document()).write_to_file(output)?;
    Ok(())
}
