use ifccad::conformance::bundled_conformance_root;
use ifccad::package::load_directory_package;
use ifccad_convert::cadcodec::{Color, DxfVersion, LineWeight};
use ifccad_convert::{convert_drawing, ConversionDiagnostic};

#[test]
fn converts_minimal_drawing_structure_units_and_layers() {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let inspected = load_directory_package(root).expect("load bundled package");
    let package = inspected
        .validated_package()
        .expect("strictly valid package");
    let drawing = package.drawings().next().expect("one drawing");

    let outcome = convert_drawing(drawing).expect("convert drawing layers");

    assert_eq!(outcome.document().version, DxfVersion::AC1032);
    assert_eq!(outcome.document().header.insertion_units, 6);
    assert_eq!(outcome.document().header.measurement, 1);

    let layer_0 = outcome.document().layers.get("0").expect("layer 0");
    assert_eq!(layer_0.color, Color::Index(7));
    assert_eq!(layer_0.line_type, "Continuous");
    assert_eq!(layer_0.line_weight, LineWeight::W0_25);
    assert!(layer_0.is_visible());

    let walls = outcome
        .document()
        .layers
        .get("A-WALL")
        .expect("A-WALL layer");
    assert_eq!(walls.color, Color::from_rgb(255, 0, 0));
    assert_eq!(walls.line_type, "Dashed");
    assert_eq!(walls.line_weight, LineWeight::W0_18);
    assert!(walls.is_visible());
    assert!(outcome.document().line_types.contains("Dashed"));
    assert!(outcome.diagnostics().iter().any(|item| matches!(
        item,
        ConversionDiagnostic::NamedLayerColorIdentityLost {
            layer,
            catalog,
            name,
            count: 1,
        } if layer == "A-WALL" && catalog == "RAL" && name == "Traffic red"
    )));
}
