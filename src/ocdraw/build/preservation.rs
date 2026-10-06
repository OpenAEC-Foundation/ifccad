use super::{OcdrawBuildError, OcdrawBuilder};
use crate::ocdraw::{
    DrawingOpaqueEntity, EntityAppearance, OcdrawPreservation, OcdrawPreservationRecordId,
};

/// Core-only opaque drawable draft; the builder allocates its ordinary entity ID.
#[derive(Clone, Debug)]
pub struct OpaqueEntityDefinition {
    pub scope_id: u32,
    pub preservation_record_id: OcdrawPreservationRecordId,
    pub layer_id: Option<u32>,
    pub appearance: Option<EntityAppearance>,
    pub visible: bool,
}

impl OcdrawBuilder {
    /// Sets owned source records. Live subject links are checked at finalization.
    pub fn set_preservation(&mut self, preservation: OcdrawPreservation) {
        self.preservation = Some(preservation);
    }

    pub fn add_opaque_entity(
        &mut self,
        draft: OpaqueEntityDefinition,
    ) -> Result<u64, OcdrawBuildError> {
        let id = self.next_entity_id;
        self.next_entity_id = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        self.scope_entities
            .entry(draft.scope_id)
            .or_default()
            .push(id);
        self.opaque_entities.push(DrawingOpaqueEntity {
            id,
            preservation_record_id: draft.preservation_record_id,
            layer_id: draft.layer_id,
            appearance: draft.appearance,
            visible: draft.visible,
        });
        Ok(id)
    }
}
