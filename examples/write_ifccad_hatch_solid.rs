use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ifccad::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("provide a new output path")?;
    let mut d = load_ifccad_bytes(include_bytes!("ifccad/hello-cad.ifcx"), Default::default())?
        .into_document();
    let mut template = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .ok_or("native fixture entity required")?
        .clone();
    d.paper_layouts.clear();
    d.blocks.clear();
    d.id_counters.next_layout_id = d.model.id + 1;
    d.id_counters.next_block_id = 1;
    d.id_counters.next_entity_id = 3;
    d.model.bounds = None;
    let plane = IfccadPlacement {
        origin: [0.0, 0.0, 0.0],
        x_axis: [1.0, 0.0, 0.0],
        y_axis: [0.0, 1.0, 0.0],
    };
    let source = 2;
    template.id = source;
    template.kind = IfccadEntityKind::Circle {
        radius: 1.0,
        placement: plane.clone(),
    };
    let source_entity = template.clone();
    let hatch = 1;
    template.id = hatch;
    template.kind = IfccadEntityKind::Hatch(IfccadHatch {
        placement: plane,
        area_rule: HatchAreaRule::Normal,
        join_tolerance: 1e-9,
        fill: HatchFill::Solid,
        loops: vec![
            IfccadHatchLoop {
                boundary: HatchBoundary2::Polyline {
                    vertices: vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]],
                    bulges: vec![0.0; 4],
                },
                source_entity_id: None,
            },
            IfccadHatchLoop {
                boundary: HatchBoundary2::Circle {
                    center: [0.0, 0.0],
                    radius: 1.0,
                },
                source_entity_id: Some(source),
            },
        ],
    });
    d.model.entities = vec![
        IfccadEntity::Native(template),
        IfccadEntity::Native(source_entity),
    ];
    recompute_ifccad_document_bounds(&mut d)?;
    let output = encode_ifccad_document(&d)?;
    load_ifccad_bytes(output.bytes(), Default::default())?;
    output.write_file(path)?;
    Ok(())
}
