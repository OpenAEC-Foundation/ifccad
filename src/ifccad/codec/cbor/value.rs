use super::*;
use minicbor::{data::Type, Decoder};
use std::collections::BTreeMap;

pub(super) trait CborValue: Sized {
    fn encode(&self, writer: &mut CborWriter) -> Result<(), IfccadCborEncodeError>;
    fn decode(reader: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError>;
    fn exact(&self, other: &Self) -> bool;
    fn present(&self) -> bool {
        true
    }
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[]
    }
    const BYTE: bool = false;
    fn byte(&self) -> u8 {
        unreachable!("only called for BYTE types")
    }
    fn from_byte(_: u8) -> Self {
        unreachable!("only called for BYTE types")
    }
    fn sort_key(&self) -> Result<Vec<u8>, IfccadCborEncodeError> {
        let mut w = CborWriter::new();
        self.encode(&mut w)?;
        Ok(w.finish()?.bytes)
    }
}
macro_rules! unsigned {
    ($($ty:ty),+) => {$(impl CborValue for $ty {
        fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> { w.u64(*self as u64) }
        fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
            let x = r.read(|d| d.u64())?; <$ty>::try_from(x).map_err(|_| r.error("unsigned integer out of range"))
        }
        fn exact(&self, other: &Self) -> bool { self == other }
    })+};
}
unsigned!(u16, u32, u64);
impl CborValue for u8 {
    const BYTE: bool = true;
    fn byte(&self) -> u8 {
        *self
    }
    fn from_byte(x: u8) -> Self {
        x
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.u64(*self as u64)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        u8::try_from(r.read(|d| d.u64())?).map_err(|_| r.error("u8 out of range"))
    }
    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}
macro_rules! signed {
    ($($ty:ty),+) => {$(impl CborValue for $ty {
        fn encode(&self,w:&mut CborWriter)->Result<(),IfccadCborEncodeError>{w.i64(*self as i64)}
        fn decode(r:&mut CborReader<'_>)->Result<Self,IfccadCborReadError>{<$ty>::try_from(r.read(|d|d.i64())?).map_err(|_|r.error("signed integer out of range"))}
        fn exact(&self,o:&Self)->bool{self==o}
    })+};
}
signed!(i8, i16, i32, i64);
impl CborValue for f64 {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.f64(*self)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        if Decoder::new(r.bytes).datatype().map_err(|e| r.error(e))? != Type::F64 {
            return Err(r.error("expected binary64"));
        }
        let x = r.read(|d| d.f64())?;
        if !x.is_finite() {
            return Err(r.error("non-finite float"));
        }
        Ok(x)
    }
    fn exact(&self, o: &Self) -> bool {
        self.to_bits() == o.to_bits()
    }
}
impl CborValue for bool {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.bool(*self)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        r.read(|d| d.bool())
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}
impl CborValue for String {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.str(self)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(r.read(|d| d.str())?.to_owned())
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
    fn sort_key(&self) -> Result<Vec<u8>, IfccadCborEncodeError> {
        Ok(self.as_bytes().to_vec())
    }
}
impl CborValue for char {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.str(self.encode_utf8(&mut [0; 4]))
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let s = String::decode(r)?;
        let mut c = s.chars();
        let first = c
            .next()
            .ok_or_else(|| r.error("expected one Unicode scalar"))?;
        if c.next().is_some() {
            return Err(r.error("expected one Unicode scalar"));
        }
        Ok(first)
    }
    fn exact(&self, o: &Self) -> bool {
        self == o
    }
}
impl<T: CborValue> CborValue for Option<T> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.as_ref()
            .ok_or_else(|| IfccadCborEncodeError::Mapping {
                location: "/".into(),
                message: "absent option outside record".into(),
            })?
            .encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Some(T::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Some(a), Some(b)) => a.exact(b),
            (None, None) => true,
            _ => false,
        }
    }
    fn present(&self) -> bool {
        self.is_some()
    }
}
impl<T: CborValue> CborValue for Box<T> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        (**self).encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Box::new(T::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        (**self).exact(o)
    }
}
impl<T: CborValue> CborValue for Vec<T> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        if T::BYTE {
            return w.bytes(&self.iter().map(CborValue::byte).collect::<Vec<_>>());
        }
        w.array(self.len() as u64)?;
        for x in self {
            x.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        if T::BYTE {
            return Ok(r
                .read(|d| d.bytes())?
                .iter()
                .map(|b| T::from_byte(*b))
                .collect());
        }
        let (mut d, len) = r.sequence()?;
        let mut out = Vec::new();
        for i in 0..len {
            out.push(T::decode(&mut r.child(&mut d, i)?)?);
        }
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        self.len() == o.len() && self.iter().zip(o).all(|(a, b)| a.exact(b))
    }
}
impl<T: CborValue, const N: usize> CborValue for [T; N] {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.array(N as u64)?;
        for x in self {
            x.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let (mut d, len) = r.sequence()?;
        if len != N as u64 {
            return Err(r.error(format!("expected {N} components")));
        }
        let mut xs = Vec::new();
        for i in 0..len {
            xs.push(T::decode(&mut r.child(&mut d, i)?)?);
        }
        xs.try_into().map_err(|_| r.error("tuple length mismatch"))
    }
    fn exact(&self, o: &Self) -> bool {
        self.iter().zip(o).all(|(a, b)| a.exact(b))
    }
}
impl<A: CborValue, B: CborValue> CborValue for (A, B) {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.array(2)?;
        self.0.encode(w)?;
        self.1.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let (mut d, n) = r.sequence()?;
        if n != 2 {
            return Err(r.error("expected pair"));
        }
        Ok((
            A::decode(&mut r.child(&mut d, 0)?)?,
            B::decode(&mut r.child(&mut d, 1)?)?,
        ))
    }
    fn exact(&self, o: &Self) -> bool {
        self.0.exact(&o.0) && self.1.exact(&o.1)
    }
}
impl<K: CborValue + Ord, V: CborValue> CborValue for BTreeMap<K, V> {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let mut entries = self
            .iter()
            .map(|(k, v)| Ok((k.sort_key()?, k, v)))
            .collect::<Result<Vec<_>, IfccadCborEncodeError>>()?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        w.map(entries.len() as u64)?;
        for (_, k, v) in entries {
            k.encode(w)?;
            v.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut d = Decoder::new(r.bytes);
        let len = d
            .map()
            .map_err(|e| r.error(e))?
            .ok_or_else(|| r.error("indefinite map"))?;
        let mut out = Self::new();
        for i in 0..len {
            let k = K::decode(&mut r.child(&mut d, format!("key{i}"))?)?;
            if out.contains_key(&k) {
                return Err(r.error("duplicate semantic map key"));
            }
            let v = V::decode(&mut r.child(&mut d, i)?)?;
            out.insert(k, v);
        }
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        self.len() == o.len()
            && self
                .iter()
                .zip(o)
                .all(|((ak, av), (bk, bv))| ak.exact(bk) && av.exact(bv))
    }
}

macro_rules! record {
    ($ty:ty; $($code:literal => $field:ident : $ft:ty = $default:expr),* $(,)?) => {
        impl CborValue for $ty {
            #[cfg(test)]
            fn registered_codes() -> &'static [(u64, &'static str)] { &[$(($code, stringify!($field))),*] }
            fn encode(&self,w:&mut CborWriter)->Result<(),IfccadCborEncodeError>{
                let Self{$($field),*}=self;
                let mut count=0_u64;$(if $field.present()&&($default as Option<$ft>).as_ref().is_none_or(|d|!$field.exact(d)){count+=1;})*
                w.map(count)?;
                $(if $field.present()&&($default as Option<$ft>).as_ref().is_none_or(|d|!$field.exact(d)){w.u64($code)?;$field.encode(w)?;})*
                Ok(())
            }
            fn decode(r:&mut CborReader<'_>)->Result<Self,IfccadCborReadError>{
                let mut fields=RecordFields::new(r,&[$(($code,stringify!($field))),*])?;
                let out=Self{$($field:fields.take::<$ft>($code,$default)?),*};fields.finish()?;Ok(out)
            }
            fn exact(&self,o:&Self)->bool{let Self{$($field),*}=self;true$(&&$field.exact(&o.$field))*}
        }
    };
}
pub(super) use record;

// Literal codes remain in each family declaration; exhaustive matching makes
// a newly added logical variant a compile error until it is mapped explicitly.
macro_rules! unit_enum {
    ($ty:ty; $($code:literal => $variant:ident),+ $(,)?) => {
        impl CborValue for $ty {
            fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
                w.u64(match self { $(Self::$variant => $code),+ })
            }
            fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
                match u64::decode(r)? { $($code => Ok(Self::$variant)),+, _ => Err(r.error("unknown variant code")) }
            }
            fn exact(&self, other: &Self) -> bool { self == other }
            #[cfg(test)]
            fn registered_codes() -> &'static [(u64, &'static str)] { &[$(($code, stringify!($variant))),+] }
        }
    };
}
pub(super) use unit_enum;
