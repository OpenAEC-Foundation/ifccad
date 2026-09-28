use ifccad_viewer::{
    export_cad_versioned, export_package_versioned, inspect_cad, inspect_package, progress,
};
use serde_json::json;
use std::path::Path;
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    progress("reading");
    let result = match args.get(1).and_then(|a| a.to_str()) {
        Some("package") if args.len() == 3 => {
            progress("validating");
            inspect_package(Path::new(&args[2]))
        }
        Some("cad") if args.len() == 4 => inspect_cad(Path::new(&args[2]), Path::new(&args[3])),
        Some("export-package") if matches!(args.len(), 5 | 6) => export_package_versioned(
            Path::new(&args[2]),
            &args[4].to_string_lossy(),
            &args[3].to_string_lossy(),
            args.get(5).and_then(|s| s.to_str()).unwrap_or("AC1032"),
        ),
        Some("export-cad") if matches!(args.len(), 6 | 7) => export_cad_versioned(
            Path::new(&args[2]),
            Path::new(&args[3]),
            &args[5].to_string_lossy(),
            &args[4].to_string_lossy(),
            args.get(6).and_then(|s| s.to_str()).unwrap_or("AC1032"),
        ),
        _ => {
            eprintln!("Usage: ifccad-viewer package DIRECTORY | cad INPUT OUTPUT_DIRECTORY");
            eprintln!("       ifccad-viewer export-package DIRECTORY FORMAT DRAWING | export-cad INPUT OUTPUT_DIRECTORY FORMAT DRAWING");
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
