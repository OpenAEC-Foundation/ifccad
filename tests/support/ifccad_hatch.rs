use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ifccad::*;
pub const SOURCE: u64 = 9007199254740993;
pub const HATCH: u64 = SOURCE + 1;
pub fn plane() -> IfccadPlacement {
    IfccadPlacement {
        origin: [0.0, 0.0, 0.0],
        x_axis: [1.0, 0.0, 0.0],
        y_axis: [0.0, 1.0, 0.0],
    }
}
pub fn drawing() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut template = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .unwrap()
        .clone();
    d.paper_layouts.clear();
    d.blocks.clear();
    d.model.bounds = None;
    template.id = SOURCE;
    template.kind = IfccadEntityKind::Circle {
        radius: 1.0,
        placement: plane(),
    };
    let source = template.clone();
    template.id = HATCH;
    template.kind = IfccadEntityKind::Hatch(IfccadHatch {
        placement: plane(),
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
                source_entity_id: Some(SOURCE),
            },
        ],
    });
    d.model.entities = vec![IfccadEntity::Native(template), IfccadEntity::Native(source)];
    d.id_counters.next_entity_id = HATCH + 1;
    validate_ifccad_document(&d).unwrap();
    d
}
