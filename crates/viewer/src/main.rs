use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use std::path::Path;
use viewer::{export_drawing_bytes, inspect_drawing, progress};
fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    let drawing_format = if let Some(index) = args.iter().position(|a| a == "--drawing-format") {
        if args.iter().filter(|a| *a == "--drawing-format").count() != 1 {
            eprintln!("Choose one drawing format");
            std::process::exit(2);
        }
        let value = args
            .get(index + 1)
            .filter(|v| matches!(v.as_str(), "ifccad" | "ocdraw"))
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("--drawing-format requires ifccad or ocdraw");
                std::process::exit(2)
            });
        args.drain(index..index + 2);
        value
    } else {
        "ocdraw".into()
    };
    let preserve_splines = args.iter().any(|arg| arg == "--preserve-splines");
    args.retain(|arg| arg != "--preserve-splines");
    let options = json!({"preserveSplines":preserve_splines}).to_string();
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp");
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
                Ok(bytes) if drawing_format == "ifccad" => {
                    viewer::inspect_cad_as_ifccad_bytes_with_options(
                        &args[2], &format, &bytes, &timestamp, &options,
                    )
                }
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
                Ok(bytes) if drawing_format == "ifccad" => {
                    let opening = viewer::inspect_cad_as_ifccad_bytes_with_options(
                        &args[2], &format, &bytes, &timestamp, &options,
                    );
                    if !opening["failure"].is_null() {
                        opening
                    } else {
                        let native = STANDARD
                            .decode(
                                opening["export"]["download"]["base64"]
                                    .as_str()
                                    .expect("strict native bytes"),
                            )
                            .expect("native download Base64");
                        let mut exported = viewer::export_ifccad_bytes_with_options(
                            &args[2],
                            &native,
                            &args[3],
                            args.get(4).map(String::as_str).unwrap_or("AC1032"),
                            &options,
                        );
                        let restoration = exported["conversion"].clone();
                        exported["conversion"] = opening["conversion"].clone();
                        exported["conversion"]["restoration"] = restoration;
                        exported["source"] = opening["source"].clone();
                        exported["reader"] = opening["reader"].clone();
                        exported
                    }
                }
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
                "Usage: viewer drawing FILE | cad FILE [--drawing-format ifccad|ocdraw] [--preserve-splines] | export-cad FILE FORMAT [VERSION] [--drawing-format ifccad|ocdraw] [--preserve-splines] | export-drawing FILE FORMAT [VERSION]"
            );
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
