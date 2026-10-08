use crate::ifccad::*;
mod encode;
mod geometry;
mod supplemental;
mod text;
mod text_layout;
mod text_values;
mod validate;
mod viewports;
mod wire;
pub(crate) use encode::encode_bytes;
pub(crate) use validate::project;

mod layout;

mod preservation;
mod workspace;
