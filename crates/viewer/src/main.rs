use serde_json::json;
use std::path::Path;
use viewer::{export_drawing_bytes, inspect_drawing, progress};
fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    let preserve_splines = args.iter().any(|arg| arg == "--preserve-splines");
    args.retain(|arg| arg != "--preserve-splines");
    progress("reading");
    let result = match args.get(1).map(String::as_str) {
        Some("drawing") if args.len() == 3 => inspect_drawing(Path::new(&args[2])),
        Some("cad") if args.len() == 3 => {
            let path = Path::new(&args[2]);
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match std::fs::read(path) {
                Ok(bytes) => viewer::inspect_cad_as_drawing_bytes_with_preservation(
                    &args[2],
                    &format,
                    &bytes,
                    preserve_splines,
                ),
                Err(e) => {
                    let mut r = viewer::result(path, &format);
                    viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
                    r
                }
            }
        }
        Some("export-cad") if matches!(args.len(), 4 | 5) => {
            let path = Path::new(&args[2]);
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match std::fs::read(path) {
                Ok(bytes) => viewer::export_cad_bytes_with_preservation(
                    &args[2],
                    &format,
                    &bytes,
                    &args[3],
                    args.get(4).map(String::as_str).unwrap_or("AC1032"),
                    preserve_splines,
                ),
                Err(e) => {
                    let mut r = viewer::result(path, &format);
                    viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
                    r
                }
            }
        }
        Some("export-drawing") if matches!(args.len(), 4 | 5) => {
            let path = Path::new(&args[2]);
            match std::fs::read(path) {
                Ok(bytes) => export_drawing_bytes(
                    &args[2],
                    &bytes,
                    &args[3],
                    args.get(4).map(String::as_str).unwrap_or("AC1032"),
                ),
                Err(e) => {
                    let mut r = viewer::result(path, "ocdraw");
                    viewer::fail(&mut r, "reading", "DRAWING_OPEN_FAILED", e);
                    r
                }
            }
        }
        _ => {
            eprintln!(
                "Usage: viewer drawing FILE | cad FILE [--preserve-splines] | export-cad FILE FORMAT [VERSION] [--preserve-splines] | export-drawing FILE FORMAT [VERSION]"
            );
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
