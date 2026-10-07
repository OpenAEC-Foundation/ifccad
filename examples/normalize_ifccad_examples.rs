//! Normalize authored IFCCAD examples without flattening their source graph.
#[path = "support/ifccad_consistency.rs"]
mod consistency;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args_os()
        .nth(1)
        .ok_or("usage: normalize_ifccad_examples <ifccad-examples-directory>")?;
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().is_none_or(|e| e != "ifcx") {
            continue;
        }
        let value = serde_json::from_slice(&std::fs::read(&path)?)?;
        std::fs::write(&path, consistency::normalize(&value)?)?;
        println!("Normalized {}", path.display());
    }
    Ok(())
}
