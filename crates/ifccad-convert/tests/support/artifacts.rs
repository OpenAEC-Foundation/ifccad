#![allow(dead_code)]

use super::documents::{loss_heavy_document, supported_model_space_document};
use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{load_directory_package, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::cadcodec::{CadDocument, DwgReader, DxfReader, DxfWriter};
use ifccad_convert::{cad_document_to_package, drawing_to_cad_document, ExportOptions};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn write_chain_artifacts(root: &Path, document: &CadDocument) -> Result<()> {
    fs::create_dir(root)?;
    let source_path = root.join("source.dxf");
    DxfWriter::new(document).write_to_file(&source_path)?;
    let reloaded = DxfReader::from_file(&source_path)?.read()?;
    let outcome = cad_document_to_package(
        &reloaded,
        package_options(
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("chain"),
        ),
        ExportOptions::default(),
    )?;
    fs::write(
        root.join("diagnostics.txt"),
        format!("{:#?}\n", outcome.diagnostics()),
    )?;

    let package_root = root.join("ifccad");
    outcome.package().write_directory(&package_root)?;
    let inspected = load_directory_package(&package_root)?;
    if !inspected.report().is_empty() {
        return Err(format!("exported package is invalid: {:#?}", inspected.report()).into());
    }
    let package = inspected
        .validated_package()
        .ok_or("strictly loaded package was not validated")?;
    let drawing = package
        .drawings()
        .next()
        .ok_or("exported package has no drawing")?;
    let imported = drawing_to_cad_document(drawing)?;
    DxfWriter::new(imported.document()).write_to_file(root.join("roundtrip.dxf"))?;
    Ok(())
}

pub fn write_review_artifacts(root: &Path) -> Result<()> {
    fs::create_dir(root)?;
    write_chain_artifacts(&root.join("supported"), &supported_model_space_document())?;
    write_chain_artifacts(&root.join("loss-heavy"), &loss_heavy_document())?;

    let fixture_root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let inspected = load_directory_package(fixture_root)?;
    let package = inspected
        .validated_package()
        .ok_or("bundled fixture was not strictly valid")?;
    let drawing = package
        .drawings()
        .next()
        .ok_or("bundled fixture has no drawing")?;
    let document = drawing_to_cad_document(drawing)?.into_document();
    write_chain_artifacts(&root.join("minimal-no-preservation"), &document)?;

    if let Some(samples) = std::env::var_os("IFCCAD_MANUAL_SAMPLES") {
        write_manual_samples(root, Path::new(&samples))?;
    }
    Ok(())
}

fn write_manual_samples(root: &Path, samples: &Path) -> Result<()> {
    let selections = [
        ("simple", samples.join("nextgis/line_2013.dxf")),
        ("rich", samples.join("libre/sample_2018.dxf")),
    ];
    let manual_root = root.join("manual");
    fs::create_dir(&manual_root)?;
    for (role, source) in selections {
        let document = read_cad(&source)?;
        let target = manual_root.join(role);
        write_chain_artifacts(&target, &document)?;
        let bytes = fs::read(&source)?;
        let digest = Sha256::digest(&bytes);
        let mut entity_types = BTreeMap::<String, usize>::new();
        for entity in document.entities() {
            *entity_types
                .entry(entity.as_entity().entity_type().to_owned())
                .or_default() += 1;
        }
        fs::write(
            target.join("source-info.txt"),
            format!(
                "role: {role}\nsource: {}\nsha256: {digest:x}\nlayers: {}\nentities: {}\nobjects: {}\nentity_types: {entity_types:#?}\ndiagnostics: diagnostics.txt\n",
                source.canonicalize()?.display(),
                document.layers.len(),
                document.entities().count(),
                document.objects.len(),
            ),
        )?;
    }
    Ok(())
}

fn read_cad(path: &Path) -> Result<CadDocument> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("dxf") => Ok(DxfReader::from_file(path)?.read()?),
        Some("dwg") => Ok(DwgReader::from_file(path)?.read()?),
        _ => Err(format!("unsupported manual sample: {}", path.display()).into()),
    }
}

fn package_options(label: &str) -> PackageOptions {
    let normalized = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    PackageOptions {
        package_id: PackageId::new(format!("export-chain-{normalized}")).unwrap(),
        data_version: "1".to_owned(),
        author: "IFCCAD chain artifact generator".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}
