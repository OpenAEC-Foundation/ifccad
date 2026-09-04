use super::{
    AppearanceDefinition, DrawingOptions, LayerDefinition, LineDefinition, PolylineDefinition,
};
use crate::ifcdr::EntityId;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct AppearanceEntry {
    pub(crate) local_id: u32,
    pub(crate) definition: AppearanceDefinition,
}

#[derive(Debug)]
pub(crate) struct LayerEntry {
    pub(crate) local_id: u32,
    pub(crate) definition: LayerDefinition,
}

#[derive(Debug)]
pub(crate) enum PendingEntity {
    Line {
        entity_id: EntityId,
        definition: LineDefinition,
    },
    Polyline {
        entity_id: EntityId,
        definition: PolylineDefinition,
    },
}

#[derive(Debug, Default)]
pub(crate) struct PackageState {
    pub(crate) drawing: Option<DrawingState>,
}

#[derive(Debug)]
pub(crate) struct DrawingState {
    pub(crate) options: DrawingOptions,
    pub(crate) token: u64,
    pub(crate) appearances: Vec<AppearanceEntry>,
    pub(crate) layers: Vec<LayerEntry>,
    pub(crate) layer_names: BTreeMap<String, usize>,
    pub(crate) entities: Vec<PendingEntity>,
}
