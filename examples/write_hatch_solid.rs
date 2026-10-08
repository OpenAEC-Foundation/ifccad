use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("provide a new output path")?;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("hatch-solid-example", "mm"))?;
    let p = b.add_line_pattern(LinePatternDefinition {
        name: "Continuous".into(),
        description: None,
        pattern: vec![],
    })?;
    let layer = b.add_layer(LayerDefinition::new("0", RgbColor::new(255, 255, 255), p))?;
    let source = b.add_circle(CircleDefinition::new(layer, [0.0, 0.0, 0.0], 1.0))?;
    let hatch = b.add_hatch(HatchEntityDefinition {
        scope_id: 0,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        area_rule: HatchAreaRule::Normal,
        join_tolerance: DEFAULT_HATCH_JOIN_TOLERANCE,
        fill: HatchFill::Solid,
        loops: vec![
            OcdrawHatchLoop {
                boundary: HatchBoundary2::Polyline {
                    vertices: vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]],
                    bulges: vec![0.0; 4],
                },
                source_entity_id: None,
            },
            OcdrawHatchLoop {
                boundary: HatchBoundary2::Circle {
                    center: [0.0, 0.0],
                    radius: 1.0,
                },
                source_entity_id: Some(source),
            },
        ],
    })?;
    let mut doc = b.build_document()?;
    doc.scopes[0].entities = vec![hatch, source];
    let bytes = encode_ocdraw_document(&doc)?;
    load_ocdraw_bytes(bytes.bytes())?;
    bytes.write_file(path)?;
    Ok(())
}
