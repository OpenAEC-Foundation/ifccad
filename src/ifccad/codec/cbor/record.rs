use super::*;
use minicbor::Decoder;
use std::collections::BTreeMap;

pub(super) struct RecordFields<'a> {
    fields: BTreeMap<u64, CborReader<'a>>,
    path: String,
    base: usize,
    names: &'static [(u64, &'static str)],
}
impl<'a> RecordFields<'a> {
    pub fn new(
        reader: &CborReader<'a>,
        names: &'static [(u64, &'static str)],
    ) -> Result<Self, IfccadCborReadError> {
        let mut d = Decoder::new(reader.bytes);
        let len = d
            .map()
            .map_err(|e| reader.error(e))?
            .ok_or_else(|| reader.error("indefinite record"))?;
        let mut fields = BTreeMap::new();
        for _ in 0..len {
            let key = d.u64().map_err(|e| reader.error(e))?;
            let name = names
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, name)| *name)
                .ok_or_else(|| reader.error(format!("unknown field {key}")))?;
            if fields.contains_key(&key) {
                return Err(reader.error(format!("duplicate field {name} ({key})")));
            }
            fields.insert(key, reader.child(&mut d, name)?);
        }
        if d.position() != reader.bytes.len() {
            return Err(reader.error("extra record data"));
        }
        Ok(Self {
            fields,
            path: reader.path.clone(),
            base: reader.base,
            names,
        })
    }
    pub fn take<T: CborValue>(
        &mut self,
        code: u64,
        default: Option<T>,
    ) -> Result<T, IfccadCborReadError> {
        match self.fields.remove(&code) {
            Some(mut r) => T::decode(&mut r),
            None => default.ok_or_else(|| self.missing(code)),
        }
    }
    pub fn span(&mut self, code: u64) -> Result<CborReader<'a>, IfccadCborReadError> {
        self.fields.remove(&code).ok_or_else(|| self.missing(code))
    }
    fn missing(&self, code: u64) -> IfccadCborReadError {
        let name = self
            .names
            .iter()
            .find(|(key, _)| *key == code)
            .map(|(_, name)| *name)
            .unwrap_or("unknown");
        token::malformed(
            &self.path,
            self.base,
            format!("missing required field {name} ({code})"),
        )
    }
    pub fn finish(self) -> Result<(), IfccadCborReadError> {
        if self.fields.is_empty() {
            Ok(())
        } else {
            Err(token::malformed(
                &self.path,
                self.base,
                "field not allowed for this variant",
            ))
        }
    }
}
