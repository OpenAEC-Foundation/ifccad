use super::{corpus, pipeline, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};

pub fn generate(corpus_path: &Path, output: &Path, practice: Option<&Path>) -> Result<Value> {
    let cases = corpus::cases(corpus_path)?;
    fs::create_dir(output)?;
    let mut results = vec![];
    for case in cases {
        results.push(run(&case.id, false, output, || corpus::generate(&case))?);
    }
    if let Some(path) = practice {
        let format = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or("practice needs a DXF or DWG extension")?;
        let id = match format.as_str() {
            "dwg" => "practice-foundation",
            "dxf" => "practice-dxf",
            _ => return Err("practice needs a DXF or DWG extension".into()),
        };
        results.push(run(id, true, output, || {
            pipeline::read_cad(&fs::read(path)?, &format)
        })?);
    }
    let manifest = json!({"version":1,"cases":results});
    fs::write(
        output.join("generation.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

fn run(
    id: &str,
    practice: bool,
    root: &Path,
    source: impl FnOnce() -> Result<ocdraw_convert::cadcodec::CadDocument>,
) -> Result<Value> {
    let directory = root.join(id);
    fs::create_dir(&directory)?;
    let result = (|| {
        let source = source()?;
        let p = pipeline::prepare(&source, practice)?;
        // Preserve initial loss evidence even if physical stabilization fails.
        fs::write(
            root.join(format!("{id}-preparation.json")),
            serde_json::to_vec_pretty(&p.evidence)?,
        )?;
        let p = if practice { pipeline::stabilize(p)? } else { p };
        pipeline::generate(&p, &directory)
    })();
    let entry = match result {
        Ok(value) => json!({"id":id,"practice":practice,"status":"passed","verification":value}),
        Err(error) => {
            json!({"id":id,"practice":practice,"status":"failed","error":error.to_string()})
        }
    };
    fs::write(
        root.join(format!("{id}-result.json")),
        serde_json::to_vec_pretty(&entry)?,
    )?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dxf_practice_uses_dxf_reader_and_four_strict_readbacks() {
        let root = std::env::temp_dir().join(format!(
            "exchange-dxf-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let corpus = root.join("corpus.json");
        fs::write(
            &corpus,
            br#"{"version":1,"unit":"mm","cases":[{"id":"empty","family":"line","count":0}]}"#,
        )
        .unwrap();
        let source = corpus::generate(&corpus::cases(&corpus).unwrap()[0]).unwrap();
        let input = root.join("drawing.DXF");
        fs::write(
            &input,
            ocdraw_convert::cadcodec::DxfWriter::new(&source)
                .write_to_vec()
                .unwrap(),
        )
        .unwrap();
        let result = generate(&corpus, &root.join("output"), Some(&input)).unwrap();
        assert_eq!(result["cases"][1]["id"], "practice-dxf");
        assert_eq!(result["cases"][1]["status"], "passed");
        assert_eq!(
            result["cases"][1]["verification"]["artifacts"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
