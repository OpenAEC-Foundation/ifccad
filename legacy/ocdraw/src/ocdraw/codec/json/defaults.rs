//! Whole-column omission of explicitly selected defaults; mixed columns remain full.
use serde_json::{Map, Value};

pub(super) fn is_positive_zero(value: f64) -> bool {
    value.to_bits() == 0.0_f64.to_bits()
}
pub(super) fn is_one(value: f64) -> bool {
    value.to_bits() == 1.0_f64.to_bits()
}
pub(super) fn number(value: &Value, default: f64) -> bool {
    value
        .as_f64()
        .is_some_and(|v| v.to_bits() == default.to_bits())
}
pub(super) fn omit_default_column(
    columns: &mut Map<String, Value>,
    field: &str,
    predicate: impl Fn(&Value) -> bool,
) {
    if columns
        .get(field)
        .and_then(Value::as_array)
        .is_some_and(|rows| rows.iter().all(predicate))
    {
        columns.remove(field);
    }
}
