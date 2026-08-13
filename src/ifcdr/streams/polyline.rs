use super::{bool_at, payload, u32_at, u64_at};
use crate::ifcdr::entity::{EntityId, ScopeId};
use crate::ifcdr::resource::{AppearanceId, LayerId, Point2, ValidatedIfcdrResource};
use serde_json::Value;

#[derive(Clone, Copy)]
pub(crate) struct PolylineStreamView<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> PolylineStreamView<'a> {
    pub(super) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }
    pub(crate) fn len(&self) -> usize {
        self.resource.evidence().streams["polyline"].row_count
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) fn get(&self, row: usize) -> Option<PolylineRef<'a>> {
        (row < self.len()).then_some(PolylineRef {
            resource: self.resource,
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
    resource: &'a ValidatedIfcdrResource,
    row: usize,
}

impl<'a> PolylineRef<'a> {
    fn data(&self) -> &'a serde_json::Map<String, Value> {
        payload(self.resource, "polylineStream")
    }
    pub(crate) fn entity_id(&self) -> EntityId {
        EntityId::new(u64_at(self.data(), "entityId", self.row)).expect("validated entity ID")
    }
    pub(crate) fn scope_id(&self) -> ScopeId {
        ScopeId::new(u32_at(self.data(), "scopeId", self.row))
    }
    pub(crate) fn closed(&self) -> bool {
        bool_at(self.data(), "closed", self.row, false)
    }
    pub(crate) fn layer_id(&self) -> LayerId {
        LayerId::new(u32_at(self.data(), "layerId", self.row))
    }
    pub(crate) fn appearance_id(&self) -> AppearanceId {
        AppearanceId::new(u32_at(self.data(), "appearanceId", self.row))
    }
    pub(crate) fn visible(&self) -> bool {
        bool_at(self.data(), "visible", self.row, true)
    }
    pub(crate) fn points(&self) -> PointIterator<'a> {
        let data = self.data();
        let start =
            usize::try_from(u64_at(data, "vertexOffset", self.row)).expect("validated offset");
        let count =
            usize::try_from(u64_at(data, "vertexCount", self.row)).expect("validated count");
        PointIterator {
            x: data["x"].as_array().expect("validated x pool"),
            y: data["y"].as_array().expect("validated y pool"),
            next: start,
            end: start + count,
        }
    }
}

pub(crate) struct PointIterator<'a> {
    x: &'a [Value],
    y: &'a [Value],
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
            self.x[index].as_f64().expect("validated x"),
            self.y[index].as_f64().expect("validated y"),
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
