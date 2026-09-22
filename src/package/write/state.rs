use super::{
    AppearanceDefinition, DrawingOptions, EntityAppearance, LayerDefinition, LineDefinition,
    PolylineDefinition,
};
use crate::ifcdr::{AppearanceId, EntityId};
use std::collections::{BTreeMap, BTreeSet};

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
    BlockInstance {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: super::BlockInstanceDefinition,
    },
    Line {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: LineDefinition,
    },
    Polyline {
        scope_id: u32,
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
    pub(crate) scopes: Vec<crate::ifcdr::logical::IfcdrScope>,
    pub(crate) block_definitions: Vec<crate::ifcdr::logical::IfcdrBlockDefinition>,
    pub(crate) block_names: BTreeMap<String, u32>,
    pub(crate) paper_layouts: Vec<(u32, String)>,
    pub(crate) options: DrawingOptions,
    pub(crate) storage: super::DrawingResourceStorage,
    pub(crate) token: u64,
    pub(crate) appearances: Vec<AppearanceEntry>,
    pub(crate) appearance_bindings: Vec<AppearanceBindingEntry>,
    pub(crate) layers: Vec<LayerEntry>,
    pub(crate) layer_names: BTreeMap<String, usize>,
    pub(crate) entities: Vec<PendingEntity>,
    pub(crate) next_entity_id: u64,
    pub(crate) assigned_entity_ids: BTreeSet<EntityId>,
}
