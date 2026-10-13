//! Drawing-independent line pattern values. External symbol geometry is not resolved here.
mod text_extent;
pub use text_extent::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinePatternBoundsReason {
    SymbolFootprintUnavailable,
    AppearanceContextUnavailable,
    TextEstimated,
}

pub(crate) fn inflate_range(
    (min, max): ([f64; 3], [f64; 3]),
    radius: f64,
) -> Option<([f64; 3], [f64; 3])> {
    use crate::geometry_kernel::numeric::Interval;
    let mut lo = [0.; 3];
    let mut hi = [0.; 3];
    for i in 0..3 {
        let v = Interval {
            lower: min[i],
            upper: max[i],
        }
        .add(Interval {
            lower: -radius,
            upper: radius,
        })?;
        lo[i] = v.lower;
        hi[i] = v.upper;
    }
    Some((lo, hi))
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinePatternElement<M> {
    pub length: f64,
    pub marker: Option<M>,
}
impl<M> LinePatternElement<M> {
    pub fn simple(length: f64) -> Self {
        Self {
            length,
            marker: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinePatternRotation {
    Relative(f64),
    Absolute(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinePatternTransform {
    pub scale: f64,
    pub offset: [f64; 2],
    pub rotation: LinePatternRotation,
}
impl Default for LinePatternTransform {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: [0.0; 2],
            rotation: LinePatternRotation::Relative(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolSource {
    Shx { file_name: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LinePatternValueError {
    #[error("invalid line pattern period")]
    InvalidPeriod,
    #[error("invalid line pattern marker transform")]
    InvalidTransform,
    #[error("symbol source requires a nonempty bare filename")]
    InvalidFileName,
}

pub fn validate_line_pattern_values<M>(
    elements: &[LinePatternElement<M>],
) -> Result<(), LinePatternValueError> {
    if elements.is_empty() {
        return Ok(());
    }
    let total: f64 = elements.iter().map(|e| e.length.abs()).sum();
    if elements.iter().any(|e| !e.length.is_finite())
        || elements[0].length < 0.0
        || !total.is_finite()
        || total <= 0.0
        || (elements.len() < 2 && elements.iter().all(|e| e.marker.is_none()))
    {
        return Err(LinePatternValueError::InvalidPeriod);
    }
    Ok(())
}

pub fn validate_line_pattern_transform(
    value: &LinePatternTransform,
) -> Result<(), LinePatternValueError> {
    let angle = match value.rotation {
        LinePatternRotation::Relative(v) | LinePatternRotation::Absolute(v) => v,
    };
    if !value.scale.is_finite()
        || value.scale <= 0.0
        || !angle.is_finite()
        || value.offset.iter().any(|v| !v.is_finite())
    {
        return Err(LinePatternValueError::InvalidTransform);
    }
    Ok(())
}

pub fn validate_symbol_source(value: &SymbolSource) -> Result<(), LinePatternValueError> {
    let SymbolSource::Shx { file_name } = value;
    if file_name.is_empty()
        || matches!(file_name.as_str(), "." | "..")
        || file_name.contains(['/', '\\', '\0'])
    {
        return Err(LinePatternValueError::InvalidFileName);
    }
    Ok(())
}
