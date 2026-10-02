mod common;
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;
use serde_json::{json, Value};

fn options(policy: IfcxCadLossPolicy) -> IfcxCadConversionOptions {
    IfcxCadConversionOptions {
        loss_policy: policy,
    }
}

fn assert_mappings(a: &IfcxCadMappings, b: &IfcxCadMappings) {
    for (a, b) in [
        (&a.entities, &b.entities),
        (&a.layers, &b.layers),
        (&a.layouts, &b.layouts),
        (&a.blocks, &b.blocks),
        (&a.line_patterns, &b.line_patterns),
    ] {
        assert_eq!(a.iter().collect::<Vec<_>>(), b.iter().collect::<Vec<_>>());
    }
}

#[test]
fn logical_and_encoded_imports_have_equivalent_documents_diagnostics_and_mappings() {
    let mut patterned = primitives();
    patterned.line_patterns.push(IfcxCadLinePattern {
        id: IfcxCadLinePatternId(2),
        name: "Dashed".into(),
        description: Some("Unused pattern".into()),
        pattern: vec![2., -1.],
    });
    patterned.id_counters.next_line_pattern_id = 3;
    let mut lossy = to_cad(&validated(&primitives())).unwrap().into_document();
    lossy.header.project_name = "Project metadata".into();
    let mut sources: Vec<_> = [empty(), primitives(), nested([0.; 3]), patterned]
        .iter()
        .map(|d| to_cad(&validated(d)).unwrap().into_document())
        .collect();
    sources.push(lossy);
    for source in sources {
        for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
            let logical = cad_document_to_ifcx_cad_document_with_options(
                &source,
                metadata(),
                options(policy),
            );
            let encoded =
                cad_document_to_ifcx_cad_with_options(&source, metadata(), options(policy));
            match (logical, encoded) {
                (Ok(logical), Ok(encoded)) => {
                    validate_ifcx_cad_document(logical.document()).unwrap();
                    assert_eq!(logical.document(), encoded.validated_ifcx().document());
                    assert_eq!(logical.diagnostics(), encoded.diagnostics());
                    assert_mappings(logical.mappings(), encoded.mappings());
                    let direct = ifcx_cad_document_to_cad_document_with_options(
                        logical.document(),
                        options(policy),
                    )
                    .unwrap();
                    let loaded = ifcx_cad_to_cad_document_with_options(
                        encoded.validated_ifcx(),
                        options(policy),
                    )
                    .unwrap();
                    assert_eq!(direct.diagnostics(), loaded.diagnostics());
                    // Compare emitted CAD semantics through the logical importer.
                    let a =
                        cad_document_to_ifcx_cad_document(direct.document(), metadata()).unwrap();
                    let b =
                        cad_document_to_ifcx_cad_document(loaded.document(), metadata()).unwrap();
                    assert_eq!(a.document(), b.document());
                    assert_eq!(
                        logical.into_document(),
                        encoded.validated_ifcx().document().clone()
                    );
                }
                (
                    Err(IfcxCadConversionError::Unsupported(a)),
                    Err(IfcxCadConversionError::Unsupported(b)),
                ) => assert_eq!(a, b),
                (a, b) => panic!(
                    "mismatched route results: logical error {:?}; encoded error {:?}",
                    a.err(),
                    b.err()
                ),
            }
        }
    }
}

#[test]
fn invalid_direct_document_never_reaches_cad_construction() {
    for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
        for change in 0..3 {
            let mut document = primitives();
            match change {
                0 => document.model.entities[0].layer_id = 900,
                1 => document.model.entities[0].kind = instance(0, 900, [0.; 3]).kind,
                _ => document
                    .model
                    .entities
                    .push(document.model.entities[0].clone()),
            }
            assert!(matches!(
                ifcx_cad_document_to_cad_document_with_options(&document, options(policy)),
                Err(IfcxCadConversionError::CoreValidation(_))
            ));
        }
        let mut bad_metadata = metadata();
        bad_metadata.header.id.clear();
        assert!(matches!(
            cad_document_to_ifcx_cad_document_with_options(&cad(), bad_metadata, options(policy)),
            Err(IfcxCadConversionError::CoreValidation(_))
        ));
    }
}

#[test]
fn source_aware_conversion_keeps_foreign_loss_and_exact_precision_checks() {
    let mut root: Value =
        serde_json::from_slice(&encode_ifcx_cad_document(&primitives()).unwrap()).unwrap();
    root["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"foreign","attributes":{"example::note":"retained"}}));
    let bytes = serde_json::to_vec(&root).unwrap();
    let source = read_native_cad_ifcx(&bytes).unwrap();
    let direct = ifcx_cad_document_to_cad_document(source.document()).unwrap();
    assert!(!direct
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx"));
    let allow = ifcx_cad_to_cad_document(&source).unwrap();
    assert!(allow.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    assert!(
        matches!(ifcx_cad_to_cad_document_with_options(&source, options(IfcxCadLossPolicy::Reject)), Err(IfcxCadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "foreign-ifcx"))
    );
    assert_eq!(source.graph().source_bytes(), bytes);
    root["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e90")
        .unwrap()["attributes"]["ifccad::geom::lineSegment"]["start"][0] =
        json!(9007199254740993_u64);
    let bytes = serde_json::to_vec(&root).unwrap();
    let source = read_native_cad_ifcx(&bytes).unwrap();
    for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
        assert!(
            matches!(ifcx_cad_to_cad_document_with_options(&source, options(policy)), Err(IfcxCadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "precision"))
        );
    }
    assert_eq!(source.graph().source_bytes(), bytes);
}
