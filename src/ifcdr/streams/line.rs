use super::store::ValidatedIfcdrStreamRef;
use crate::ifcdr::entity::{EntityId, ScopeId};
use crate::ifcdr::resource::{AppearanceId, LayerId, Point2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    entity_id: EntityId,
    scope_id: ScopeId,
    start: Point2,
    end: Point2,
    layer_id: LayerId,
    appearance_id: AppearanceId,
    visible: bool,
}

impl Line {
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
    pub fn start(&self) -> Point2 {
        self.start
    }
    pub fn end(&self) -> Point2 {
        self.end
    }
    pub fn layer_id(&self) -> LayerId {
        self.layer_id
    }
    pub fn appearance_id(&self) -> AppearanceId {
        self.appearance_id
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LineStreamView<'a> {
    stream: ValidatedIfcdrStreamRef<'a>,
}

impl<'a> LineStreamView<'a> {
    pub(super) fn new(stream: ValidatedIfcdrStreamRef<'a>) -> Self {
        Self { stream }
    }
    pub(crate) fn len(&self) -> usize {
        self.stream.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<Line> {
        if row >= self.len() {
            return None;
        }
        Some(Line {
            entity_id: EntityId::new(self.stream.uint64("entityId").get(row)?)?,
            scope_id: ScopeId::new(self.stream.uint32("scopeId").get(row)?),
            start: Point2::new(
                self.stream.float64("x1").get(row)?,
                self.stream.float64("y1").get(row)?,
            ),
            end: Point2::new(
                self.stream.float64("x2").get(row)?,
                self.stream.float64("y2").get(row)?,
            ),
            layer_id: LayerId::new(self.stream.uint32("layerId").get(row)?),
            appearance_id: AppearanceId::new(self.stream.uint32("appearanceId").get(row)?),
            visible: self.stream.boolean("visible").get(row)?,
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
