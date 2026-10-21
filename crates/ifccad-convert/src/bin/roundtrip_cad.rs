//! Conservative qualification of the native binary projection through real CAD files.
use ifccad::{
    encode_ifccad_cbor_document, encode_ifccad_document, load_ifccad_cbor_bytes, IfccadHeader,
};
use ifccad_convert::{
    cad_document_to_ifccad_document, ifccad_document_to_cad_document, CadToIfccadOptions,
    IfccadConversionError, IfccadDiagnostic, IfccadGeometryTolerance, IfccadTargetMetadata,
    IfccadToCadOptions,
};
use opencadcodec::{io::read::ReadOutcome, DwgReader, DwgWriter, DxfReader, DxfWriter};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    io::{Cursor, Write},
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn read(bytes: Vec<u8>, format: &str) -> Result<ReadOutcome> {
    Ok(match format {
        "dwg" => DwgReader::from_stream(Cursor::new(bytes)).read_with_stats()?,
        "dxf" => DxfReader::from_reader(Cursor::new(bytes))?.read_with_stats()?,
        _ => return Err("unsupported CAD format".into()),
    })
}

fn reader_evidence(outcome: &ReadOutcome) -> Value {
    let mut kinds = std::collections::BTreeMap::new();
    for entity in outcome.document.entities() {
        *kinds
            .entry(entity.as_entity().entity_type())
            .or_insert(0usize) += 1;
    }
    let stats = &outcome.stats;
    let unclassified = outcome
        .document
        .notifications
        .iter()
        .filter(|notice| !informational_notice(notice, outcome.document.version))
        .count();
    json!({"stats": stats, "notifications": outcome.document.notifications,
        "entityKinds": kinds, "unclassifiedNotificationCount":unclassified,
        "clean": stats.stream_completed && stats.record_stream_read
            && !stats.recovery_mode && stats.skipped_source_records == 0
            && stats.recovered_errors == 0 && stats.diagnostics.is_empty()
            && unclassified == 0 && outcome.document.notifications.omitted_count() == 0})
}

// The pinned reader emits these successful header/page-map progress messages
// with Warning severity. Only these exact templates are informational. All
// notices are retained in the report; unfamiliar warnings prevent qualification.
fn informational_notice(
    notice: &opencadcodec::notification::Notification,
    version: opencadcodec::DxfVersion,
) -> bool {
    use opencadcodec::{io::dwg::dwg_version::DwgVersion, notification::NotificationType};
    if notice.notification_type != NotificationType::Warning {
        return false;
    }
    let message = notice.message.as_str();
    if DwgVersion::from_dxf_version(version)
        .is_ok_and(|v| message == format!("Reading DWG file version: {} ({v:?})", version.as_str()))
    {
        return true;
    }
    for prefix in ["AC18: Read ", "AC1021: Read "] {
        for suffix in [
            " page records from page map",
            " section descriptors from section map",
        ] {
            if message
                .strip_prefix(prefix)
                .and_then(|s| s.strip_suffix(suffix))
                .is_some_and(|s| s.parse::<u64>().is_ok())
            {
                return true;
            }
        }
    }
    message
        .strip_prefix("AC18 inner header: page_map_address=0x")
        .and_then(|s| s.split_once(", section_map_id="))
        .is_some_and(|(address, id)| {
            u64::from_str_radix(address, 16).is_ok() && id.parse::<u64>().is_ok()
        })
}

fn conversion_evidence(diagnostics: &[IfccadDiagnostic]) -> Value {
    json!({"diagnostics": diagnostics,
        "lossCount": diagnostics.iter().filter(|d| d.is_loss()).count()})
}

// Exact comparison is deliberately conservative: identities, array order and all
// numeric values are retained. No field is dropped to manufacture equivalence.
// CBOR byte equality additionally detects signed zero and exact numeric bits.
fn differences(expected: &Value, actual: &Value, path: &str, output: &mut Vec<Value>) {
    if output.len() >= 100 || expected == actual {
        return;
    }
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for key in keys {
                let next = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
                match (a.get(key), b.get(key)) {
                    (Some(a), Some(b)) => differences(a, b, &next, output),
                    (a, b) if output.len() < 100 => output.push(json!({"path":next,
                        "expectedPresent":a.is_some(), "actualPresent":b.is_some(),
                        "expected":a, "actual":b})),
                    _ => {}
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for i in 0..a.len().max(b.len()) {
                if output.len() >= 100 {
                    break;
                }
                let next = format!("{path}/{i}");
                match (a.get(i), b.get(i)) {
                    (Some(a), Some(b)) => differences(a, b, &next, output),
                    (a, b) => output.push(json!({"path":next,
                        "expectedPresent":a.is_some(), "actualPresent":b.is_some(),
                        "expected":a, "actual":b})),
                }
            }
        }
        _ => output.push(json!({"path":path,"expected":expected,"actual":actual})),
    }
}

fn pipeline(input: &Path, output: &Path, format: &str, report: &mut Value) -> Result<bool> {
    report["phase"] = json!("source-read");
    let bytes = fs::read(input)?;
    report["source"] = json!({"name":input.file_name().map(|s|s.to_string_lossy()),
        "sha256":digest(&bytes),"bytes":bytes.len(),"format":format});
    let source = read(bytes, format)?;
    report["sourceReader"] = reader_evidence(&source);
    let version = source.document.version;
    report["source"]["version"] = json!(version.as_str());
    let metadata = IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: "cad-roundtrip".into(),
            data_version: "0.1.0".into(),
            author: "IFCCAD roundtrip qualification".into(),
            timestamp: "1970-01-01T00:00:00Z".into(),
        },
    };
    // Allow collects located loss evidence; it never makes a run qualify. No
    // partial opening or opaque source capture is enabled. Geometry is exact.
    let import_options = CadToIfccadOptions {
        geometry_tolerance: IfccadGeometryTolerance::exact(),
        ..Default::default()
    };
    report["phase"] = json!("native-import");
    let native =
        cad_document_to_ifccad_document(&source.document, metadata.clone(), import_options)?;
    report["import"] = conversion_evidence(native.diagnostics());
    report["phase"] = json!("binary-write-readback");
    let binary = encode_ifccad_cbor_document(native.document())?;
    write_new(&output.join("drawing.ifccad.cbor"), binary.bytes())?;
    // Read the persisted bytes, not the pre-serialization in-memory drawing.
    let loaded = load_ifccad_cbor_bytes(
        &fs::read(output.join("drawing.ifccad.cbor"))?,
        Default::default(),
    )?;
    let projection = encode_ifccad_document(loaded.document())?;
    write_new(&output.join("drawing.ifcx"), projection.bytes())?;
    report["binary"] = json!({"sha256":digest(binary.bytes()),
        "bytes":binary.bytes().len(),"strictReadback":true,
        "encoding":"experimental IFCCAD base-CBOR revision 1",
        "embeddedSourceFile":false});
    report["phase"] = json!("native-export");
    let exported = ifccad_document_to_cad_document(
        loaded.document(),
        IfccadToCadOptions {
            geometry_tolerance: IfccadGeometryTolerance::exact(),
            ..Default::default()
        },
    )?;
    report["export"] = conversion_evidence(exported.diagnostics());
    let mut cad = exported.into_document();
    cad.version = version;
    report["phase"] = json!("cad-write");
    let target = if format == "dwg" {
        DwgWriter::write_to_vec(&cad)?
    } else {
        DxfWriter::new(&cad).write_to_vec()?
    };
    let filename = format!("roundtrip.{format}");
    write_new(&output.join(&filename), &target)?;
    report["target"] = json!({"name":filename,"sha256":digest(&target),"bytes":target.len()});
    report["phase"] = json!("cad-readback");
    let restored = read(fs::read(output.join(filename))?, format)?;
    report["targetReader"] = reader_evidence(&restored);
    report["target"]["version"] = json!(restored.document.version.as_str());
    report["phase"] = json!("native-reimport");
    let reimported = cad_document_to_ifccad_document(&restored.document, metadata, import_options)?;
    report["reimport"] = conversion_evidence(reimported.diagnostics());
    report["phase"] = json!("projection-comparison");
    let final_binary = encode_ifccad_cbor_document(reimported.document())?;
    let final_projection = encode_ifccad_document(reimported.document())?;
    write_new(&output.join("readback.ifcx"), final_projection.bytes())?;
    write_new(&output.join("readback.ifccad.cbor"), final_binary.bytes())?;
    let equal = binary.bytes() == final_binary.bytes();
    let mut delta = Vec::new();
    differences(
        &serde_json::from_slice(projection.bytes())?,
        &serde_json::from_slice(final_projection.bytes())?,
        "",
        &mut delta,
    );
    report["comparison"] = json!({"exactNativeProjectionEqual":equal,
        "cadVersionEqual":version == restored.document.version,
        "firstDifferences":delta,"differenceLimit":100,
        "identityNormalization":false});
    Ok(equal
        && version == restored.document.version
        && report["sourceReader"]["clean"] == true
        && report["targetReader"]["clean"] == true
        && ["import", "export", "reimport"]
            .iter()
            .all(|p| report[p]["lossCount"] == 0))
}

fn run() -> Result<i32> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 || args.iter().any(|a| a.to_string_lossy().starts_with("--")) {
        return Err("usage: roundtrip_cad SOURCE.dwg|SOURCE.dxf FRESH_OUTPUT_DIRECTORY".into());
    }
    let input = Path::new(&args[0]);
    let output = Path::new(&args[1]);
    let format = input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(format.as_str(), "dwg" | "dxf") {
        return Err("source must be DWG or DXF".into());
    }
    fs::create_dir(output)?;
    let mut report = json!({"schemaVersion":1,"status":"incomplete", "qualified":false,
        "scope":"Exact native IFCCAD projection and diagnosed CAD semantics at the pinned codec boundary",
        "universalLosslessnessClaim":false,"independentCadReaderChecked":false,
        "unassessed":["source information not exposed by the codec", "font contours and CAD rendering",
            "evaluated Hatch fill", "complete Dimension display", "editing behavior"],
        "partialOpening":false,"opaquePreservation":false,"geometryTolerance":"exact",
        "artifactsAreCandidates":true});
    // A killed process leaves an explicit incomplete marker, never a stale pass.
    write_new(
        &output.join("report.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    let code = match pipeline(input, output, &format, &mut report) {
        Ok(qualified) => {
            report["qualified"] = json!(qualified);
            report["status"] = json!(if qualified {
                "qualified-native-roundtrip"
            } else {
                "not-qualified"
            });
            report["phase"] = json!("complete");
            i32::from(!qualified)
        }
        Err(error) => {
            report["status"] = json!("failed");
            report["error"] = json!({"message":error.to_string(),"debug":format!("{error:?}")});
            if let Some(IfccadConversionError::Unsupported(diagnostics)) =
                error.downcast_ref::<IfccadConversionError>()
            {
                report["error"]["diagnostics"] = json!(diagnostics);
            }
            if matches!(report["phase"].as_str(), Some("source-read"))
                || error.is::<std::io::Error>()
            {
                2
            } else {
                1
            }
        }
    };
    report["exitCode"] = json!(code);
    write_new(
        &output.join("report.json.tmp"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    fs::rename(output.join("report.json.tmp"), output.join("report.json"))?;
    println!(
        "{}: {}",
        report["status"].as_str().unwrap(),
        output.display()
    );
    Ok(code)
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("roundtrip_cad: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_pinned_progress_notices_are_informational() {
        use opencadcodec::{
            notification::{Notification, NotificationType},
            DxfVersion,
        };
        let version = DxfVersion::AC1032;
        for text in [
            "Reading DWG file version: AC1032 (AC24)",
            "AC18: Read 16 page records from page map",
            "AC18 inner header: page_map_address=0x5020, section_map_id=15",
        ] {
            assert!(informational_notice(
                &Notification::new(NotificationType::Warning, text),
                version
            ));
            assert!(!informational_notice(
                &Notification::new(NotificationType::Error, text),
                version
            ));
        }
        for text in [
            "AC18: Read unknown page records from page map",
            "entity skipped",
            "Reading DWG file version: AC1032 (AC24); corrupt",
            "AC18: Read 16 page records from page map; skipped",
        ] {
            assert!(!informational_notice(
                &Notification::new(NotificationType::Warning, text),
                version
            ));
        }
    }
    #[test]
    fn differences_retain_missing_null_and_reference_changes() {
        let mut out = Vec::new();
        differences(
            &json!({"a/b":null,"ref":"/cad/d1/e1"}),
            &json!({"ref":"/cad/d1/e2"}),
            "",
            &mut out,
        );
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["path"], "/a~1b");
        assert_eq!(out[0]["expectedPresent"], true);
        assert_eq!(out[0]["actualPresent"], false);
        assert_eq!(out[1]["expected"], "/cad/d1/e1");
    }
}
