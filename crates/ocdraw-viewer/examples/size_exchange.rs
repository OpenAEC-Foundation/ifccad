#[path = "size_exchange/corpus.rs"]
mod corpus;
#[path = "size_exchange/pipeline.rs"]
mod pipeline;
#[path = "size_exchange/projection.rs"]
mod projection;
#[path = "size_exchange/report.rs"]
mod report;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = args.first().ok_or("expected generate or verify")?;
    let mut options = std::collections::BTreeMap::new();
    if !(args.len() - 1).is_multiple_of(2) {
        return Err("options require values".into());
    }
    for pair in args[1..].as_chunks::<2>().0 {
        if options.insert(pair[0].as_str(), pair[1].as_str()).is_some() {
            return Err("duplicate option".into());
        }
    }
    let value = match mode.as_str() {
        "verify" if options.len() == 1 => pipeline::verify(std::path::Path::new(
            options.get("--directory").ok_or("missing directory")?,
        ))?,
        "generate"
            if options
                .keys()
                .all(|k| matches!(*k, "--corpus" | "--output" | "--practice")) =>
        {
            report::generate(
                std::path::Path::new(options.get("--corpus").ok_or("missing corpus")?),
                std::path::Path::new(options.get("--output").ok_or("missing output")?),
                options.get("--practice").map(std::path::Path::new),
            )?
        }
        _ => return Err("invalid mode/options".into()),
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
