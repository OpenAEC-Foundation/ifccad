use super::DrawingColor;
use crate::ocdraw::{Bounds3d, LayoutSettings, UcsDefinition};

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingLayer {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub visible: bool,
    pub frozen: bool,
    pub locked: bool,
    pub plottable: bool,
    pub frozen_in_new_viewports: bool,
    pub color: DrawingColor,
    pub opacity: f64,
    pub line_pattern_id: super::LinePatternId,
    pub line_weight: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingLayoutKind {
    Model,
    Paper,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingLayout {
    pub id: u32,
    pub scope_id: u32,
    pub kind: DrawingLayoutKind,
    pub name: String,
    pub tab_index: u32,
    pub settings: LayoutSettings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingUcsDefinition {
    pub id: u32,
    pub definition: UcsDefinition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawingWorkspaceState {
    pub current_layer_id: Option<u32>,
    pub active_layout_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingBlockDefinition {
    pub scope_id: u32,
    pub name: String,
    pub base_point: [f64; 3],
    pub description: String,
    pub anonymous: bool,
    pub insertion_unit: String,
    pub explodable: bool,
    pub uniform_scaling: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingScopeKind {
    Model,
    Paper,
    Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingScope {
    pub id: u32,
    pub kind: DrawingScopeKind,
    pub bounds: Option<Bounds3d>,
    /// Missing with a box means the producer declares an enclosure.
    pub bounds_quality: Option<super::OcdrawBoundsQuality>,
    pub entities: Vec<u64>,
}

/// Rebuilds the inverse ownership index from authoritative scope lists.
/// Call after shared ownership validation has established unique membership.
pub(crate) fn owner_index(scopes: &[DrawingScope]) -> std::collections::BTreeMap<u64, u32> {
    scopes
        .iter()
        .flat_map(|scope| scope.entities.iter().map(move |id| (*id, scope.id)))
        .collect()
}
