use cadcodec::Handle;
use ifccad::ifcdr::EntityId;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExportEntityMapping {
    entries: BTreeMap<Handle, EntityId>,
}

impl ExportEntityMapping {
    pub fn target_entity_id(&self, source: Handle) -> Option<EntityId> {
        self.entries.get(&source).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (Handle, EntityId)> + '_ {
        self.entries
            .iter()
            .map(|(&source, &target)| (source, target))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn insert(&mut self, source: Handle, target: EntityId) {
        self.entries.insert(source, target);
    }
}
