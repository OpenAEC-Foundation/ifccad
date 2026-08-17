use super::discovery::{discover_resources, ResourceKind};
use super::graph::validate_ifcx_graph;
use super::loader::{DirectoryPackageLoader, PackageLoadLimits};
use super::model::{LoadedIfccadPackage, PackageAnalysis, PackageLoadOutcome};
use super::schema::{validate_ifcpr, validate_ifcx};
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity, PackageOpenError,
    PackageValidationReport,
};
use crate::ifcdr::{validate_ifcdr, LoadedIfcdrResource};
use crate::package::codes::IFCCAD_PACKAGE_CHECKSUM_MISMATCH;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

// The conformance composer will become this internal orchestrator's production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_directory_package(
    root: impl AsRef<Path>,
) -> Result<PackageLoadOutcome, PackageOpenError> {
    let mut loader = DirectoryPackageLoader::open(root, PackageLoadLimits::default())?;
    let Some(entrypoint) = loader.load_entrypoint()? else {
        return Ok(PackageLoadOutcome {
            package: None,
            analysis: None,
            validated_package: None,
            report: loader.into_report(),
        });
    };

    let discovery = discover_resources(&entrypoint.value);
    let mut declarations = discovery.declarations;
    declarations.sort_by(|left, right| {
        (left.uri.as_str(), left.kind, left.uri_location.as_str()).cmp(&(
            right.uri.as_str(),
            right.kind,
            right.uri_location.as_str(),
        ))
    });
    let mut attempted_uris = BTreeSet::new();
    let mut resources = BTreeMap::new();
    for declaration in &declarations {
        if !attempted_uris.insert(declaration.uri.as_str()) {
            continue;
        }
        if let Some(resource) =
            loader.load_json_resource(&declaration.uri, Some(&declaration.uri_location))?
        {
            resources.insert(declaration.uri.clone(), Arc::new(resource));
        }
    }

    let mut diagnostics = loader.into_report().into_diagnostics();
    diagnostics.extend(discovery.diagnostics);
    let package = Arc::new(LoadedIfccadPackage {
        entrypoint,
        declarations,
        resources,
    });
    diagnostics.extend(validate_ifcx(&package.entrypoint.value));
    let mut validated_ifcpr_uris = BTreeSet::new();
    for declaration in &package.declarations {
        if declaration.kind != ResourceKind::Ifcpr
            || !validated_ifcpr_uris.insert(declaration.uri.as_str())
        {
            continue;
        }
        if let Some(resource) = package.resources.get(&declaration.uri) {
            diagnostics.extend(validate_ifcpr(&declaration.uri, &resource.value));
        }
    }
    diagnostics.extend(verify_resource_checksums(&package));
    let graph = validate_ifcx_graph(&package.entrypoint.value);
    diagnostics.extend(graph.diagnostics);

    let mut validated_ifcdr_resources = BTreeMap::new();
    let mut attempted_ifcdr_uris = BTreeSet::new();
    for declaration in &package.declarations {
        if declaration.kind != ResourceKind::Ifcdr
            || !attempted_ifcdr_uris.insert(declaration.uri.as_str())
        {
            continue;
        }
        let Some(source) = package.resources.get(&declaration.uri) else {
            continue;
        };
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            declaration.uri.clone(),
            source.clone(),
        ));
        let (validated, resource_diagnostics) = outcome.into_parts();
        diagnostics.extend(resource_diagnostics);
        if let Some(validated) = validated {
            validated_ifcdr_resources.insert(declaration.uri.clone(), Arc::new(validated));
        }
    }

    let binding_analysis = super::bindings::analyze_resource_bindings(
        &package,
        &graph.node_indices_by_path,
        &validated_ifcdr_resources,
    );
    diagnostics.extend(binding_analysis.diagnostics);

    let analysis = Arc::new(PackageAnalysis {
        node_indices_by_path: graph.node_indices_by_path,
        validated_ifcdr_resources,
        bindings: binding_analysis.bindings,
    });

    let report = PackageValidationReport::from_diagnostics(diagnostics);
    let validated_package =
        super::analysis::build_strict_proof(package.clone(), analysis.clone(), report.is_valid());

    Ok(PackageLoadOutcome {
        package: Some(package),
        analysis: Some(analysis),
        validated_package,
        report,
    })
}

fn verify_resource_checksums(package: &LoadedIfccadPackage) -> Vec<PackageDiagnostic> {
    package
        .declarations
        .iter()
        .filter_map(|declaration| {
            let expected = declaration.checksum.as_deref()?;
            if !is_sha256_checksum(expected) {
                return None;
            }
            let resource = package.resources.get(&declaration.uri)?;
            let actual = sha256_checksum(resource.bytes());
            if actual == expected {
                return None;
            }
            Some(PackageDiagnostic {
                code: IFCCAD_PACKAGE_CHECKSUM_MISMATCH.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_uri: Some(declaration.uri.clone()),
                location: Some(declaration.checksum_location.clone()),
                context: BTreeMap::from([
                    (
                        "actualChecksum".to_owned(),
                        PackageDiagnosticContextValue::String(actual),
                    ),
                    (
                        "expectedChecksum".to_owned(),
                        PackageDiagnosticContextValue::String(expected.to_owned()),
                    ),
                ]),
                message: format!(
                    "resource checksum does not match the exact bytes for {:?}",
                    declaration.uri
                ),
            })
        })
        .collect()
}

fn is_sha256_checksum(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn sha256_checksum(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn validate_directory_package(
    root: impl AsRef<Path>,
) -> Result<PackageValidationReport, PackageOpenError> {
    Ok(load_directory_package(root)?.report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::ifcdr::{AppearanceId, LayerId, ScopeId};
    use crate::package::codes::{
        IFCCAD_PACKAGE_CHECKSUM_MISMATCH, IFCCAD_PACKAGE_ENTRYPOINT_INVALID,
        IFCCAD_PACKAGE_JSON_INVALID, IFCCAD_PACKAGE_NODE_PATH_DUPLICATE,
        IFCCAD_PACKAGE_NODE_REFERENCE_MISSING, IFCCAD_PACKAGE_RESOURCE_MISSING,
        IFCCAD_PACKAGE_SCHEMA_INVALID,
    };
    use crate::package::{PackageDiagnosticContextValue, DIRECTORY_PACKAGE_ENTRYPOINT};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ifccad-package-validation-{name}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn valid_checksum() -> String {
        format!("sha256:{}", "a".repeat(64))
    }

    fn write_geometry_entrypoint(root: &Path, uri: &str, checksum: &str) {
        let mut entrypoint = serde_json::json!({
            "data": [{
                "path": "geometry",
                "type": "openaec:DrawingGeometryRepresentation",
                "attributes": {
                    "geometry": {
                        "format": "openaec.ifcdr",
                        "version": "0.5.0",
                        "uri": uri,
                        "checksum": checksum,
                        "role": "modelspace"
                    }
                }
            }]
        });
        entrypoint["data"]
            .as_array_mut()
            .unwrap()
            .extend(ifcx_identity_nodes());
        fs::write(
            root.join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&entrypoint).expect("serialize entrypoint"),
        )
        .expect("write entrypoint");
    }

    fn copy_minimal_package(root: &Path) -> serde_json::Value {
        let source = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("minimal-no-preservation");
        let entrypoint = serde_json::from_slice::<serde_json::Value>(
            &fs::read(source.join(DIRECTORY_PACKAGE_ENTRYPOINT)).expect("read minimal entrypoint"),
        )
        .expect("parse minimal entrypoint");
        fs::write(
            root.join("drawing.ifcdr.json"),
            fs::read(source.join("drawing.ifcdr.json")).expect("read minimal IFCDR"),
        )
        .expect("copy minimal IFCDR");
        entrypoint
    }

    fn write_entrypoint(root: &Path, entrypoint: &serde_json::Value) {
        fs::write(
            root.join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(entrypoint).expect("serialize entrypoint"),
        )
        .expect("write entrypoint");
    }

    fn read_ifcdr(root: &Path) -> serde_json::Value {
        serde_json::from_slice(
            &fs::read(root.join("drawing.ifcdr.json")).expect("read copied IFCDR"),
        )
        .expect("parse copied IFCDR")
    }

    fn write_ifcdr_and_update_checksum(
        root: &Path,
        entrypoint: &mut serde_json::Value,
        ifcdr: &serde_json::Value,
    ) {
        let bytes = serde_json::to_vec(ifcdr).expect("serialize IFCDR");
        fs::write(root.join("drawing.ifcdr.json"), &bytes).expect("write IFCDR");
        entrypoint["data"][3]["attributes"]["geometry"]["checksum"] =
            serde_json::json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
    }

    fn minimal_ifcdr_bytes() -> Vec<u8> {
        fs::read(
            bundled_conformance_root()
                .join("packages")
                .join("valid")
                .join("minimal-no-preservation")
                .join("drawing.ifcdr.json"),
        )
        .expect("read valid IFCDR fixture")
    }

    fn ifcx_identity_nodes() -> Vec<serde_json::Value> {
        let source = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("minimal-no-preservation")
            .join(DIRECTORY_PACKAGE_ENTRYPOINT);
        let entrypoint = serde_json::from_slice::<serde_json::Value>(
            &fs::read(source).expect("read valid IFCX fixture"),
        )
        .expect("parse valid IFCX fixture");
        entrypoint["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| {
                matches!(
                    node.get("type").and_then(serde_json::Value::as_str),
                    Some("openaec:Layer" | "openaec:Appearance")
                )
            })
            .cloned()
            .collect()
    }

    #[test]
    fn load_outcome_retains_entrypoint_resources_and_exact_bytes() {
        let root = TestDirectory::new("loaded-model");
        let entrypoint = br#"{"data":[{"path":"geometry","type":"openaec:DrawingGeometryRepresentation","attributes":{"geometry":{"format":"openaec.ifcdr","version":"0.5.0","uri":"drawing.ifcdr.json","checksum":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","role":"modelspace"}}}]}"#;
        let drawing = b"{\r\n  \"header\": {}\r\n}\r\n";
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), entrypoint)
            .expect("write entrypoint");
        fs::write(root.path().join("drawing.ifcdr.json"), drawing).expect("write drawing resource");

        let outcome = load_directory_package(root.path()).expect("load directory package");
        let package = outcome.package.expect("entrypoint produced package");

        assert_eq!(package.entrypoint.bytes, entrypoint);
        assert_eq!(package.entrypoint.value["data"][0]["path"], "geometry");
        assert_eq!(package.declarations.len(), 1);
        assert_eq!(package.declarations[0].uri, "drawing.ifcdr.json");
        let source = package.resources["drawing.ifcdr.json"].clone();
        let second = package.resources["drawing.ifcdr.json"].clone();
        assert!(Arc::ptr_eq(&source, &second));
        assert_eq!(source.bytes(), drawing);
        assert_eq!(source.value()["header"], serde_json::json!({}));
    }

    #[test]
    fn ifcdr_valid_fixture_produces_a_shared_validated_resource() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("minimal-no-preservation");
        let outcome = load_directory_package(root).expect("load valid fixture");
        let package = outcome.package.as_ref().expect("loaded package");
        let validated = &outcome
            .analysis
            .as_ref()
            .expect("package analysis")
            .validated_ifcdr_resources["drawing.ifcdr.json"];

        assert!(Arc::ptr_eq(
            validated.loaded().source(),
            &package.resources["drawing.ifcdr.json"]
        ));
        assert_eq!(
            validated.loaded().source().bytes(),
            package.resources["drawing.ifcdr.json"].bytes()
        );
    }

    #[test]
    fn package_analysis_shares_loaded_package_and_ifcdr_proof() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("minimal-no-preservation");
        let outcome = load_directory_package(root).expect("load valid fixture");
        let package = outcome.package.as_ref().expect("loaded package");
        let second = package.clone();
        let analysis = outcome.analysis.as_ref().expect("package analysis");
        let ifcdr = &analysis.validated_ifcdr_resources["drawing.ifcdr.json"];

        assert!(Arc::ptr_eq(package, &second));
        assert_eq!(analysis.node_indices_by_path["drawing-main"], 1);
        assert!(Arc::ptr_eq(
            ifcdr.loaded().source(),
            &package.resources["drawing.ifcdr.json"]
        ));
    }

    #[test]
    fn package_analysis_valid_package_produces_an_independent_strict_proof() {
        let proof = {
            let root = bundled_conformance_root()
                .join("packages")
                .join("valid")
                .join("minimal-no-preservation");
            let outcome = load_directory_package(root).expect("load valid fixture");
            let package = outcome.package.as_ref().expect("loaded package");
            let analysis = outcome.analysis.as_ref().expect("package analysis");
            let proof = outcome
                .validated_package
                .as_ref()
                .expect("strict package proof");

            assert!(Arc::ptr_eq(proof.loaded().package(), package));
            assert!(Arc::ptr_eq(proof.evidence(), analysis));
            outcome
                .validated_package
                .expect("owned strict package proof")
        };

        assert_eq!(
            proof.loaded().package().entrypoint.value()["data"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
    }

    #[test]
    fn package_analysis_invalid_package_retains_partial_state_without_proof() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("invalid")
            .join("package-missing-resource");
        let outcome = load_directory_package(root).expect("load invalid fixture");

        assert!(outcome.package.is_some());
        assert!(outcome.analysis.is_some());
        assert!(outcome.validated_package.is_none());
        assert!(!outcome.report.is_valid());
    }

    #[test]
    fn resource_same_kind_uri_reuses_one_ifcdr_proof_for_two_representations() {
        let root = TestDirectory::new("shared-ifcdr-uri");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut second = entrypoint["data"][3].clone();
        second["path"] = serde_json::json!("representation-modelspace-copy");
        entrypoint["data"].as_array_mut().unwrap().push(second);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load shared resource package");
        let analysis = outcome.analysis.as_ref().expect("package analysis");
        let first = &analysis.bindings.geometry_ifcdr_by_path["representation-modelspace-main"];
        let second = &analysis.bindings.geometry_ifcdr_by_path["representation-modelspace-copy"];

        assert!(outcome.validated_package.is_some());
        assert_eq!(analysis.validated_ifcdr_resources.len(), 1);
        assert!(Arc::ptr_eq(first, second));
    }

    #[test]
    fn resource_cross_kind_uri_blocks_the_strict_proof() {
        let root = TestDirectory::new("cross-kind-uri");
        let mut entrypoint = copy_minimal_package(root.path());
        let geometry = entrypoint["data"][3]["attributes"]["geometry"].clone();
        entrypoint["data"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "path": "preservation-conflict",
                "type": "openaec:PreservationRepresentation",
                "attributes": {"preservation": {
                    "format": "openaec.ifcpr",
                    "version": "0.1.0",
                    "uri": geometry["uri"],
                    "checksum": geometry["checksum"],
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceUris": ["drawing.ifcdr.json"]
                }}
            }));
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load conflicting package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.location.as_deref() == Some("/data/8/attributes/preservation/uri")
        }));
    }

    #[test]
    fn resource_cross_kind_diagnostic_identifies_the_later_source_declaration() {
        let root = TestDirectory::new("cross-kind-source-order");
        let mut entrypoint = copy_minimal_package(root.path());
        let geometry = entrypoint["data"][3]["attributes"]["geometry"].clone();
        entrypoint["data"].as_array_mut().unwrap().insert(
            0,
            serde_json::json!({
                "path": "preservation-first",
                "type": "openaec:PreservationRepresentation",
                "attributes": {"preservation": {
                    "format": "openaec.ifcpr",
                    "version": "0.1.0",
                    "uri": geometry["uri"],
                    "checksum": geometry["checksum"],
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceUris": ["drawing.ifcdr.json"]
                }}
            }),
        );
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load conflicting package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.location.as_deref() == Some("/data/4/attributes/geometry/uri")
        }));
    }

    #[test]
    fn preservation_link_requires_a_declared_ifcdr_uri() {
        let root = TestDirectory::new("undeclared-preservation-link");
        let mut entrypoint = copy_minimal_package(root.path());
        let source = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("source-archive")
            .join("preservation.ifcpr.json");
        let preservation = fs::read(source).expect("read valid IFCPR");
        fs::write(root.path().join("preservation.ifcpr.json"), &preservation)
            .expect("copy valid IFCPR");
        entrypoint["data"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "path": "preservation",
                "type": "openaec:PreservationRepresentation",
                "attributes": {"preservation": {
                    "format": "openaec.ifcpr",
                    "version": "0.1.0",
                    "uri": "preservation.ifcpr.json",
                    "checksum": format!("sha256:{:x}", Sha256::digest(&preservation)),
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceUris": ["undeclared.ifcdr.json"]
                }}
            }));
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load preservation package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.location.as_deref()
                    == Some("/data/8/attributes/preservation/linkedDrawingResourceUris/0")
        }));
    }

    #[test]
    fn layout_scope_must_exist_in_its_geometry_resource() {
        let root = TestDirectory::new("missing-layout-scope");
        let mut entrypoint = copy_minimal_package(root.path());
        entrypoint["data"][2]["attributes"]["scopeId"] = serde_json::json!(99);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.resource_uri.as_deref() == Some(DIRECTORY_PACKAGE_ENTRYPOINT)
                && diagnostic.location.as_deref() == Some("/data/2/attributes/scopeId")
        }));
    }

    #[test]
    fn invalid_ifcdr_does_not_cascade_to_layout_scope_binding() {
        let root = TestDirectory::new("invalid-ifcdr-layout-scope");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["header"]["version"] = serde_json::json!("99.0.0");
        entrypoint["data"][2]["attributes"]["scopeId"] = serde_json::json!(99);
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(!outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.location.as_deref() == Some("/data/2/attributes/scopeId")
        }));
    }

    #[test]
    fn ifcdr_layer_binding_requires_an_ifcx_layer_node() {
        let root = TestDirectory::new("missing-ifcx-layer");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["layerBindings"][0]["ifcxLayer"] = serde_json::json!("missing-layer");
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/layerBindings/0/ifcxLayer")
        }));
    }

    #[test]
    fn ifcdr_appearance_binding_requires_an_ifcx_appearance_node() {
        let root = TestDirectory::new("wrong-ifcx-appearance-type");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["appearanceBindings"][2]["ifcxAppearance"] = serde_json::json!("layer-0");
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_BINDING_INVALID"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/appearanceBindings/2/ifcxAppearance")
        }));
    }

    #[test]
    fn rejects_unknown_appearance_modes() {
        for (mode_field, property) in [
            ("colorMode", "color"),
            ("opacityMode", "opacity"),
            ("linePatternMode", "linePattern"),
            ("lineWeightMode", "lineWeight"),
        ] {
            let root = TestDirectory::new(&format!("unknown-{property}-mode"));
            let mut entrypoint = copy_minimal_package(root.path());
            let mut ifcdr = read_ifcdr(root.path());
            ifcdr["appearanceBindings"][0][mode_field] = serde_json::json!(3);
            write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
            write_entrypoint(root.path(), &entrypoint);

            let outcome = load_directory_package(root.path()).expect("load package");
            let expected_location = format!("/appearanceBindings/0/{mode_field}");

            assert!(outcome.validated_package.is_none());
            assert!(outcome.report.iter().any(|diagnostic| {
                diagnostic.code == "IFCCAD_PACKAGE_APPEARANCE_INVALID"
                    && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                    && diagnostic.location.as_deref() == Some(expected_location.as_str())
                    && diagnostic.context.get("property")
                        == Some(&PackageDiagnosticContextValue::String(property.to_owned()))
            }));
        }
    }

    #[test]
    fn rejects_explicit_appearance_properties_without_a_value_source() {
        for (mode_field, property) in [
            ("colorMode", "color"),
            ("opacityMode", "opacity"),
            ("linePatternMode", "linePattern"),
            ("lineWeightMode", "lineWeight"),
        ] {
            let root = TestDirectory::new(&format!("unresolved-{property}-appearance"));
            let mut entrypoint = copy_minimal_package(root.path());
            let mut ifcdr = read_ifcdr(root.path());
            ifcdr["appearanceBindings"][0][mode_field] = serde_json::json!(1);
            write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
            write_entrypoint(root.path(), &entrypoint);

            let outcome = load_directory_package(root.path()).expect("load package");
            let expected_location = format!("/appearanceBindings/0/{mode_field}");

            assert!(outcome.validated_package.is_none());
            assert!(outcome.report.iter().any(|diagnostic| {
                diagnostic.code == "IFCCAD_PACKAGE_APPEARANCE_INVALID"
                    && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                    && diagnostic.location.as_deref() == Some(expected_location.as_str())
                    && diagnostic.context.get("property")
                        == Some(&PackageDiagnosticContextValue::String(property.to_owned()))
            }));
        }
    }

    #[test]
    fn rejects_case_insensitive_duplicate_layer_names_per_resource() {
        let root = TestDirectory::new("duplicate-layer-name");
        let mut entrypoint = copy_minimal_package(root.path());
        entrypoint["data"][5]["attributes"]["name"] = serde_json::json!("0");
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_LAYER_NAME_DUPLICATE"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/layerBindings/1/ifcxLayer")
        }));
    }

    #[test]
    fn explicit_appearance_uses_a_valid_override_before_the_ifcx_default() {
        let root = TestDirectory::new("valid-appearance-override");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["appearanceBindings"][2]["overrideId"] = serde_json::json!(9);
        ifcdr["appearanceOverrides"] = serde_json::json!([{
            "id": 9,
            "color": {"rgb": [0, 255, 0]},
            "opacity": null,
            "ifcxLinePattern": null,
            "lineWeight": null
        }]);
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(
            outcome.validated_package.is_some(),
            "valid override should retain strict proof: {:?}",
            outcome.report.diagnostics()
        );
    }

    #[test]
    fn invalid_appearance_override_does_not_fall_back_to_the_ifcx_default() {
        let root = TestDirectory::new("invalid-appearance-override");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["appearanceBindings"][2]["overrideId"] = serde_json::json!(9);
        ifcdr["appearanceOverrides"] = serde_json::json!([{
            "id": 9,
            "color": "green",
            "opacity": null,
            "ifcxLinePattern": null,
            "lineWeight": null
        }]);
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_APPEARANCE_INVALID"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/appearanceBindings/2/colorMode")
        }));
    }

    #[test]
    fn explicit_line_pattern_override_requires_an_existing_ifcx_identity() {
        let root = TestDirectory::new("missing-line-pattern-identity");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["appearanceBindings"][2]["overrideId"] = serde_json::json!(9);
        ifcdr["appearanceOverrides"] = serde_json::json!([{
            "id": 9,
            "color": null,
            "opacity": null,
            "ifcxLinePattern": "missing-line-pattern",
            "lineWeight": null
        }]);
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_APPEARANCE_INVALID"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/appearanceBindings/2/linePatternMode")
        }));
    }

    #[test]
    fn valid_package_retains_proven_layout_layer_and_appearance_bindings() {
        let root = TestDirectory::new("proven-cross-resource-bindings");
        let entrypoint = copy_minimal_package(root.path());
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");
        let bindings = &outcome.analysis.as_ref().expect("analysis").bindings;
        let layout = &bindings.layout_by_path["drawing-main-layout-model"];

        assert!(outcome.validated_package.is_some());
        assert_eq!(layout.representation_path, "representation-modelspace-main");
        assert_eq!(layout.ifcdr_uri, "drawing.ifcdr.json");
        assert_eq!(layout.scope_id, ScopeId::new(0));
        assert_eq!(
            bindings.ifcx_layer_by_ifcdr_id[&("drawing.ifcdr.json".to_owned(), LayerId::new(1))],
            "layer-a-wall"
        );
        assert_eq!(
            bindings.ifcx_appearance_by_ifcdr_id
                [&("drawing.ifcdr.json".to_owned(), AppearanceId::new(2))],
            "appearance-default-solid"
        );
    }

    #[test]
    fn ifcdr_invalid_resource_remains_loaded_but_has_no_validation_proof() {
        let root = TestDirectory::new("invalid-ifcdr-proof");
        let bytes = br#"{"header":{"format":"openaec.ifcdr","version":"0.6.0","resourceId":"x","unit":"m","nextEntityId":1}}"#;
        let checksum = format!("sha256:{:x}", Sha256::digest(bytes));
        write_geometry_entrypoint(root.path(), "drawing.ifcdr.json", &checksum);
        fs::write(root.path().join("drawing.ifcdr.json"), bytes).expect("write IFCDR");

        let outcome = load_directory_package(root.path()).expect("load invalid IFCDR");
        assert!(outcome
            .package
            .as_ref()
            .unwrap()
            .resources
            .contains_key("drawing.ifcdr.json"));
        assert!(!outcome
            .analysis
            .as_ref()
            .unwrap()
            .validated_ifcdr_resources
            .contains_key("drawing.ifcdr.json"));
        assert!(outcome.report.iter().any(|item| {
            item.code == "IFCCAD_IFCDR_VERSION_UNSUPPORTED"
                && item.resource_uri.as_deref() == Some("drawing.ifcdr.json")
        }));
    }

    #[test]
    fn ifcdr_resources_validate_independently_within_one_package() {
        let root = TestDirectory::new("independent-ifcdr-proofs");
        let valid = fs::read(
            bundled_conformance_root()
                .join("packages")
                .join("valid")
                .join("minimal-no-preservation")
                .join("drawing.ifcdr.json"),
        )
        .expect("read valid IFCDR");
        let invalid = br#"{"header":{"format":"openaec.ifcdr","version":"0.6.0","resourceId":"x","unit":"m","nextEntityId":1}}"#;
        let descriptor = |uri: &str, bytes: &[u8]| {
            serde_json::json!({
                "format": "openaec.ifcdr",
                "version": "0.5.0",
                "uri": uri,
                "checksum": format!("sha256:{:x}", Sha256::digest(bytes)),
                "role": "modelspace"
            })
        };
        let entrypoint = serde_json::json!({"data": [
            {"path": "valid", "type": "openaec:DrawingGeometryRepresentation", "attributes": {"geometry": descriptor("valid.ifcdr.json", &valid)}},
            {"path": "invalid", "type": "openaec:DrawingGeometryRepresentation", "attributes": {"geometry": descriptor("invalid.ifcdr.json", invalid)}}
        ]});
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&entrypoint).unwrap(),
        )
        .expect("write entrypoint");
        fs::write(root.path().join("valid.ifcdr.json"), valid).expect("write valid IFCDR");
        fs::write(root.path().join("invalid.ifcdr.json"), invalid).expect("write invalid IFCDR");

        let outcome = load_directory_package(root.path()).expect("load package");
        let package = outcome.package.unwrap();
        assert_eq!(package.resources.len(), 2);
        assert!(outcome
            .analysis
            .as_ref()
            .unwrap()
            .validated_ifcdr_resources
            .contains_key("valid.ifcdr.json"));
        assert!(!outcome
            .analysis
            .as_ref()
            .unwrap()
            .validated_ifcdr_resources
            .contains_key("invalid.ifcdr.json"));
    }

    #[test]
    fn every_loaded_ifcdr_in_committed_valid_packages_gets_a_proof() {
        let valid_root = bundled_conformance_root().join("packages").join("valid");
        for entry in fs::read_dir(valid_root).expect("read valid package fixtures") {
            let entry = entry.expect("valid package fixture entry");
            if !entry.file_type().expect("fixture type").is_dir() {
                continue;
            }
            let outcome = load_directory_package(entry.path()).expect("load valid package fixture");
            let package = outcome.package.as_ref().expect("loaded valid package");
            let ifcdr_uris = package
                .declarations
                .iter()
                .filter(|declaration| declaration.kind == ResourceKind::Ifcdr)
                .filter(|declaration| package.resources.contains_key(&declaration.uri))
                .map(|declaration| declaration.uri.as_str())
                .collect::<BTreeSet<_>>();
            let validated_uris = outcome
                .analysis
                .as_ref()
                .expect("package analysis")
                .validated_ifcdr_resources
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                validated_uris,
                ifcdr_uris,
                "fixture {:?}",
                entry.file_name()
            );
        }
    }

    #[test]
    fn missing_resource_keeps_partial_loaded_package() {
        let root = TestDirectory::new("partial-missing-resource");
        write_geometry_entrypoint(root.path(), "missing.ifcdr.json", &valid_checksum());

        let outcome = load_directory_package(root.path()).expect("load directory package");
        let package = outcome.package.expect("parsed entrypoint is retained");

        assert_eq!(package.declarations[0].uri, "missing.ifcdr.json");
        assert!(!package.resources.contains_key("missing.ifcdr.json"));
        assert_eq!(
            outcome.report.diagnostics()[0].code,
            IFCCAD_PACKAGE_RESOURCE_MISSING
        );
    }

    #[test]
    fn malformed_entrypoint_has_no_loaded_package() {
        let root = TestDirectory::new("malformed-entrypoint-outcome");
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), b"{")
            .expect("write malformed entrypoint");

        let outcome = load_directory_package(root.path()).expect("inspect directory package");

        assert!(outcome.package.is_none());
        assert_eq!(outcome.report.len(), 1);
        assert_eq!(
            outcome.report.diagnostics()[0].code,
            IFCCAD_PACKAGE_JSON_INVALID
        );
    }

    #[test]
    fn ifcx_schema_error_keeps_loaded_entrypoint() {
        let root = TestDirectory::new("ifcx-schema-partial");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{"data":[{"path":"layout","type":"openaec:DrawingLayout","attributes":{"name":"Model","kind":"sheet","scopeId":0},"children":{"Representation":"geometry"}}]}"#,
        )
        .expect("write entrypoint");

        let outcome = load_directory_package(root.path()).expect("load directory package");

        assert!(outcome.package.is_some());
        assert!(outcome.report.iter().any(|item| {
            item.code == IFCCAD_PACKAGE_SCHEMA_INVALID
                && item.location.as_deref() == Some("/data/0/attributes/kind")
        }));
    }

    #[test]
    fn checksum_uses_exact_resource_bytes() {
        let root = TestDirectory::new("exact-checksum");
        let bytes = b"{\r\n  \"header\": {}\r\n}\r\n";
        let checksum = format!("sha256:{:x}", Sha256::digest(bytes));
        write_geometry_entrypoint(root.path(), "drawing.ifcdr.json", &checksum);
        fs::write(root.path().join("drawing.ifcdr.json"), bytes).expect("write drawing resource");

        let matching = load_directory_package(root.path()).expect("load matching package");
        assert!(!matching
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_CHECKSUM_MISMATCH));

        fs::write(root.path().join("drawing.ifcdr.json"), b"{\"header\":{}}")
            .expect("change exact resource bytes");
        let changed = load_directory_package(root.path()).expect("load changed package");
        let diagnostic = changed
            .report
            .iter()
            .find(|item| item.code == IFCCAD_PACKAGE_CHECKSUM_MISMATCH)
            .expect("changed exact bytes must mismatch");

        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/geometry/checksum")
        );
        assert_eq!(
            diagnostic.context.get("expectedChecksum"),
            Some(&PackageDiagnosticContextValue::String(checksum))
        );
        assert_eq!(
            diagnostic.context.get("actualChecksum"),
            Some(&PackageDiagnosticContextValue::String(format!(
                "sha256:{:x}",
                Sha256::digest(b"{\"header\":{}}")
            )))
        );
    }

    #[test]
    fn malformed_checksum_is_not_reported_as_a_mismatch() {
        let root = TestDirectory::new("malformed-checksum");
        write_geometry_entrypoint(root.path(), "drawing.ifcdr.json", "sha256:ABC");
        fs::write(root.path().join("drawing.ifcdr.json"), b"{}").expect("write drawing resource");

        let outcome = load_directory_package(root.path()).expect("load package");
        let codes: Vec<_> = outcome
            .report
            .iter()
            .map(|item| item.code.as_str())
            .collect();

        assert!(codes.contains(&IFCCAD_PACKAGE_SCHEMA_INVALID));
        assert!(!codes.contains(&IFCCAD_PACKAGE_CHECKSUM_MISMATCH));
    }

    #[test]
    fn loaded_package_retains_the_partial_graph_index() {
        let root = TestDirectory::new("loaded-graph-index");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{"data":[{"path":"same","type":"example:First"},{"path":"same","type":"example:Second"}]}"#,
        )
        .expect("write entrypoint");

        let outcome = load_directory_package(root.path()).expect("load package");
        assert!(outcome.package.is_some());
        let analysis = outcome.analysis.as_ref().expect("package analysis");

        assert_eq!(analysis.node_indices_by_path["same"], 0);
        assert!(outcome
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_NODE_PATH_DUPLICATE));
    }

    #[test]
    fn combined_diagnostics_follow_the_report_ordering_contract() {
        let root = TestDirectory::new("orchestration-order");
        let checksum = valid_checksum();
        let mut entrypoint = serde_json::json!({
            "data": [
                {
                    "path": "drawing-set",
                    "type": "openaec:DrawingSet",
                    "children": {"Drawings": ["missing-drawing"]}
                },
                {
                    "path": "missing-geometry",
                    "type": "openaec:DrawingGeometryRepresentation",
                    "attributes": {
                        "geometry": {
                            "format": "openaec.ifcdr",
                            "version": "0.5.0",
                            "uri": "z-missing.ifcdr.json",
                            "checksum": checksum,
                            "role": "modelspace"
                        }
                    }
                },
                {
                    "path": "loaded-geometry",
                    "type": "openaec:DrawingGeometryRepresentation",
                    "attributes": {
                        "geometry": {
                            "format": "openaec.ifcdr",
                            "version": "0.5.0",
                            "uri": "a-loaded.ifcdr.json",
                            "checksum": checksum,
                            "role": ""
                        }
                    }
                },
                {"path": "duplicate", "type": "example:First"},
                {"path": "duplicate", "type": "example:Second"}
            ]
        });
        entrypoint["data"]
            .as_array_mut()
            .unwrap()
            .extend(ifcx_identity_nodes());
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&entrypoint).expect("serialize entrypoint"),
        )
        .expect("write entrypoint");
        let valid_ifcdr = minimal_ifcdr_bytes();
        fs::write(root.path().join("a-loaded.ifcdr.json"), valid_ifcdr)
            .expect("write loaded resource");

        let outcome = load_directory_package(root.path()).expect("load package");
        let sequence: Vec<_> = outcome
            .report
            .iter()
            .map(|item| {
                (
                    item.code.as_str(),
                    item.resource_uri.as_deref(),
                    item.location.as_deref(),
                )
            })
            .collect();

        assert_eq!(
            sequence,
            [
                (
                    IFCCAD_PACKAGE_CHECKSUM_MISMATCH,
                    Some("a-loaded.ifcdr.json"),
                    Some("/data/2/attributes/geometry/checksum"),
                ),
                (
                    IFCCAD_PACKAGE_NODE_REFERENCE_MISSING,
                    Some(DIRECTORY_PACKAGE_ENTRYPOINT),
                    Some("/data/0/children/Drawings/0"),
                ),
                (
                    IFCCAD_PACKAGE_SCHEMA_INVALID,
                    Some(DIRECTORY_PACKAGE_ENTRYPOINT),
                    Some("/data/2/attributes/geometry/role"),
                ),
                (
                    IFCCAD_PACKAGE_NODE_PATH_DUPLICATE,
                    Some(DIRECTORY_PACKAGE_ENTRYPOINT),
                    Some("/data/4/path"),
                ),
                (
                    IFCCAD_PACKAGE_RESOURCE_MISSING,
                    Some("z-missing.ifcdr.json"),
                    Some("/data/1/attributes/geometry/uri"),
                ),
            ]
        );
    }

    #[test]
    fn validates_a_directory_from_ifcx_discovered_resources() {
        let root = TestDirectory::new("discovered-resources");
        let resource = minimal_ifcdr_bytes();
        let checksum = format!("sha256:{:x}", Sha256::digest(&resource));
        write_geometry_entrypoint(root.path(), "custom-name.ifcdr.json", &checksum);
        fs::write(root.path().join("custom-name.ifcdr.json"), &resource)
            .expect("write discovered resource");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert!(report.is_valid());
        assert!(report.is_empty());
    }

    #[test]
    fn reports_a_resource_missing_at_its_ifcx_declaration() {
        let root = TestDirectory::new("missing-discovered-resource");
        write_geometry_entrypoint(root.path(), "custom-name.ifcdr.json", &valid_checksum());

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_RESOURCE_MISSING);
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some("custom-name.ifcdr.json")
        );
        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/geometry/uri")
        );
    }

    #[test]
    fn keeps_discovery_diagnostics_with_loader_diagnostics() {
        let root = TestDirectory::new("combined-diagnostics");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{
              "data": [
                {
                  "type": "openaec:DrawingGeometryRepresentation",
                  "attributes": { "geometry": { "url": "ignored.json" } }
                },
                {
                  "type": "openaec:PreservationRepresentation",
                  "attributes": {
                    "preservation": { "uri": "malformed.ifcpr.json" }
                  }
                }
              ]
            }"#,
        )
        .expect("write entrypoint");
        fs::write(root.path().join("malformed.ifcpr.json"), b"{")
            .expect("write malformed resource");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        let codes: Vec<_> = report.iter().map(|item| item.code.as_str()).collect();
        assert!(codes.contains(&IFCCAD_PACKAGE_ENTRYPOINT_INVALID));
        assert!(codes.contains(&IFCCAD_PACKAGE_JSON_INVALID));
        assert!(codes.contains(&IFCCAD_PACKAGE_SCHEMA_INVALID));
    }

    #[test]
    fn returns_entrypoint_loading_diagnostics_without_discovery() {
        let root = TestDirectory::new("malformed-entrypoint");
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), b"{")
            .expect("write malformed entrypoint");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert_eq!(report.len(), 1);
        assert_eq!(report.diagnostics()[0].code, IFCCAD_PACKAGE_JSON_INVALID);
    }

    #[test]
    fn loads_committed_bootstrap_packages_and_reports_legacy_links() {
        for case in [
            "minimal-no-preservation",
            "unrepresented-packed",
            "source-archive",
            "generated-snapshot",
            "multi-drawing-projections",
        ] {
            let root = bundled_conformance_root()
                .join("packages")
                .join("valid")
                .join(case);
            let outcome = load_directory_package(root).expect("inspect committed package");
            let package = outcome.package.as_ref().expect("retain committed package");
            let analysis = outcome.analysis.as_ref().expect("package analysis");
            assert_eq!(package.entrypoint.uri, DIRECTORY_PACKAGE_ENTRYPOINT);
            assert!(!analysis.node_indices_by_path.is_empty(), "case {case}");

            if case == "minimal-no-preservation" {
                assert!(
                    outcome.report.is_empty(),
                    "case {case}: {:?}",
                    outcome.report.diagnostics()
                );
                continue;
            }

            let properties: Vec<_> = outcome
                .report
                .iter()
                .filter_map(|item| item.context.get("property"))
                .collect();
            assert!(properties.contains(&&PackageDiagnosticContextValue::String(
                "linkedDrawingResourceIds".to_owned()
            )));
            assert!(properties.contains(&&PackageDiagnosticContextValue::String(
                "linkedDrawingResourceUris".to_owned()
            )));
        }
    }

    #[test]
    fn discovers_the_committed_missing_resource_case() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("invalid")
            .join("package-missing-resource");
        let outcome = load_directory_package(root).expect("inspect missing-resource case");

        let package = outcome.package.as_ref().expect("retain partial package");
        assert!(package
            .declarations
            .iter()
            .any(|declaration| declaration.uri == "drawing.ifcdr.json"));
        assert!(outcome
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_RESOURCE_MISSING));
    }
}
