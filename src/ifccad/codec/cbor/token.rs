use super::*;
use minicbor::{data::Type, Decoder, Encoder};

pub(super) fn malformed(path: &str, offset: usize, message: impl ToString) -> IfccadCborReadError {
    IfccadCborReadError::Malformed {
        location: path.into(),
        offset,
        message: message.to_string(),
    }
}
fn limit(
    path: &str,
    name: &'static str,
    observed: u64,
    allowed: u64,
) -> Result<(), IfccadCborReadError> {
    if observed > allowed {
        Err(IfccadCborReadError::ResourceLimit {
            location: path.into(),
            limit: name,
            observed,
            allowed,
        })
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ScanSummary {
    pub items: u64,
    pub depth: u32,
    pub container: u64,
}

/// Iterative physical preflight. No allocation is based on an unverified length.
pub(super) fn scan_one_item(
    bytes: &[u8],
    limits: IfccadCborReadLimits,
) -> Result<ScanSummary, IfccadCborReadError> {
    limit(
        "/",
        "input bytes",
        bytes.len() as u64,
        limits.max_input_bytes,
    )?;
    let mut d = Decoder::new(bytes);
    let mut stack = vec![(1_u64, 0_u32)];
    let mut stats = ScanSummary::default();
    while let Some((left, parent_depth)) = stack.pop() {
        if left == 0 {
            continue;
        }
        stack.push((left - 1, parent_depth));
        let offset = d.position();
        let fail = |e| malformed("/", offset, e);
        stats.items = stats
            .items
            .checked_add(1)
            .ok_or_else(|| malformed("/", offset, "item count overflow"))?;
        limit("/", "total items", stats.items, limits.max_total_items)?;
        match d.datatype().map_err(fail)? {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                d.u64().map_err(fail)?;
            }
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => {
                d.i64().map_err(fail)?;
            }
            Type::Bool => {
                d.bool().map_err(fail)?;
            }
            Type::F64 => {
                if !d.f64().map_err(fail)?.is_finite() {
                    return Err(malformed("/", offset, "non-finite float"));
                }
            }
            Type::String => {
                d.str().map_err(fail)?;
            }
            Type::Bytes => {
                d.bytes().map_err(fail)?;
            }
            Type::Array | Type::Map => {
                let map = d.datatype().map_err(fail)? == Type::Map;
                let len = if map { d.map() } else { d.array() }
                    .map_err(fail)?
                    .ok_or_else(|| malformed("/", offset, "indefinite container"))?;
                let depth = parent_depth
                    .checked_add(1)
                    .ok_or_else(|| malformed("/", offset, "depth overflow"))?;
                limit("/", "nesting depth", depth as u64, limits.max_depth as u64)?;
                limit("/", "container entries", len, limits.max_container_entries)?;
                let children = len
                    .checked_mul(if map { 2 } else { 1 })
                    .ok_or_else(|| malformed("/", offset, "length overflow"))?;
                limit(
                    "/",
                    "total items",
                    stats
                        .items
                        .checked_add(children)
                        .ok_or_else(|| malformed("/", offset, "item count overflow"))?,
                    limits.max_total_items,
                )?;
                if usize::try_from(children).is_err()
                    || children > (bytes.len() - d.position()) as u64
                {
                    return Err(malformed(
                        "/",
                        offset,
                        "container length exceeds remaining input",
                    ));
                }
                stats.depth = stats.depth.max(depth);
                stats.container = stats.container.max(len);
                stack.push((children, depth));
            }
            other => {
                return Err(malformed(
                    "/",
                    offset,
                    format!("unsupported CBOR token {other}"),
                ))
            }
        }
    }
    if d.position() != bytes.len() {
        return Err(malformed("/", d.position(), "trailing bytes"));
    }
    Ok(stats)
}

pub(super) struct CborWriter {
    encoder: Encoder<Vec<u8>>,
    stack: Vec<u64>,
    stats: ScanSummary,
}
macro_rules! scalar_writer {
    ($name:ident, $ty:ty) => {
        pub fn $name(&mut self, value: $ty) -> Result<(), IfccadCborEncodeError> {
            self.mark(None)?;
            self.encoder
                .$name(value)
                .map(|_| ())
                .map_err(|e| self.error(e))
        }
    };
}
impl CborWriter {
    pub fn new() -> Self {
        Self {
            encoder: Encoder::new(Vec::new()),
            stack: vec![1],
            stats: ScanSummary::default(),
        }
    }
    fn error(&self, message: impl ToString) -> IfccadCborEncodeError {
        IfccadCborEncodeError::Mapping {
            location: "/".into(),
            message: message.to_string(),
        }
    }
    fn mark(&mut self, children: Option<u64>) -> Result<(), IfccadCborEncodeError> {
        while self.stack.last() == Some(&0) {
            self.stack.pop();
        }
        let Some(left) = self.stack.last_mut() else {
            return Err(self.error("extra output item"));
        };
        *left -= 1;
        self.stats.items = self
            .stats
            .items
            .checked_add(1)
            .ok_or_else(|| self.error("item count overflow"))?;
        if let Some(children) = children {
            self.stats.depth = self
                .stats
                .depth
                .max(u32::try_from(self.stack.len()).map_err(|_| self.error("depth overflow"))?);
            self.stack.push(children);
        }
        Ok(())
    }
    pub fn map(&mut self, len: u64) -> Result<(), IfccadCborEncodeError> {
        self.mark(Some(
            len.checked_mul(2)
                .ok_or_else(|| self.error("map length overflow"))?,
        ))?;
        self.stats.container = self.stats.container.max(len);
        self.encoder.map(len).map(|_| ()).map_err(|e| self.error(e))
    }
    pub fn array(&mut self, len: u64) -> Result<(), IfccadCborEncodeError> {
        self.mark(Some(len))?;
        self.stats.container = self.stats.container.max(len);
        self.encoder
            .array(len)
            .map(|_| ())
            .map_err(|e| self.error(e))
    }
    scalar_writer!(u64, u64);
    scalar_writer!(i64, i64);
    scalar_writer!(f64, f64);
    scalar_writer!(bool, bool);
    scalar_writer!(str, &str);
    scalar_writer!(bytes, &[u8]);
    pub fn finish(mut self) -> Result<EncodedCborBuffer, IfccadCborEncodeError> {
        while self.stack.last() == Some(&0) {
            self.stack.pop();
        }
        if !self.stack.is_empty() {
            return Err(self.error("incomplete output container"));
        }
        let bytes = self.encoder.into_writer();
        let readback_limits = IfccadCborReadLimits {
            max_input_bytes: bytes.len() as u64,
            max_depth: self.stats.depth,
            max_container_entries: self.stats.container,
            max_total_items: self.stats.items,
        };
        Ok(EncodedCborBuffer {
            bytes,
            readback_limits,
        })
    }
}

pub(super) struct CborReader<'a> {
    pub bytes: &'a [u8],
    pub base: usize,
    pub path: String,
}
impl<'a> CborReader<'a> {
    pub fn new(bytes: &'a [u8], base: usize, path: String) -> Self {
        Self { bytes, base, path }
    }
    pub fn error(&self, message: impl ToString) -> IfccadCborReadError {
        malformed(&self.path, self.base, message)
    }
    pub fn read<T>(
        &self,
        f: impl FnOnce(&mut Decoder<'a>) -> Result<T, minicbor::decode::Error>,
    ) -> Result<T, IfccadCborReadError> {
        let mut d = Decoder::new(self.bytes);
        let result = f(&mut d).map_err(|e| {
            malformed(
                &self.path,
                self.base + e.position().unwrap_or(d.position()),
                e,
            )
        })?;
        if d.position() != self.bytes.len() {
            return Err(self.error("extra data in value span"));
        }
        Ok(result)
    }
    pub fn child(
        &self,
        d: &mut Decoder<'a>,
        name: impl std::fmt::Display,
    ) -> Result<Self, IfccadCborReadError> {
        let start = d.position();
        // The complete input has already passed the bounded iterative preflight.
        d.skip()
            .map_err(|e| malformed(&self.path, self.base + start, e))?;
        Ok(Self::new(
            &self.bytes[start..d.position()],
            self.base + start,
            format!("{}/{name}", self.path),
        ))
    }
    pub fn sequence(&self) -> Result<(Decoder<'a>, u64), IfccadCborReadError> {
        let mut d = Decoder::new(self.bytes);
        let len = d
            .array()
            .map_err(|e| self.error(e))?
            .ok_or_else(|| self.error("indefinite array"))?;
        Ok((d, len))
    }
}
