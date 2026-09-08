use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const LOGICAL: &str = include_str!("../../../../schemas/ifcdr/registry-0.7.0.json");
const MAPPING: &str = include_str!("../../../../schemas/ifcdr/json-mapping-0.7.0.json");

pub(crate) fn canonical_registry() -> &'static IfcdrRegistry {
    static REGISTRY: OnceLock<IfcdrRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let logical: Value = serde_json::from_str(LOGICAL).expect("embedded logical registry");
        let mapping: Value = serde_json::from_str(MAPPING).expect("embedded JSON mapping");
        let mut registry: IfcdrRegistry = serde_json::from_value(materialize(&logical, &mapping))
            .expect("physical mapping metadata");
        registry.build_indexes();
        registry
    })
}

fn field(logical: &Value, name: &str, kind: &str, nullable: bool, optional: bool) -> Value {
    use serde_json::json;
    let definition = &logical["types"][kind];
    let physical = match kind {
        "appearanceMode" => "uint32",
        "entityId" => "uint64",
        "nonEmptyString" | "unit" => "string",
        _ => match definition["kind"].as_str() {
            Some("record") => "object",
            Some("sequence") => "array",
            Some("scalar") => definition["base"].as_str().unwrap(),
            _ => kind,
        },
    };
    let mut value = json!({"name": name, "valueType": physical, "nullable": nullable, "presence": if optional { "optional" } else { "required" }});
    if physical == "object" {
        value["fields"] = Value::Array(
            definition["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    field(
                        logical,
                        f["name"].as_str().unwrap(),
                        f["valueType"].as_str().unwrap(),
                        f["nullable"].as_bool().unwrap_or(false),
                        f["optional"].as_bool().unwrap_or(false),
                    )
                })
                .collect(),
        );
    }
    value
}

fn materialize(logical: &Value, mapping: &Value) -> Value {
    use serde_json::json;
    let mut header: Vec<Value> = ["format", "version"]
        .into_iter()
        .map(|name| field(logical, name, "string", false, false))
        .collect();
    let mut resource_fields = Vec::new();
    for f in logical["resource"]["fields"].as_array().unwrap() {
        let name = f["name"].as_str().unwrap();
        let value = field(
            logical,
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
                    f["name"].as_str().unwrap(),
                    f["valueType"].as_str().unwrap(),
                    f["nullable"].as_bool().unwrap(),
                    false,
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
                    let mut c = field(logical, fm[key].as_str().unwrap(), "uint32", false, false);
                    c["cardinality"] = json!("row");
                    columns.push(c);
                }
                if fm["encoding"] == "pointPoolRange" {
                    for pool in fm["pools"].as_array().unwrap() {
                        let mut c = field(logical, pool.as_str().unwrap(), "float64", false, false);
                        c["cardinality"] = json!("pool");
                        columns.push(c);
                    }
                    ranges.push(json!({"offsetColumn":fm["offset"],"countColumn":fm["count"],"targetColumns":fm["pools"]}));
                } else {
                    let target_name = fm["target"].as_str().unwrap().split('.').next().unwrap();
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
                    fm["payload"].as_str().unwrap(),
                    f["valueType"].as_str().unwrap(),
                    f["nullable"].as_bool().unwrap(),
                    fm["omission"] == "logicalDefault",
                );
                c["cardinality"] = json!("row");
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
        streams.push(stream);
    }
    let directory_fields: Vec<_> = ["version", "streams"]
        .into_iter()
        .map(|n| {
            field(
                logical,
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
