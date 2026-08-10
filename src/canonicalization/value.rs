use super::{CanonicalizationError, CanonicalizationErrorCode};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// A value in the language-neutral IFCCAD canonicalization domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalValue {
    Null,
    Bool(bool),
    Integer(String),
    Float(String),
    String(String),
    Sequence(Vec<CanonicalValue>),
    Mapping(BTreeMap<String, CanonicalValue>),
}

pub fn canonicalize(value: &CanonicalValue) -> Result<Vec<u8>, CanonicalizationError> {
    let normalized = normalize(value)?;
    Ok(serde_json::to_vec(&normalized).expect("canonical JSON value is serializable"))
}

fn normalize(value: &CanonicalValue) -> Result<Value, CanonicalizationError> {
    match value {
        CanonicalValue::Null => Ok(object("null", None)),
        CanonicalValue::Bool(value) => Ok(object("bool", Some(Value::Bool(*value)))),
        CanonicalValue::Integer(value) => {
            if !is_canonical_integer(value) {
                return Err(CanonicalizationError::new(
                    CanonicalizationErrorCode::IntegerInvalid,
                    "integer value must use canonical decimal text",
                ));
            }
            Ok(object("int", Some(Value::String(value.clone()))))
        }
        CanonicalValue::Float(value) => {
            if !is_canonical_float(value) {
                return Err(CanonicalizationError::new(
                    CanonicalizationErrorCode::FloatInvalid,
                    "float value must use finite canonical hexadecimal text",
                ));
            }
            Ok(object("float", Some(Value::String(value.clone()))))
        }
        CanonicalValue::String(value) => Ok(object("string", Some(Value::String(value.clone())))),
        CanonicalValue::Sequence(values) => Ok(object(
            "sequence",
            Some(Value::Array(
                values
                    .iter()
                    .map(normalize)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        )),
        CanonicalValue::Mapping(values) => {
            let entries = values
                .iter()
                .map(|(key, value)| {
                    Ok(Value::Array(vec![
                        Value::String(key.clone()),
                        normalize(value)?,
                    ]))
                })
                .collect::<Result<Vec<_>, CanonicalizationError>>()?;
            Ok(object("mapping", Some(Value::Array(entries))))
        }
    }
}

fn object(kind: &str, value: Option<Value>) -> Value {
    let mut object = Map::new();
    object.insert("type".to_owned(), Value::String(kind.to_owned()));
    if let Some(value) = value {
        object.insert("value".to_owned(), value);
    }
    Value::Object(object)
}

fn is_canonical_integer(value: &str) -> bool {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty() || !digits.bytes().all(|value| value.is_ascii_digit()) {
        return false;
    }
    if digits == "0" {
        return !value.starts_with('-');
    }
    !digits.starts_with('0')
}

fn is_canonical_float(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let Some((significand, exponent)) = unsigned.split_once('p') else {
        return false;
    };
    if exponent.contains('p') || !is_canonical_exponent(exponent) {
        return false;
    }

    let Some(significand) = significand.strip_prefix("0x") else {
        return false;
    };
    let Some((leading, fraction)) = significand.split_once('.') else {
        return false;
    };
    if leading.len() != 1 || fraction.is_empty() {
        return false;
    }
    if !fraction
        .bytes()
        .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        return false;
    }

    if leading == "0" && fraction == "0" {
        return exponent == "+0";
    }
    if fraction.len() != 13 {
        return false;
    }

    let Ok(exponent_value) = exponent.parse::<i32>() else {
        return false;
    };
    match leading {
        "0" => exponent_value == -1022,
        "1" => (-1022..=1023).contains(&exponent_value),
        _ => false,
    }
}

fn is_canonical_exponent(value: &str) -> bool {
    let Some(digits) = value.strip_prefix(['+', '-']) else {
        return false;
    };
    !digits.is_empty()
        && digits.bytes().all(|value| value.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'))
}
