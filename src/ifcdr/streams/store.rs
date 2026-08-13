use super::columns::{BooleanColumn, Float64Column, UInt32Column, UInt64Column};
use crate::ifcdr::resource::ValidatedIfcdrResource;
use serde_json::{Map, Value};

#[derive(Clone, Copy)]
pub(super) struct ValidatedIfcdrStreamRef<'a> {
    resource: &'a ValidatedIfcdrResource,
    stream_name: &'static str,
    payload_key: &'static str,
}

impl<'a> ValidatedIfcdrStreamRef<'a> {
    pub(super) fn new(
        resource: &'a ValidatedIfcdrResource,
        stream_name: &'static str,
        payload_key: &'static str,
    ) -> Option<Self> {
        resource
            .evidence()
            .streams
            .contains_key(stream_name)
            .then_some(Self {
                resource,
                stream_name,
                payload_key,
            })
    }
    pub(super) fn len(self) -> usize {
        self.resource.evidence().streams[self.stream_name].row_count
    }
    fn payload(self) -> &'a Map<String, Value> {
        self.resource.loaded().source().value()["streams"][self.payload_key]
            .as_object()
            .expect("validated stream payload")
    }
    pub(super) fn uint64(self, name: &str) -> UInt64Column<'a> {
        UInt64Column::new(array(self.payload(), name))
    }
    pub(super) fn uint32(self, name: &str) -> UInt32Column<'a> {
        UInt32Column::new(array(self.payload(), name))
    }
    pub(super) fn float64(self, name: &str) -> Float64Column<'a> {
        Float64Column::new(array(self.payload(), name))
    }
    pub(super) fn boolean(self, name: &str) -> BooleanColumn<'a> {
        let omission_default = crate::ifcdr::registry::canonical_registry()
            .stream_by_name(self.stream_name)
            .and_then(|stream| stream.columns.iter().find(|column| column.name == name))
            .and_then(|column| column.omission_default.as_ref())
            .and_then(Value::as_bool);
        BooleanColumn::new(
            self.payload()
                .get(name)
                .and_then(Value::as_array)
                .map(Vec::as_slice),
            omission_default,
        )
    }
}

fn array<'a>(payload: &'a Map<String, Value>, name: &str) -> &'a [Value] {
    payload[name]
        .as_array()
        .map(Vec::as_slice)
        .expect("validated stream column")
}
