use ifccad::canonicalization::{fingerprint, fingerprint_typed_value, CanonicalValue};
use ifccad::conformance::{bundled_conformance_root, verify_fingerprint_vectors};
use serde_json::json;

#[test]
fn fingerprints_exact_canonical_bytes_with_sha256() {
    let actual = fingerprint(&CanonicalValue::Bool(true)).expect("fingerprint canonical value");

    assert_eq!(
        actual,
        "sha256:e89ea334229178466a873d14c56bea013221be78be952866c5ea904bad274b87"
    );
}

#[test]
fn fingerprint_is_independent_of_mapping_input_order() {
    let first = json!({
        "kind": "mapping",
        "entries": [
            ["z", {"kind": "int", "value": "2"}],
            ["a", {"kind": "string", "value": "first"}],
        ],
    });
    let second = json!({
        "kind": "mapping",
        "entries": [
            ["a", {"kind": "string", "value": "first"}],
            ["z", {"kind": "int", "value": "2"}],
        ],
    });

    let first = fingerprint_typed_value(&first).expect("first mapping fingerprint");
    let second = fingerprint_typed_value(&second).expect("second mapping fingerprint");

    assert_eq!(first, second);
    assert_eq!(
        first,
        "sha256:b2b46f8c76ba14918f7073587c97b66f95cd71625b5ac080ab6827c2cfb8f06b"
    );
}

#[test]
fn verifies_every_bundled_fingerprint_vector() {
    let path = bundled_conformance_root()
        .join("vectors")
        .join("fingerprints.json");
    let verified = verify_fingerprint_vectors(path).expect("fingerprint vectors");

    assert_eq!(verified, 9);
}
