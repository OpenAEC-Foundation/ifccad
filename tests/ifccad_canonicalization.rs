use ifccad::canonicalization::{canonicalize, canonicalize_typed_value, CanonicalValue};
use ifccad::conformance::{bundled_conformance_root, verify_canonicalization_vectors};
use serde_json::json;

#[test]
fn canonicalizes_supported_scalars_to_exact_utf8() {
    let cases = [
        (CanonicalValue::Null, r#"{"type":"null"}"#),
        (
            CanonicalValue::Bool(false),
            r#"{"type":"bool","value":false}"#,
        ),
        (
            CanonicalValue::Integer("18446744073709551615".to_owned()),
            r#"{"type":"int","value":"18446744073709551615"}"#,
        ),
        (
            CanonicalValue::Float("-0x0.0p+0".to_owned()),
            r#"{"type":"float","value":"-0x0.0p+0"}"#,
        ),
        (
            CanonicalValue::String("café".to_owned()),
            r#"{"type":"string","value":"café"}"#,
        ),
    ];

    for (value, expected) in cases {
        assert_eq!(
            canonicalize(&value).expect("canonical value"),
            expected.as_bytes()
        );
    }
}

#[test]
fn rejects_noncanonical_numeric_text_with_stable_codes() {
    let cases = [
        (
            CanonicalValue::Integer("01".to_owned()),
            "VECTOR_INTEGER_INVALID",
        ),
        (
            CanonicalValue::Integer("-0".to_owned()),
            "VECTOR_INTEGER_INVALID",
        ),
        (
            CanonicalValue::Float("1.0".to_owned()),
            "VECTOR_FLOAT_INVALID",
        ),
        (
            CanonicalValue::Float("nan".to_owned()),
            "VECTOR_FLOAT_INVALID",
        ),
    ];

    for (value, expected_code) in cases {
        let error = canonicalize(&value).expect_err("invalid canonical value");
        assert_eq!(error.code().as_str(), expected_code);
    }
}

#[test]
fn canonicalizes_typed_compounds_with_stable_ordering() {
    let cases = [
        (
            json!({
                "kind": "sequence",
                "items": [
                    {"kind": "int", "value": "1"},
                    {"kind": "enum", "value": "value"},
                    {"kind": "null"},
                ],
            }),
            r#"{"type":"sequence","value":[{"type":"int","value":"1"},{"type":"string","value":"value"},{"type":"null"}]}"#,
        ),
        (
            json!({
                "kind": "mapping",
                "entries": [
                    ["z", {"kind": "int", "value": "2"}],
                    ["a", {"kind": "string", "value": "first"}],
                ],
            }),
            r#"{"type":"mapping","value":[["a",{"type":"string","value":"first"}],["z",{"type":"int","value":"2"}]]}"#,
        ),
        (
            json!({
                "kind": "record",
                "fields": [
                    ["z", {"kind": "float", "value": "-0x0.0p+0"}],
                    ["name", {"kind": "string", "value": "café"}],
                    ["mode", {"kind": "enum", "value": "value"}],
                ],
            }),
            r#"{"type":"mapping","value":[["mode",{"type":"string","value":"value"}],["name",{"type":"string","value":"café"}],["z",{"type":"float","value":"-0x0.0p+0"}]]}"#,
        ),
    ];

    for (value, expected) in cases {
        assert_eq!(
            canonicalize_typed_value(&value).expect("typed canonical value"),
            expected.as_bytes()
        );
    }
}

#[test]
fn rejects_malformed_typed_values_with_stable_codes() {
    let cases = [
        (json!(null), "VECTOR_TYPE_INVALID"),
        (json!({"kind": "bool", "value": 1}), "VECTOR_BOOL_INVALID"),
        (
            json!({"kind": "string", "value": false}),
            "VECTOR_STRING_INVALID",
        ),
        (
            json!({"kind": "enum", "value": false}),
            "VECTOR_ENUM_INVALID",
        ),
        (
            json!({"kind": "sequence", "items": {}}),
            "VECTOR_SEQUENCE_INVALID",
        ),
        (
            json!({"kind": "mapping", "entries": [["a"]]}),
            "VECTOR_MAPPING_INVALID",
        ),
        (
            json!({"kind": "mapping", "entries": [[1, {"kind": "null"}]]}),
            "VECTOR_MAPPING_KEY_INVALID",
        ),
        (
            json!({
                "kind": "mapping",
                "entries": [
                    ["a", {"kind": "null"}],
                    ["a", null],
                ],
            }),
            "VECTOR_MAPPING_KEY_DUPLICATE",
        ),
        (
            json!({"kind": "record", "fields": {}}),
            "VECTOR_RECORD_INVALID",
        ),
        (
            json!({"kind": "record", "fields": [["not-valid!", {"kind": "null"}]]}),
            "VECTOR_RECORD_FIELD_INVALID",
        ),
        (
            json!({
                "kind": "record",
                "fields": [
                    ["name", {"kind": "null"}],
                    ["name", null],
                ],
            }),
            "VECTOR_RECORD_FIELD_INVALID",
        ),
        (
            json!({"kind": "bytes", "value": "AA=="}),
            "VECTOR_TYPE_UNSUPPORTED",
        ),
    ];

    for (value, expected_code) in cases {
        let error = canonicalize_typed_value(&value).expect_err("malformed typed value");
        assert_eq!(error.code().as_str(), expected_code);
    }
}

#[test]
fn verifies_every_bundled_canonicalization_vector() {
    let path = bundled_conformance_root()
        .join("vectors")
        .join("canonicalization.json");
    let verified = verify_canonicalization_vectors(path).expect("canonicalization vectors");

    assert_eq!(verified, 23);
}
