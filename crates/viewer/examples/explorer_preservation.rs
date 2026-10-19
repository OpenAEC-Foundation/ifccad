//! Produce a strict-readable source-preservation example from the pinned DXF.
use base64::{engine::general_purpose::STANDARD, Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide a new example output path")?;
    if std::path::Path::new(&path).exists() {
        return Err("output already exists".into());
    }
    let result = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "source-spline.dxf",
        "dxf",
        include_bytes!("../tests/fixtures/open-cubic.dxf"),
        "2026-10-19T00:00:00Z",
        r#"{"preserveSplines":true}"#,
    );
    if !result["failure"].is_null() {
        return Err(result["failure"].to_string().into());
    }
    let bytes = STANDARD.decode(
        result["export"]["download"]["base64"]
            .as_str()
            .ok_or("no native download")?,
    )?;
    ifccad::load_ifccad_bytes(&bytes, Default::default())?;
    std::fs::write(path, bytes)?;
    Ok(())
}
