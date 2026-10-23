use crate::geometry_kernel::hatch::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
fn one() -> f64 {
    1.
}
fn zero(v: &f64) -> bool {
    v.to_bits() == 0
}
fn is_one(v: &f64) -> bool {
    *v == 1.
}
fn is_false(v: &bool) -> bool {
    !*v
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClassicValue {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    preferred_fill: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    creation_type: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    description: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    origin: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "zero")]
    rotation: f64,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    scale: f64,
    #[serde(default, skip_serializing_if = "is_false")]
    double: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::presentation::present"
    )]
    definition_lines: Option<Vec<DefinitionLineValue>>,
}
pub(super) fn decode_classic(v: &Value) -> Result<HatchClassicSettings, serde_json::Error> {
    use serde::de::Error;
    let c: ClassicValue = serde_json::from_value(v.clone())?;
    Ok(HatchClassicSettings {
        preferred_fill: c
            .preferred_fill
            .map(|s| match s.as_str() {
                "solid" => Ok(HatchClassicFill::Solid),
                "linePattern" => Ok(HatchClassicFill::LinePattern),
                _ => Err(serde_json::Error::custom("invalid preferredFill")),
            })
            .transpose()?,
        creation_type: c
            .creation_type
            .map(|s| match s.as_str() {
                "userDefined" => Ok(HatchCreationType::UserDefined),
                "predefined" => Ok(HatchCreationType::Predefined),
                "custom" => Ok(HatchCreationType::Custom),
                _ => Err(serde_json::Error::custom("invalid creationType")),
            })
            .transpose()?,
        name: c.name,
        description: c.description,
        origin: c.origin,
        rotation: c.rotation,
        scale: c.scale,
        double: c.double,
        definition_lines: c.definition_lines.map(|lines| {
            lines
                .into_iter()
                .map(|l| HatchPatternDefinitionLine {
                    angle: l.angle,
                    base_point: l.base_point,
                    offset_vector: l.offset_vector,
                    dash_lengths: l.dash_lengths,
                })
                .collect()
        }),
    })
}
pub(super) fn encode_classic(c: &HatchClassicSettings) -> Value {
    let value = ClassicValue {
        preferred_fill: c.preferred_fill.map(|v| {
            match v {
                HatchClassicFill::Solid => "solid",
                HatchClassicFill::LinePattern => "linePattern",
            }
            .into()
        }),
        creation_type: c.creation_type.map(|v| {
            match v {
                HatchCreationType::UserDefined => "userDefined",
                HatchCreationType::Predefined => "predefined",
                HatchCreationType::Custom => "custom",
            }
            .into()
        }),
        name: c.name.clone(),
        description: c.description.clone(),
        origin: c.origin,
        rotation: c.rotation,
        scale: c.scale,
        double: c.double,
        definition_lines: c.definition_lines.as_ref().map(|lines| {
            lines
                .iter()
                .map(|l| DefinitionLineValue {
                    angle: l.angle,
                    base_point: l.base_point,
                    offset_vector: l.offset_vector,
                    dash_lengths: l.dash_lengths.clone(),
                })
                .collect()
        }),
    };
    serde_json::to_value(value).expect("validated classic settings")
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConstructionValue {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    seed_points: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "zero")]
    pixel_size: f64,
}
pub(super) fn decode_construction(
    v: &Value,
) -> Result<HatchBoundaryConstruction, serde_json::Error> {
    let c: ConstructionValue = serde_json::from_value(v.clone())?;
    Ok(HatchBoundaryConstruction {
        seed_points: c.seed_points,
        pixel_size: c.pixel_size,
    })
}
pub(super) fn encode_construction(c: &HatchBoundaryConstruction) -> Value {
    serde_json::to_value(ConstructionValue {
        seed_points: c.seed_points.clone(),
        pixel_size: c.pixel_size,
    })
    .expect("validated construction")
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DefinitionLineValue {
    angle: f64,
    base_point: [f64; 2],
    offset_vector: [f64; 2],
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dash_lengths: Vec<f64>,
}
