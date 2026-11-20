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
        &[(0, "Dwg")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let Self::Dwg {
            base_point,
            insertion_unit,
            fingerprint_guid,
        } = self;
        w.map(2)?;
        w.u64(0)?;
        w.u64(0)?;
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
        if code != 0 {
            return Err(r.error("unknown external source format"));
        }
        let body = f.span(1)?;
        f.finish()?;
        let mut b = RecordFields::new(
            &body,
            &[
                (0, "base_point"),
                (1, "insertion_unit"),
                (2, "fingerprint_guid"),
            ],
        )?;
        let value = Self::Dwg {
            base_point: b.take(0, None)?,
            insertion_unit: b.take(1, Some(None))?,
            fingerprint_guid: b.take(2, Some(None))?,
        };
        b.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        let Self::Dwg {
            base_point: a,
            insertion_unit: b,
            fingerprint_guid: c,
        } = self;
        let Self::Dwg {
            base_point: d,
            insertion_unit: e,
            fingerprint_guid: f,
        } = o;
        a.exact(d) && b.exact(e) && c.exact(f)
    }
}
