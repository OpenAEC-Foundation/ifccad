//! Closed physical presentation values for the independent IFCCAD codec.
use crate::ifccad::{
    IfccadColor, IfccadPointDisplay, IfccadPointGlyph, IfccadPointSize, IfccadReport,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ColorValue {
    rgb: [u8; 3],
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    indexed_color: Option<IndexedColor>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    named_color: Option<NamedColor>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedColor {
    system: String,
    index: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedColor {
    catalog: String,
    name: String,
}
pub(super) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
impl Serialize for IfccadColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ColorValue {
            rgb: self.rgb,
            indexed_color: self.indexed.as_ref().map(|(system, index)| IndexedColor {
                system: system.clone(),
                index: *index,
            }),
            named_color: self.named.as_ref().map(|(catalog, name)| NamedColor {
                catalog: catalog.clone(),
                name: name.clone(),
            }),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for IfccadColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = ColorValue::deserialize(deserializer)?;
        let color = Self {
            rgb: value.rgb,
            indexed: value.indexed_color.map(|c| (c.system, c.index)),
            named: value.named_color.map(|c| (c.catalog, c.name)),
        };
        if !color.is_valid() {
            return Err(serde::de::Error::custom(
                "color identity strings must be nonempty",
            ));
        }
        Ok(color)
    }
}
pub(super) fn read_color(value: &serde_json::Value) -> Option<IfccadColor> {
    serde_json::from_value(value.clone()).ok()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PointDisplayValue {
    form: PointFormValue,
    size: PointSizeValue,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PointFormValue {
    glyph: String,
    circle: bool,
    square: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum PointSizeValue {
    DefaultFivePercent {},
    Absolute { value: f64 },
    ViewportPercent { value: f64 },
}

pub(super) fn decode_point_display(
    value: &serde_json::Value,
) -> Result<IfccadPointDisplay, IfccadReport> {
    let value: PointDisplayValue = serde_json::from_value(value.clone())
        .map_err(|e| IfccadReport::one(format!("invalid point display: {e}")))?;
    let glyph = match value.form.glyph.as_str() {
        "dot" => IfccadPointGlyph::Dot,
        "hidden" => IfccadPointGlyph::Hidden,
        "plus" => IfccadPointGlyph::Plus,
        "cross" => IfccadPointGlyph::Cross,
        "shortLine" => IfccadPointGlyph::ShortLine,
        _ => return Err(IfccadReport::one("invalid point display glyph")),
    };
    Ok(IfccadPointDisplay {
        glyph,
        circle: value.form.circle,
        square: value.form.square,
        size: match value.size {
            PointSizeValue::DefaultFivePercent {} => IfccadPointSize::DefaultFivePercent,
            PointSizeValue::Absolute { value } => IfccadPointSize::Absolute(value),
            PointSizeValue::ViewportPercent { value } => IfccadPointSize::ViewportPercent(value),
        },
    })
}
pub(super) fn encode_point_display(value: IfccadPointDisplay) -> serde_json::Value {
    let glyph = match value.glyph {
        IfccadPointGlyph::Dot => "dot",
        IfccadPointGlyph::Hidden => "hidden",
        IfccadPointGlyph::Plus => "plus",
        IfccadPointGlyph::Cross => "cross",
        IfccadPointGlyph::ShortLine => "shortLine",
    }
    .to_owned();
    let size = match value.size {
        IfccadPointSize::DefaultFivePercent => PointSizeValue::DefaultFivePercent {},
        IfccadPointSize::Absolute(value) => PointSizeValue::Absolute { value },
        IfccadPointSize::ViewportPercent(value) => PointSizeValue::ViewportPercent { value },
    };
    serde_json::to_value(PointDisplayValue {
        form: PointFormValue {
            glyph,
            circle: value.circle,
            square: value.square,
        },
        size,
    })
    .expect("validated point display")
}
