//! Source information transported independently of provider runtimes and encodings.

use super::LinePatternId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct OcdrawPreservationRecordId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservation {
    pub version: u32,
    pub next_record_id: u64,
    pub sources: Vec<OcdrawPreservationSource>,
    pub records: Vec<OcdrawPreservationRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum OcdrawPreservationAllocationError {
    #[error("preservation record ID space is exhausted")]
    IdExhausted,
    #[error("preservation record allocation watermark is invalid")]
    InvalidWatermark,
}

impl OcdrawPreservation {
    /// Reserves an ID without reusing deleted records. Failure leaves the counter intact.
    pub fn allocate_record_id(
        &mut self,
    ) -> Result<OcdrawPreservationRecordId, OcdrawPreservationAllocationError> {
        if self.next_record_id == 0 || self.records.iter().any(|r| r.id.0 >= self.next_record_id) {
            return Err(OcdrawPreservationAllocationError::InvalidWatermark);
        }
        let id = self.next_record_id;
        self.next_record_id = id
            .checked_add(1)
            .ok_or(OcdrawPreservationAllocationError::IdExhausted)?;
        Ok(OcdrawPreservationRecordId(id))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationOrigin {
    CadDocument,
    Dwg,
    Dxf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationSource {
    pub id: String,
    pub provider: String,
    pub provider_revision: String,
    pub origin: OcdrawPreservationOrigin,
    pub source_version: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationCategory {
    Entity,
    Object,
    Table,
    Drawing,
    Layout,
    Shared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationRole {
    Complete,
    Supplement,
    Shared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationRepresentation {
    CodecTyped,
    CodecOpaque,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationDependencyCoverage {
    Qualified,
    Conservative,
    Unknown,
}

/// Targets select distinct identity domains; dependency targets need not currently exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationTarget {
    Drawing,
    Entity(u64),
    Layer(u32),
    LinePattern(LinePatternId),
    Layout(u32),
    Scope(u32),
    BlockDefinition(u32),
    Record(OcdrawPreservationRecordId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationBinding {
    pub slot: String,
    pub source_key: String,
    pub target: OcdrawPreservationTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationCondition {
    pub target: OcdrawPreservationTarget,
    pub predicate: String,
    pub version: u32,
    pub baseline: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OcdrawPreservationPayloadKind {
    AdapterSnapshot,
    RawDwgRecord,
    RawDxfGroups,
    RawSection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationPayload {
    pub schema: String,
    pub version: u32,
    pub kind: OcdrawPreservationPayloadKind,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationRecord {
    pub id: OcdrawPreservationRecordId,
    pub source_id: String,
    pub source_key: String,
    pub category: OcdrawPreservationCategory,
    pub role: OcdrawPreservationRole,
    pub representation: OcdrawPreservationRepresentation,
    pub subject: Option<OcdrawPreservationTarget>,
    pub dependency_coverage: OcdrawPreservationDependencyCoverage,
    pub bindings: Vec<OcdrawPreservationBinding>,
    pub conditions: Vec<OcdrawPreservationCondition>,
    pub payload: OcdrawPreservationPayload,
}
