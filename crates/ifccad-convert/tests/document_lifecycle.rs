mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn options(policy: IfccadLossPolicy) -> CadToIfccadOptions {
    CadToIfccadOptions {
        loss_policy: policy,
    }
}

fn assert_mappings(a: &IfccadMappings, b: &IfccadMappings) {
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
    patterned.line_patterns.push(IfccadLinePattern {
        id: IfccadLinePatternId(2),
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
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let logical = cad_document_to_ifccad_document(&source, metadata(), options(policy));
            let encoded = cad_document_to_encoded_ifccad(&source, metadata(), options(policy));
            match (logical, encoded) {
                (Ok(logical), Ok(encoded)) => {
                    validate_ifccad_document(logical.document()).unwrap();
                    assert_eq!(logical.document(), encoded.validated_source().document());
                    assert_eq!(logical.diagnostics(), encoded.diagnostics());
                    assert_mappings(logical.mappings(), encoded.mappings());
                    let direct = ifccad_document_to_cad_document(
                        logical.document(),
                        IfccadToCadOptions {
                            loss_policy: (options(policy)).loss_policy,
                        },
                    )
                    .unwrap();
                    let loaded = ifccad_source_to_cad_document(
                        encoded.validated_source(),
                        IfccadToCadOptions {
                            loss_policy: (options(policy)).loss_policy,
                        },
                    )
                    .unwrap();
                    assert_eq!(direct.diagnostics(), loaded.diagnostics());
                    // Compare emitted CAD semantics through the logical importer.
                    let a = cad_document_to_ifccad_document(
                        direct.document(),
                        metadata(),
                        Default::default(),
                    )
                    .unwrap();
                    let b = cad_document_to_ifccad_document(
                        loaded.document(),
                        metadata(),
                        Default::default(),
                    )
                    .unwrap();
                    assert_eq!(a.document(), b.document());
                    assert_eq!(
                        logical.into_document(),
                        encoded.validated_source().document().clone()
                    );
                }
                (
                    Err(IfccadConversionError::Unsupported(a)),
                    Err(IfccadConversionError::Unsupported(b)),
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
    use std::error::Error;

    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
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
            let expected = validate_ifccad_document(&document).unwrap_err();
            let error = ifccad_document_to_cad_document(
                &document,
                IfccadToCadOptions {
                    loss_policy: policy,
                },
            )
            .err()
            .expect("invalid document");
            let IfccadConversionError::CoreValidation(report) = &error else {
                panic!("expected logical validation phase");
            };
            assert_eq!(report, &expected);
            assert_eq!(
                error.source().unwrap().downcast_ref::<IfccadReport>(),
                Some(&expected)
            );
        }
        let mut bad_metadata = metadata();
        bad_metadata.header.id.clear();
        for error in [
            cad_document_to_ifccad_document(&cad(), bad_metadata.clone(), options(policy))
                .err()
                .expect("invalid metadata"),
            cad_document_to_encoded_ifccad(&cad(), bad_metadata.clone(), options(policy))
                .err()
                .expect("invalid metadata"),
        ] {
            let IfccadConversionError::CoreValidation(report) = &error else {
                panic!("metadata validation precedes encoding");
            };
            assert_eq!(report.errors, ["incomplete IFCX header"]);
            assert_eq!(
                error.source().unwrap().downcast_ref::<IfccadReport>(),
                Some(report)
            );
        }
    }
}

#[test]
fn conversion_wrappers_retain_phase_and_source() {
    use std::error::Error;

    let report = IfccadReport {
        errors: vec!["first failure".into(), "second failure".into()],
    };
    let validation = IfccadConversionError::CoreValidation(report.clone());
    assert_eq!(
        validation.source().unwrap().downcast_ref::<IfccadReport>(),
        Some(&report)
    );
    let readback = IfccadConversionError::CoreReadback(IfccadReadError::from(report.clone()));
    let source = readback
        .source()
        .unwrap()
        .downcast_ref::<IfccadReadError>()
        .unwrap();
    assert_eq!(source.report(), &report);
    assert_eq!(
        source.source().unwrap().downcast_ref::<IfccadReport>(),
        Some(&report)
    );
    assert!(matches!(
        validation,
        IfccadConversionError::CoreValidation(_)
    ));
    assert!(matches!(readback, IfccadConversionError::CoreReadback(_)));

    let encoding = IfccadConversionError::CoreEncoding(IfccadEncodeError::Readback(
        IfccadReadError::from(report.clone()),
    ));
    let source = encoding
        .source()
        .unwrap()
        .downcast_ref::<IfccadEncodeError>()
        .unwrap();
    assert!(matches!(source, IfccadEncodeError::Readback(_)));
    assert_eq!(source.report(), Some(&report));
    let reader = source
        .source()
        .unwrap()
        .downcast_ref::<IfccadReadError>()
        .unwrap();
    assert_eq!(
        reader.source().unwrap().downcast_ref::<IfccadReport>(),
        Some(&report)
    );
}

#[test]
fn source_aware_conversion_keeps_foreign_loss_and_exact_precision_checks() {
    let mut root: Value =
        serde_json::from_slice(encode_ifccad_document(&primitives()).unwrap().bytes()).unwrap();
    root["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"foreign","attributes":{"example::note":"retained"}}));
    let bytes = serde_json::to_vec(&root).unwrap();
    let source = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let direct = ifccad_document_to_cad_document(source.document(), Default::default()).unwrap();
    assert!(!direct
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx"));
    let allow = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    assert!(allow.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    assert!(
        matches!(ifccad_source_to_cad_document(&source, IfccadToCadOptions { loss_policy: (options(IfccadLossPolicy::Reject)).loss_policy }), Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "foreign-ifcx"))
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
    let source = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(
            matches!(ifccad_source_to_cad_document(&source, IfccadToCadOptions { loss_policy: (options(policy)).loss_policy }), Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "precision"))
        );
    }
    assert_eq!(source.graph().source_bytes(), bytes);
}
