use super::*;
use crate::geometry_kernel::{CoordinateFrame3, Point2, Point3, Vector3};

pub(super) fn encode_line_point(
    point: &[f64; 3],
    w: &mut CborWriter,
) -> Result<(), IfccadCborEncodeError> {
    let n = if point[2].to_bits() == 0.0_f64.to_bits() {
        2
    } else {
        3
    };
    w.array(n)?;
    for v in &point[..n as usize] {
        v.encode(w)?;
    }
    Ok(())
}
pub(super) fn decode_line_point(r: &mut CborReader<'_>) -> Result<[f64; 3], IfccadCborReadError> {
    let (mut d, n) = r.sequence()?;
    if n != 2 && n != 3 {
        return Err(r.error("line point requires XY or XYZ"));
    }
    let mut p = [0.0; 3];
    for i in 0..n {
        p[i as usize] = f64::decode(&mut r.child(&mut d, i)?)?;
    }
    Ok(p)
}
macro_rules! point {
    ($ty:ty,$n:literal,$($field:ident),+) => {
        impl CborValue for $ty {
            fn encode(&self,w:&mut CborWriter)->Result<(),IfccadCborEncodeError>{[$(self.$field()),+].encode(w)}
            fn decode(r:&mut CborReader<'_>)->Result<Self,IfccadCborReadError>{let[$($field),+]=<[f64;$n]>::decode(r)?;Ok(Self::new($($field),+))}
            fn exact(&self,o:&Self)->bool{true$(&&self.$field().to_bits()==o.$field().to_bits())+}
        }
    };
}
point!(Point2, 2, x, y);
point!(Point3, 3, x, y, z);
point!(Vector3, 3, x, y, z);
impl CborValue for CoordinateFrame3 {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(3)?;
        w.u64(0)?;
        self.origin().encode(w)?;
        w.u64(1)?;
        self.x_axis().encode(w)?;
        w.u64(2)?;
        self.y_axis().encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "origin"), (1, "x_axis"), (2, "y_axis")])?;
        let out = Self::try_new(f.take(0, None)?, f.take(1, None)?, f.take(2, None)?)
            .map_err(|e| r.error(e))?;
        f.finish()?;
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        self.origin().exact(&o.origin())
            && self.x_axis().exact(&o.x_axis())
            && self.y_axis().exact(&o.y_axis())
    }
}
