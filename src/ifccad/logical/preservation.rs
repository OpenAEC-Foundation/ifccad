//! Source information transported independently of provider runtimes and encodings.

use super::IfccadLinePatternId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationRecordId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservation {
    pub version: u32,
    pub sources: Vec<IfccadPreservationSource>,
    pub records: Vec<IfccadPreservationRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationOrigin {
    CadDocument,
    Dwg,
    Dxf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationSource {
    pub id: String,
    pub provider: String,
    pub provider_revision: String,
    pub origin: IfccadPreservationOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_version: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationCategory {
    Entity,
    Object,
    Table,
    Drawing,
    Layout,
    Shared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationRole {
    Complete,
    Supplement,
    Shared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationRepresentation {
    CodecTyped,
    CodecOpaque,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationDependencyCoverage {
    Qualified,
    Conservative,
    Unknown,
}

/// Targets select distinct identity domains; dependency targets need not currently exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "role",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum IfccadPreservationTarget {
    Drawing,
    Entity(u64),
    Layer(u64),
    LinePattern(IfccadLinePatternId),
    Layout(u64),
    BlockDefinition(u64),
    Record(IfccadPreservationRecordId),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationBinding {
    pub slot: String,
    pub source_key: String,
    pub target: IfccadPreservationTarget,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationCondition {
    pub target: IfccadPreservationTarget,
    pub predicate: String,
    pub version: u32,
    pub baseline: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationPayloadKind {
    AdapterSnapshot,
    RawDwgRecord,
    RawDxfGroups,
    RawSection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationPayload {
    pub schema: String,
    pub version: u32,
    pub kind: IfccadPreservationPayloadKind,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPreservationRecord {
    pub id: IfccadPreservationRecordId,
    pub source_id: String,
    pub source_key: String,
    pub category: IfccadPreservationCategory,
    pub role: IfccadPreservationRole,
    pub representation: IfccadPreservationRepresentation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<IfccadPreservationTarget>,
    pub dependency_coverage: IfccadPreservationDependencyCoverage,
    pub bindings: Vec<IfccadPreservationBinding>,
    pub conditions: Vec<IfccadPreservationCondition>,
    pub payload: IfccadPreservationPayload,
}
