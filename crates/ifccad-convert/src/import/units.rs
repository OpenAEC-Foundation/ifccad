use cadcodec::CadDocument;
use ifccad::ifcdr::IfcdrLengthUnit;

pub(crate) fn apply_units(document: &mut CadDocument, unit: IfcdrLengthUnit) {
    document.header.insertion_units = crate::units::cad_code(unit);
    if let Some(measurement) = crate::units::measurement(unit) {
        document.header.measurement = measurement;
    }
}

#[cfg(test)]
mod tests {
    use super::apply_units;
    use cadcodec::CadDocument;
    use ifccad::ifcdr::IfcdrLengthUnit::{
        Centimetre, Foot, Inch, Kilometre, Metre, Millimetre, Unitless,
    };
    #[test]
    fn all_cad_unit_codes_roundtrip_separately_from_measurement_metadata() {
        for code in 0..=24 {
            let unit = crate::units::from_cad_code(code).unwrap();
            let mut document = CadDocument::new();
            document.header.measurement = 1;
            apply_units(&mut document, unit);
            assert_eq!(document.header.insertion_units, code);
            if crate::units::measurement(unit).is_none() {
                assert_eq!(document.header.measurement, 1);
            }
        }
    }

    #[test]
    fn maps_every_ifcdr_unit_to_cadcodec_header_semantics() {
        let cases = [
            (Unitless, 0, 0),
            (Millimetre, 4, 1),
            (Centimetre, 5, 1),
            (Metre, 6, 1),
            (Kilometre, 7, 1),
            (Inch, 1, 0),
            (Foot, 2, 0),
        ];

        for (unit, insertion_units, measurement) in cases {
            let mut document = CadDocument::new();
            apply_units(&mut document, unit);
            assert_eq!(document.header.insertion_units, insertion_units);
            assert_eq!(document.header.measurement, measurement);
        }
    }
}
