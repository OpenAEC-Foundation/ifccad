mod decode;
mod diagnostic;
mod encode;
mod mapping;
mod physical;
pub(crate) use decode::color as project_color;
pub(crate) use decode::decode_json;
pub(crate) use diagnostic::logical_diagnostic;
pub(crate) use encode::unit_name;
pub(crate) use encode::{encode_json, EncodedIfcdrResource, IfcdrEncodeError};

#[cfg(test)]
mod tests;
