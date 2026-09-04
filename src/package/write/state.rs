use super::{
    AppearanceDefinition, DrawingOptions, EntityAppearance, LayerDefinition, LineDefinition,
    PolylineDefinition,
};
use crate::ifcdr::{AppearanceId, EntityId};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct AppearanceEntry {
    pub(crate) definition: AppearanceDefinition,
}

#[derive(Debug)]
pub(crate) struct AppearanceBindingEntry {
    pub(crate) id: AppearanceId,
    pub(crate) definition: EntityAppearance,
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
        appearance_id: AppearanceId,
        definition: LineDefinition,
    },
    Polyline {
        entity_id: EntityId,
        appearance_id: AppearanceId,
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
    pub(crate) appearance_bindings: Vec<AppearanceBindingEntry>,
    pub(crate) layers: Vec<LayerEntry>,
    pub(crate) layer_names: BTreeMap<String, usize>,
    pub(crate) entities: Vec<PendingEntity>,
}
