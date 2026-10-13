use crate::ifccad::*;
mod defaults;
mod dimension;
mod dimension_style;
mod dimension_values;
mod encode;
mod geometry;
mod hatch;
mod hatch_gradient;
mod hatch_pattern;
mod hatch_values;
mod line_pattern_values;
mod presentation;
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
