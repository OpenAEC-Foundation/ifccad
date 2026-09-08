mod line;
mod polyline;

pub use line::Line;
pub(crate) use line::LineStreamView;
pub(crate) use polyline::PolylineStreamView;
pub use polyline::{PointIterator, PolylineRef};

use super::resource::ValidatedIfcdrResource;

pub(crate) struct IfcdrStreams<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrStreams<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }

    pub(crate) fn lines(&self) -> Option<LineStreamView<'a>> {
        Some(LineStreamView::new(&self.resource.typed().lines))
    }

    pub(crate) fn polylines(&self) -> Option<PolylineStreamView<'a>> {
        Some(PolylineStreamView::new(&self.resource.typed().polylines))
    }
}
