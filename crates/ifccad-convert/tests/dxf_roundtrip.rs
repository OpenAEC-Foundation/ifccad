use ifccad::conformance::bundled_conformance_root;
use ifccad::package::load_directory_package;
use ifccad_convert::cadcodec::{
    CadDocument, Color, DxfReader, DxfWriter, EntityType, LineWeight, Transparency,
};
use ifccad_convert::convert_drawing;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq)]
struct EntityProjection {
    geometry: GeometryProjection,
    layer: String,
    invisible: bool,
    color: Color,
    color_name: Option<String>,
    linetype: String,
    line_weight: LineWeight,
    transparency: Transparency,
}

#[derive(Debug, PartialEq)]
enum GeometryProjection {
    Line { start: [f64; 3], end: [f64; 3] },
    LwPolyline { points: Vec<[f64; 2]>, closed: bool },
}

#[test]
fn dxf_roundtrip_preserves_supported_conversion_semantics() {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let inspected = load_directory_package(root).expect("load bundled package");
    let package = inspected
        .validated_package()
        .expect("strictly valid package");
    let drawing = package.drawings().next().expect("one drawing");
    let outcome = convert_drawing(drawing).expect("convert drawing");
    let expected_entities = project_entities(outcome.document());

    let path = TestDxf::new("minimal-conversion");
    DxfWriter::new(outcome.document())
        .write_to_file(path.as_ref())
        .expect("write converted DXF");
    let roundtripped = DxfReader::from_file(path.as_ref())
        .expect("open converted DXF")
        .read()
        .expect("read converted DXF");

    assert_eq!(project_entities(&roundtripped), expected_entities);
    assert_eq!(roundtripped.header.insertion_units, 6);
    assert_eq!(roundtripped.header.measurement, 1);
    assert_eq!(roundtripped.layers.len(), 2);
    let layer_0 = roundtripped.layers.get("0").expect("layer 0");
    assert_eq!(layer_0.color, Color::Index(7));
    assert_eq!(layer_0.line_type, "Continuous");
    assert_eq!(layer_0.line_weight, LineWeight::W0_25);
    assert!(layer_0.is_visible());
    let walls = roundtripped.layers.get("A-WALL").expect("A-WALL");
    assert_eq!(walls.color, Color::from_rgb(255, 0, 0));
    assert_eq!(walls.color_name.as_deref(), Some("Traffic red"));
    assert_eq!(walls.book_name.as_deref(), Some("RAL"));
    assert_eq!(walls.line_type, "Dashed");
    assert_eq!(walls.line_weight, LineWeight::W0_18);
    assert!(walls.is_visible());
    assert!(roundtripped.line_types.contains("Dashed"));
}

fn project_entities(document: &CadDocument) -> Vec<EntityProjection> {
    document
        .entities()
        .map(|entity| {
            let common = entity.common();
            let geometry = match entity {
                EntityType::Line(line) => GeometryProjection::Line {
                    start: [line.start.x, line.start.y, line.start.z],
                    end: [line.end.x, line.end.y, line.end.z],
                },
                EntityType::LwPolyline(polyline) => GeometryProjection::LwPolyline {
                    points: polyline
                        .vertices
                        .iter()
                        .map(|vertex| [vertex.location.x, vertex.location.y])
                        .collect(),
                    closed: polyline.is_closed,
                },
                _ => panic!("unexpected converted entity kind"),
            };
            EntityProjection {
                geometry,
                layer: common.layer.clone(),
                invisible: common.invisible,
                color: common.color,
                color_name: common.color_name.clone(),
                linetype: normalize_linetype(&common.linetype).to_owned(),
                line_weight: common.line_weight,
                transparency: common.transparency,
            }
        })
        .collect()
}

fn normalize_linetype(value: &str) -> &str {
    if value.is_empty() {
        "ByLayer"
    } else {
        value
    }
}

struct TestDxf(PathBuf);

impl TestDxf {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "ifccad-convert-{name}-{}-{nonce}.dxf",
            std::process::id()
        )))
    }
}

impl AsRef<Path> for TestDxf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDxf {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
