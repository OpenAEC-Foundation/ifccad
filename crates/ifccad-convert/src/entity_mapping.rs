use cadcodec::Handle;
use ifccad::ifcdr::EntityId;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct EntityMapping {
    entries: BTreeMap<EntityId, Handle>,
}

impl EntityMapping {
    pub fn target_handle(&self, source: EntityId) -> Option<Handle> {
        self.entries.get(&source).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (EntityId, Handle)> + '_ {
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

    pub(crate) fn insert(&mut self, source: EntityId, target: Handle) {
        self.entries.insert(source, target);
    }
}

#[cfg(test)]
mod tests {
    use super::EntityMapping;
    use cadcodec::Handle;
    use ifccad::conformance::bundled_conformance_root;
    use ifccad::ifcdr::{EntityId, IfcdrEntityRef};
    use ifccad::package::load_directory_package;

    #[test]
    fn mapping_exposes_source_ids_in_order() {
        let ids = fixture_entity_ids();
        let mut mapping = EntityMapping::default();
        mapping.insert(ids[0], Handle::new(0x20));
        mapping.insert(ids[1], Handle::new(0x21));

        assert_eq!(mapping.len(), 2);
        assert_eq!(mapping.target_handle(ids[0]), Some(Handle::new(0x20)));
        assert_eq!(
            mapping.iter().collect::<Vec<_>>(),
            [(ids[0], Handle::new(0x20)), (ids[1], Handle::new(0x21)),]
        );
    }

    fn fixture_entity_ids() -> Vec<EntityId> {
        let root = bundled_conformance_root()
            .join("packages")
            .join("valid")
            .join("minimal-no-preservation");
        let outcome = load_directory_package(root).expect("load bundled package");
        let package = outcome.validated_package().expect("strictly valid package");
        let drawing = package.drawings().next().expect("one drawing");
        let layout = drawing.layouts().next().expect("one layout");

        drawing
            .representation()
            .resource()
            .entities(layout.scope().id())
            .map(|entity| match entity {
                IfcdrEntityRef::Line(line) => line.entity_id(),
                IfcdrEntityRef::Polyline(polyline) => polyline.entity_id(),
                IfcdrEntityRef::Unmodeled(entity) => entity.entity_id(),
            })
            .collect()
    }
}
