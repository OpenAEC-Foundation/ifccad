mod columns;
mod line;
mod polyline;
mod store;

pub use line::Line;
pub(crate) use line::LineStreamView;
pub(crate) use polyline::PolylineStreamView;
pub use polyline::{PointIterator, PolylineRef};

use super::resource::ValidatedIfcdrResource;
use store::ValidatedIfcdrStreamRef;

pub(crate) struct IfcdrStreams<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrStreams<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }

    pub(crate) fn lines(&self) -> Option<LineStreamView<'a>> {
        ValidatedIfcdrStreamRef::new(self.resource, "line", "lineStream").map(LineStreamView::new)
    }

    pub(crate) fn polylines(&self) -> Option<PolylineStreamView<'a>> {
        ValidatedIfcdrStreamRef::new(self.resource, "polyline", "polylineStream")
            .map(PolylineStreamView::new)
    }
}
