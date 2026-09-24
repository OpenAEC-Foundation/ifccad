use super::{
    AppearanceDefinition, ArcDefinition, CircleDefinition, DrawingOptions, EllipseArcDefinition,
    EllipseDefinition, EntityAppearance, LayerDefinition, LineDefinition, PlanarPolylineDefinition,
    PointDefinition, SpatialPolylineDefinition,
};
use crate::ifcdr::{AppearanceId, EntityId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(crate) struct AppearanceEntry {
    pub(crate) definition: AppearanceDefinition,
}

#[derive(Debug)]
pub(crate) struct AppearanceBindingEntry {
    pub(crate) id: AppearanceId,
    pub(crate) definition: EntityAppearance,
}

#[derive(Debug)]
pub(crate) struct LayerEntry {
    pub(crate) local_id: u32,
    pub(crate) definition: LayerDefinition,
}

#[derive(Debug)]
pub(crate) enum PendingEntity {
    Point {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: PointDefinition,
    },
    Circle {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: CircleDefinition,
    },
    Arc {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: ArcDefinition,
    },
    Ellipse {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: EllipseDefinition,
    },
    EllipseArc {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: EllipseArcDefinition,
    },
    BlockInstance {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: super::BlockInstanceDefinition,
    },
    Line {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: LineDefinition,
    },
    PlanarPolyline {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: PlanarPolylineDefinition,
    },
    SpatialPolyline {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: SpatialPolylineDefinition,
    },
    Viewport {
        scope_id: u32,
        entity_id: EntityId,
        appearance_id: AppearanceId,
        definition: super::ViewportDefinition,
    },
}

#[derive(Debug, Default)]
pub(crate) struct PackageState {
    pub(crate) drawing: Option<DrawingState>,
}

#[derive(Debug)]
pub(crate) struct DrawingState {
    pub(crate) scopes: Vec<crate::ifcdr::logical::IfcdrScope>,
    pub(crate) block_definitions: Vec<crate::ifcdr::logical::IfcdrBlockDefinition>,
    pub(crate) block_names: BTreeMap<String, u32>,
    pub(crate) paper_layouts: Vec<(u32, String)>,
    pub(crate) model_layout_settings: super::LayoutSettings,
    pub(crate) paper_layout_settings: BTreeMap<u32, super::LayoutSettings>,
    pub(crate) plot_style_mode: super::PlotStyleMode,
    pub(crate) point_display: super::PointDisplay,
    pub(crate) options: DrawingOptions,
    pub(crate) storage: super::DrawingResourceStorage,
    pub(crate) token: u64,
    pub(crate) appearances: Vec<AppearanceEntry>,
    pub(crate) appearance_bindings: Vec<AppearanceBindingEntry>,
    pub(crate) layers: Vec<LayerEntry>,
    pub(crate) layer_names: BTreeMap<String, usize>,
    pub(crate) entities: Vec<PendingEntity>,
    pub(crate) next_entity_id: u64,
    pub(crate) assigned_entity_ids: BTreeSet<EntityId>,
}
