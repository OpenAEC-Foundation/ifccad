use ocdraw::drawing::{load_drawing_file, DrawingLoadStatus};
use std::path::Path;

#[test]
fn candidate_drawing_cases_use_the_production_reader() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ocdraw");
    let valid = load_drawing_file(root.join("valid/empty.ocdraw.json")).unwrap();
    assert_eq!(
        valid.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        valid.diagnostics()
    );
    for (name, code) in [
        ("unknown-core-field", "SCHEMA"),
        ("duplicate-layer-id", "DUPLICATE_ID"),
        ("model-layout-paper-scope", "LAYOUT_SCOPE"),
        ("reversed-plot-area", "PLOT_RECT"),
        ("missing-model-window", "MODEL_WINDOW_REF"),
        ("degenerate-ucs-frame", "UCS_FRAME"),
    ] {
        let case = load_drawing_file(root.join(format!("invalid/{name}.ocdraw.json"))).unwrap();
        assert_eq!(case.status(), DrawingLoadStatus::Invalid, "{name}");
        assert!(
            case.diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{name}: {:?}",
            case.diagnostics()
        );
    }
}
