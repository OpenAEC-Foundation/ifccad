use super::{accounting, adapters, projection, recipe, report};
use adapters::Result;
use ocdraw::ocdraw::{load_drawing_bytes, load_drawing_file};
use ocdraw_convert::cadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use ocdraw_convert::{
    cad_document_to_drawing, ocdraw_to_cad_document, ExportLossPolicy, ExportOptions, ImportOptions,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
pub fn fresh_directory(path: &Path) -> Result<()> {
    fs::create_dir(path)?;
    Ok(())
}
fn write_cad(cad: &CadDocument, format: &str) -> Result<Vec<u8>> {
    Ok(match format {
        "dxf" => DxfWriter::new(cad).write_to_vec()?,
        "dwg" => DwgWriter::write_to_vec(cad)?,
        _ => return Err("unknown format".into()),
    })
}
fn read_cad(bytes: Vec<u8>, format: &str) -> Result<CadDocument> {
    Ok(match format {
        "dxf" => DxfReader::from_reader(std::io::Cursor::new(bytes))?.read()?,
        "dwg" => DwgReader::from_stream(std::io::Cursor::new(bytes)).read()?,
        _ => return Err("unknown format".into()),
    })
}
fn measure(root: &Path, case: &recipe::Case) -> Result<Value> {
    fresh_directory(root)?;
    let recipe = recipe::generate(case)?;
    let native = adapters::drawing(&recipe)?;
    native.write_file(root.join("drawing.ocdraw.json"))?;
    let loaded = load_drawing_file(root.join("drawing.ocdraw.json"))?;
    let drawing = loaded
        .validated_drawing()
        .ok_or_else(|| format!("native readback: {:?}", loaded.diagnostics()))?;
    let (projection, ids) = projection::ocdraw(drawing)?;
    projection::verify(&recipe, &projection)?;
    let mut outputs = json!({"ocdraw":{"bytes":native.bytes().len(),"sha256":accounting::digest(native.bytes())}});
    let mut chains = json!({});
    for format in ["dxf", "dwg"] {
        let direct = write_cad(&adapters::cad(&recipe)?, format)?;
        fs::write(root.join(format!("direct.{format}")), &direct)?;
        projection::verify(
            &recipe,
            &projection::cad(&read_cad(direct.clone(), format)?)?,
        )?;
        outputs[format] = json!({"bytes":direct.len(),"sha256":accounting::digest(&direct)});
        let imported = ocdraw_to_cad_document(drawing, ImportOptions::default())?;
        let import_issues = imported
            .diagnostics()
            .iter()
            .map(|d| json!({"code":d.code,"location":d.location,"message":d.message}))
            .collect::<Vec<_>>();
        if imported
            .diagnostics()
            .iter()
            .any(|d| d.code != "PARAMETERIZATION_CHANGED")
        {
            return Err(format!("unexpected import loss: {:?}", imported.diagnostics()).into());
        }
        let mut cad = imported.into_document();
        adapters::fix_cad_metadata(&mut cad);
        let bytes = write_cad(&cad, format)?;
        fs::write(root.join(format!("chain.{format}")), &bytes)?;
        let returned = read_cad(bytes, format)?;
        projection::verify(&recipe, &projection::cad(&returned)?)?;
        let exported = cad_document_to_drawing(
            &returned,
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            },
        )
        .map_err(|e| format!("{format} return export: {e:?}"))?;
        let export_issues = exported
            .diagnostics()
            .iter()
            .map(|d| format!("{d:?}"))
            .collect::<Vec<_>>();
        exported
            .drawing()
            .write_file(root.join(format!("returned-{format}.ocdraw.json")))?;
        let readback = load_drawing_bytes(exported.drawing().bytes());
        let final_drawing = readback
            .validated_drawing()
            .ok_or("return readback invalid")?;
        projection::verify(&recipe, &projection::ocdraw(final_drawing)?.0)?;
        chains[format] = json!({"status":"passed","import_diagnostics":import_issues,"export_diagnostics":export_issues});
    }
    Ok(
        json!({"id":case.id,"status":"passed","entity_ids":ids,"counts":recipe.counts(),"outputs":outputs,"chains":chains,"artifacts":report::artifact_manifest(root)?}),
    )
}
pub fn execute(
    _repo: &Path,
    output: &Path,
    corpus: &Value,
    selected: Option<&str>,
) -> Result<Value> {
    fresh_directory(output)?;
    let mut rows = Vec::new();
    for value in corpus["cases"].as_array().ok_or("cases missing")? {
        let case: recipe::Case = serde_json::from_value(value.clone())?;
        if selected.is_some_and(|id| id != case.id) {
            continue;
        }
        println!("Measuring {}", case.id);
        rows.push(match measure(&output.join(&case.id), &case) {
            Ok(row) => row,
            Err(e) => json!({"id":case.id,"status":"failed","error":e.to_string()}),
        });
    }
    Ok(json!({"cases":rows}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_readback_preserves_mixed_recipe_and_identity() {
        let recipe = recipe::generate(&recipe::Case {
            id: "mixed".into(),
            family: "mixed".into(),
            count: 8,
        })
        .unwrap();
        let bytes = adapters::drawing(&recipe).unwrap();
        let loaded = load_drawing_bytes(bytes.bytes());
        let (actual, ids) = projection::ocdraw(loaded.validated_drawing().unwrap()).unwrap();
        projection::verify(&recipe, &actual).unwrap();
        assert_eq!(ids, (1..=8).collect::<Vec<_>>());
    }
}
