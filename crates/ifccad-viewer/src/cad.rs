use crate::{fail, inspect_package, inspect_package_files, progress, result};
use ifccad::{
    package::{EncodedPackage, PackageOptions},
    PackageId,
};
use ifccad_convert::cadcodec::{CadDocument, DwgReader, DxfReader};
use ifccad_convert::{
    cad_document_to_package, ExportAction, ExportDiagnosticSource, ExportOptions,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Cursor, path::Path};
pub fn inspect_cad(input: &Path, output: &Path) -> Value {
    let format = if input
        .extension()
        .is_some_and(|v| v.eq_ignore_ascii_case("dwg"))
    {
        "dwg"
    } else {
        "dxf"
    };
    let mut r = result(input, format);
    let read = (|| -> Result<CadDocument, Box<dyn std::error::Error>> {
        Ok(if format == "dwg" {
            DwgReader::from_file(input)?.read()?
        } else {
            DxfReader::from_file(input)?.read()?
        })
    })();
    let doc = match read {
        Ok(v) => v,
        Err(e) => {
            fail(&mut r, "reading", "CAD_READ_FAILED", e);
            return r;
        }
    };
    r["reader"]["messages"] = json!(doc
        .notifications
        .iter()
        .map(|n| format!("{n:?}"))
        .collect::<Vec<_>>());
    convert_document(&doc, output, r)
}

/// Read CAD bytes and validate the converted package without filesystem I/O.
pub fn inspect_cad_bytes(name: &str, format: &str, bytes: &[u8], timestamp: &str) -> Value {
    let mut r = result(Path::new(name), format);
    r["source"]["name"] = json!(name);
    let read = match format {
        "dxf" => {
            DxfReader::from_reader(Cursor::new(bytes.to_vec())).and_then(|reader| reader.read())
        }
        "dwg" => DwgReader::from_stream(Cursor::new(bytes.to_vec())).read(),
        _ => {
            fail(
                &mut r,
                "reading",
                "CAD_FORMAT_UNSUPPORTED",
                "Choose DXF or DWG",
            );
            return r;
        }
    };
    let doc = match read {
        Ok(doc) => doc,
        Err(error) => {
            fail(&mut r, "reading", "CAD_READ_FAILED", error);
            return r;
        }
    };
    r["reader"]["messages"] = json!(doc
        .notifications
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>());
    convert_document_with(&doc, r, timestamp, |encoded| {
        let files = encoded
            .files()
            .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
            .collect();
        Ok(inspect_package_files(name, &files))
    })
}

fn convert_document(doc: &CadDocument, output: &Path, r: Value) -> Value {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp");
    convert_document_with(doc, r, &timestamp, |encoded| {
        encoded.write_directory(output)?;
        Ok(inspect_package(output))
    })
}

fn convert_document_with(
    doc: &CadDocument,
    mut r: Value,
    timestamp: &str,
    inspect: impl FnOnce(&EncodedPackage) -> Result<Value, Box<dyn std::error::Error>>,
) -> Value {
    progress("converting");
    let options = PackageOptions {
        package_id: PackageId::new("viewer-local-conversion").unwrap(),
        data_version: "1".into(),
        author: "IFCCAD Format Explorer".into(),
        timestamp: timestamp.to_owned(),
    };
    let out = match cad_document_to_package(doc, options, ExportOptions::default()) {
        Ok(v) => v,
        Err(e) => {
            r["conversion"] = json!({"assessment":null,"geometry":null,"diagnostics":[],"entities":doc.entities().map(|e|json!({"handle":e.common().handle.to_string(),"kind":e.as_entity().entity_type(),"disposition":"unclassified","target":null,"reasons":[]})).collect::<Vec<_>>()});
            fail(&mut r, "converting", "CONVERSION_FAILED", format!("{e:?}"));
            return r;
        }
    };
    let assessment = out.transfer_assessment();
    let geometry = out.geometry_assessment();
    let diagnostics:Vec<_>=out.diagnostics().iter().map(|d|json!({"source":format!("{:?}",d.source()),"action":format!("{:?}",d.action()),"reasons":d.reasons().iter().map(|r|format!("{r:?}")).collect::<Vec<_>>()})).collect();
    let mut counts = BTreeMap::new();
    for e in doc.entities() {
        *counts.entry(e.common().handle).or_insert(0) += 1;
    }
    let mut entity_diagnostics = BTreeMap::<_, Vec<_>>::new();
    for diagnostic in out.diagnostics() {
        if let ExportDiagnosticSource::Entity { handle, .. } = diagnostic.source() {
            entity_diagnostics
                .entry(*handle)
                .or_default()
                .push(diagnostic);
        }
    }
    let mut entities:Vec<_>=doc.entities().map(|e|{
  let handle=e.common().handle;
  let losses=entity_diagnostics.get(&handle).map(Vec::as_slice).unwrap_or(&[]);
  let target=out.entity_mapping().target_entity_id(handle);
  let skipped=losses.iter().any(|d|d.action()==ExportAction::Skipped);
  let disposition=if counts[&handle]!=1||target.is_some()&&skipped{"unclassified"}else if target.is_some(){if losses.is_empty(){"emitted"}else{"partial"}}else if skipped{"skipped"}else{"unclassified"};
  json!({"handle":handle.to_string(),"kind":e.as_entity().entity_type(),"disposition":disposition,"target":target.map(|id|json!({"resourceId":null,"entityId":id.get().to_string()})),"reasons":losses.iter().flat_map(|d|d.reasons().iter().map(|r|format!("{r:?}"))).collect::<Vec<_>>()})
 }).collect();
    r["conversion"] = json!({"assessment":{"conclusion":format!("{:?}",assessment.conclusion()),"scope":format!("{:?}",assessment.scope()),"coverage":format!("{:?}",assessment.coverage()),"limitations":assessment.limitations()},"geometry":{"status":format!("{:?}",geometry.status()),"drawingUnit":format!("{:?}",geometry.drawing_unit()),"requestedTolerance":format!("{:?}",geometry.requested_tolerance()),"resolvedTolerance":{"lower":geometry.resolved_tolerance().lower(),"upper":geometry.resolved_tolerance().upper()},"assessedEntities":geometry.assessed_entities(),"assessedVertices":geometry.assessed_vertices(),"roundedEntities":geometry.rounded_entities(),"maxDeviationUpperBound":geometry.max_deviation_upper_bound()},"diagnostics":diagnostics,"entities":[]});
    progress("validating");
    let package = match inspect(out.package()) {
        Ok(package) => package,
        Err(error) => {
            fail(&mut r, "preparing", "PACKAGE_WRITE_FAILED", error);
            r["conversion"]["entities"] = json!(entities);
            return r;
        }
    };
    // Read actual resource identity from the generated package, not the demo fixture.
    if let Some(text) = package["presentation"]["documents"][0]["text"].as_str() {
        if let Ok(ifcx) = serde_json::from_str::<Value>(text) {
            if let Some(nodes) = ifcx["data"].as_array() {
                if let Some(rid) = nodes
                    .iter()
                    .find_map(|n| n["attributes"]["resource"]["resourceId"].as_str())
                {
                    for e in &mut entities {
                        if !e["target"].is_null() {
                            e["target"]["resourceId"] = json!(rid);
                        }
                    }
                }
            }
        }
    }
    r["conversion"]["entities"] = json!(entities);
    for key in ["validation", "presentation", "failure"] {
        r[key] = package[key].clone();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use ifccad_convert::cadcodec::{EntityType, Handle, Line};
    #[test]
    fn fatal_conversion_retains_inventory_without_graph_or_fidelity_claim() {
        let mut doc = CadDocument::new();
        let handle = doc
            .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 1., 0.)))
            .unwrap();
        doc.get_entity_mut(handle)
            .unwrap()
            .common_mut()
            .owner_handle = Handle::new(0xDEADBEEF);
        let r = convert_document(
            &doc,
            Path::new("unused-output"),
            result(Path::new("broken.dxf"), "dxf"),
        );
        assert_eq!(r["failure"]["stage"], "converting");
        assert!(r["presentation"].is_null());
        assert!(r["conversion"]["assessment"].is_null());
        assert_eq!(
            r["conversion"]["entities"][0]["disposition"],
            "unclassified"
        );
    }
}
