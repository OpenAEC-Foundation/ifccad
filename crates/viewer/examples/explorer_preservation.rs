//! Produce a strict-readable source-preservation example from the pinned DXF.
use base64::{engine::general_purpose::STANDARD, Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide a new example output path")?;
    if std::path::Path::new(&path).exists() {
        return Err("output already exists".into());
    }
    let result = viewer::inspect_cad_as_drawing_bytes_with_preservation(
        "source-spline.dxf",
        "dxf",
        include_bytes!("../../ocdraw-convert/tests/fixtures/splines/open-cubic.dxf"),
        true,
    );
    if !result["failure"].is_null() {
        return Err(result["failure"].to_string().into());
    }
    let bytes = STANDARD.decode(
        result["export"]["download"]["base64"]
            .as_str()
            .ok_or("no native download")?,
    )?;
    ocdraw::ocdraw::load_ocdraw_bytes(&bytes)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
