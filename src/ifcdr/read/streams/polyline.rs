use crate::ifcdr::logical::{IfcdrPolylineAccess, IfcdrPolylinesAccess};
use crate::ifcdr::read::decoded::{DecodedPolyline, DecodedPolylines, PolylineColumns};
use crate::ifcdr::{AppearanceId, EntityId, LayerId, Point2, ScopeId};
#[derive(Clone, Copy)]
pub(crate) struct PolylineStreamView<'a> {
    columns: &'a PolylineColumns,
}
impl<'a> PolylineStreamView<'a> {
    pub(super) fn new(columns: &'a PolylineColumns) -> Self {
        Self { columns }
    }
    pub(crate) fn len(&self) -> usize {
        self.columns.entity.ids.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<PolylineRef<'a>> {
        DecodedPolylines(self.columns).get(row)?;
        Some(PolylineRef {
            view: DecodedPolyline {
                columns: self.columns,
                row,
            },
        })
    }
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = PolylineRef<'a>> + 'a {
        let view = *self;
        (0..self.len()).map(move |i| view.get(i).unwrap())
    }
}
#[derive(Clone, Copy)]
pub struct PolylineRef<'a> {
    view: DecodedPolyline<'a>,
}
impl<'a> PolylineRef<'a> {
    pub fn entity_id(&self) -> EntityId {
        EntityId::new(self.view.entity().entity_id).unwrap()
    }
    pub fn scope_id(&self) -> ScopeId {
        ScopeId::new(self.view.entity().scope_id)
    }
    pub fn layer_id(&self) -> LayerId {
        LayerId::new(self.view.entity().layer_id)
    }
    pub fn appearance_id(&self) -> AppearanceId {
        AppearanceId::new(self.view.entity().appearance_id)
    }
    pub fn visible(&self) -> bool {
        self.view.entity().visible
    }
    pub fn closed(&self) -> bool {
        self.view.closed()
    }
    pub fn points(&self) -> PointIterator<'a> {
        PointIterator {
            view: self.view,
            next: 0,
        }
    }
}
pub struct PointIterator<'a> {
    view: DecodedPolyline<'a>,
    next: usize,
}
impl Iterator for PointIterator<'_> {
    type Item = Point2;
    fn next(&mut self) -> Option<Point2> {
        let p = self.view.vertex(self.next)?;
        self.next += 1;
        Some(p)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.view.vertex_count() - self.next;
        (n, Some(n))
    }
}
impl ExactSizeIterator for PointIterator<'_> {}
#[cfg(test)]
mod tests {
    use crate::ifcdr::read::resource::{fixture_source, LoadedIfcdrResource};
    use crate::ifcdr::read::validation::validate_ifcdr;
    use crate::ifcdr::Point2;

    #[test]
    fn borrows_each_polyline_point_range() {
        let outcome = validate_ifcdr(LoadedIfcdrResource::new(
            "drawing.ifcdr.json".to_owned(),
            fixture_source(),
        ));
        let resource = outcome.validated().unwrap();
        let polylines = resource.streams().polylines().unwrap();

        assert_eq!(polylines.len(), 2);
        assert!(polylines.get(2).is_none());
        let first = polylines.get(0).unwrap();
        assert_eq!(first.entity_id().get(), 3);
        assert!(first.closed());
        assert_eq!(
            first.points().collect::<Vec<_>>(),
            [
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 0.0),
                Point2::new(10.0, 5.0),
                Point2::new(0.0, 5.0),
            ]
        );
        let second = polylines.get(1).unwrap();
        assert!(!second.closed());
        assert_eq!(
            second.points().collect::<Vec<_>>(),
            [
                Point2::new(20.0, 10.0),
                Point2::new(25.0, 15.0),
                Point2::new(30.0, 10.0),
            ]
        );
    }
}
