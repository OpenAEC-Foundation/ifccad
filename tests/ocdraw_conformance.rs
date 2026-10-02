use ocdraw::ocdraw::{load_ocdraw_bytes, OcdrawReadStatus};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};
#[test]
fn candidate_drawing_cases_use_the_production_reader() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ocdraw");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    assert_eq!(manifest["ocdrawVersion"], "0.1.0");
    let mut declared = BTreeSet::new();
    for case in manifest["cases"].as_array().unwrap() {
        let path = case["path"].as_str().unwrap();
        assert!(declared.insert(path.to_owned()));
        let read = load_ocdraw_bytes(&std::fs::read(root.join(path)).unwrap());
        let expected = if case["status"] == "valid" {
            OcdrawReadStatus::Valid
        } else {
            OcdrawReadStatus::Invalid
        };
        assert_eq!(
            read.as_ref()
                .map(|_| OcdrawReadStatus::Valid)
                .unwrap_or_else(|e| e.status()),
            expected,
            "{path}: {:?}",
            read.as_ref()
                .err()
                .map(|e| e.diagnostics())
                .unwrap_or_default()
        );
        if let Some(code) = case["diagnostic"].as_str() {
            assert!(
                read.as_ref()
                    .err()
                    .map(|e| e.diagnostics())
                    .unwrap_or_default()
                    .iter()
                    .any(|d| d.code == code),
                "{path}: {:?}",
                read.as_ref()
                    .err()
                    .map(|e| e.diagnostics())
                    .unwrap_or_default()
            );
        }
    }
    let actual = ["valid", "invalid"]
        .into_iter()
        .flat_map(|dir| {
            std::fs::read_dir(root.join(dir))
                .unwrap()
                .map(move |entry| format!("{dir}/{}", entry.unwrap().file_name().to_str().unwrap()))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, declared,
        "every candidate fixture requires an expected outcome"
    );
}
