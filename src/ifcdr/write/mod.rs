//! Resource construction independent of package builders and physical codecs.

mod prepare;

pub(crate) use prepare::{
    prepare_resource, IfcdrWriteEntity, IfcdrWriteInput, PreparedIfcdrResource,
};
