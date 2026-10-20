//! Reproduce the two curated Explorer drawings in a new output directory.
#[path = "support/explorer_catalogue.rs"]
mod catalogue;
#[path = "support/ifccad_consistency.rs"]
mod consistency;
#[path = "support/explorer_drawing.rs"]
mod drawing;
#[path = "support/explorer_house.rs"]
mod house;

fn main() -> drawing::Result<()> {
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: write_explorer_examples <new-output-directory>")?,
    );
    let root = root.join("ifccad");
    std::fs::create_dir_all(&root)?;
    let examples = [
        ("entities.ifcx", catalogue::build()?),
        ("house.ifcx", house::build()?),
    ];
    for (name, bytes) in examples {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(name))?;
        file.write_all(&bytes)?;
        println!("Wrote {name}");
    }
    Ok(())
}
