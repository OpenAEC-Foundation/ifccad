//! CAD-independent resolution of symbolic drawing colors.

/// Resolve a foreground color against an effective opaque sRGB background.
///
/// Selects pure black or white with the larger luminance contrast ratio.
/// Consumers supply the composited background after resolving appearance
/// inheritance; this function does not store a resolved color in a drawing.
pub fn foreground_srgb(background: [u8; 3]) -> [u8; 3] {
    let linear = background.map(|byte| {
        let c = f64::from(byte) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let luminance = 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    if (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) {
        [0, 0, 0]
    } else {
        [255, 255, 255]
    }
}
