use ocdraw::ifccad::{IfccadPointDisplay, IfccadPointGlyph, IfccadPointSize};

pub(crate) fn from_cad(mode: i16, size: f64) -> Option<IfccadPointDisplay> {
    if mode < 0 || mode & !0x67 != 0 || mode & 7 > 4 || !size.is_finite() {
        return None;
    }
    let glyph = match mode & 7 {
        0 => IfccadPointGlyph::Dot,
        1 => IfccadPointGlyph::Hidden,
        2 => IfccadPointGlyph::Plus,
        3 => IfccadPointGlyph::Cross,
        4 => IfccadPointGlyph::ShortLine,
        _ => return None,
    };
    let size = if size == 0.0 {
        IfccadPointSize::DefaultFivePercent
    } else if size > 0.0 {
        IfccadPointSize::Absolute(size)
    } else {
        IfccadPointSize::ViewportPercent(-size)
    };
    Some(IfccadPointDisplay {
        glyph,
        circle: mode & 32 != 0,
        square: mode & 64 != 0,
        size,
    })
}
pub(crate) fn to_cad(display: IfccadPointDisplay) -> (i16, f64) {
    let glyph = match display.glyph {
        IfccadPointGlyph::Dot => 0,
        IfccadPointGlyph::Hidden => 1,
        IfccadPointGlyph::Plus => 2,
        IfccadPointGlyph::Cross => 3,
        IfccadPointGlyph::ShortLine => 4,
    };
    let mode = glyph + if display.circle { 32 } else { 0 } + if display.square { 64 } else { 0 };
    let size = match display.size {
        IfccadPointSize::DefaultFivePercent => 0.0,
        IfccadPointSize::Absolute(v) => v,
        IfccadPointSize::ViewportPercent(v) => -v,
    };
    (mode, size)
}
