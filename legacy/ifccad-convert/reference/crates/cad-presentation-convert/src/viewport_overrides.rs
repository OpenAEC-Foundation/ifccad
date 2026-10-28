use crate::{
    color_to_cad, explicit_color_from_cad, CadColorLoss, CadColorValue, PresentationValueError,
};
use opencadcodec::objects::XRecordValue;
use opencadcodec::{Color, LineWeight, Transparency};

/// Decode AcCmColorBase method-tagged RGB or concrete ACI. Inherited and custom methods are not concrete colors.
pub fn decode_override_color(
    value: &XRecordValue,
) -> Result<CadColorValue, PresentationValueError> {
    let XRecordValue::Int32(raw) = value else {
        return Err(PresentationValueError::UnsupportedOverride);
    };
    let raw = *raw as u32;
    let color = match raw >> 24 {
        0xc2 => Color::from_rgb((raw >> 16) as u8, (raw >> 8) as u8, raw as u8),
        0xc3 if raw & 0x00ffff00 == 0 && raw as u8 != 0 => Color::Index(raw as u8),
        _ => return Err(PresentationValueError::UnsupportedOverride),
    };
    explicit_color_from_cad(color)
}

/// Named identity needs additional, independently qualified metadata; this scalar encodes RGB/ACI only.
pub fn encode_override_color(
    value: &CadColorValue,
) -> Result<(XRecordValue, Vec<CadColorLoss>), PresentationValueError> {
    let mapped = color_to_cad(value)?;
    let raw = match mapped.color {
        Color::Rgb { r, g, b } => {
            0xc2000000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
        }
        Color::Index(index) => 0xc3000000 | u32::from(index),
        _ => return Err(PresentationValueError::UnsupportedOverride),
    };
    Ok((XRecordValue::Int32(raw as i32), mapped.losses))
}

/// Explicit packed alpha: method 2 in DXF, method 3 in the codec's DWG representation.
pub fn decode_override_opacity(value: &XRecordValue) -> Result<f64, PresentationValueError> {
    let XRecordValue::Int32(raw) = value else {
        return Err(PresentationValueError::UnsupportedOverride);
    };
    let raw = *raw as u32;
    if !matches!(raw >> 24, 2 | 3) || raw & 0x00ffff00 != 0 {
        return Err(PresentationValueError::UnsupportedOverride);
    }
    // Use the same canonical opacity as ordinary CAD transparency decoding.
    // alpha/255 can differ by one binary64 bit and break exact byte recognition.
    Ok(1.0 - f64::from(255 - raw as u8) / 255.0)
}
pub fn encode_override_opacity(
    value: Transparency,
) -> Result<XRecordValue, PresentationValueError> {
    if !value.is_explicit() {
        return Err(PresentationValueError::UnsupportedOverride);
    }
    Ok(XRecordValue::Int32(value.to_dxf_value()))
}

/// XRecord code 91 carries the explicit weight in hundredths of a millimeter, not a DWG table index.
pub fn decode_override_lineweight(value: &XRecordValue) -> Result<f64, PresentationValueError> {
    let XRecordValue::Int32(raw @ 0..=211) = value else {
        return Err(PresentationValueError::UnsupportedOverride);
    };
    Ok(f64::from(*raw) / 100.0)
}
pub fn encode_override_lineweight(
    value: LineWeight,
) -> Result<XRecordValue, PresentationValueError> {
    let LineWeight::Value(raw @ 0..=211) = value else {
        return Err(PresentationValueError::UnsupportedOverride);
    };
    Ok(XRecordValue::Int32(i32::from(raw)))
}
