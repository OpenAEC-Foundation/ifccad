use super::{accounting, adapters, projection, recipe};
use adapters::Result;
use ifccad::package::{load_directory_package, AssessmentCompleteness, PackageValidity};
use ifccad_convert::cadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use ifccad_convert::{
    cad_document_to_package, drawing_to_cad_document, ExportError, ExportLossPolicy, ExportOptions,
    TransferAssessment,
};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

pub fn fresh_directory(path: &Path) -> Result<()> {
    // create_dir is atomic and rejects an existing directory; never overwrite a run.
    fs::create_dir(path)?;
    Ok(())
}

fn assessment(value: &TransferAssessment) -> Value {
    json!({"conclusion":format!("{:?}",value.conclusion()), "scope":format!("{:?}",value.scope()),
        "coverage":format!("{:?}",value.coverage()), "limitations":value.limitations()})
}

fn check(name: &str, result: Result<()>) -> Value {
    match result {
        Ok(()) => json!({"stage":name,"status":"passed"}),
        Err(error) => json!({"stage":name,"status":"failed","error":error.to_string()}),
    }
}

fn read_package(path: &Path, expected: &recipe::Drawing) -> Result<Vec<u64>> {
    let loaded = load_directory_package(path)?;
    if loaded.report().assessment().validity() != PackageValidity::Valid
        || loaded.report().assessment().completeness() != AssessmentCompleteness::Complete
    {
        return Err(format!("package assessment: {:?}", loaded.report()).into());
    }
    let package = loaded
        .validated_package()
        .ok_or("strict package access unavailable")?;
    if package.drawings().count() != 1 {
        return Err("expected one drawing".into());
    }
    let (actual, ids) = projection::ifccad(package.drawings().next().unwrap())?;
    projection::verify(expected, &actual)?;
    Ok(ids)
}

fn write_cad(document: &CadDocument, path: &Path, format: &str) -> Result<()> {
    let bytes = match format {
        "dxf" => DxfWriter::new(document).write_to_vec()?,
        "dwg" => DwgWriter::write_to_vec(document)?,
        _ => return Err("unknown CAD format".into()),
    };
    fs::write(path, bytes)?;
    Ok(())
}
fn read_cad(path: &Path, format: &str) -> Result<CadDocument> {
    Ok(match format {
        "dxf" => DxfReader::from_file(path)?.read()?,
        "dwg" => DwgReader::from_file(path)?.read()?,
        _ => return Err("unknown CAD format".into()),
    })
}

fn chain(root: &Path, expected: &recipe::Drawing, format: &str) -> Value {
    let mut stages = Vec::new();
    let mut metadata = json!({"loss_policy":"Reject", "source":"ifccad-external", "intermediate":format!("chain.{format}"),
        "final_package":format!("chain-{format}-ifccad")});
    let mut stage = "read source IFCCAD";
    let result = (|| -> Result<()> {
        let loaded = load_directory_package(root.join("ifccad-external"))?;
        let package = loaded
            .validated_package()
            .ok_or("strict source unavailable")?;
        let drawing = package.drawings().next().ok_or("source drawing missing")?;
        projection::verify(expected, &projection::ifccad(drawing)?.0)?;
        stages.push(check(stage, Ok(())));
        stage = "IFCCAD -> CadDocument";
        let imported = drawing_to_cad_document(drawing)?;
        metadata["import_diagnostics"] = json!(imported
            .diagnostics()
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>());
        metadata["import_assessment"] = assessment(imported.transfer_assessment());
        projection::verify(expected, &projection::cad(imported.document())?)?;
        stages.push(check(stage, Ok(())));
        let mut document = imported.into_document();
        adapters::fix_cad_metadata(&mut document);
        stage = "write CAD";
        let intermediate = root.join(format!("chain.{format}"));
        write_cad(&document, &intermediate, format)?;
        stages.push(check(stage, Ok(())));
        stage = "read CAD and compare recipe";
        let read = read_cad(&intermediate, format)?;
        // Retain header evidence without changing or normalizing the actual source.
        fs::write(
            root.join(format!("chain-{format}-source-header.txt")),
            format!("{:#?}\n", document.header),
        )?;
        fs::write(
            root.join(format!("chain-{format}-read-header.txt")),
            format!("{:#?}\n", read.header),
        )?;
        projection::verify(expected, &projection::cad(&read)?)?;
        stages.push(check(stage, Ok(())));
        stage = "CadDocument -> IFCCAD (Reject)";
        let exported = match cad_document_to_package(
            &read,
            adapters::options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
            },
        ) {
            Ok(outcome) => outcome,
            Err(ExportError::LossRejected { diagnostics }) => {
                metadata["export_diagnostics"] =
                    json!(diagnostics.iter().map(|d| json!({
                    "source":format!("{:?}",d.source()), "action":format!("{:?}",d.action()),
                    "reasons":d.reasons().iter().map(|r| format!("{r:?}")).collect::<Vec<_>>()
                })).collect::<Vec<_>>());
                return Err(format!("LossRejected {{ diagnostics: {diagnostics:?} }}").into());
            }
            Err(error) => return Err(format!("{error:?}").into()),
        };
        metadata["export_diagnostics"] = json!(exported
            .diagnostics()
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>());
        metadata["export_assessment"] = assessment(exported.transfer_assessment());
        let final_root = root.join(format!("chain-{format}-ifccad"));
        exported.package().write_directory(&final_root)?;
        stages.push(check(stage, Ok(())));
        stage = "read final IFCCAD and compare recipe";
        read_package(&final_root, expected)?;
        stages.push(check(stage, Ok(())));
        Ok(())
    })();
    let success = result.is_ok();
    if !success {
        stages.push(check(stage, result));
    }
    metadata["status"] = json!(if success { "passed" } else { "failed" });
    metadata["stages"] = json!(stages);
    metadata
}

fn generated(root: &Path, case: &recipe::Case) -> Result<Value> {
    let expected = recipe::generate(case)?;
    let (lines, polylines, vertices) = expected.counts();
    fresh_directory(root)?;
    let mut outputs = serde_json::Map::new();
    let mut external_ids = None;
    for inline in [false, true] {
        let mode = if inline {
            "ifccad-inline"
        } else {
            "ifccad-external"
        };
        let mut row = json!({"status":"failed"});
        let result = (|| -> Result<()> {
            let encoded = adapters::package(&expected, inline)?;
            encoded.write_directory(root.join(mode))?;
            let files = encoded
                .files()
                .map(|(path, _)| {
                    Ok(accounting::file(
                        path,
                        if path.ends_with(".ifcx.json") {
                            "ifcx"
                        } else {
                            "ifcdr"
                        },
                        &fs::read(root.join(mode).join(path))?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            row = accounting::account(
                files,
                lines + polylines,
                vertices,
                true,
                !inline && lines == 0 && polylines > 0,
            )?;
            let ids = read_package(&root.join(mode), &expected)?;
            if inline {
                if external_ids.as_ref() != Some(&ids) {
                    return Err("IFCCAD entity IDs/order differ across storage modes or external validation failed".into());
                }
            } else {
                external_ids = Some(ids);
            }
            row["assessment"] = json!({"validity":"valid","completeness":"complete"});
            Ok(())
        })();
        row["validation"] = check("production readback and exact recipe comparison", result);
        row["status"] = row["validation"]["status"].clone();
        row["artifact"] = json!(mode);
        outputs.insert(mode.into(), row);
    }
    for format in ["dxf", "dwg"] {
        let mut row = json!({});
        let result = (|| -> Result<()> {
            let document = adapters::cad(&expected)?;
            projection::verify(&expected, &projection::cad(&document)?)?;
            let path = root.join(format!("drawing.{format}"));
            write_cad(&document, &path, format)?;
            row = accounting::account(
                vec![accounting::file(
                    &format!("drawing.{format}"),
                    "cad",
                    &fs::read(&path)?,
                )],
                lines + polylines,
                vertices,
                true,
                false,
            )?;
            projection::verify(&expected, &projection::cad(&read_cad(&path, format)?)?)?;
            Ok(())
        })();
        row["validation"] = check("production readback and exact recipe comparison", result);
        row["status"] = row["validation"]["status"].clone();
        row["artifact"] = json!(format!("drawing.{format}"));
        row["version"] = json!("AC1032");
        outputs.insert(format.into(), row);
    }
    for mode in ["ifccad-external", "ifccad-inline"] {
        for format in ["dxf", "dwg"] {
            let ratio =
                if outputs[mode]["status"] == "passed" && outputs[format]["status"] == "passed" {
                    Some(
                        outputs[mode]["bytes"]["total"].as_u64().unwrap() as f64
                            / outputs[format]["bytes"]["total"].as_u64().unwrap() as f64,
                    )
                } else {
                    None
                };
            outputs.get_mut(mode).unwrap()[format!("ratio_to_{format}")] = json!(ratio);
        }
    }
    let mut chains = serde_json::Map::new();
    for format in ["dxf", "dwg"] {
        chains.insert(format.into(), chain(root, &expected, format));
    }
    Ok(
        json!({"id":case.id,"kind":"generated-native", "counts":{"drawings":1,"scopes":1,"lines":lines,"polylines":polylines,"polyline_vertices":vertices},
        "outputs":outputs,"chains":chains}),
    )
}

fn fixture(repo: &Path, inventory: &Value) -> Result<Value> {
    let id = inventory["id"].as_str().ok_or("fixture id missing")?;
    let native = inventory["native"]
        .as_bool()
        .ok_or("fixture kind missing")?;
    let root = repo.join("conformance/next/packages/valid").join(id);
    let mut files = Vec::new();
    for pair in inventory["files"]
        .as_array()
        .ok_or("fixture files missing")?
    {
        let path = pair[0].as_str().ok_or("fixture path missing")?;
        let role = pair[1].as_str().ok_or("fixture role missing")?;
        files.push(accounting::file(
            path,
            role,
            &fs::read(root.join(path)).map_err(|e| format!("fixture {id}/{path}: {e}"))?,
        ));
    }
    let loaded = load_directory_package(&root)?;
    let package = loaded
        .validated_package()
        .ok_or_else(|| format!("fixture {id}: {:?}", loaded.report()))?;
    let expected_validity = if native {
        PackageValidity::Valid
    } else {
        PackageValidity::NotFullyAssessed
    };
    if loaded.report().assessment().validity() != expected_validity {
        return Err(format!("fixture {id}: unexpected assessment").into());
    }
    let (mut drawings, mut scopes, mut lines, mut polylines, mut vertices) = (0, 0, 0, 0, 0);
    for drawing in package.drawings() {
        drawings += 1;
        let resource = drawing.representation().resource();
        for scope in resource.scopes() {
            scopes += 1;
            for entity in resource.entities(scope.id()) {
                match entity {
                    ifccad::ifcdr::IfcdrEntityRef::Line(_) => lines += 1,
                    ifccad::ifcdr::IfcdrEntityRef::Polyline(p) => {
                        polylines += 1;
                        vertices += p.points().count() as u64;
                    }
                }
            }
        }
    }
    let mut row = accounting::account(
        files,
        lines + polylines,
        vertices,
        native,
        lines == 0 && polylines > 0,
    )?;
    row["id"] = json!(id);
    row["kind"] = json!(if native {
        "fixture-native"
    } else {
        "fixture-preservation-inclusive"
    });
    row["status"] = json!("passed");
    row["assessment"] = serde_json::to_value(loaded.report().assessment())?;
    row["counts"] = json!({"drawings":drawings,"scopes":scopes,"lines":lines,"polylines":polylines,"polyline_vertices":vertices});
    Ok(row)
}

pub fn execute(repo: &Path, root: &Path, corpus: &Value, selected: Option<&str>) -> Result<Value> {
    fresh_directory(root)?;
    let mut cases = Vec::new();
    for definition in corpus["cases"].as_array().ok_or("corpus cases missing")? {
        let case: recipe::Case = serde_json::from_value(definition.clone())?;
        if selected.is_some_and(|id| id != case.id) {
            continue;
        }
        eprintln!("Measuring {}", case.id);
        let row = match generated(&root.join(&case.id), &case) {
            Ok(row) => row,
            Err(error) => json!({"id":case.id,"status":"failed","error":error.to_string()}),
        };
        cases.push(row);
    }
    if cases.is_empty() {
        return Err("no selected corpus cases".into());
    }
    let mut fixtures = Vec::new();
    for inventory in corpus["fixtures"]
        .as_array()
        .ok_or("fixture inventory missing")?
    {
        fixtures.push(match fixture(repo, inventory) {
            Ok(row) => row,
            Err(error) => json!({"id":inventory["id"],"status":"failed","error":error.to_string()}),
        });
    }
    Ok(json!({"cases":cases,"fixtures":fixtures}))
}

pub fn successful(result: &Value) -> bool {
    let Some(cases) = result["cases"].as_array() else {
        return false;
    };
    let Some(fixtures) = result["fixtures"].as_array() else {
        return false;
    };
    !cases.is_empty()
        && fixtures.len() == 3
        && cases.iter().all(|case| {
            ["ifccad-external", "ifccad-inline", "dxf", "dwg"]
                .iter()
                .all(|mode| case["outputs"][mode]["status"] == "passed")
                && ["dxf", "dwg"]
                    .iter()
                    .all(|format| case["chains"][format]["status"] == "passed")
        })
        && fixtures.iter().all(|f| f["status"] == "passed")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_readers_preserve_small_mixed_drawing_and_storage_ids() {
        let root = std::env::temp_dir().join(format!("ifccad-size-readers-{}", std::process::id()));
        fresh_directory(&root).unwrap();
        let expected = recipe::generate(&recipe::Case {
            id: "test".into(),
            family: "mixed".into(),
            count: 8,
        })
        .unwrap();
        let mut ids = None;
        for inline in [false, true] {
            let path = root.join(if inline { "inline" } else { "external" });
            adapters::package(&expected, inline)
                .unwrap()
                .write_directory(&path)
                .unwrap();
            let actual = read_package(&path, &expected).unwrap();
            if let Some(ids) = &ids {
                assert_eq!(ids, &actual);
            } else {
                ids = Some(actual);
            }
        }
        let document = adapters::cad(&expected).unwrap();
        for format in ["dxf", "dwg"] {
            let path = root.join(format!("drawing.{format}"));
            write_cad(&document, &path, format).unwrap();
            projection::verify(
                &expected,
                &projection::cad(&read_cad(&path, format).unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
    #[test]
    fn a_partial_or_failed_run_cannot_pass() {
        assert!(!successful(&json!({"cases":[],"fixtures":[]})));
        assert!(!successful(
            &json!({"cases":[{"id":"broken","status":"failed"}],"fixtures":[]})
        ));
        let check = check("read", Err("missing file".into()));
        assert_eq!(check["status"], "failed");
        assert_eq!(check["error"], "missing file");
    }
    #[test]
    fn existing_output_is_not_overwritten_and_missing_inventory_fails() {
        let root = std::env::temp_dir().join(format!("ifccad-size-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("sentinel"), b"keep").unwrap();
        assert!(fresh_directory(&root).is_err());
        assert_eq!(fs::read(root.join("sentinel")).unwrap(), b"keep");
        assert!(fixture(
            &root,
            &json!({"id":"missing","native":true,"files":[["drawing.json","ifcdr"]]})
        )
        .is_err());
    }
}
