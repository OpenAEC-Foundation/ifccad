use crate::ifcdr::logical::{IfcdrPolylineAccess, IfcdrPolylinesAccess};
use crate::ifcdr::read::decoded::{DecodedPolyline, DecodedPolylines, PolylineColumns};
use crate::ifcdr::{
    AppearanceId, EntityId, GeometryEvaluationError, LayerId, PlanePlacement, Point2, Point3,
    ScopeId,
};
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
    pub fn placement(&self) -> PlanePlacement {
        PlanePlacement::from_validated_components(self.view.placement())
    }
    pub fn scope_points(&self) -> ScopePointIterator<'a> {
        ScopePointIterator {
            local: self.local_points(),
            placement: self.placement(),
        }
    }
    #[deprecated(
        note = "use local_points() for local XY coordinates or scope_points() for placed XYZ coordinates"
    )]
    pub fn points(&self) -> LocalPointIterator<'a> {
        self.local_points()
    }
    pub fn local_points(&self) -> LocalPointIterator<'a> {
        LocalPointIterator {
            view: self.view,
            next: 0,
        }
    }
}
pub struct LocalPointIterator<'a> {
    view: DecodedPolyline<'a>,
    next: usize,
}
impl Iterator for LocalPointIterator<'_> {
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
impl ExactSizeIterator for LocalPointIterator<'_> {}

/// Compatibility name for the iterator over local XY points.
pub type PointIterator<'a> = LocalPointIterator<'a>;
pub struct ScopePointIterator<'a> {
    local: LocalPointIterator<'a>,
    placement: PlanePlacement,
}
impl Iterator for ScopePointIterator<'_> {
    type Item = Result<Point3, GeometryEvaluationError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.local
            .next()
            .map(|p| self.placement.try_to_scope_point(p))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.local.size_hint()
    }
}
impl ExactSizeIterator for ScopePointIterator<'_> {}

#[cfg(test)]
mod tests {
    use crate::ifcdr::read::resource::{fixture_source, LoadedIfcdrResource};
    use crate::ifcdr::read::validation::validate_ifcdr;
    use crate::ifcdr::Point2;

    #[test]
    fn scope_iterator_continues_after_a_vertex_evaluation_error() {
        use super::*;
        use crate::ifcdr::Vector3;
        let columns = PolylineColumns {
            offsets: vec![0],
            counts: vec![3],
            x: vec![0.0, 1.0, -f64::MAX],
            y: vec![0.0; 3],
            ..Default::default()
        };
        let mut points = ScopePointIterator {
            local: LocalPointIterator {
                view: DecodedPolyline {
                    columns: &columns,
                    row: 0,
                },
                next: 0,
            },
            placement: PlanePlacement::try_new(
                Point3::new(f64::MAX, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            )
            .unwrap(),
        };
        assert_eq!(points.len(), 3);
        assert_eq!(points.next().unwrap().unwrap().x(), f64::MAX);
        assert!(points.next().unwrap().is_err());
        assert_eq!(points.len(), 1);
        assert_eq!(points.next().unwrap().unwrap().x(), 0.0);
        assert!(points.next().is_none());
    }

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
            first.local_points().collect::<Vec<_>>(),
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
            second.local_points().collect::<Vec<_>>(),
            [
                Point2::new(20.0, 10.0),
                Point2::new(25.0, 15.0),
                Point2::new(30.0, 10.0),
            ]
        );
    }
}
