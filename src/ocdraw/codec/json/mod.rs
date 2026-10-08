mod decode_appearance;
mod hatch;
mod hatch_values;
pub(crate) use hatch::{decode_hatch, encode_hatch};
mod mtext;
mod mtext_values;
mod text;
mod text_style;
pub(crate) use mtext::{decode_mtext, encode_mtext};
pub(crate) use text::{decode_text, encode_text};
pub(crate) use text_style::{decode_text_styles, encode_text_styles};
mod decode_geometry;
mod decode_line_pattern;
pub(crate) use decode_line_pattern::decode_line_patterns;
mod decode_layer;
mod decode_layout;

mod bytes;
mod decode_plot;
mod decode_scope;
mod decode_state;
mod decode_view_state;
mod decode_viewport;
mod document;
mod opaque;
mod preservation;
pub(crate) use opaque::{decode_opaque_entities, encode_opaque_entities};
pub(crate) use preservation::{decode_preservation, encode_preservation};
mod encode_color;
mod encode_document;
mod encode_geometry;
mod encode_plot;
mod encode_state;
mod encode_view_state;
mod stream_contract;
mod validate;

pub(crate) use decode_geometry::decode_geometric_entities;
pub(crate) use decode_layer::decode_layers;
pub(crate) use decode_layout::decode_layouts;

pub(crate) use decode_scope::decode_scopes;
pub(crate) use decode_state::{
    decode_block_definitions, decode_point_display, decode_ucs_definitions, decode_workspace_state,
    frame_from_json,
};
pub(crate) use decode_view_state::decode_view_state;
pub(crate) use decode_viewport::decode_viewports;
pub(crate) use document::{parse_document, JsonEncodedDrawing};
pub(crate) use encode_color::encode_color;
pub(crate) use encode_document::encode_document_bytes;
pub(crate) use encode_geometry::{object_columns, polyline_columns};
pub(crate) use encode_plot::{encode_plot_settings, encode_rect};
pub(crate) use encode_state::{encode_point_display, encode_ucs_definition};
pub(crate) use validate::validate_physical;
