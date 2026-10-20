use opencadcodec::{CadDocument, DxfWriter, EntityType, Line, LwPolyline, Vector2};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/diagnose-cli-tests");
        fs::create_dir_all(&root).unwrap();
        let p = root.join(format!(
            "ifccad-diagnose-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_diagnose_cad"))
}

#[test]
fn command_collects_all_errors_journals_progress_and_never_overwrites() {
    let f = Fixture::new();
    let input = f.0.join("sample.dxf");
    let output = f.0.join("report");
    let mut cad = CadDocument::new();
    for _ in 0..2 {
        // Finite scalar inputs but a major-arc enclosure outside finite range.
        let mut p = LwPolyline::from_points(vec![
            Vector2::new(f64::MAX / 2., 0.),
            Vector2::new(-f64::MAX / 2., 0.),
        ]);
        p.vertices[0].bulge = 2.;
        cad.add_entity(EntityType::LwPolyline(p)).unwrap();
    }
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let bytes = DxfWriter::new(&cad).write_to_vec().unwrap();
    fs::write(&input, &bytes).unwrap();
    let result = command().arg(&input).arg(&output).output().unwrap();
    assert_eq!(
        result.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["audit"]["conversionReady"], false);
    assert_eq!(report["audit"]["traversalComplete"], true);
    let errors = report["audit"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["status"] == "failed")
        .count();
    assert_eq!(errors, 2);
    assert_eq!(report["provenance"]["exclusionsApplied"], 0);
    assert_eq!(report["provenance"]["benchmark"], false);
    assert_eq!(
        report["provenance"]["inputSha256"].as_str().unwrap().len(),
        64
    );
    let journal = fs::read_to_string(output.join("progress.jsonl")).unwrap();
    let entries = journal
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(entries.last().unwrap()["type"], "run-finished");
    assert!(fs::read_to_string(output.join("summary.md"))
        .unwrap()
        .contains("Hard failures"));
    let first = fs::read(output.join("report.json")).unwrap();
    assert_eq!(
        command().arg(&input).arg(&output).status().unwrap().code(),
        Some(2)
    );
    assert_eq!(fs::read(output.join("report.json")).unwrap(), first);
    assert_eq!(fs::read(&input).unwrap(), bytes);
}

#[test]
fn reader_failure_has_explicit_failed_report_and_preserved_journal() {
    let f = Fixture::new();
    let input = f.0.join("broken.dxf");
    let output = f.0.join("report");
    fs::write(&input, b"this is not a DXF file").unwrap();
    assert_eq!(
        command().arg(&input).arg(&output).status().unwrap().code(),
        Some(2)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["status"], "source-read-failed");
    assert!(report.get("audit").is_none());
    assert!(output.join("progress.jsonl").exists());
}

#[test]
fn unknown_options_do_not_create_a_run_or_allow_exclusions() {
    let f = Fixture::new();
    let output = f.0.join("report");
    assert_eq!(
        command()
            .args(["--exclusions", "old.json"])
            .arg(&output)
            .status()
            .unwrap()
            .code(),
        Some(2)
    );
    assert!(!output.exists());
}

#[test]
fn command_reads_dwg_and_reports_logically_ready_content() {
    let f = Fixture::new();
    let input = f.0.join("sample.dwg");
    let output = f.0.join("report");
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    fs::write(&input, opencadcodec::DwgWriter::write_to_vec(&cad).unwrap()).unwrap();
    let result = command().arg(&input).arg(&output).output().unwrap();
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["audit"]["conversionReady"], true);
    assert_eq!(report["provenance"]["reader"], "dwg");
}
