use super::{AppearanceDefinition, LayerDefinition};
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

#[derive(Debug, Default)]
pub(crate) struct BuilderState {
    pub(crate) appearances: Vec<AppearanceEntry>,
    pub(crate) layers: Vec<LayerEntry>,
    pub(crate) layer_names: BTreeMap<String, usize>,
}
