pub use crate::ConversionLossPolicy as ExportLossPolicy;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExportOptions {
    pub loss_policy: ExportLossPolicy,
    pub geometry_tolerance: crate::ConversionGeometryTolerance,
}
