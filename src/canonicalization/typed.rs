use super::{canonicalize, CanonicalValue, CanonicalizationError, CanonicalizationErrorCode};
use serde_json::Value;
use std::collections::BTreeMap;

/// Decode the language-neutral typed-value representation used by IFCCAD.
pub fn decode_typed_value(value: &Value) -> Result<CanonicalValue, CanonicalizationError> {
    let object = value.as_object().ok_or_else(|| {
        error(
            CanonicalizationErrorCode::TypeInvalid,
            "typed value must be an object with a string kind",
        )
    })?;
    let kind = object.get("kind").and_then(Value::as_str).ok_or_else(|| {
        error(
            CanonicalizationErrorCode::TypeInvalid,
            "typed value must be an object with a string kind",
        )
    })?;

    match kind {
        "null" => Ok(CanonicalValue::Null),
        "bool" => object
            .get("value")
            .and_then(Value::as_bool)
            .map(CanonicalValue::Bool)
            .ok_or_else(|| {
                error(
                    CanonicalizationErrorCode::BoolInvalid,
                    "bool value must be boolean",
                )
            }),
        "int" => decode_numeric_text(
            object.get("value"),
            CanonicalValue::Integer,
            CanonicalizationErrorCode::IntegerInvalid,
            "integer value must use canonical decimal text",
        ),
        "float" => decode_numeric_text(
            object.get("value"),
            CanonicalValue::Float,
            CanonicalizationErrorCode::FloatInvalid,
            "float value must use finite canonical hexadecimal text",
        ),
        "string" => decode_text(
            object.get("value"),
            CanonicalValue::String,
            CanonicalizationErrorCode::StringInvalid,
            "string value must be text",
        ),
        "enum" => decode_text(
            object.get("value"),
            CanonicalValue::String,
            CanonicalizationErrorCode::EnumInvalid,
            "enum value must be text",
        ),
        "sequence" => decode_sequence(object.get("items")),
        "mapping" => decode_mapping(object.get("entries")),
        "record" => decode_record(object.get("fields")),
        _ => Err(error(
            CanonicalizationErrorCode::TypeUnsupported,
            "typed value kind is not supported by IFCCAD canonicalization",
        )),
    }
}

/// Decode and immediately serialize an IFCCAD typed value to canonical UTF-8.
pub fn canonicalize_typed_value(value: &Value) -> Result<Vec<u8>, CanonicalizationError> {
    canonicalize(&decode_typed_value(value)?)
}

fn decode_numeric_text(
    value: Option<&Value>,
    constructor: fn(String) -> CanonicalValue,
    code: CanonicalizationErrorCode,
    message: &'static str,
) -> Result<CanonicalValue, CanonicalizationError> {
    let text = value
        .and_then(Value::as_str)
        .ok_or_else(|| error(code, message))?;
    let result = constructor(text.to_owned());
    canonicalize(&result)?;
    Ok(result)
}

fn decode_text(
    value: Option<&Value>,
    constructor: fn(String) -> CanonicalValue,
    code: CanonicalizationErrorCode,
    message: &'static str,
) -> Result<CanonicalValue, CanonicalizationError> {
    value
        .and_then(Value::as_str)
        .map(|value| constructor(value.to_owned()))
        .ok_or_else(|| error(code, message))
}

fn decode_sequence(value: Option<&Value>) -> Result<CanonicalValue, CanonicalizationError> {
    let items = value.and_then(Value::as_array).ok_or_else(|| {
        error(
            CanonicalizationErrorCode::SequenceInvalid,
            "sequence items must be an array",
        )
    })?;
    let values = items
        .iter()
        .map(decode_typed_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CanonicalValue::Sequence(values))
}

fn decode_mapping(value: Option<&Value>) -> Result<CanonicalValue, CanonicalizationError> {
    let entries = value.and_then(Value::as_array).ok_or_else(|| {
        error(
            CanonicalizationErrorCode::MappingInvalid,
            "mapping entries must be an array",
        )
    })?;
    let mut values = BTreeMap::new();
    for entry in entries {
        let pair = entry
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or_else(|| {
                error(
                    CanonicalizationErrorCode::MappingInvalid,
                    "mapping entries must be key-value pairs",
                )
            })?;
        let key = pair[0].as_str().ok_or_else(|| {
            error(
                CanonicalizationErrorCode::MappingKeyInvalid,
                "mapping keys must be strings",
            )
        })?;
        if values.contains_key(key) {
            return Err(error(
                CanonicalizationErrorCode::MappingKeyDuplicate,
                "mapping keys must be unique",
            ));
        }
        let decoded = decode_typed_value(&pair[1])?;
        values.insert(key.to_owned(), decoded);
    }
    Ok(CanonicalValue::Mapping(values))
}

fn decode_record(value: Option<&Value>) -> Result<CanonicalValue, CanonicalizationError> {
    let fields = value.and_then(Value::as_array).ok_or_else(|| {
        error(
            CanonicalizationErrorCode::RecordInvalid,
            "record fields must be an array",
        )
    })?;
    let mut values = BTreeMap::new();
    for field in fields {
        let pair = field
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or_else(|| {
                error(
                    CanonicalizationErrorCode::RecordFieldInvalid,
                    "record fields must be name-value pairs",
                )
            })?;
        let name = pair[0]
            .as_str()
            .filter(|name| is_record_name(name))
            .ok_or_else(|| {
                error(
                    CanonicalizationErrorCode::RecordFieldInvalid,
                    "record field name is invalid",
                )
            })?;
        if values.contains_key(name) {
            return Err(error(
                CanonicalizationErrorCode::RecordFieldInvalid,
                "record field names must be unique",
            ));
        }
        let decoded = decode_typed_value(&pair[1])?;
        values.insert(name.to_owned(), decoded);
    }
    Ok(CanonicalValue::Mapping(values))
}

fn is_record_name(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic())
        && characters.all(|value| value == '_' || value.is_alphanumeric())
        && !is_python_keyword(value)
}

fn is_python_keyword(value: &str) -> bool {
    matches!(
        value,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

fn error(code: CanonicalizationErrorCode, message: &'static str) -> CanonicalizationError {
    CanonicalizationError::new(code, message)
}
