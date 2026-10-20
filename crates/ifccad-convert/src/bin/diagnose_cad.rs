//! Report-only DXF/DWG diagnosis. Intentionally independent of benchmark drivers.
use ifccad::IfccadHeader;
use ifccad_convert::{
    diagnose_cad_document_with_progress, CadToIfccadOptions, IfccadPreservationCapture,
    IfccadTargetMetadata,
};
use opencadcodec::{CadDocument, DwgReader, DxfReader};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufWriter, Cursor, Write},
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("diagnose_cad: {error}");
            std::process::exit(2);
        }
    }
}
fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = BufWriter::new(File::create_new(&temporary)?);
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    drop(file);
    fs::rename(temporary, path)?;
    Ok(())
}
fn journal(file: &mut BufWriter<File>, value: &impl serde::Serialize) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.flush()
}
fn read_cad(input: Vec<u8>, kind: &str) -> Result<CadDocument> {
    Ok(if kind == "dxf" {
        DxfReader::from_reader(Cursor::new(input))?.read()?
    } else {
        DwgReader::from_stream(Cursor::new(input)).read()?
    })
}
fn failed_run(
    output: &Path,
    journal_file: &mut BufWriter<File>,
    provenance: &Value,
    status: &str,
    error: &dyn std::fmt::Display,
) -> Result<i32> {
    let failure =
        json!({"status":status,"provenance":provenance,"error":error.to_string(),"complete":false});
    journal(journal_file, &failure)?;
    write_json(&output.join("report.json"), &failure)?;
    fs::write(output.join("summary.md"),format!("# CAD diagnosis failed\n\nStatus: {status}. No successful audit is available.\n\n{error}\n"))?;
    eprintln!("{status}: {error}");
    Ok(2)
}
fn run() -> Result<i32> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.len() != 2
        || arguments
            .iter()
            .any(|a| a.to_string_lossy().starts_with("--"))
    {
        return Err("usage: diagnose_cad SOURCE.dxf|SOURCE.dwg FRESH_REPORT_DIRECTORY".into());
    }
    let input = PathBuf::from(&arguments[0]);
    let output = PathBuf::from(&arguments[1]);
    let kind = input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(kind.as_str(), "dxf" | "dwg") {
        return Err("source must be DXF or DWG".into());
    }
    // A fresh directory prevents replacing historical or interrupted evidence.
    fs::create_dir(&output)?;
    let mut journal_file = BufWriter::new(File::create_new(output.join("progress.jsonl"))?);
    let mut provenance = json!({"inputName":input.file_name().map(|n|n.to_string_lossy()),"reader":kind,"exclusionsApplied":0,"benchmark":false,"lossPolicy":"Allow","preservation":"SupportedTyped","geometryTolerance":"Default: 1e-9 in each coordinate domain","hatchJoinTolerance":"Default production creation tolerance","command":std::env::args_os().map(|a|a.to_string_lossy().into_owned()).collect::<Vec<_>>()});
    journal(
        &mut journal_file,
        &json!({"type":"run-started","provenance":provenance}),
    )?;
    let bytes = match fs::read(&input) {
        Ok(bytes) => bytes,
        Err(error) => {
            return failed_run(
                &output,
                &mut journal_file,
                &provenance,
                "source-read-failed",
                &error,
            )
        }
    };
    provenance["inputSha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    provenance["inputLength"] = json!(bytes.len());
    write_json(&output.join("provenance.json"), &provenance)?;
    let source = match read_cad(bytes, &kind) {
        Ok(source) => source,
        Err(error) => {
            return failed_run(
                &output,
                &mut journal_file,
                &provenance,
                "source-read-failed",
                &error,
            )
        }
    };
    journal(
        &mut journal_file,
        &json!({"type":"source-read","storedEntities":source.entities().count()}),
    )?;
    let metadata = IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: "cad-diagnosis".into(),
            data_version: "0.1.0".into(),
            author: "IFCCAD diagnosis".into(),
            timestamp: "1970-01-01T00:00:00Z".into(),
        },
    };
    let options = CadToIfccadOptions {
        preservation: IfccadPreservationCapture::SupportedTyped,
        ..Default::default()
    };
    let mut completed = 0usize;
    let report = diagnose_cad_document_with_progress(&source, metadata, options, &mut |event| {
        journal(&mut journal_file, event)?;
        if event.phase == "entity-preparation" {
            completed += 1;
            if completed.is_multiple_of(1000) {
                eprintln!("Prepared {completed} source entities");
            }
        }
        Ok(())
    })?;
    let code = if report.conversion_ready { 0 } else { 1 };
    let status = if report.conversion_ready {
        "logical-conversion-ready"
    } else {
        "conversion-blocked"
    };
    let summary=format!("{}\n## Source identity\n\nSHA-256: `{}`.\n\nThe selected source/configuration/dependency/toolchain manifest must accompany this report when used as reproducible evidence.\n",report.markdown(),provenance["inputSha256"].as_str().unwrap());
    fs::write(output.join("summary.md"), summary)?;
    write_json(
        &output.join("report.json"),
        &json!({"status":status,"provenance":provenance,"audit":report}),
    )?;
    journal(
        &mut journal_file,
        &json!({"type":"run-finished","status":status,"exitCode":code}),
    )?;
    println!("Diagnosis written to {} ({status})", output.display());
    Ok(code)
}
