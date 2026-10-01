//! Drawing meaning shared by JSON decoding and in-memory construction.

#[derive(Clone, Debug)]
pub(crate) struct NamedId {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScopeKind {
    Model,
    Paper,
    Block,
}

#[derive(Clone, Debug)]
pub(crate) struct Scope {
    pub entities: Vec<u64>,
    pub id: u32,
    pub kind: ScopeKind,
    pub has_bounds: Option<bool>,
    pub bounds: Option<crate::ocdraw::Bounds3d>,
}

#[derive(Clone, Debug)]
pub(crate) struct Layout {
    pub id: u32,
    pub name: String,
    pub scope_id: u32,
    pub kind: ScopeKind,
    pub tab_index: u32,
    pub limits: Option<crate::ocdraw::LayoutRect>,
    pub plot_rectangles: Option<PlotRectangles>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlotRectangles {
    pub printable_area: crate::ocdraw::LayoutRect,
    pub window: Option<crate::ocdraw::LayoutRect>,
}

#[derive(Clone, Debug)]
pub(crate) struct UcsChoiceCheck {
    pub location: String,
    pub kind: String,
    pub has_id: bool,
    pub has_frame: bool,
    pub frame_valid: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NamedUcs {
    pub id: u32,
    pub name: String,
    pub frame: Option<crate::ocdraw::CoordinateFrame3>,
}

#[derive(Clone, Debug)]
pub(crate) struct BlockDefinition {
    pub scope_id: u32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Entity {
    pub id: u64,
    pub layer_id: u32,
    pub definition_scope_id: Option<u32>,
    pub appearance: [AppearancePair; 4],
    pub location: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppearanceMode {
    ByLayer,
    ByBlock,
    Explicit,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AppearancePair {
    pub mode: AppearanceMode,
    pub has_value: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct DrawingModel {
    pub next_entity_id: u64,
    pub next_layer_id: u32,
    pub next_layout_id: u32,
    pub layers: Vec<NamedId>,
    pub layouts: Vec<Layout>,
    pub scopes: Vec<Scope>,
    pub blocks: Vec<BlockDefinition>,
    pub entities: Vec<Entity>,
    pub current_layer_id: Option<u32>,
    pub active_layout_id: Option<u32>,
    pub ucs_definitions: Vec<NamedUcs>,
    pub model_window_ids: Vec<u32>,
    pub active_model_window_id: Option<u32>,
    pub named_ucs_refs: Vec<(Option<u32>, String)>,
    pub ucs_choices: Vec<UcsChoiceCheck>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LogicalError {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}
