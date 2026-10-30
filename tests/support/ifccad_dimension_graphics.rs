#[path = "ifccad_dimension.rs"]
mod definition;
use ifccad::*;
pub fn dimension(d: &mut IfccadDocument) -> &mut IfccadDimension {
    let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!("Dimension")
    };
    v
}
pub fn part(kind: IfccadDimensionGraphicsKind) -> IfccadDimensionGraphicsPart {
    IfccadDimensionGraphicsPart {
        layer_id: None,
        kind,
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        visible: true,
        line_pattern_scale: 1.,
        purpose: IfccadDimensionGraphicsPurpose::Display,
    }
}
pub fn drawing() -> IfccadDocument {
    let mut d = definition::drawing();
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts: (0..12)
            .map(|i| {
                part(IfccadDimensionGraphicsKind::LineSegment {
                    start: [i as f64, -0., 0.],
                    end: [i as f64, 3., 0.],
                    thickness: 0.,
                    extrusion_direction: [0., 0., 1.],
                })
            })
            .collect(),
    });
    d
}
