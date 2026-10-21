use opencadcodec::entities::Ray;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Line, Vector3,
};
use serde_json::Value;
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/roundtrip-cli-tests");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, name: &str, bytes: &[u8]) -> (i32, Value) {
        fs::write(self.0.join(name), bytes).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_roundtrip_cad"))
            .arg(self.0.join(name))
            .arg(self.0.join("result"))
            .output()
            .unwrap();
        let report = fs::read(self.0.join("result/report.json"))
            .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&result.stderr)));
        (
            result.status.code().unwrap(),
            serde_json::from_slice(&report).unwrap(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn drawing() -> CadDocument {
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 2., 8., 4., 3.)))
        .unwrap();
    cad
}

#[test]
fn dxf_and_dwg_roundtrip_use_editable_native_binary_and_preserve_input() {
    for format in ["dxf", "dwg"] {
        let f = Fixture::new();
        let cad = drawing();
        let bytes = if format == "dwg" {
            DwgWriter::write_to_vec(&cad).unwrap()
        } else {
            DxfWriter::new(&cad).write_to_vec().unwrap()
        };
        let name = format!("source.{format}");
        let (code, report) = f.run(&name, &bytes);
        assert_eq!(code, 0, "{report:#}");
        assert_eq!(report["qualified"], true);
        assert_eq!(report["universalLosslessnessClaim"], false);
        assert_eq!(report["comparison"]["exactNativeProjectionEqual"], true);
        assert_eq!(fs::read(f.0.join(name)).unwrap(), bytes);
        let binary = fs::read(f.0.join("result/drawing.ifccad.cbor")).unwrap();
        let mut doc = ifccad::load_ifccad_cbor_bytes(&binary, Default::default())
            .unwrap()
            .into_document();
        assert!(doc.preservation.is_none());
        assert!(doc.model.entities.iter().all(|e| e.as_native().is_some()));
        let line = doc
            .model
            .entities
            .iter_mut()
            .find_map(|e| match &mut e.as_native_mut()?.kind {
                ifccad::IfccadEntityKind::LineSegment { end, .. } => Some(end),
                _ => None,
            })
            .unwrap();
        line[0] = 16.;
        ifccad::recompute_ifccad_document_bounds(&mut doc).unwrap();
        let edited = ifccad::encode_ifccad_cbor_document(&doc).unwrap();
        let loaded = ifccad::load_ifccad_cbor_bytes(edited.bytes(), Default::default()).unwrap();
        let export =
            ifccad_convert::ifccad_document_to_cad_document(loaded.document(), Default::default())
                .unwrap();
        let restored = if format == "dwg" {
            DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(export.document()).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(
                DxfWriter::new(export.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        assert!(restored.entities().any(
            |e| matches!(e,EntityType::Line(l) if l.end.x==16. && l.start.z==2. && l.end.z==3.)
        ));
    }
}

#[test]
fn unsupported_content_never_qualifies_even_when_the_native_projection_roundtrips() {
    let f = Fixture::new();
    let mut cad = drawing();
    cad.add_entity(EntityType::Ray(Ray::new(
        Vector3::ZERO,
        Vector3::new(1., 0., 0.),
    )))
    .unwrap();
    let (code, report) = f.run(
        "unsupported.dxf",
        &DxfWriter::new(&cad).write_to_vec().unwrap(),
    );
    assert_eq!(code, 1, "{report:#}");
    assert_eq!(report["qualified"], false);
    assert!(report["import"]["lossCount"].as_u64().unwrap() > 0);
    assert_eq!(report["opaquePreservation"], false);
}

#[test]
fn failed_reader_is_recorded_and_existing_runs_are_never_overwritten() {
    let f = Fixture::new();
    let (code, report) = f.run("broken.dwg", b"not a drawing");
    assert_eq!(code, 2);
    assert_eq!(report["phase"], "source-read");
    assert_eq!(report["qualified"], false);
    let before = fs::read(f.0.join("result/report.json")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_roundtrip_cad"))
        .arg(f.0.join("broken.dwg"))
        .arg(f.0.join("result"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(fs::read(f.0.join("result/report.json")).unwrap(), before);
}

#[test]
fn missing_input_and_unknown_options_cannot_produce_a_pass() {
    let f = Fixture::new();
    let result = Command::new(env!("CARGO_BIN_EXE_roundtrip_cad"))
        .arg(f.0.join("absent.dxf"))
        .arg(f.0.join("missing"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let report: Value =
        serde_json::from_slice(&fs::read(f.0.join("missing/report.json")).unwrap()).unwrap();
    assert_eq!(report["status"], "failed");
    let result = Command::new(env!("CARGO_BIN_EXE_roundtrip_cad"))
        .args(["--allow-loss", "ignored"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
}
