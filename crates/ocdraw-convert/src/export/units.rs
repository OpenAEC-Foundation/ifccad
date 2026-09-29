use super::ExportLossReason;
use ocdraw::ifcdr::IfcdrLengthUnit;

pub(crate) fn map_length_unit(code: i16) -> (IfcdrLengthUnit, Option<ExportLossReason>) {
    let unit = match crate::units::from_cad_code(code) {
        Some(unit) => unit,
        None => {
            return (
                IfcdrLengthUnit::Unitless,
                Some(ExportLossReason::UnsupportedUnit { code }),
            )
        }
    };
    (unit, None)
}
