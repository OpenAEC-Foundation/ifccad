use cadcodec::CadDocument;
use ifccad::ifcdr::IfcdrLengthUnit;

pub(crate) fn apply_units(document: &mut CadDocument, unit: IfcdrLengthUnit) {
    use IfcdrLengthUnit::{Centimetre, Foot, Inch, Kilometre, Metre, Millimetre, Unitless};

    let (insertion_units, measurement) = match unit {
        Unitless => (0, None),
        Millimetre => (4, Some(1)),
        Centimetre => (5, Some(1)),
        Metre => (6, Some(1)),
        Kilometre => (7, Some(1)),
        Inch => (1, Some(0)),
        Foot => (2, Some(0)),
    };
    document.header.insertion_units = insertion_units;
    if let Some(measurement) = measurement {
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
