use ifccad::package::{PointDisplay, PointGlyph, PointSize};

pub(crate) fn from_cad(mode: i16, size: f64) -> Option<PointDisplay> {
    if mode < 0 || mode & !0x67 != 0 || mode & 7 > 4 || !size.is_finite() {
        return None;
    }
    let glyph = match mode & 7 {
        0 => PointGlyph::Dot,
        1 => PointGlyph::Hidden,
        2 => PointGlyph::Plus,
        3 => PointGlyph::Cross,
        4 => PointGlyph::ShortLine,
        _ => return None,
    };
    let size = if size == 0.0 {
        PointSize::DefaultFivePercent
    } else if size > 0.0 {
        PointSize::Absolute(size)
    } else {
        PointSize::ViewportPercent(-size)
    };
    Some(PointDisplay {
        glyph,
        circle: mode & 32 != 0,
        square: mode & 64 != 0,
        size,
    })
}

pub(crate) fn to_cad(display: PointDisplay) -> (i16, f64) {
    let glyph = match display.glyph {
        PointGlyph::Dot => 0,
        PointGlyph::Hidden => 1,
        PointGlyph::Plus => 2,
        PointGlyph::Cross => 3,
        PointGlyph::ShortLine => 4,
    };
    let mode = glyph + if display.circle { 32 } else { 0 } + if display.square { 64 } else { 0 };
    let size = match display.size {
        PointSize::DefaultFivePercent => 0.0,
        PointSize::Absolute(value) => value,
        PointSize::ViewportPercent(value) => -value,
    };
    (mode, size)
}
