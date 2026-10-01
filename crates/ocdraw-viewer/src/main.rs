use ocdraw_viewer::{
    export_drawing_bytes, inspect_cad_as_drawing_bytes, inspect_drawing, progress,
};
use serde_json::json;
use std::path::Path;
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
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
                Ok(bytes) => inspect_cad_as_drawing_bytes(&args[2], &format, &bytes),
                Err(e) => {
                    let mut r = ocdraw_viewer::result(path, &format);
                    ocdraw_viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
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
                Ok(bytes) => ocdraw_viewer::export_cad_bytes(
                    &args[2],
                    &format,
                    &bytes,
                    &args[3],
                    args.get(4).map(String::as_str).unwrap_or("AC1032"),
                ),
                Err(e) => {
                    let mut r = ocdraw_viewer::result(path, &format);
                    ocdraw_viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
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
                    let mut r = ocdraw_viewer::result(path, "ocdraw");
                    ocdraw_viewer::fail(&mut r, "reading", "DRAWING_OPEN_FAILED", e);
                    r
                }
            }
        }
        _ => {
            eprintln!("Usage: ocdraw-viewer drawing FILE | cad FILE | export-drawing FILE FORMAT [VERSION]");
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
