use super::columns::Float64Column;
use super::store::ValidatedIfcdrStreamRef;
use crate::ifcdr::entity::{EntityId, ScopeId};
use crate::ifcdr::resource::{AppearanceId, LayerId, Point2};

#[derive(Clone, Copy)]
pub(crate) struct PolylineStreamView<'a> {
    stream: ValidatedIfcdrStreamRef<'a>,
}

impl<'a> PolylineStreamView<'a> {
    pub(super) fn new(stream: ValidatedIfcdrStreamRef<'a>) -> Self {
        Self { stream }
    }
    pub(crate) fn len(&self) -> usize {
        self.stream.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<PolylineRef<'a>> {
        (row < self.len()).then_some(PolylineRef {
            stream: self.stream,
            row,
        })
    }
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = PolylineRef<'a>> + 'a {
        let view = *self;
        (0..self.len()).map(move |row| view.get(row).expect("row within validated polyline stream"))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PolylineRef<'a> {
    stream: ValidatedIfcdrStreamRef<'a>,
    row: usize,
}

impl<'a> PolylineRef<'a> {
    pub(crate) fn entity_id(&self) -> EntityId {
        EntityId::new(
            self.stream
                .uint64("entityId")
                .get(self.row)
                .expect("validated entity ID"),
        )
        .expect("validated entity ID")
    }
    pub(crate) fn scope_id(&self) -> ScopeId {
        ScopeId::new(
            self.stream
                .uint32("scopeId")
                .get(self.row)
                .expect("validated scope ID"),
        )
    }
    pub(crate) fn closed(&self) -> bool {
        self.stream
            .boolean("closed")
            .get(self.row)
            .expect("validated closed state")
    }
    pub(crate) fn layer_id(&self) -> LayerId {
        LayerId::new(
            self.stream
                .uint32("layerId")
                .get(self.row)
                .expect("validated layer ID"),
        )
    }
    pub(crate) fn appearance_id(&self) -> AppearanceId {
        AppearanceId::new(
            self.stream
                .uint32("appearanceId")
                .get(self.row)
                .expect("validated appearance ID"),
        )
    }
    pub(crate) fn visible(&self) -> bool {
        self.stream
            .boolean("visible")
            .get(self.row)
            .expect("validated visibility")
    }
    pub(crate) fn points(&self) -> PointIterator<'a> {
        let start = usize::try_from(
            self.stream
                .uint32("vertexOffset")
                .get(self.row)
                .expect("validated offset"),
        )
        .expect("validated offset");
        let count = usize::try_from(
            self.stream
                .uint32("vertexCount")
                .get(self.row)
                .expect("validated count"),
        )
        .expect("validated count");
        PointIterator {
            x: self.stream.float64("x"),
            y: self.stream.float64("y"),
            next: start,
            end: start + count,
        }
    }
}

pub(crate) struct PointIterator<'a> {
    x: Float64Column<'a>,
    y: Float64Column<'a>,
    next: usize,
    end: usize,
}
impl Iterator for PointIterator<'_> {
    type Item = Point2;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.end {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(Point2::new(
            self.x.get(index).expect("validated x"),
            self.y.get(index).expect("validated y"),
        ))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.next;
        (len, Some(len))
    }
}
impl ExactSizeIterator for PointIterator<'_> {}

#[cfg(test)]
mod tests {
    use crate::ifcdr::resource::{fixture_source, LoadedIfcdrResource, Point2};
    use crate::ifcdr::validation::validate_ifcdr;

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
