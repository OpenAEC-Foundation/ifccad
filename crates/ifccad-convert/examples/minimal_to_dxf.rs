use ifccad::conformance::bundled_conformance_root;
use ifccad::package::load_directory_package;
use ifccad_convert::cadcodec::DxfWriter;
use ifccad_convert::convert_drawing;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let inspected = load_directory_package(root)?;
    let package = inspected
        .validated_package()
        .ok_or_else(|| io::Error::other("bundled package is not strictly valid"))?;
    let drawing = package
        .drawings()
        .next()
        .ok_or_else(|| io::Error::other("bundled package contains no drawing"))?;
    let outcome = convert_drawing(drawing)?;

    for diagnostic in outcome.diagnostics() {
        eprintln!("conversion diagnostic: {diagnostic}");
    }

    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("target")
        });
    let output = target
        .join("manual")
        .join("ifccad-convert")
        .join("minimal-no-preservation.dxf");
    std::fs::create_dir_all(
        output
            .parent()
            .ok_or_else(|| io::Error::other("DXF output has no parent directory"))?,
    )?;
    DxfWriter::new(outcome.document()).write_to_file(&output)?;
    println!("{}", output.canonicalize()?.display());
    Ok(())
}
