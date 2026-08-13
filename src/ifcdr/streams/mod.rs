mod line;
mod polyline;

pub(crate) use line::{Line, LineStreamView};
pub(crate) use polyline::{PolylineRef, PolylineStreamView};

use crate::ifcdr::resource::ValidatedIfcdrResource;

pub(crate) struct IfcdrStreams<'a> {
    resource: &'a ValidatedIfcdrResource,
}

impl<'a> IfcdrStreams<'a> {
    pub(crate) fn new(resource: &'a ValidatedIfcdrResource) -> Self {
        Self { resource }
    }

    pub(crate) fn lines(&self) -> Option<LineStreamView<'a>> {
        self.resource
            .evidence()
            .streams
            .contains_key("line")
            .then(|| LineStreamView::new(self.resource))
    }

    pub(crate) fn polylines(&self) -> Option<PolylineStreamView<'a>> {
        self.resource
            .evidence()
            .streams
            .contains_key("polyline")
            .then(|| PolylineStreamView::new(self.resource))
    }
}

pub(super) fn payload<'a>(
    resource: &'a ValidatedIfcdrResource,
    key: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    resource.loaded().source().value()["streams"][key]
        .as_object()
        .expect("validated stream payload")
}

pub(super) fn u64_at(
    payload: &serde_json::Map<String, serde_json::Value>,
    column: &str,
    row: usize,
) -> u64 {
    payload[column][row]
        .as_u64()
        .expect("validated uint64 column")
}

pub(super) fn u32_at(
    payload: &serde_json::Map<String, serde_json::Value>,
    column: &str,
    row: usize,
) -> u32 {
    u32::try_from(u64_at(payload, column, row)).expect("validated uint32 column")
}

pub(super) fn f64_at(
    payload: &serde_json::Map<String, serde_json::Value>,
    column: &str,
    row: usize,
) -> f64 {
    payload[column][row]
        .as_f64()
        .expect("validated float64 column")
}

pub(super) fn bool_at(
    payload: &serde_json::Map<String, serde_json::Value>,
    column: &str,
    row: usize,
    omission_default: bool,
) -> bool {
    payload.get(column).map_or(omission_default, |values| {
        values[row].as_bool().expect("validated boolean column")
    })
}
