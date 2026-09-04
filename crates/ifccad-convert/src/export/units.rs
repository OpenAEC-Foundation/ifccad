use super::ExportLossReason;
use ifccad::ifcdr::IfcdrLengthUnit;

pub(crate) fn map_length_unit(code: i16) -> (IfcdrLengthUnit, Option<ExportLossReason>) {
    use IfcdrLengthUnit::{Centimetre, Foot, Inch, Kilometre, Metre, Millimetre, Unitless};

    let unit = match code {
        0 => Unitless,
        4 => Millimetre,
        5 => Centimetre,
        6 => Metre,
        7 => Kilometre,
        1 => Inch,
        2 => Foot,
        code => return (Unitless, Some(ExportLossReason::UnsupportedUnit { code })),
    };
    (unit, None)
}
