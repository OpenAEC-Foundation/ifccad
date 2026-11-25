use super::*;

record!(IfccadExternalSource;
0 => id: IfccadExternalSourceId = None,
1 => name: String = None,
2 => location: IfccadExternalSourceLocation = None,
3 => source: IfccadExternalSourceContent = None,
);
record!(IfccadXref;
0 => source_id: IfccadExternalSourceId = None,
1 => placement: IfccadPlacement = None,
2 => scale: f64 = None,
3 => reference_type: IfccadXrefReferenceType = Some(IfccadXrefReferenceType::Attach),
);
impl CborValue for IfccadExternalSourceId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(u64::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}

impl CborValue for IfccadExternalSourceLocation {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "UriReference"), (1, "SearchFileName")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let (code, text) = match self {
            Self::UriReference { uri } => (0, uri),
            Self::SearchFileName { file_name } => (1, file_name),
        };
        w.map(2)?;
        w.u64(0)?;
        w.u64(code)?;
        w.u64(1)?;
        w.map(1)?;
        w.u64(0)?;
        text.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
        let text = b.take(0, None)?;
        b.finish()?;
        match code {
            0 => Ok(Self::UriReference { uri: text }),
            1 => Ok(Self::SearchFileName { file_name: text }),
            _ => Err(r.error("unknown external source location")),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}

impl CborValue for IfccadExternalSourceContent {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Dwg"), (1, "Dxf"), (2, "Ifccad")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let (code, base_point, insertion_unit, fingerprint_guid) = match self {
            Self::Dwg {
                base_point,
                insertion_unit,
                fingerprint_guid,
            } => (0, base_point, insertion_unit, fingerprint_guid.as_ref()),
            Self::Dxf {
                base_point,
                insertion_unit,
                fingerprint_guid,
            } => (1, base_point, insertion_unit, fingerprint_guid.as_ref()),
            Self::Ifccad {
                base_point,
                insertion_unit,
            } => (2, base_point, insertion_unit, None),
        };
        w.map(2)?;
        w.u64(0)?;
        w.u64(code)?;
        w.u64(1)?;
        w.map(1 + u64::from(insertion_unit.is_some()) + u64::from(fingerprint_guid.is_some()))?;
        w.u64(0)?;
        base_point.encode(w)?;
        if let Some(v) = insertion_unit {
            w.u64(1)?;
            v.encode(w)?;
        }
        if let Some(v) = fingerprint_guid {
            w.u64(2)?;
            v.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        if code > 2 {
            return Err(r.error("unknown external source format"));
        }
        let body = f.span(1)?;
        f.finish()?;
        let dwg_fields = &[
            (0, "base_point"),
            (1, "insertion_unit"),
            (2, "fingerprint_guid"),
        ];
        let fields = if code == 2 {
            &dwg_fields[..2]
        } else {
            &dwg_fields[..]
        };
        let mut b = RecordFields::new(&body, fields)?;
        let base_point = b.take(0, None)?;
        let insertion_unit = b.take(1, Some(None))?;
        let value = match code {
            0 => Self::Dwg {
                base_point,
                insertion_unit,
                fingerprint_guid: b.take(2, Some(None))?,
            },
            1 => Self::Dxf {
                base_point,
                insertion_unit,
                fingerprint_guid: b.take(2, Some(None))?,
            },
            2 => Self::Ifccad {
                base_point,
                insertion_unit,
            },
            _ => unreachable!(),
        };
        b.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        self.format() == o.format()
            && self.base_point().exact(&o.base_point())
            && self.insertion_unit() == o.insertion_unit()
            && self.fingerprint_guid() == o.fingerprint_guid()
    }
}

unit_enum!(IfccadXrefReferenceType; 0 => Attach, 1 => Overlay);
