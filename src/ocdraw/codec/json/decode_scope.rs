use crate::ocdraw::logical::{DrawingScope, DrawingScopeKind};
use crate::ocdraw::{Bounds3d, Point3};
use serde_json::Value;

fn bounds(value: &Value) -> Option<Bounds3d> {
    Some(Bounds3d {
        min: Point3::new(
            value.get("minX")?.as_f64()?,
            value.get("minY")?.as_f64()?,
            value.get("minZ")?.as_f64()?,
        ),
        max: Point3::new(
            value.get("maxX")?.as_f64()?,
            value.get("maxY")?.as_f64()?,
            value.get("maxZ")?.as_f64()?,
        ),
    })
}

pub(crate) fn decode_scopes(value: &Value) -> Option<Vec<DrawingScope>> {
    value
        .get("scopes")?
        .as_array()?
        .iter()
        .map(|row| {
            let id = u32::try_from(row.get("id")?.as_u64()?).ok()?;
            let kind = match row.get("kind")?.as_u64()? {
                0 => DrawingScopeKind::Model,
                1 => DrawingScopeKind::Paper,
                2 => DrawingScopeKind::Block,
                _ => return None,
            };
            let bounds = match row.get("bounds") {
                Some(value) if !value.is_null() => Some(bounds(value)?),
                _ => None,
            };
            Some(DrawingScope {
                id,
                kind,
                bounds,
                entities: row
                    .get("entities")?
                    .as_array()?
                    .iter()
                    .map(Value::as_u64)
                    .collect::<Option<Vec<_>>>()?,
            })
        })
        .collect()
}
