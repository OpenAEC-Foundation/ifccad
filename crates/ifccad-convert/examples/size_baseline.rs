//! Development-only size and exchange baseline; see the approved design.
#[path = "size_baseline/accounting.rs"]
mod accounting;
#[path = "size_baseline/adapters.rs"]
mod adapters;
#[path = "size_baseline/projection.rs"]
mod projection;
#[path = "size_baseline/recipe.rs"]
mod recipe;
#[path = "size_baseline/report.rs"]
mod report;
#[path = "size_baseline/run.rs"]
mod run;
use std::{fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut args = std::env::args().skip(1);
    let mut name = None;
    let mut selected = None;
    let mut cargo_config = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--run" => name = Some(args.next().ok_or("--run requires a new directory name")?),
            "--case" => selected = Some(args.next().ok_or("--case requires a corpus case ID")?),
            "--cargo-config" => {
                cargo_config = Some(args.next().ok_or(
                    "--cargo-config requires the same config file used to build this example",
                )?)
            }
            _ => return Err("usage: size_baseline --run NEW-NAME [--case CASE-ID]".into()),
        }
    }
    let name = name.ok_or("usage: size_baseline --run NEW-NAME [--case CASE-ID]")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(
            "run name must contain only ASCII letters, digits, hyphens or underscores".into(),
        );
    }
    let corpus_bytes = fs::read(repo.join("benchmarks/size/corpus-v1.json"))?;
    let corpus: serde_json::Value = serde_json::from_slice(&corpus_bytes)?;
    let parent = repo.join("target/size-baseline");
    fs::create_dir_all(&parent)?;
    let output = parent.join(&name);
    run::fresh_directory(&output)?;
    let provenance = report::provenance(&repo, &output, cargo_config.as_deref())?;
    run::write_json(&output.join("provenance.json"), &provenance)?;
    fs::write(output.join("corpus-v1.json"), &corpus_bytes)?;
    let first = run::execute(&repo, &output.join("first"), &corpus, selected.as_deref())?;
    let second = run::execute(&repo, &output.join("second"), &corpus, selected.as_deref())?;
    let success = report::finish(
        &output,
        provenance,
        accounting::digest(&corpus_bytes),
        selected.as_deref(),
        first,
        second,
    )?;
    println!("Report: target/size-baseline/{name}/report.md");
    if !success {
        return Err("incomplete baseline; inspect the retained report".into());
    }
    Ok(())
}
