use crate::ifcdr::logical::{IfcdrLineRow, IfcdrLinesAccess};
use crate::ifcdr::read::decoded::{DecodedLines, LineColumns};
use crate::ifcdr::{AppearanceId, EntityId, LayerId, Point2, ScopeId};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    row: IfcdrLineRow,
}
impl Line {
    pub fn entity_id(&self) -> EntityId {
        EntityId::new(self.row.entity.entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.row.entity.scope_id)
    }
    pub fn start(&self) -> Point2 {
        self.row.start
    }
    pub fn end(&self) -> Point2 {
        self.row.end
    }
    pub fn layer_id(&self) -> LayerId {
        LayerId::new(self.row.entity.layer_id)
    }
    pub fn appearance_id(&self) -> AppearanceId {
        AppearanceId::new(self.row.entity.appearance_id)
    }
    pub fn visible(&self) -> bool {
        self.row.entity.visible
    }
}
#[derive(Clone, Copy)]
pub(crate) struct LineStreamView<'a> {
    columns: &'a LineColumns,
}
impl<'a> LineStreamView<'a> {
    pub(super) fn new(columns: &'a LineColumns) -> Self {
        Self { columns }
    }
    pub(crate) fn len(&self) -> usize {
        self.columns.entity.ids.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<Line> {
        DecodedLines(self.columns).get(row).map(|row| Line { row })
    }
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = Line> + 'a {
        let view = *self;
        (0..self.len()).map(move |i| view.get(i).unwrap())
    }
}
#[cfg(test)]
mod tests {
    use super::Line;
    use crate::ifcdr::read::resource::{fixture_source, LoadedIfcdrResource};
    use crate::ifcdr::read::validation::validate_ifcdr;
    use crate::ifcdr::Point2;

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
