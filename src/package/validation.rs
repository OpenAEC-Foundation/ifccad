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
use crate::package::codes::{
    IFCCAD_PACKAGE_CHECKSUM_MISMATCH, IFCCAD_PACKAGE_RESOURCE_ID_DUPLICATE,
    IFCCAD_PACKAGE_RESOURCE_ID_MISMATCH, IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING,
};
use crate::ResourceId;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

// The conformance composer will become this internal orchestrator's production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub fn load_directory_package(
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
        (
            left.external_uri.as_str(),
            left.kind,
            left.external_uri_location.as_str(),
        )
            .cmp(&(
                right.external_uri.as_str(),
                right.kind,
                right.external_uri_location.as_str(),
            ))
    });
    let mut attempted_uris = BTreeSet::new();
    let mut resources = BTreeMap::new();
    for declaration in &declarations {
        if !attempted_uris.insert(declaration.external_uri.as_str()) {
            continue;
        }
        if let Some(resource) = loader.load_json_resource(
            &declaration.external_uri,
            Some(&declaration.external_uri_location),
        )? {
            resources.insert(declaration.external_uri.clone(), Arc::new(resource));
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
    diagnostics.extend(validate_declared_resource_id_uniqueness(
        &package.declarations,
    ));
    let mut validated_ifcpr_uris = BTreeSet::new();
    let mut ifcpr_resource_ids_by_uri = BTreeMap::new();
    for declaration in &package.declarations {
        if declaration.kind != ResourceKind::Ifcpr
            || !validated_ifcpr_uris.insert(declaration.external_uri.as_str())
        {
            continue;
        }
        if let Some(resource) = package.resources.get(&declaration.external_uri) {
            let resource_diagnostics = validate_ifcpr(
                Some(&declaration.resource_id),
                &declaration.external_uri,
                &resource.value,
            );
            if resource_diagnostics.is_empty() {
                let content_resource_id = resource
                    .value()
                    .pointer("/header/resourceId")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|value| ResourceId::new(value).ok())
                    .expect("IFCPR 0.2.0 schema proves a non-empty header resourceId");
                ifcpr_resource_ids_by_uri
                    .insert(declaration.external_uri.clone(), content_resource_id);
            }
            diagnostics.extend(resource_diagnostics);
        }
    }
    for declaration in package
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == ResourceKind::Ifcpr)
    {
        if let Some(content_resource_id) = ifcpr_resource_ids_by_uri.get(&declaration.external_uri)
        {
            if content_resource_id != &declaration.resource_id {
                diagnostics.push(resource_id_mismatch_diagnostic(
                    declaration,
                    content_resource_id,
                ));
            }
        }
    }
    diagnostics.extend(verify_resource_checksums(&package));
    let graph = validate_ifcx_graph(&package.entrypoint.value);
    diagnostics.extend(graph.diagnostics);

    let mut validated_ifcdr_by_uri = BTreeMap::new();
    let mut attempted_ifcdr_uris = BTreeSet::new();
    for declaration in &package.declarations {
        if declaration.kind != ResourceKind::Ifcdr
            || !attempted_ifcdr_uris.insert(declaration.external_uri.as_str())
        {
            continue;
        }
        let Some(source) = package.resources.get(&declaration.external_uri) else {
            continue;
        };
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            declaration.external_uri.clone(),
            source.clone(),
        ));
        let (validated, mut resource_diagnostics) = outcome.into_parts();
        for diagnostic in &mut resource_diagnostics {
            diagnostic
                .resource_id
                .get_or_insert_with(|| declaration.resource_id.clone());
        }
        diagnostics.extend(resource_diagnostics);
        if let Some(validated) = validated {
            validated_ifcdr_by_uri.insert(declaration.external_uri.clone(), Arc::new(validated));
        }
    }

    let mut validated_ifcdr_resources = BTreeMap::new();
    for declaration in package
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == ResourceKind::Ifcdr)
    {
        let Some(validated) = validated_ifcdr_by_uri.get(&declaration.external_uri) else {
            continue;
        };
        let content_resource_id = validated.header().resource_id();
        if content_resource_id != &declaration.resource_id {
            diagnostics.push(resource_id_mismatch_diagnostic(
                declaration,
                content_resource_id,
            ));
            continue;
        }
        validated_ifcdr_resources
            .entry(declaration.resource_id.clone())
            .or_insert_with(|| validated.clone());
    }

    diagnostics.extend(validate_ifcpr_drawing_resource_ids(
        &package,
        &ifcpr_resource_ids_by_uri,
        &validated_ifcdr_resources,
    ));

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

fn validate_ifcpr_drawing_resource_ids(
    package: &LoadedIfccadPackage,
    ifcpr_resource_ids_by_uri: &BTreeMap<String, ResourceId>,
    validated_ifcdr_resources: &BTreeMap<ResourceId, Arc<crate::ifcdr::ValidatedIfcdrResource>>,
) -> Vec<PackageDiagnostic> {
    let mut diagnostics = Vec::new();
    for (external_uri, ifcpr_resource_id) in ifcpr_resource_ids_by_uri {
        let Some(resource) = package.resources.get(external_uri) else {
            continue;
        };
        let Some(links) = resource
            .value()
            .get("linkedDrawingResources")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for (index, value) in links.iter().enumerate() {
            let Some(target_resource_id) =
                value.as_str().and_then(|value| ResourceId::new(value).ok())
            else {
                continue;
            };
            if validated_ifcdr_resources.contains_key(&target_resource_id)
                || ifcx_descriptor_reports_missing_target(
                    package,
                    external_uri,
                    &target_resource_id,
                )
            {
                continue;
            }
            diagnostics.push(PackageDiagnostic {
                code: IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(ifcpr_resource_id.clone()),
                resource_uri: Some(external_uri.clone()),
                location: Some(format!("/linkedDrawingResources/{index}")),
                context: BTreeMap::from([(
                    "resourceId".to_owned(),
                    PackageDiagnosticContextValue::String(target_resource_id.to_string()),
                )]),
                message: "IFCPR link does not identify a validated IFCDR resource".to_owned(),
            });
        }
    }
    diagnostics
}

fn ifcx_descriptor_reports_missing_target(
    package: &LoadedIfccadPackage,
    ifcpr_external_uri: &str,
    target_resource_id: &ResourceId,
) -> bool {
    package
        .entrypoint
        .value()
        .get("data")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|node| node.pointer("/attributes/preservation"))
        .filter(|descriptor| {
            descriptor.get("uri").and_then(serde_json::Value::as_str) == Some(ifcpr_external_uri)
        })
        .filter_map(|descriptor| {
            descriptor
                .get("linkedDrawingResourceIds")
                .and_then(serde_json::Value::as_array)
        })
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .any(|value| value == target_resource_id.as_str())
}

fn validate_declared_resource_id_uniqueness(
    declarations: &[super::discovery::ResourceDeclaration],
) -> Vec<PackageDiagnostic> {
    let mut source_order = declarations.iter().collect::<Vec<_>>();
    source_order.sort_by_key(|declaration| declaration_source_index(declaration));
    let mut first_by_id = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for declaration in source_order {
        let identity = (declaration.kind, declaration.external_uri.as_str());
        match first_by_id.get(&declaration.resource_id) {
            None => {
                first_by_id.insert(declaration.resource_id.clone(), identity);
            }
            Some(first) if first == &identity => {}
            Some((first_kind, first_uri)) => diagnostics.push(PackageDiagnostic {
                code: IFCCAD_PACKAGE_RESOURCE_ID_DUPLICATE.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(declaration.resource_id.clone()),
                resource_uri: Some(declaration.external_uri.clone()),
                location: Some(declaration.resource_id_location.clone()),
                context: BTreeMap::from([
                    (
                        "resourceId".to_owned(),
                        PackageDiagnosticContextValue::String(declaration.resource_id.to_string()),
                    ),
                    (
                        "firstKind".to_owned(),
                        PackageDiagnosticContextValue::String(
                            resource_kind_name(*first_kind).to_owned(),
                        ),
                    ),
                    (
                        "firstExternalUri".to_owned(),
                        PackageDiagnosticContextValue::String((*first_uri).to_owned()),
                    ),
                    (
                        "actualKind".to_owned(),
                        PackageDiagnosticContextValue::String(
                            resource_kind_name(declaration.kind).to_owned(),
                        ),
                    ),
                    (
                        "actualExternalUri".to_owned(),
                        PackageDiagnosticContextValue::String(declaration.external_uri.clone()),
                    ),
                ]),
                message: "one package resource ID cannot identify different resources".to_owned(),
            }),
        }
    }
    diagnostics
}

fn resource_id_mismatch_diagnostic(
    declaration: &super::discovery::ResourceDeclaration,
    content_resource_id: &ResourceId,
) -> PackageDiagnostic {
    PackageDiagnostic {
        code: IFCCAD_PACKAGE_RESOURCE_ID_MISMATCH.to_owned(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: Some(declaration.resource_id.clone()),
        resource_uri: Some(declaration.external_uri.clone()),
        location: Some(declaration.resource_id_location.clone()),
        context: BTreeMap::from([
            (
                "declaredResourceId".to_owned(),
                PackageDiagnosticContextValue::String(declaration.resource_id.to_string()),
            ),
            (
                "contentResourceId".to_owned(),
                PackageDiagnosticContextValue::String(content_resource_id.to_string()),
            ),
        ]),
        message: "IFCX resource ID does not match the referenced resource header".to_owned(),
    }
}

fn declaration_source_index(declaration: &super::discovery::ResourceDeclaration) -> usize {
    declaration
        .resource_id_location
        .strip_prefix("/data/")
        .and_then(|suffix| suffix.split('/').next())
        .and_then(|index| index.parse().ok())
        .expect("resource declarations retain their IFCX data location")
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Ifcdr => "ifcdr",
        ResourceKind::Ifcpr => "ifcpr",
    }
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
            let resource = package.resources.get(&declaration.external_uri)?;
            let actual = sha256_checksum(resource.bytes());
            if actual == expected {
                return None;
            }
            Some(PackageDiagnostic {
                code: IFCCAD_PACKAGE_CHECKSUM_MISMATCH.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(declaration.resource_id.clone()),
                resource_uri: Some(declaration.external_uri.clone()),
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
                    declaration.external_uri
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

    fn resource_id(value: &str) -> ResourceId {
        ResourceId::new(value).expect("test resource ID")
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
                        "resourceId": "geometry-modelspace-main",
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

    fn next_package(category: &str, name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("conformance")
            .join("next")
            .join("packages")
            .join(category)
            .join(name)
    }

    fn copy_next_preservation_package(root: &Path) -> serde_json::Value {
        let source = next_package("valid", "unrepresented-packed");
        for relative in [
            DIRECTORY_PACKAGE_ENTRYPOINT,
            "drawing.ifcdr.json",
            "preservation.ifcpr.json",
        ] {
            fs::write(
                root.join(relative),
                fs::read(source.join(relative)).expect("read next package resource"),
            )
            .expect("copy next package resource");
        }
        serde_json::from_slice(
            &fs::read(root.join(DIRECTORY_PACKAGE_ENTRYPOINT)).expect("read copied IFCX"),
        )
        .expect("parse copied IFCX")
    }

    #[test]
    fn external_uri_can_differ_from_resource_identity() {
        let outcome =
            load_directory_package(next_package("valid", "resource-id-distinct-from-uri"))
                .expect("load resource identity package");

        assert!(outcome.report().is_empty());
        assert!(outcome.validated_package().is_some());
    }

    #[test]
    fn descriptor_and_ifcdr_header_identity_must_match() {
        let outcome = load_directory_package(next_package("invalid", "resource-id-mismatch"))
            .expect("load mismatched identity package");
        let diagnostic = outcome
            .report()
            .iter()
            .find(|item| item.code == "IFCCAD_PACKAGE_RESOURCE_ID_MISMATCH")
            .expect("resource ID mismatch diagnostic");

        assert_eq!(
            diagnostic.context.get("declaredResourceId"),
            Some(&PackageDiagnosticContextValue::String(
                "geometry-declared".to_owned()
            ))
        );
        assert_eq!(
            diagnostic.context.get("contentResourceId"),
            Some(&PackageDiagnosticContextValue::String(
                "geometry-modelspace-main".to_owned()
            ))
        );
    }

    #[test]
    fn resource_identity_is_unique_across_kinds() {
        let outcome = load_directory_package(next_package("invalid", "resource-id-duplicate"))
            .expect("load duplicate identity package");

        assert!(outcome
            .report()
            .iter()
            .any(|item| item.code == "IFCCAD_PACKAGE_RESOURCE_ID_DUPLICATE"));
        assert!(outcome.validated_package().is_none());
    }

    #[test]
    fn preservation_links_resolve_drawing_resource_ids() {
        let outcome = load_directory_package(next_package("valid", "unrepresented-packed"))
            .expect("load preservation package");

        assert!(outcome.report().is_empty());
        assert!(outcome.validated_package().is_some());
    }

    #[test]
    fn missing_preservation_link_reports_unknown_resource_id() {
        let outcome = load_directory_package(next_package(
            "invalid",
            "linked-drawing-resource-id-missing",
        ))
        .expect("load missing preservation link package");

        assert_eq!(
            outcome
                .report()
                .iter()
                .filter(|item| item.code == "IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING")
                .count(),
            1
        );
    }

    #[test]
    fn descriptor_and_ifcpr_header_identity_must_match() {
        let root = TestDirectory::new("ifcpr-resource-id-mismatch");
        let mut entrypoint = copy_next_preservation_package(root.path());
        entrypoint["data"][8]["attributes"]["preservation"]["resourceId"] =
            serde_json::json!("preservation-declared");
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load mismatched IFCPR package");
        assert!(outcome.report().iter().any(|item| {
            item.code == "IFCCAD_PACKAGE_RESOURCE_ID_MISMATCH"
                && item.context.get("contentResourceId")
                    == Some(&PackageDiagnosticContextValue::String(
                        "preservation-golden-source".to_owned(),
                    ))
        }));
    }

    #[test]
    fn multiple_preservation_resources_with_distinct_ids_are_allowed() {
        let root = TestDirectory::new("multiple-ifcpr-resources");
        let mut entrypoint = copy_next_preservation_package(root.path());
        let mut second_resource: serde_json::Value = serde_json::from_slice(
            &fs::read(root.path().join("preservation.ifcpr.json")).expect("read first IFCPR"),
        )
        .expect("parse first IFCPR");
        second_resource["header"]["resourceId"] = serde_json::json!("preservation-second");
        let second_bytes = serde_json::to_vec(&second_resource).expect("serialize second IFCPR");
        fs::write(
            root.path().join("preservation-second.ifcpr.json"),
            &second_bytes,
        )
        .expect("write second IFCPR");

        let mut second_node = entrypoint["data"][8].clone();
        second_node["path"] = serde_json::json!("preservation-second");
        second_node["attributes"]["preservation"]["resourceId"] =
            serde_json::json!("preservation-second");
        second_node["attributes"]["preservation"]["uri"] =
            serde_json::json!("preservation-second.ifcpr.json");
        second_node["attributes"]["preservation"]["checksum"] =
            serde_json::json!(format!("sha256:{:x}", Sha256::digest(&second_bytes)));
        entrypoint["data"].as_array_mut().unwrap().push(second_node);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load two IFCPR resources");
        assert!(
            outcome.report().is_empty(),
            "{:?}",
            outcome.report().diagnostics()
        );
        assert!(outcome.validated_package().is_some());
    }

    #[test]
    fn ifcpr_links_must_resolve_validated_drawing_resource_ids() {
        let root = TestDirectory::new("ifcpr-missing-drawing-resource-id");
        let mut entrypoint = copy_next_preservation_package(root.path());
        let ifcpr_path = root.path().join("preservation.ifcpr.json");
        let mut ifcpr: serde_json::Value =
            serde_json::from_slice(&fs::read(&ifcpr_path).expect("read copied IFCPR"))
                .expect("parse copied IFCPR");
        ifcpr["linkedDrawingResources"] = serde_json::json!(["geometry-content-missing"]);
        let bytes = serde_json::to_vec(&ifcpr).expect("serialize changed IFCPR");
        fs::write(&ifcpr_path, &bytes).expect("write changed IFCPR");
        entrypoint["data"][8]["attributes"]["preservation"]["checksum"] =
            serde_json::json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load IFCPR link package");
        assert!(outcome.report().iter().any(|item| {
            item.code == "IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING"
                && item.resource_id.as_ref().map(ResourceId::as_str)
                    == Some("preservation-golden-source")
                && item.location.as_deref() == Some("/linkedDrawingResources/0")
        }));
        assert!(outcome.validated_package().is_none());
    }

    #[test]
    fn load_outcome_retains_entrypoint_resources_and_exact_bytes() {
        let root = TestDirectory::new("loaded-model");
        let entrypoint = br#"{"data":[{"path":"geometry","type":"openaec:DrawingGeometryRepresentation","attributes":{"geometry":{"format":"openaec.ifcdr","version":"0.5.0","resourceId":"geometry-main","uri":"drawing.ifcdr.json","checksum":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","role":"modelspace"}}}]}"#;
        let drawing = b"{\r\n  \"header\": {}\r\n}\r\n";
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), entrypoint)
            .expect("write entrypoint");
        fs::write(root.path().join("drawing.ifcdr.json"), drawing).expect("write drawing resource");

        let outcome = load_directory_package(root.path()).expect("load directory package");
        let package = outcome.package.expect("entrypoint produced package");

        assert_eq!(package.entrypoint.bytes, entrypoint);
        assert_eq!(package.entrypoint.value["data"][0]["path"], "geometry");
        assert_eq!(package.declarations.len(), 1);
        assert_eq!(package.declarations[0].external_uri, "drawing.ifcdr.json");
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
            .validated_ifcdr_resources[&resource_id("geometry-modelspace-main")];

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
        let ifcdr = &analysis.validated_ifcdr_resources[&resource_id("geometry-modelspace-main")];

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
                    "version": "0.2.0",
                    "resourceId": "preservation-conflict",
                    "uri": geometry["uri"],
                    "checksum": geometry["checksum"],
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceIds": ["geometry-modelspace-main"]
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
                    "version": "0.2.0",
                    "resourceId": "preservation-conflict",
                    "uri": geometry["uri"],
                    "checksum": geometry["checksum"],
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceIds": ["geometry-modelspace-main"]
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
    fn preservation_link_requires_a_validated_ifcdr_resource_id() {
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
                    "version": "0.2.0",
                    "resourceId": "preservation-golden-source",
                    "uri": "preservation.ifcpr.json",
                    "checksum": format!("sha256:{:x}", Sha256::digest(&preservation)),
                    "sourceDocumentId": "source",
                    "linkedDrawingResourceIds": ["geometry-undeclared"]
                }}
            }));
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load preservation package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_PACKAGE_TARGET_RESOURCE_MISSING"
                && diagnostic.location.as_deref()
                    == Some("/data/8/attributes/preservation/linkedDrawingResourceIds/0")
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
        let package = outcome.validated_package.as_ref().expect("strict package");
        let representation = package.drawings().next().expect("drawing").representation();
        let appearance = representation
            .appearance(AppearanceId::new(2))
            .expect("appearance binding");
        let super::super::AppearanceProperty::Explicit(color) = appearance.color() else {
            panic!("override remains explicit");
        };
        assert_eq!(color.rgb().components(), [0, 255, 0]);
        assert_eq!(
            appearance.opacity(),
            super::super::AppearanceProperty::Explicit(1.0),
            "an absent property override falls back to the IFCX definition"
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
        assert_eq!(
            layout.ifcdr_resource_id,
            resource_id("geometry-modelspace-main")
        );
        assert_eq!(layout.scope_id, ScopeId::new(0));
        assert_eq!(
            bindings.ifcx_layer_by_ifcdr_id
                [&(resource_id("geometry-modelspace-main"), LayerId::new(1))],
            "layer-a-wall"
        );
        assert_eq!(
            bindings.ifcx_appearance_by_ifcdr_id[&(
                resource_id("geometry-modelspace-main"),
                AppearanceId::new(2)
            )],
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
            .contains_key(&resource_id("x")));
        assert!(outcome.report.iter().any(|item| {
            item.code == "IFCCAD_IFCDR_VERSION_UNSUPPORTED"
                && item.resource_uri.as_deref() == Some("drawing.ifcdr.json")
        }));
    }

    #[test]
    fn unsupported_ifcdr_unit_blocks_the_strict_package() {
        let root = TestDirectory::new("unsupported-ifcdr-unit");
        let mut entrypoint = copy_minimal_package(root.path());
        let mut ifcdr = read_ifcdr(root.path());
        ifcdr["header"]["unit"] = serde_json::json!("parsec");
        write_ifcdr_and_update_checksum(root.path(), &mut entrypoint, &ifcdr);
        write_entrypoint(root.path(), &entrypoint);

        let outcome = load_directory_package(root.path()).expect("load package");

        assert!(outcome.validated_package.is_none());
        assert!(outcome.report.iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_UNIT_UNSUPPORTED"
                && diagnostic.resource_uri.as_deref() == Some("drawing.ifcdr.json")
                && diagnostic.location.as_deref() == Some("/header/unit")
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
        let descriptor = |resource_id: &str, uri: &str, bytes: &[u8]| {
            serde_json::json!({
                "format": "openaec.ifcdr",
                "version": "0.5.0",
                "resourceId": resource_id,
                "uri": uri,
                "checksum": format!("sha256:{:x}", Sha256::digest(bytes)),
                "role": "modelspace"
            })
        };
        let entrypoint = serde_json::json!({"data": [
            {"path": "valid", "type": "openaec:DrawingGeometryRepresentation", "attributes": {"geometry": descriptor("geometry-modelspace-main", "valid.ifcdr.json", &valid)}},
            {"path": "invalid", "type": "openaec:DrawingGeometryRepresentation", "attributes": {"geometry": descriptor("x", "invalid.ifcdr.json", invalid)}}
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
            .contains_key(&resource_id("geometry-modelspace-main")));
        assert!(!outcome
            .analysis
            .as_ref()
            .unwrap()
            .validated_ifcdr_resources
            .contains_key(&resource_id("x")));
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
            let ifcdr_resource_ids = package
                .declarations
                .iter()
                .filter(|declaration| declaration.kind == ResourceKind::Ifcdr)
                .filter(|declaration| package.resources.contains_key(&declaration.external_uri))
                .map(|declaration| declaration.resource_id.as_str())
                .collect::<BTreeSet<_>>();
            let validated_uris = outcome
                .analysis
                .as_ref()
                .expect("package analysis")
                .validated_ifcdr_resources
                .keys()
                .map(ResourceId::as_str)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                validated_uris,
                ifcdr_resource_ids,
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

        assert_eq!(package.declarations[0].external_uri, "missing.ifcdr.json");
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
                            "resourceId": "geometry-missing",
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
                            "resourceId": "geometry-modelspace-main",
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
                (
                    IFCCAD_PACKAGE_CHECKSUM_MISMATCH,
                    Some("a-loaded.ifcdr.json"),
                    Some("/data/2/attributes/geometry/checksum"),
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
                  "attributes": {
                    "geometry": {
                      "resourceId": "geometry-ignored",
                      "url": "ignored.json"
                    }
                  }
                },
                {
                  "type": "openaec:PreservationRepresentation",
                  "attributes": {
                    "preservation": {
                      "resourceId": "preservation-malformed",
                      "uri": "malformed.ifcpr.json"
                    }
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
    fn loads_committed_active_packages_without_diagnostics() {
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

            assert!(
                outcome.report.is_empty(),
                "case {case}: {:?}",
                outcome.report.diagnostics()
            );
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
            .any(|declaration| declaration.external_uri == "drawing.ifcdr.json"));
        assert!(outcome
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_RESOURCE_MISSING));
    }
}
