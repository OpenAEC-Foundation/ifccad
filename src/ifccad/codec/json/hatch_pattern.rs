use crate::geometry_kernel::hatch::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
fn one() -> f64 {
    1.
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum DashValue {
    Dash { length: f64 },
    Gap { length: f64 },
    Dot,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FamilyValue {
    angle: f64,
    base_point: [f64; 2],
    offset: [f64; 2],
    dashes: Vec<DashValue>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum FillValue {
    Solid,
    LinePattern {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default)]
        origin: [f64; 2],
        #[serde(default)]
        rotation: f64,
        #[serde(default = "one")]
        scale: f64,
        families: Vec<FamilyValue>,
    },
}
pub(super) fn encode(fill: &HatchFill) -> Value {
    let v = match fill {
        HatchFill::Solid => FillValue::Solid,
        HatchFill::LinePattern(p) => FillValue::LinePattern {
            name: p.name.clone(),
            description: p.description.clone(),
            origin: p.origin,
            rotation: p.rotation,
            scale: p.scale,
            families: p
                .families
                .iter()
                .map(|f| FamilyValue {
                    angle: f.angle,
                    base_point: f.base_point,
                    offset: f.offset,
                    dashes: f
                        .dashes
                        .iter()
                        .map(|d| match *d {
                            HatchDash::Dash { length } => DashValue::Dash { length },
                            HatchDash::Gap { length } => DashValue::Gap { length },
                            HatchDash::Dot => DashValue::Dot,
                        })
                        .collect(),
                })
                .collect(),
        },
    };
    serde_json::to_value(v).expect("validated fill values")
}
pub(super) fn decode(v: &Value) -> Result<HatchFill, serde_json::Error> {
    use serde::de::Error as _;
    if ["name", "description", "origin", "rotation", "scale"]
        .iter()
        .any(|k| v.get(k).is_some_and(Value::is_null))
    {
        return Err(serde_json::Error::custom(
            "omit absent pattern members; null is forbidden",
        ));
    }
    Ok(match serde_json::from_value::<FillValue>(v.clone())? {
        FillValue::Solid => HatchFill::Solid,
        FillValue::LinePattern {
            name,
            description,
            origin,
            rotation,
            scale,
            families,
        } => HatchFill::LinePattern(HatchLinePattern {
            name,
            description,
            origin,
            rotation,
            scale,
            families: families
                .into_iter()
                .map(|f| HatchLineFamily {
                    angle: f.angle,
                    base_point: f.base_point,
                    offset: f.offset,
                    dashes: f
                        .dashes
                        .into_iter()
                        .map(|d| match d {
                            DashValue::Dash { length } => HatchDash::Dash { length },
                            DashValue::Gap { length } => HatchDash::Gap { length },
                            DashValue::Dot => HatchDash::Dot,
                        })
                        .collect(),
                })
                .collect(),
        }),
    })
}
