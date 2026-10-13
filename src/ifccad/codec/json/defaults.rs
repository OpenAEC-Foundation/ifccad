//! Exact omission of explicitly selected physical defaults, never authored overrides.
use serde_json::Value;

pub(super) fn is_positive_zero(value: f64) -> bool {
    value.to_bits() == 0.0_f64.to_bits()
}
pub(super) fn is_one(value: f64) -> bool {
    value.to_bits() == 1.0_f64.to_bits()
}
pub(super) fn omit_number(value: &mut Value, field: &str, default: f64) {
    if value
        .get(field)
        .and_then(Value::as_f64)
        .is_some_and(|v| v.to_bits() == default.to_bits())
    {
        value.as_object_mut().expect("encoded object").remove(field);
    }
}
pub(super) fn omit_bool(value: &mut Value, field: &str, default: bool) {
    if value.get(field).and_then(Value::as_bool) == Some(default) {
        value.as_object_mut().expect("encoded object").remove(field);
    }
}
