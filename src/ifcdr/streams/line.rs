use super::{bool_at, f64_at, payload, u32_at, u64_at};
use crate::ifcdr::entity::{EntityId, ScopeId};
use crate::ifcdr::resource::{AppearanceId, LayerId, Point2, ValidatedIfcdrResource};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Line {
    entity_id: EntityId,
    scope_id: ScopeId,
    start: Point2,
    end: Point2,
    layer_id: LayerId,
    appearance_id: AppearanceId,
    visible: bool,
}

impl Line {
    pub(crate) fn entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub(crate) fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
    pub(crate) fn start(&self) -> Point2 {
        self.start
    }
    pub(crate) fn end(&self) -> Point2 {
        self.end
    }
    pub(crate) fn layer_id(&self) -> LayerId {
        self.layer_id
    }
    pub(crate) fn appearance_id(&self) -> AppearanceId {
        self.appearance_id
    }
    pub(crate) fn visible(&self) -> bool {
        self.visible
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LineStreamView<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> LineStreamView<'a> {
    pub(super) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }
    pub(crate) fn len(&self) -> usize {
        self.resource.evidence().streams["line"].row_count
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<Line> {
        (row < self.len()).then(|| {
            let data = payload(self.resource, "lineStream");
            Line {
                entity_id: EntityId::new(u64_at(data, "entityId", row))
                    .expect("validated entity ID"),
                scope_id: ScopeId::new(u32_at(data, "scopeId", row)),
                start: Point2::new(f64_at(data, "x1", row), f64_at(data, "y1", row)),
                end: Point2::new(f64_at(data, "x2", row), f64_at(data, "y2", row)),
                layer_id: LayerId::new(u32_at(data, "layerId", row)),
                appearance_id: AppearanceId::new(u32_at(data, "appearanceId", row)),
                visible: bool_at(data, "visible", row, true),
            }
        })
    }
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = Line> + 'a {
        let view = *self;
        (0..self.len()).map(move |row| view.get(row).expect("row within validated line stream"))
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::ifcdr::resource::{fixture_source, LoadedIfcdrResource, Point2};
    use crate::ifcdr::validation::validate_ifcdr;

    #[test]
    fn reads_lines_as_copied_typed_rows() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let resource = outcome.validated().unwrap();
        let lines = resource.streams().lines().unwrap();

        assert_eq!(lines.len(), 2);
        assert!(!lines.is_empty());
        assert!(lines.get(2).is_none());
        let rows = lines.iter().collect::<Vec<_>>();
        assert_eq!(rows[0].entity_id().get(), 1);
        assert_eq!(rows[0].scope_id().get(), 0);
        assert_eq!(rows[0].start(), Point2::new(0.0, 0.0));
        assert_eq!(rows[0].end(), Point2::new(5.0, 5.0));
        assert_eq!(rows[1].layer_id().get(), 1);
        assert_eq!(rows[0].appearance_id().get(), 0);
        assert!(rows.iter().all(Line::visible));
    }
}
