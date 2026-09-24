use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const LOGICAL: &str = include_str!("../../../../schemas/ifcdr/registry-0.9.0.json");
const MAPPING: &str = include_str!("../../../../schemas/ifcdr/json-mapping-0.9.0.json");
const LOGICAL_0_10: &str = include_str!("../../../../schemas/ifcdr/registry-0.10.0.json");
const MAPPING_0_10: &str = include_str!("../../../../schemas/ifcdr/json-mapping-0.10.0.json");
const LOGICAL_0_11: &str = include_str!("../../../../schemas/ifcdr/registry-0.11.0.json");
const MAPPING_0_11: &str = include_str!("../../../../schemas/ifcdr/json-mapping-0.11.0.json");

fn validate_default_markers(logical: &Value, mapping: &Value) -> Result<(), String> {
    for stream in mapping["streams"].as_array().into_iter().flatten() {
        for field in stream["fields"].as_array().into_iter().flatten() {
            let Some(marker) = field.get("nullEncoding") else {
                continue;
            };
            let name = field["logical"].as_str().unwrap_or("");
            let invalid = || format!("invalid whole-default row marker for {name}");
            if marker != "logicalDefault"
                || field["omission"] != "logicalDefault"
                || field.get("encoding").is_some()
            {
                return Err(invalid());
            }
            let (owner, local) = name.split_once('.').ok_or_else(invalid)?;
            if stream["name"] != owner {
                return Err(invalid());
            }
            let logical_field = logical["streams"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|s| s["name"] == owner)
                .and_then(|s| s["fields"].as_array())
                .and_then(|fields| fields.iter().find(|f| f["name"] == local))
                .ok_or_else(invalid)?;
            if logical_field["nullable"] != false
                || logical_field.get("default").is_none_or(Value::is_null)
            {
                return Err(invalid());
            }
            let kind = logical_field["valueType"].as_str().ok_or_else(invalid)?;
            if logical["types"][kind]["kind"] == "sequence" {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod spatial_mapping_tests {
    use super::*;

    #[test]
    fn row_default_markers_require_a_nonnullable_defaulted_logical_field() {
        let logical: Value = serde_json::from_str(include_str!(
            "../../../../schemas/ifcdr/registry-0.8.0.json"
        ))
        .unwrap();
        let mapping: Value = serde_json::from_str(include_str!(
            "../../../../schemas/ifcdr/json-mapping-0.8.0.json"
        ))
        .unwrap();
        assert!(validate_default_markers(&logical, &mapping).is_ok());
        for mutation in ["nullable", "no_default", "wrong_type"] {
            let mut changed = logical.clone();
            let field = changed["streams"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|stream| stream["name"] == "polyline")
                .unwrap()["fields"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|field| field["name"] == "placement")
                .unwrap();
            match mutation {
                "nullable" => field["nullable"] = serde_json::json!(true),
                "no_default" => {
                    field.as_object_mut().unwrap().remove("default");
                }
                _ => field["valueType"] = serde_json::json!("vertices"),
            }
            assert!(
                validate_default_markers(&changed, &mapping).is_err(),
                "{mutation}"
            );
        }
        for mutation in ["omission", "unknown_logical", "range", "marker_value"] {
            let mut changed = mapping.clone();
            let field = changed["streams"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|stream| stream["name"] == "polyline")
                .unwrap()["fields"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|field| field["logical"] == "polyline.placement")
                .unwrap();
            match mutation {
                "omission" => field["omission"] = serde_json::json!("forbidden"),
                "unknown_logical" => field["logical"] = serde_json::json!("polyline.unknown"),
                "range" => field["encoding"] = serde_json::json!("pointPoolRange"),
                _ => field["nullEncoding"] = serde_json::json!("anything"),
            }
            assert!(
                validate_default_markers(&logical, &changed).is_err(),
                "{mutation}"
            );
        }
    }
}

pub(crate) fn canonical_registry() -> &'static IfcdrRegistry {
    static REGISTRY: OnceLock<IfcdrRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| build_registry(LOGICAL, MAPPING))
}

pub(crate) fn registry_0_10() -> &'static IfcdrRegistry {
    static REGISTRY: OnceLock<IfcdrRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| build_registry(LOGICAL_0_10, MAPPING_0_10))
}

pub(crate) fn registry_0_11() -> &'static IfcdrRegistry {
    static REGISTRY: OnceLock<IfcdrRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| build_registry(LOGICAL_0_11, MAPPING_0_11))
}

fn build_registry(logical: &str, mapping: &str) -> IfcdrRegistry {
    let logical: Value = serde_json::from_str(logical).expect("embedded logical registry");
    let mapping: Value = serde_json::from_str(mapping).expect("embedded JSON mapping");
    let mut registry: IfcdrRegistry =
        serde_json::from_value(materialize(&logical, &mapping)).expect("physical mapping metadata");
    registry.build_indexes();
    registry
}

fn field(
    logical: &Value,
    mapping: &Value,
    name: &str,
    kind: &str,
    nullable: bool,
    optional: bool,
) -> Value {
    use serde_json::json;
    let definition = &logical["types"][kind];
    let physical = match kind {
        "appearanceMode" | "scopeKind" | "blockScaling" => "uint32",
        "entityId" => "uint64",
        "nonEmptyString" | "unit" => "string",
        _ if mapping["valueMappings"].get(kind).is_some() => "uint32",
        _ => match definition["kind"].as_str() {
            Some("record") => "object",
            Some("sequence") => "array",
            Some("scalar") => definition["base"].as_str().unwrap(),
            _ => kind,
        },
    };
    let mut value = json!({"name": name, "valueType": physical, "nullable": nullable, "presence": if optional { "optional" } else { "required" }});
    if mapping["valueMappings"].get(kind).is_some() && kind != "appearanceMode" {
        value["allowedValues"] = Value::Array(
            mapping["valueMappings"][kind]
                .as_object()
                .expect("registered enum mapping")
                .values()
                .cloned()
                .collect(),
        );
    }
    if physical == "object" {
        value["fields"] = Value::Array(
            definition["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    field(
                        logical,
                        mapping,
                        f["name"].as_str().unwrap(),
                        f["valueType"].as_str().unwrap(),
                        f["nullable"].as_bool().unwrap_or(false),
                        f["optional"].as_bool().unwrap_or(false) || f.get("default").is_some(),
                    )
                })
                .collect(),
        );
    }
    value
}

fn materialize(logical: &Value, mapping: &Value) -> Value {
    use serde_json::json;
    validate_default_markers(logical, mapping).expect("valid embedded whole-default markers");
    let mut header: Vec<Value> = ["format", "version"]
        .into_iter()
        .map(|name| field(logical, mapping, name, "string", false, false))
        .collect();
    let mut resource_fields = Vec::new();
    for f in logical["resource"]["fields"].as_array().unwrap() {
        let name = f["name"].as_str().unwrap();
        let value = field(
            logical,
            mapping,
            name,
            f["valueType"].as_str().unwrap(),
            f["nullable"].as_bool().unwrap(),
            false,
        );
        if name == "bounds" {
            resource_fields.push(value);
        } else {
            header.push(value);
        }
    }
    resource_fields.push(json!({"name":"header","valueType":"object","nullable":false,"presence":"required","fields":header}));
    resource_fields.push(json!({"name":"streamDirectory","valueType":"object","nullable":false,"presence":"required"}));
    resource_fields.push(
        json!({"name":"streams","valueType":"object","nullable":false,"presence":"required"}),
    );
    let mut tables = Vec::new();
    for t in logical["tables"].as_array().unwrap() {
        let m = mapping["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == t["name"])
            .unwrap();
        let fields: Vec<_> = t["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                field(
                    logical,
                    mapping,
                    f["name"].as_str().unwrap(),
                    f["valueType"].as_str().unwrap(),
                    f["nullable"].as_bool().unwrap(),
                    m["fields"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|fm| {
                            fm["logical"]
                                == format!(
                                    "{}.{}",
                                    t["name"].as_str().unwrap(),
                                    f["name"].as_str().unwrap()
                                )
                        })
                        .unwrap()["omission"]
                        == "logicalDefault",
                )
            })
            .collect();
        tables.push(json!({"name":t["name"],"schemaId":t["schemaId"],"payloadPath":m["payload"],"presence":if m["omission"] == "forbidden" {"required"} else {"optional"},"rowFields":fields}));
    }
    let mut streams = Vec::new();
    for s in logical["streams"].as_array().unwrap() {
        let m = mapping["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == s["name"])
            .unwrap();
        let mut columns = Vec::new();
        let mut ranges = Vec::new();
        for f in s["fields"].as_array().unwrap() {
            let full_name = format!(
                "{}.{}",
                s["name"].as_str().unwrap(),
                f["name"].as_str().unwrap()
            );
            let fm = m["fields"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["logical"] == full_name)
                .unwrap();
            if fm.get("encoding").is_some() {
                for key in ["offset", "count"] {
                    let mut c = field(
                        logical,
                        mapping,
                        fm[key].as_str().unwrap(),
                        "uint32",
                        false,
                        false,
                    );
                    c["cardinality"] = json!("row");
                    columns.push(c);
                }
                if fm["encoding"] == "pointPoolRange" {
                    for pool in fm["pools"].as_array().unwrap() {
                        let mut c = field(
                            logical,
                            mapping,
                            pool.as_str().unwrap(),
                            "float64",
                            false,
                            false,
                        );
                        if mapping["ifcdrVersion"] == "0.11.0"
                            && s["name"] == "planarPolyline"
                            && pool == "bulge"
                        {
                            c["presence"] = json!("optional");
                        }
                        c["cardinality"] = json!("pool");
                        columns.push(c);
                    }
                    ranges.push(json!({"offsetColumn":fm["offset"],"countColumn":fm["count"],"targetColumns":fm["pools"]}));
                } else {
                    let target_name = fm["targetStream"]
                        .as_str()
                        .or_else(|| {
                            fm["target"]
                                .as_str()
                                .and_then(|target| target.split('.').next())
                        })
                        .expect("child range target");
                    let target = mapping["streams"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|t| t["name"] == target_name)
                        .unwrap();
                    ranges.push(json!({"offsetColumn":fm["offset"],"countColumn":fm["count"],"targetPath":target["payload"]}));
                }
            } else {
                let mut c = field(
                    logical,
                    mapping,
                    fm["payload"].as_str().unwrap(),
                    f["valueType"].as_str().unwrap(),
                    f["nullable"].as_bool().unwrap(),
                    fm["omission"] == "logicalDefault",
                );
                if f["valueType"] == "planePlacement" && mapping["ifcdrVersion"] == "0.11.0" {
                    for axis in c["fields"].as_array_mut().unwrap() {
                        if matches!(axis["name"].as_str(), Some("X" | "Y")) {
                            axis["presence"] = json!("optional");
                        }
                    }
                }
                c["cardinality"] = json!("row");
                c["nullDefault"] = json!(fm["nullEncoding"] == "logicalDefault");
                if let Some(default) = f.get("default") {
                    c["omissionDefault"] = default.clone();
                }
                columns.push(c);
            }
        }
        let mut stream = json!({"name":s["name"],"schemaId":s["schemaId"],"payloadKey":m["payload"].as_str().unwrap().strip_prefix("streams.").unwrap(),"role":s["role"],"columns":columns,"ranges":ranges});
        if s["name"] == "entityOrder" {
            stream["children"] = json!(["entityOrderEntry"]);
        }
        if s["name"] == "entityOrderEntry" {
            stream["parent"] = json!("entityOrder");
        }
        if s["name"] == "viewport" {
            stream["children"] = json!(["viewportLayerOverride"]);
        }
        if s["name"] == "viewportLayerOverride" {
            stream["parent"] = json!("viewport");
        }
        streams.push(stream);
    }
    let directory_fields: Vec<_> = ["version", "streams"]
        .into_iter()
        .map(|n| {
            field(
                logical,
                mapping,
                n,
                if n == "version" { "string" } else { "array" },
                false,
                false,
            )
        })
        .collect();
    let entry_fields: Vec<_> = [
        "name", "schema", "role", "count", "columns", "parent", "children",
    ]
    .into_iter()
    .map(|n| {
        field(
            logical,
            mapping,
            n,
            match n {
                "count" => "uint32",
                "columns" | "children" => "array",
                _ => "string",
            },
            false,
            matches!(n, "parent" | "children"),
        )
    })
    .collect();
    json!({"$schema":"internal","registrySchemaVersion":"internal","ifcdrVersion":logical["ifcdrVersion"],"resource":{"schemaId":logical["resource"]["schemaId"],"fields":resource_fields},"directory":{"schemaId":mapping["directory"]["schemaId"],"fields":directory_fields,"entryFields":entry_fields},"tables":tables,"streams":streams})
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IfcdrRegistry {
    #[serde(rename = "$schema")]
    schema: String,
    registry_schema_version: String,
    ifcdr_version: String,
    resource: ObjectSchema,
    directory: DirectorySchema,
    tables: Vec<TableSchema>,
    streams: Vec<StreamSchema>,
    #[serde(skip)]
    table_by_payload_path: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_name: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_schema_id: BTreeMap<String, usize>,
    #[serde(skip)]
    stream_by_payload_key: BTreeMap<String, usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectSchema {
    pub(crate) schema_id: String,
    pub(crate) fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectorySchema {
    schema_id: String,
    pub(crate) fields: Vec<FieldSchema>,
    pub(crate) entry_fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FieldSchema {
    #[serde(default)]
    pub(crate) allowed_values: Vec<Value>,
    pub(crate) name: String,
    pub(crate) value_type: ValueType,
    pub(crate) presence: Presence,
    pub(crate) nullable: bool,
    #[serde(default)]
    pub(crate) fields: Vec<FieldSchema>,
    #[serde(default)]
    pub(crate) omission_default: Option<Value>,
    #[serde(default)]
    pub(crate) reference: Option<Reference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TableSchema {
    name: String,
    schema_id: String,
    payload_path: String,
    pub(crate) presence: Presence,
    pub(crate) row_fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamSchema {
    name: String,
    schema_id: String,
    payload_key: String,
    role: StreamRole,
    pub(crate) columns: Vec<ColumnSchema>,
    #[serde(default)]
    pub(crate) ranges: Vec<RangeSchema>,
    #[serde(default)]
    pub(crate) parent: Option<String>,
    #[serde(default)]
    pub(crate) children: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnSchema {
    #[serde(default)]
    pub(crate) allowed_values: Vec<Value>,
    #[serde(default)]
    pub(crate) fields: Vec<FieldSchema>,
    #[serde(default)]
    pub(crate) null_default: bool,
    pub(crate) name: String,
    pub(crate) value_type: ValueType,
    pub(crate) presence: Presence,
    pub(crate) nullable: bool,
    pub(crate) cardinality: Cardinality,
    #[serde(default)]
    pub(crate) omission_default: Option<Value>,
    #[serde(default)]
    pub(crate) reference: Option<Reference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Reference {
    pub(crate) category: ReferenceCategory,
    pub(crate) target: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RangeSchema {
    pub(crate) offset_column: String,
    pub(crate) count_column: String,
    #[serde(default)]
    pub(crate) target_columns: Vec<String>,
    #[serde(default)]
    pub(crate) target_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ValueType {
    Array,
    Boolean,
    Float64,
    IfcxId,
    Int32,
    JsonValue,
    Object,
    String,
    Uint32,
    Uint64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Presence {
    Optional,
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Cardinality {
    Pool,
    Row,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum StreamRole {
    Child,
    Object,
    Order,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ReferenceCategory {
    Entity,
    Ifcx,
    StreamColumn,
    TableField,
}

impl IfcdrRegistry {
    fn build_indexes(&mut self) {
        self.table_by_payload_path = self
            .tables
            .iter()
            .enumerate()
            .map(|(index, table)| (table.payload_path.clone(), index))
            .collect();
        self.stream_by_name = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.name.clone(), index))
            .collect();
        self.stream_by_schema_id = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.schema_id.clone(), index))
            .collect();
        self.stream_by_payload_key = self
            .streams
            .iter()
            .enumerate()
            .map(|(index, stream)| (stream.payload_key.clone(), index))
            .collect();
    }

    pub(crate) fn ifcdr_version(&self) -> &str {
        &self.ifcdr_version
    }

    pub(crate) fn resource(&self) -> &ObjectSchema {
        &self.resource
    }

    pub(crate) fn directory(&self) -> &DirectorySchema {
        &self.directory
    }

    pub(crate) fn tables(&self) -> &[TableSchema] {
        &self.tables
    }

    pub(crate) fn streams(&self) -> &[StreamSchema] {
        &self.streams
    }

    pub(crate) fn table_by_payload_path(&self, path: &str) -> Option<&TableSchema> {
        self.table_by_payload_path
            .get(path)
            .map(|index| &self.tables[*index])
    }

    pub(crate) fn stream_by_name(&self, name: &str) -> Option<&StreamSchema> {
        self.stream_by_name
            .get(name)
            .map(|index| &self.streams[*index])
    }

    pub(crate) fn stream_by_schema_id(&self, id: &str) -> Option<&StreamSchema> {
        self.stream_by_schema_id
            .get(id)
            .map(|index| &self.streams[*index])
    }

    pub(crate) fn stream_by_payload_key(&self, key: &str) -> Option<&StreamSchema> {
        self.stream_by_payload_key
            .get(key)
            .map(|index| &self.streams[*index])
    }
}

impl DirectorySchema {
    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }
}

impl TableSchema {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn payload_path(&self) -> &str {
        &self.payload_path
    }
}

impl StreamSchema {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn payload_key(&self) -> &str {
        &self.payload_key
    }

    pub(crate) fn role(&self) -> StreamRole {
        self.role
    }
}
