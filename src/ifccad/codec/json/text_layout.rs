//! Closed IFCCAD single-line layout payload.
use crate::text::*;
use serde_json::{json, Value};
pub(super) fn encode_layout(layout: TextLayout) -> Value {
    match layout {
        TextLayout::Anchored {
            horizontal,
            vertical,
            height,
            width_factor,
        } => {
            json!({"kind": "anchored", "horizontal": match horizontal { TextHorizontalAlignment::Left => "left", TextHorizontalAlignment::Center => "center", TextHorizontalAlignment::Right => "right" }, "vertical": match vertical { TextVerticalAlignment::Baseline => "baseline", TextVerticalAlignment::Bottom => "bottom", TextVerticalAlignment::Middle => "middle", TextVerticalAlignment::Top => "top" }, "height": height, "widthFactor": width_factor})
        }
        TextLayout::WholeTextMiddle {
            height,
            width_factor,
        } => json!({"kind": "wholeTextMiddle", "height": height, "widthFactor": width_factor}),
        TextLayout::Aligned {
            length,
            width_factor,
        } => json!({"kind": "aligned", "length": length, "widthFactor": width_factor}),
        TextLayout::Fit { length, height } => {
            json!({"kind": "fit", "length": length, "height": height})
        }
    }
}
pub(super) fn decode_layout(v: &Value) -> Option<TextLayout> {
    Some(match v.get("kind")?.as_str()? {
        "anchored" => TextLayout::Anchored {
            horizontal: match v.get("horizontal")?.as_str()? {
                "left" => TextHorizontalAlignment::Left,
                "center" => TextHorizontalAlignment::Center,
                "right" => TextHorizontalAlignment::Right,
                _ => return None,
            },
            vertical: match v.get("vertical")?.as_str()? {
                "baseline" => TextVerticalAlignment::Baseline,
                "bottom" => TextVerticalAlignment::Bottom,
                "middle" => TextVerticalAlignment::Middle,
                "top" => TextVerticalAlignment::Top,
                _ => return None,
            },
            height: v.get("height")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "wholeTextMiddle" => TextLayout::WholeTextMiddle {
            height: v.get("height")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "aligned" => TextLayout::Aligned {
            length: v.get("length")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "fit" => TextLayout::Fit {
            length: v.get("length")?.as_f64()?,
            height: v.get("height")?.as_f64()?,
        },
        _ => return None,
    })
}
