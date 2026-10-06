use super::{EntityAppearance, OcdrawPreservationRecordId};

/// Drawable source content with no native geometric interpretation or certified bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawingOpaqueEntity {
    pub id: u64,
    pub preservation_record_id: OcdrawPreservationRecordId,
    pub layer_id: Option<u32>,
    pub appearance: Option<EntityAppearance>,
    pub visible: bool,
}
