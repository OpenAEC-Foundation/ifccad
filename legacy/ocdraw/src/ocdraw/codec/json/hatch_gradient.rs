use crate::geometry_kernel::hatch::*;
use crate::ocdraw::DrawingHatchGradient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TypeValue {
    Linear,
    Cylinder,
    InverseCylinder,
    Spherical,
    InverseSpherical,
    Hemispherical,
    InverseHemispherical,
    Curved,
    InverseCurved,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ModeValue {
    OneColor,
    #[default]
    TwoColor,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GradientValue {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    gradient_type: Option<TypeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    colors: Option<[Value; 2]>,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    shift: f64,
    #[serde(default)]
    color_mode: ModeValue,
    #[serde(default)]
    tint: f64,
}
pub(super) fn encode(g: &DrawingHatchGradient) -> Value {
    let v = GradientValue {
        gradient_type: g.gradient_type.map(|t| match t {
            HatchGradientType::Linear => TypeValue::Linear,
            HatchGradientType::Cylinder => TypeValue::Cylinder,
            HatchGradientType::InverseCylinder => TypeValue::InverseCylinder,
            HatchGradientType::Spherical => TypeValue::Spherical,
            HatchGradientType::InverseSpherical => TypeValue::InverseSpherical,
            HatchGradientType::Hemispherical => TypeValue::Hemispherical,
            HatchGradientType::InverseHemispherical => TypeValue::InverseHemispherical,
            HatchGradientType::Curved => TypeValue::Curved,
            HatchGradientType::InverseCurved => TypeValue::InverseCurved,
        }),
        colors: g
            .colors
            .as_ref()
            .map(|cs| cs.each_ref().map(super::encode_color::encode_color)),
        rotation: g.rotation,
        shift: g.shift,
        tint: g.tint,
        color_mode: match g.color_mode {
            HatchGradientColorMode::OneColor => ModeValue::OneColor,
            HatchGradientColorMode::TwoColor => ModeValue::TwoColor,
        },
    };
    serde_json::to_value(v).expect("validated gradient")
}
pub(super) fn decode(v: &Value) -> Result<DrawingHatchGradient, serde_json::Error> {
    use serde::de::Error as _;
    if ["type", "colors", "rotation", "shift", "colorMode", "tint"]
        .iter()
        .any(|k| v.get(k).is_some_and(Value::is_null))
    {
        return Err(serde_json::Error::custom("gradient fields cannot be null"));
    }
    let g: GradientValue = serde_json::from_value(v.clone())?;
    let colors = g
        .colors
        .as_ref()
        .map(|cs| {
            let read = |v: &Value| {
                super::decode_appearance::color(v)
                    .ok_or_else(|| serde_json::Error::custom("invalid concrete gradient color"))
            };
            Ok::<_, serde_json::Error>([read(&cs[0])?, read(&cs[1])?])
        })
        .transpose()?;
    Ok(HatchGradient {
        gradient_type: g.gradient_type.map(|t| match t {
            TypeValue::Linear => HatchGradientType::Linear,
            TypeValue::Cylinder => HatchGradientType::Cylinder,
            TypeValue::InverseCylinder => HatchGradientType::InverseCylinder,
            TypeValue::Spherical => HatchGradientType::Spherical,
            TypeValue::InverseSpherical => HatchGradientType::InverseSpherical,
            TypeValue::Hemispherical => HatchGradientType::Hemispherical,
            TypeValue::InverseHemispherical => HatchGradientType::InverseHemispherical,
            TypeValue::Curved => HatchGradientType::Curved,
            TypeValue::InverseCurved => HatchGradientType::InverseCurved,
        }),
        colors,
        rotation: g.rotation,
        shift: g.shift,
        tint: g.tint,
        color_mode: match g.color_mode {
            ModeValue::OneColor => HatchGradientColorMode::OneColor,
            ModeValue::TwoColor => HatchGradientColorMode::TwoColor,
        },
    })
}
