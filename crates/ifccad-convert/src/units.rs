pub(crate) use cad_geometry_convert::units::UNIT_TOKENS;

// IFCCAD stores units as tokens; the shared list supplies their CAD code order.
pub(crate) fn unit_code(unit: &str) -> i16 {
    UNIT_TOKENS
        .iter()
        .position(|u| *u == unit)
        .expect("validated unit") as i16
}
