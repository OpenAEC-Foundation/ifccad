//! Explorer's shared Studio mapping. The reference converter is not a fallback.
use crate::{fail, ifccad_input, options, result};
use ifccad::IfccadDocument;
use opencaddocument::{Document, Dwg, Dxf, DxfVersion, Format, IfccadImportOutcome, Ifcx, Loss};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};

const LIMITATIONS: &str = "Shared Studio mapping: numerical error bounds and rendered appearance are unassessed. Partial recovery, spline preservation and Reject policy are unavailable. Reported losses are not an exhaustive preservation audit.";

pub fn document_capabilities() -> Value {
    json!({"ifccad":{"backend":"opencaddocument","revision":"4723b0e6b4b38354853eb5ce1d0f9af23a8bbb2f","partialOpening":false,"adjustableTolerance":false,"coordinateToleranceDefault":false,"coordinateToleranceFallback":false,"splinePreservation":false,"lossRejection":false,"numericalAssessment":"unassessed","directIfcxPreview":true,"limitations":LIMITATIONS}})
}

fn parse_options(
    name: &str,
    format: &str,
    text: &str,
) -> Result<options::ConversionOptions, Value> {
    let parsed = options::ConversionOptions::parse(text)
        .map_err(|e| unperformed_options(name, format, "INVALID_CONVERSION_OPTIONS", e))?;
    if parsed.partial_open
        || parsed.preserve_splines
        || parsed.loss_policy == options::ExportLossPolicy::Reject
        || !matches!(parsed.tolerance, options::Tolerance::Default)
    {
        return Err(unperformed_options(
            name,
            format,
            "DOCUMENT_OPTION_UNAVAILABLE",
            LIMITATIONS,
        ));
    }
    Ok(parsed)
}

fn diagnostics(losses: &[Loss]) -> Vec<Value> {
    let mut rows = vec![
        json!({"code":"DOCUMENT_MIGRATION_LIMITS","message":LIMITATIONS,"action":"unassessed"}),
    ];
    rows.extend(losses.iter().map(|loss| json!({"code":"DOCUMENT_CONVERSION_LOSS","message":loss.message,"handle":loss.handle.map(|h|format!("{:X}",h.value())),"action":"loss"})));
    rows
}

fn unperformed_options(name: &str, format: &str, code: &str, message: impl ToString) -> Value {
    let mut output = options::invalid(name, format, code, message);
    output["reader"]["status"] = json!("notRun");
    output
}

fn selection(drawing: &IfccadDocument, imported: &IfccadImportOutcome) -> Value {
    let mut entities = Vec::new();
    for (layout, members) in std::iter::once(("Model", &drawing.model.entities)).chain(
        drawing
            .paper_layouts
            .iter()
            .map(|p| (p.name.as_str(), &p.entities)),
    ) {
        for entity in members {
            if let Some(handle) = imported.entity_handles.get(&entity.id()) {
                if imported.document.get_entity(*handle).is_some() {
                    entities.push(json!({"path":format!("/cad/d{}/e{}",drawing.drawing_id,entity.id()),"handle":format!("{:X}",handle.value()),"layout":layout}));
                }
            }
        }
    }
    json!({"format":"ifccad","qualification":"pinnedStudioIfcxImport","entities":entities})
}

fn preview(output: &mut Value, drawing: &IfccadDocument) {
    match Ifcx.import_ifccad(drawing) {
        Ok(imported) => {
            output["preview"] = json!({"backend":"opencaddocument","viewerSelection":selection(drawing,&imported),"diagnostics":diagnostics(&imported.losses),"rendering":"unassessed"})
        }
        Err(error) => {
            output["preview"] =
                json!({"failure":{"message":error.to_string()},"rendering":"unavailable"})
        }
    }
}

pub fn document_open_ifccad(name: &str, bytes: &[u8]) -> Value {
    let mut output = crate::inspect_ifccad_bytes(name, bytes);
    if output["validation"]["strictAvailable"] == true {
        if let Ok(drawing) = ifccad_input::load(name, bytes) {
            preview(&mut output, drawing.document());
        }
    }
    output
}

fn format(name: &str) -> Result<&'static dyn Format, String> {
    match name {
        "dxf" => Ok(&Dxf),
        "dwg" => Ok(&Dwg),
        _ => Err("Choose DXF or DWG".into()),
    }
}

pub fn document_cad_to_ifccad(
    name: &str,
    cad_format: &str,
    bytes: &[u8],
    timestamp: &str,
    text: &str,
) -> Value {
    let options = match parse_options(name, cad_format, text) {
        Ok(value) => value,
        Err(output) => return output,
    };
    let mut output = result(Path::new(name), cad_format);
    let (document, mut losses) =
        match format(cad_format).and_then(|codec| codec.read(bytes).map_err(|e| e.to_string())) {
            Ok(value) => value,
            Err(error) => {
                fail(&mut output, "reading", "DOCUMENT_CAD_READ_FAILED", error);
                return output;
            }
        };
    let converted = (|| -> Result<(), String> {
        output["reader"]["version"] = json!(document
            .dwg_source_version
            .unwrap_or(document.version)
            .as_str());
        output["reader"]["messages"] = json!(document
            .notifications
            .iter()
            .map(|n| format!("{n:?}"))
            .collect::<Vec<_>>());
        crate::progress("converting");
        let mut native = Ifcx.export_ifccad(&document).map_err(|e| e.to_string())?;
        losses.append(&mut native.losses);
        native.drawing.header.id = name.into();
        native.drawing.header.author = "IFCCAD Explorer / opencaddocument".into();
        native.drawing.header.timestamp = timestamp.into();
        output["conversion"] = json!({"backend":"opencaddocument","format":"ifccad","options":options.value(),"diagnostics":diagnostics(&losses),"lossCoverage":"reportedLossesOnly","geometryAssessment":{"status":"unassessed"},"preservation":{"status":"unavailable"}});
        let encoded = ifccad::encode_ifccad_document(&native.drawing).map_err(|e| e.to_string())?;
        let checked = ifccad::load_ifccad_bytes(encoded.bytes(), Default::default())
            .map_err(|e| e.to_string())?;
        crate::native_inspection::present_document(
            &mut output,
            checked.document(),
            checked.graph().composed_ifcx(),
        );
        preview(&mut output, checked.document());
        crate::download::download(&mut output, name, "ifccad", encoded.bytes());
        Ok(())
    })();
    if let Err(error) = converted {
        fail(
            &mut output,
            "converting",
            "DOCUMENT_CONVERSION_FAILED",
            error,
        );
    }
    output
}

fn kinds(document: &Document) -> BTreeMap<String, usize> {
    let mut result = BTreeMap::new();
    for entity in document.entities() {
        *result
            .entry(format!("{:?}", entity.as_entity().entity_type()))
            .or_default() += 1;
    }
    result
}

pub fn document_export_ifccad(
    name: &str,
    bytes: &[u8],
    cad_format: &str,
    version: &str,
    text: &str,
) -> Value {
    let options = match parse_options(name, "ifccad", text) {
        Ok(value) => value,
        Err(output) => return output,
    };
    let mut output = document_open_ifccad(name, bytes);
    if output["validation"]["strictAvailable"] != true {
        return output;
    }
    let exported = (|| -> Result<(), String> {
        let native = ifccad_input::load(name, bytes)?;
        if cad_format == "ifccad" {
            let encoded =
                ifccad::encode_ifccad_document(native.document()).map_err(|e| e.to_string())?;
            let data = if ifccad_input::encoding(name) == "ifcx" {
                bytes
            } else {
                encoded.bytes()
            };
            ifccad::load_ifccad_bytes(data, Default::default()).map_err(|e| e.to_string())?;
            let native_name = format!("{}.ifcx", ifccad_input::stem(name));
            crate::download::download(&mut output, &native_name, "ifccad", data);
            return Ok(());
        }
        let target = match version {
            "AC1015" => DxfVersion::AC1015,
            "AC1018" => DxfVersion::AC1018,
            "AC1021" => DxfVersion::AC1021,
            "AC1024" => DxfVersion::AC1024,
            "AC1027" => DxfVersion::AC1027,
            "AC1032" => DxfVersion::AC1032,
            _ => return Err("Unsupported CAD version".into()),
        };
        let mut imported = Ifcx
            .import_ifccad(native.document())
            .map_err(|e| e.to_string())?;
        let codec = format(cad_format)?;
        imported.document.version = target;
        output["conversion"] = json!({"backend":"opencaddocument","diagnostics":diagnostics(&imported.losses),"geometryAssessment":{"status":"unassessed"}});
        crate::progress("writing");
        let (written, mut losses) = codec.write(&imported.document).map_err(|e| e.to_string())?;
        crate::progress("readback");
        let (readback, mut read_losses) = codec.read(&written).map_err(|e| e.to_string())?;
        losses.append(&mut read_losses);
        if readback.version != target {
            return Err("Written CAD version does not match requested version".into());
        }
        let expected = kinds(&imported.document);
        let actual = kinds(&readback);
        if expected != actual {
            losses.push(Loss {
                handle: None,
                message: "CAD readback entity kinds/counts differ from the outgoing Document"
                    .into(),
            });
        }
        output["conversion"]["diagnostics"] =
            json!(diagnostics(&[imported.losses, losses.clone()].concat()));
        // Readback also traverses the shared native mapping and strict production reader.
        let returned = Ifcx.export_ifccad(&readback).map_err(|e| e.to_string())?;
        output["conversion"]["diagnostics"]
            .as_array_mut()
            .expect("diagnostic list")
            .extend(diagnostics(&returned.losses).into_iter().skip(1));
        let encoded =
            ifccad::encode_ifccad_document(&returned.drawing).map_err(|e| e.to_string())?;
        ifccad::load_ifccad_bytes(encoded.bytes(), Default::default())
            .map_err(|e| e.to_string())?;
        crate::download::download(&mut output, name, cad_format, &written);
        output["export"]["requestedVersion"] = json!(version);
        output["export"]["options"] = options.value();
        output["export"]["diagnostics"] = output["conversion"]["diagnostics"].clone();
        output["export"]["fileCheck"] = json!({"cadReadback":true,"versionMatched":true,"version":version,"strictNativeReadback":true,"semanticComparison":"entityKindsAndCounts","expected":expected,"actual":actual,"completeSemanticFidelity":"unassessed"});
        Ok(())
    })();
    if let Err(error) = exported {
        fail(&mut output, "exporting", "DOCUMENT_EXPORT_FAILED", error);
    }
    output
}
