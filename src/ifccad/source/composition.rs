use super::{IfccadCompositionPolicy, IfccadReport};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;
use std::fmt;

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;

        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON value without duplicate object keys")
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Bool(value)))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(Number::from(value))))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(Number::from(value))))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Number::from_f64(value)
                    .map(Value::Number)
                    .map(UniqueValue)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value.to_owned())))
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value)))
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }

            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<UniqueValue>()? {
                    values.push(value.0);
                }
                Ok(UniqueValue(Value::Array(values)))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some((key, value)) = entries.next_entry::<String, UniqueValue>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate JSON key {key:?}")));
                    }
                    values.insert(key, value.0);
                }
                Ok(UniqueValue(Value::Object(values)))
            }
        }

        deserializer.deserialize_any(UniqueVisitor)
    }
}

pub(super) fn compose(
    bytes: &[u8],
    policy: IfccadCompositionPolicy,
) -> Result<Value, IfccadReport> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let mut root = UniqueValue::deserialize(&mut deserializer)
        .map_err(|error| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-WIRE-001",
                "/",
                format!("invalid IFCX JSON: {error}"),
            )
        })?
        .0;
    deserializer.end().map_err(|error| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-WIRE-001",
            "/",
            format!("trailing IFCX JSON: {error}"),
        )
    })?;
    let data = root
        .get_mut("data")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-WIRE-001",
                "/data",
                "IFCX data must be an array",
            )
        })?;
    let mut indices = BTreeMap::<String, usize>::new();
    let mut composed = Vec::<Value>::new();
    for node in std::mem::take(data) {
        let path = node
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                crate::ifccad::diagnostics::failure(
                    "IFCCAD-WIRE-001",
                    "/data/path",
                    "IFCX node path must be a string",
                )
            })?
            .to_owned();
        if let Some(index) = indices.get(&path).copied() {
            merge_node(&mut composed[index], node, policy, &path)?;
        } else {
            indices.insert(path.clone(), composed.len());
            if policy == IfccadCompositionPolicy::RejectConflicts {
                composed.push(node);
            } else {
                let mut initial = Map::new();
                initial.insert("path".to_owned(), Value::String(path.clone()));
                let mut result = Value::Object(initial);
                merge_node(&mut result, node, policy, &path)?;
                composed.push(result);
            }
        }
    }
    *data = composed;
    Ok(root)
}

fn merge_node(
    existing: &mut Value,
    additional: Value,
    policy: IfccadCompositionPolicy,
    path: &str,
) -> Result<(), IfccadReport> {
    let target = existing.as_object_mut().ok_or_else(|| {
        crate::ifccad::diagnostics::failure("IFCCAD-WIRE-001", path, "IFCX node must be an object")
    })?;
    let fields = additional.as_object().ok_or_else(|| {
        crate::ifccad::diagnostics::failure("IFCCAD-WIRE-001", path, "IFCX node must be an object")
    })?;
    for (field, value) in fields {
        if field == "path" {
            continue;
        }
        if policy == IfccadCompositionPolicy::RejectConflicts {
            if let Some(present) = target.get_mut(field) {
                if let (Some(present_fields), Some(new_fields)) =
                    (present.as_object_mut(), value.as_object())
                {
                    for (key, new_value) in new_fields {
                        match present_fields.get(key) {
                            Some(old_value) if old_value != new_value => {
                                return Err(crate::ifccad::diagnostics::failure(
                                    "IFCCAD-WIRE-002",
                                    &format!("{path}/{field}/{key}"),
                                    format!("conflict at {path}/{field}/{key}"),
                                ));
                            }
                            Some(_) => {}
                            None => {
                                present_fields.insert(key.to_owned(), new_value.clone());
                            }
                        }
                    }
                } else if present != value {
                    return Err(crate::ifccad::diagnostics::failure(
                        "IFCCAD-WIRE-002",
                        &format!("{path}/{field}"),
                        "conflicting fragments",
                    ));
                }
            } else {
                target.insert(field.to_owned(), value.clone());
            }
            continue;
        }
        if matches!(field.as_str(), "children" | "inherits" | "attributes") {
            if let Some(new_fields) = value.as_object() {
                let present = target
                    .entry(field.to_owned())
                    .or_insert_with(|| Value::Object(Map::new()));
                if let Some(present_fields) = present.as_object_mut() {
                    for (key, new_value) in new_fields {
                        if field == "inherits" && new_value.is_null() {
                            present_fields.remove(key);
                        } else {
                            present_fields.insert(key.to_owned(), new_value.clone());
                        }
                    }
                    continue;
                }
            }
        }
        target.insert(field.to_owned(), value.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compose;
    use crate::ifccad::IfccadCompositionPolicy;
    use serde_json::json;

    #[test]
    fn parse_disjoint_fragments_compose() {
        let source = br#"{"header":{},"imports":[],"schemas":{},"data":[{"path":"/cad/d1/e1","attributes":{"ifccad::entity":{"layer":"L"}}},{"path":"/cad/d1/e1","attributes":{"ifccad::geom::circle":{"radius":2}}}]}"#;
        let result = compose(source, IfccadCompositionPolicy::LaterWins)
            .expect("disjoint fragments must compose");
        let nodes = result["data"].as_array().expect("data array");
        assert_eq!(nodes.len(), 1);
        assert_eq!(
            nodes[0]["attributes"]["ifccad::geom::circle"],
            json!({"radius":2})
        );
        assert!(nodes[0]["attributes"].get("ifccad::entity").is_some());
    }

    #[test]
    fn parse_duplicate_json_child_key_is_an_error() {
        let source = br#"{"data":[{"path":"/cad/d1/layout/1","children":{"0":"/cad/d1/e1","0":"/cad/d1/e2"}}]}"#;
        let error = compose(source, IfccadCompositionPolicy::LaterWins)
            .expect_err("duplicate object key must fail");
        assert!(error
            .errors
            .iter()
            .any(|message| message.contains("duplicate")));
    }

    #[test]
    fn parse_later_attribute_fragment_wins() {
        let source = br#"{"data":[{"path":"/cad/d1/e1","attributes":{"ifccad::geom::circle":{"radius":2}}},{"path":"/cad/d1/e1","attributes":{"ifccad::geom::circle":{"radius":3}}}]}"#;
        let result =
            compose(source, IfccadCompositionPolicy::LaterWins).expect("later fragment must win");
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["data"][0]["attributes"]["ifccad::geom::circle"],
            json!({"radius": 3})
        );
    }

    #[test]
    fn parse_later_child_fragment_wins() {
        let source = br#"{"data":[{"path":"node1","children":{"one":"node2"}},{"path":"node1","children":{"one":"node3"}}]}"#;
        let result =
            compose(source, IfccadCompositionPolicy::LaterWins).expect("later child must win");
        assert_eq!(result["data"][0]["children"]["one"], "node3");
    }

    #[test]
    fn parse_identical_fragments_are_harmless() {
        let source = br#"{"data":[{"path":"node1","attributes":{"test::a":"A"}},{"path":"node1","attributes":{"test::a":"A"}}]}"#;
        let result = compose(source, IfccadCompositionPolicy::LaterWins)
            .expect("identical fragments must compose");
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
        assert_eq!(result["data"][0]["attributes"]["test::a"], "A");
    }

    #[test]
    fn parse_null_markers_follow_current_upstream_flattening() {
        for field in ["children", "inherits", "attributes"] {
            let mut first = json!({"path": "node1"});
            first[field] = json!({"key": "A"});
            let mut second = json!({"path": "node1"});
            second[field] = json!({"key": null});
            let source = serde_json::to_vec(&json!({"data": [first, second]})).unwrap();
            let result = compose(&source, IfccadCompositionPolicy::LaterWins)
                .expect("null marker must compose");
            if field == "inherits" {
                assert!(result["data"][0][field].get("key").is_none());
            } else {
                assert!(result["data"][0][field]["key"].is_null());
            }
        }
    }

    #[test]
    fn strict_policy_accepts_disjoint_and_equal_fragments_but_rejects_overwrites() {
        let source = br#"{"data":[{"path":"node1","attributes":{"test::a":"A"}},{"path":"node1","attributes":{"test::a":"A","test::b":"B"}}]}"#;
        let result = compose(source, IfccadCompositionPolicy::RejectConflicts)
            .expect("disjoint and identical keys must compose");
        assert_eq!(result["data"][0]["attributes"]["test::b"], "B");

        let conflicting = br#"{"data":[{"path":"node1","attributes":{"test::a":"A"}},{"path":"node1","attributes":{"test::a":"B"}}]}"#;
        let error = compose(conflicting, IfccadCompositionPolicy::RejectConflicts)
            .expect_err("strict mode must reject overwrite");
        assert!(error
            .errors
            .iter()
            .any(|e| e.contains("conflict at node1/attributes/test::a")));
    }
}
