use ifccad_viewer::{inspect_cad, inspect_package, progress};
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
        _ => {
            eprintln!("Usage: ifccad-viewer package DIRECTORY | cad INPUT OUTPUT_DIRECTORY");
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
