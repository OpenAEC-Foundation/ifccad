use crate::ocdraw::{PointDisplay, PointGlyph, PointSize, UcsDefinition};
use serde_json::{json, Value};

pub(crate) fn encode_point_display(display: PointDisplay) -> Value {
    let glyph = match display.glyph {
        PointGlyph::Dot => "dot",
        PointGlyph::Hidden => "hidden",
        PointGlyph::Plus => "plus",
        PointGlyph::Cross => "cross",
        PointGlyph::ShortLine => "shortLine",
    };
    let size = match display.size {
        PointSize::DefaultFivePercent => json!({"kind":"defaultFivePercent"}),
        PointSize::Absolute(value) => json!({"kind":"absolute","value":value}),
        PointSize::ViewportPercent(value) => json!({"kind":"viewportPercent","value":value}),
    };
    json!({"form":{"glyph":glyph,"circle":display.circle,"square":display.square},"size":size})
}

pub(crate) fn encode_ucs_definition(definition: &UcsDefinition, id: u32) -> Value {
    let frame = definition.frame;
    let origin = frame.origin();
    let x = frame.x_axis();
    let y = frame.y_axis();
    json!({
        "ucsId": id,
        "name": definition.name,
        "frame": {
            "origin": {"x": origin.x(), "y": origin.y(), "z": origin.z()},
            "X": {"x": x.x(), "y": x.y(), "z": x.z()},
            "Y": {"x": y.x(), "y": y.y(), "z": y.z()}
        },
        "elevation": definition.elevation
    })
}
