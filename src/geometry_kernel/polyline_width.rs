//! Authored planar full widths, independent of CAD runtimes and derived geometry.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PlanarPolylineWidths {
    Constant {
        width: f64,
    },
    PerVertex {
        widths: Vec<[f64; 2]>,
    },
    Defaulted {
        #[serde(rename = "defaultStartWidth")]
        default_start_width: f64,
        #[serde(rename = "defaultEndWidth")]
        default_end_width: f64,
        overrides: Vec<PolylineVertexWidthOverride>,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolylineVertexWidthOverride {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "explicit_width"
    )]
    pub start: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "explicit_width"
    )]
    pub end: Option<f64>,
}
fn explicit_width<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    f64::deserialize(d).map(Some)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PolylineWidthValidationError {
    #[error("width count {actual} differs from vertex count {expected}")]
    CountMismatch { expected: usize, actual: usize },
    #[error("polyline width must be finite")]
    NonFinite,
    #[error("polyline width must be nonnegative")]
    Negative,
    #[error("width index {index} is outside {vertex_count} vertices")]
    IndexOutOfRange { index: usize, vertex_count: usize },
}
impl Default for PlanarPolylineWidths {
    fn default() -> Self {
        Self::Constant { width: 0.0 }
    }
}
impl PlanarPolylineWidths {
    /// Only the intrinsic positive-zero Constant may be omitted on native wire.
    pub fn is_intrinsic_default(&self) -> bool {
        matches!(self, Self::Constant{width} if width.to_bits()==0)
    }
    pub fn validate(&self, vertex_count: usize) -> Result<(), PolylineWidthValidationError> {
        let scalar = |v: f64| {
            if !v.is_finite() {
                Err(PolylineWidthValidationError::NonFinite)
            } else if v < 0.0 {
                Err(PolylineWidthValidationError::Negative)
            } else {
                Ok(())
            }
        };
        let count = |actual| {
            if actual == vertex_count {
                Ok(())
            } else {
                Err(PolylineWidthValidationError::CountMismatch {
                    expected: vertex_count,
                    actual,
                })
            }
        };
        match self {
            Self::Constant { width } => scalar(*width),
            Self::PerVertex { widths } => {
                count(widths.len())?;
                for pair in widths {
                    for v in pair {
                        scalar(*v)?;
                    }
                }
                Ok(())
            }
            Self::Defaulted {
                default_start_width,
                default_end_width,
                overrides,
            } => {
                count(overrides.len())?;
                scalar(*default_start_width)?;
                scalar(*default_end_width)?;
                for item in overrides {
                    for v in [item.start, item.end].into_iter().flatten() {
                        scalar(v)?;
                    }
                }
                Ok(())
            }
        }
    }
    pub fn effective_at(
        &self,
        index: usize,
        vertex_count: usize,
    ) -> Result<[f64; 2], PolylineWidthValidationError> {
        self.validate(vertex_count)?;
        if index >= vertex_count {
            return Err(PolylineWidthValidationError::IndexOutOfRange {
                index,
                vertex_count,
            });
        }
        Ok(self.resolved_at(index))
    }
    /// Resolve all records after one shared validation; the returned view is derived only.
    pub fn effective_widths(
        &self,
        vertex_count: usize,
    ) -> Result<Vec<[f64; 2]>, PolylineWidthValidationError> {
        self.validate(vertex_count)?;
        Ok((0..vertex_count).map(|i| self.resolved_at(i)).collect())
    }
    fn resolved_at(&self, index: usize) -> [f64; 2] {
        match self {
            Self::Constant { width } => [*width; 2],
            Self::PerVertex { widths } => widths[index],
            Self::Defaulted {
                default_start_width,
                default_end_width,
                overrides,
            } => [
                overrides[index].start.unwrap_or(*default_start_width),
                overrides[index].end.unwrap_or(*default_end_width),
            ],
        }
    }
}
