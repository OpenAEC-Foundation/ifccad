mod state;
use super::logical::{
    AppearanceMode as LogicalAppearanceMode, AppearancePair, AppearanceSelection,
    BlockDefinition as LogicalBlockDefinition, DrawingColor, DrawingEntityRecord, DrawingModel,
    Entity as LogicalEntity, EntityAppearance, EntityGeometry, Layout as LogicalLayout, NamedId,
    NamedUcs, Scope, ScopeKind,
};
use super::{load_drawing_bytes, DrawingLoadStatus, LayoutSettings, PointDisplay, UcsDefinition};
pub use state::{DrawingSavedState, ViewportDefinition};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct DrawingOptions {
    pub drawing_id: String,
    pub unit: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlotStyleMode {
    #[default]
    ColorDependent,
    Named,
}

impl DrawingOptions {
    pub fn new(drawing_id: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            drawing_id: drawing_id.into(),
            unit: unit.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RgbColor(pub [u8; 3]);

impl RgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self([red, green, blue])
    }
}

impl From<RgbColor> for DrawingColor {
    fn from(value: RgbColor) -> Self {
        Self {
            rgb: value.0,
            indexed: None,
            named: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayerDefinition {
    pub name: String,
    pub description: Option<String>,
    pub visible: bool,
    pub frozen: bool,
    pub locked: bool,
    pub plottable: bool,
    pub frozen_in_new_viewports: bool,
    pub color: DrawingColor,
    pub opacity: f64,
    pub line_pattern: String,
    pub line_weight: f64,
}

impl LayerDefinition {
    pub fn new(name: impl Into<String>, color: impl Into<DrawingColor>) -> Self {
        Self {
            name: name.into(),
            description: None,
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            color: color.into(),
            opacity: 1.0,
            line_pattern: "Continuous".into(),
            line_weight: 0.25,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LineDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct GeometricEntityDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub geometry: EntityGeometry,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

impl GeometricEntityDefinition {
    pub fn new(scope_id: u32, layer_id: u32, geometry: EntityGeometry) -> Self {
        Self {
            scope_id,
            layer_id,
            geometry,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }
}

impl LineDefinition {
    pub fn new(layer_id: u32, start: [f64; 3], end: [f64; 3]) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            start,
            end,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

#[derive(Clone, Debug)]
pub struct PointDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub position: [f64; 3],
    pub appearance: EntityAppearance,
    pub visible: bool,
}

impl PointDefinition {
    pub fn new(layer_id: u32, position: [f64; 3]) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            position,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

#[derive(Clone, Debug)]
pub struct CircleDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub center: [f64; 3],
    pub radius: f64,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct ArcDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub center: [f64; 3],
    pub radius: f64,
    pub start_parameter: f64,
    pub sweep_parameter: f64,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct EllipseDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub center: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub semi_major_radius: f64,
    pub semi_minor_radius: f64,
    pub arc: Option<(f64, f64)>,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct PlanarPolylineDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub vertices: Vec<[f64; 3]>,
    pub closed: bool,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

impl PlanarPolylineDefinition {
    pub fn new(layer_id: u32, vertices: Vec<[f64; 3]>, closed: bool) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            vertices,
            closed,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

#[derive(Clone, Debug)]
pub struct SpatialPolylineDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub vertices: Vec<[f64; 3]>,
    pub closed: bool,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

impl SpatialPolylineDefinition {
    pub fn new(layer_id: u32, vertices: Vec<[f64; 3]>, closed: bool) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            vertices,
            closed,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

impl EllipseDefinition {
    pub fn new(
        layer_id: u32,
        center: [f64; 3],
        x_axis: [f64; 3],
        semi_major_radius: f64,
        semi_minor_radius: f64,
    ) -> Self {
        let y_axis = [-x_axis[1], x_axis[0], 0.0];
        Self {
            scope_id: 0,
            layer_id,
            center,
            x_axis,
            y_axis,
            semi_major_radius,
            semi_minor_radius,
            arc: None,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }

    pub fn with_arc(mut self, start: f64, sweep: f64) -> Self {
        self.arc = Some((start, sweep));
        self
    }
}

impl ArcDefinition {
    pub fn new(
        layer_id: u32,
        center: [f64; 3],
        radius: f64,
        start_parameter: f64,
        sweep_parameter: f64,
    ) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            center,
            radius,
            start_parameter,
            sweep_parameter,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

#[derive(Clone, Debug)]
pub struct BlockInstanceDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub definition_scope_id: u32,
    pub origin: [f64; 3],
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct BlockDefinition {
    pub name: String,
    pub base_point: [f64; 3],
    pub description: String,
    pub anonymous: bool,
    pub insertion_unit: String,
    pub explodable: bool,
    pub uniform_scaling: bool,
}

impl BlockDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_point: [0.0; 3],
            description: String::new(),
            anonymous: false,
            insertion_unit: "unitless".into(),
            explodable: true,
            uniform_scaling: false,
        }
    }
}

impl From<&str> for BlockDefinition {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for BlockDefinition {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl BlockInstanceDefinition {
    pub fn new(layer_id: u32, definition_scope_id: u32, origin: [f64; 3]) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            definition_scope_id,
            origin,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

impl CircleDefinition {
    pub fn new(layer_id: u32, center: [f64; 3], radius: f64) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            center,
            radius,
            appearance: EntityAppearance::default(),
            visible: true,
        }
    }

    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DrawingBuildError {
    #[error("drawing content is invalid: {0}")]
    Invalid(String),
    #[error("drawing ID space is exhausted")]
    IdExhausted,
    #[error("drawing serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DrawingWriteError {
    #[error("could not write drawing: {0}")]
    Io(#[from] std::io::Error),
}

pub struct EncodedDrawing {
    bytes: Vec<u8>,
}

impl EncodedDrawing {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), DrawingWriteError> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&self.bytes)?;
        Ok(())
    }
}

pub struct DrawingBuilder {
    pub(crate) options: DrawingOptions,
    pub(crate) model_layout_name: String,
    pub(crate) layers: Vec<LayerDefinition>,
    pub(crate) objects: Vec<DrawingEntityRecord>,
    pub(crate) scope_entities: std::collections::BTreeMap<u32, Vec<u64>>,
    pub(crate) next_entity_id: u64,
    pub(crate) paper_layouts: Vec<String>,
    pub(crate) layout_settings: Vec<LayoutSettings>,
    pub(crate) block_definitions: Vec<BlockDefinition>,
    pub(crate) current_layer: Option<u32>,
    pub(crate) active_layout: Option<u32>,
    pub(crate) plot_style_mode: PlotStyleMode,
    pub(crate) point_display: Option<PointDisplay>,
    pub(crate) ucs_definitions: Vec<UcsDefinition>,
    pub(crate) viewports: Vec<super::logical::DrawingViewport>,
    pub(crate) saved_state: DrawingSavedState,
}

impl DrawingBuilder {
    fn logical_model(&self) -> Result<DrawingModel, DrawingBuildError> {
        let layer_count =
            u32::try_from(self.layers.len()).map_err(|_| DrawingBuildError::IdExhausted)?;
        let layout_count = self
            .paper_layouts
            .len()
            .checked_add(1)
            .and_then(|count| u32::try_from(count).ok())
            .ok_or(DrawingBuildError::IdExhausted)?;
        let layers = self
            .layers
            .iter()
            .enumerate()
            .map(|(id, layer)| NamedId {
                id: id as u32,
                name: layer.name.clone(),
            })
            .collect();
        let mut layouts = vec![LogicalLayout {
            id: 0,
            name: self.model_layout_name.clone(),
            scope_id: 0,
            kind: ScopeKind::Model,
            tab_index: 0,
            limits: self.layout_settings[0].limits,
            plot_rectangles: self.layout_settings[0].plot_settings.as_ref().map(|plot| {
                super::logical::PlotRectangles {
                    printable_area: plot.media.printable_area,
                    window: match plot.area {
                        super::PlotArea::Window(rect) => Some(rect),
                        _ => None,
                    },
                }
            }),
        }];
        let mut scopes = vec![Scope {
            entities: Vec::new(),
            id: 0,
            kind: ScopeKind::Model,
            has_bounds: None,
            bounds: None,
        }];
        for (index, name) in self.paper_layouts.iter().enumerate() {
            let id = u32::try_from(index + 1).map_err(|_| DrawingBuildError::IdExhausted)?;
            layouts.push(LogicalLayout {
                id,
                name: name.clone(),
                scope_id: id,
                kind: ScopeKind::Paper,
                tab_index: id,
                limits: self.layout_settings[index + 1].limits,
                plot_rectangles: self.layout_settings[index + 1].plot_settings.as_ref().map(
                    |plot| super::logical::PlotRectangles {
                        printable_area: plot.media.printable_area,
                        window: match plot.area {
                            super::PlotArea::Window(rect) => Some(rect),
                            _ => None,
                        },
                    },
                ),
            });
            scopes.push(Scope {
                entities: Vec::new(),
                id,
                kind: ScopeKind::Paper,
                has_bounds: None,
                bounds: None,
            });
        }
        let mut blocks = Vec::new();
        for (index, definition) in self.block_definitions.iter().enumerate() {
            let id = self
                .paper_layouts
                .len()
                .checked_add(index + 1)
                .and_then(|id| u32::try_from(id).ok())
                .ok_or(DrawingBuildError::IdExhausted)?;
            scopes.push(Scope {
                entities: Vec::new(),
                id,
                kind: ScopeKind::Block,
                has_bounds: None,
                bounds: None,
            });
            blocks.push(LogicalBlockDefinition {
                scope_id: id,
                name: definition.name.clone(),
            });
        }
        let mut entities: Vec<LogicalEntity> = self
            .objects
            .iter()
            .enumerate()
            .map(|(index, row)| LogicalEntity {
                id: row.id,
                layer_id: row.layer_id,
                definition_scope_id: match row.geometry {
                    EntityGeometry::BlockInstance {
                        definition_scope_id,
                        ..
                    } => Some(definition_scope_id),
                    _ => None,
                },
                appearance: [
                    logical_pair(&row.appearance.color),
                    logical_pair(&row.appearance.opacity),
                    logical_pair(&row.appearance.line_pattern),
                    logical_pair(&row.appearance.line_weight),
                ],
                location: format!("/entities/{index}"),
            })
            .collect();
        entities.extend(self.viewports.iter().map(|row| LogicalEntity {
            id: row.id,
            layer_id: row.layer_id,
            definition_scope_id: None,
            appearance: [
                logical_pair(&row.appearance.color),
                logical_pair(&row.appearance.opacity),
                logical_pair(&row.appearance.line_pattern),
                logical_pair(&row.appearance.line_weight),
            ],
            location: format!("/entities/{}", row.id),
        }));
        for scope in &mut scopes {
            scope.entities = self
                .scope_entities
                .get(&scope.id)
                .cloned()
                .unwrap_or_default();
        }
        Ok(DrawingModel {
            next_entity_id: self.next_entity_id,
            next_layer_id: layer_count,
            next_layout_id: layout_count,
            layers,
            layouts,
            scopes,
            blocks,
            entities,
            current_layer_id: self.current_layer,
            active_layout_id: self.active_layout,
            ucs_definitions: self
                .ucs_definitions
                .iter()
                .enumerate()
                .map(|(id, ucs)| NamedUcs {
                    id: id as u32,
                    name: ucs.name.clone(),
                    frame: Some(ucs.frame),
                })
                .collect(),
            model_window_ids: self
                .saved_state
                .model_windows
                .iter()
                .map(|w| w.id)
                .collect(),
            active_model_window_id: self
                .saved_state
                .view_state
                .map(|s| s.active_model_window_id),
            named_ucs_refs: Vec::new(),
            ucs_choices: Vec::new(),
        })
    }

    pub fn new(options: DrawingOptions) -> Result<Self, DrawingBuildError> {
        if options.drawing_id.is_empty() || options.unit.is_empty() {
            return Err(DrawingBuildError::Invalid(
                "drawing identity and unit must be nonempty".into(),
            ));
        }
        Ok(Self {
            options,
            model_layout_name: "Model".into(),
            layers: Vec::new(),
            objects: Vec::new(),
            scope_entities: Default::default(),
            next_entity_id: 1,
            paper_layouts: Vec::new(),
            layout_settings: vec![LayoutSettings::default()],
            block_definitions: Vec::new(),
            current_layer: None,
            active_layout: None,
            plot_style_mode: PlotStyleMode::ColorDependent,
            point_display: None,
            ucs_definitions: Vec::new(),
            viewports: Vec::new(),
            saved_state: DrawingSavedState::default(),
        })
    }

    pub fn add_paper_layout(&mut self, name: impl Into<String>) -> Result<u32, DrawingBuildError> {
        if !self.block_definitions.is_empty() {
            return Err(DrawingBuildError::Invalid(
                "paper layouts must be added before block definitions".into(),
            ));
        }
        let id = u32::try_from(self.paper_layouts.len())
            .map_err(|_| DrawingBuildError::IdExhausted)?
            .checked_add(1)
            .ok_or(DrawingBuildError::IdExhausted)?;
        self.paper_layouts.push(name.into());
        self.layout_settings.push(LayoutSettings::default());
        Ok(id)
    }

    pub fn add_block_definition(
        &mut self,
        definition: impl Into<BlockDefinition>,
    ) -> Result<u32, DrawingBuildError> {
        let id = self
            .paper_layouts
            .len()
            .checked_add(self.block_definitions.len())
            .and_then(|count| count.checked_add(1))
            .and_then(|count| u32::try_from(count).ok())
            .ok_or(DrawingBuildError::IdExhausted)?;
        self.block_definitions.push(definition.into());
        Ok(id)
    }

    pub fn set_model_layout_name(&mut self, name: impl Into<String>) {
        self.model_layout_name = name.into();
    }

    pub fn add_ucs_definition(
        &mut self,
        definition: UcsDefinition,
    ) -> Result<u32, DrawingBuildError> {
        if definition.name.is_empty() || !definition.elevation.is_finite() {
            return Err(DrawingBuildError::Invalid("invalid UCS definition".into()));
        }
        let id = u32::try_from(self.ucs_definitions.len())
            .map_err(|_| DrawingBuildError::IdExhausted)?;
        self.ucs_definitions.push(definition);
        Ok(id)
    }

    pub fn set_layout_settings(
        &mut self,
        layout_id: u32,
        settings: LayoutSettings,
    ) -> Result<(), DrawingBuildError> {
        if settings.limits.is_some_and(|limits| !limits.is_valid()) {
            return Err(DrawingBuildError::Invalid("invalid layout limits".into()));
        }
        if settings
            .plot_settings
            .as_ref()
            .is_some_and(|plot| !plot.is_valid())
        {
            return Err(DrawingBuildError::Invalid("invalid plot settings".into()));
        }
        let slot = self
            .layout_settings
            .get_mut(layout_id as usize)
            .ok_or_else(|| DrawingBuildError::Invalid("unknown layout ID".into()))?;
        *slot = settings;
        Ok(())
    }

    pub fn set_current_layer(&mut self, id: u32) {
        self.current_layer = Some(id);
    }

    pub fn set_active_layout(&mut self, id: u32) {
        self.active_layout = Some(id);
    }

    pub fn set_plot_style_mode(&mut self, mode: PlotStyleMode) {
        self.plot_style_mode = mode;
    }

    pub fn set_point_display(&mut self, display: PointDisplay) -> Result<(), DrawingBuildError> {
        if !display.is_valid() {
            return Err(DrawingBuildError::Invalid(
                "invalid point display size".into(),
            ));
        }
        self.point_display = Some(display);
        Ok(())
    }

    pub fn add_layer(&mut self, layer: LayerDefinition) -> Result<u32, DrawingBuildError> {
        let id = u32::try_from(self.layers.len()).map_err(|_| DrawingBuildError::IdExhausted)?;
        self.layers.push(layer);
        Ok(id)
    }

    pub fn add_line(&mut self, line: LineDefinition) -> Result<u64, DrawingBuildError> {
        let min = std::array::from_fn(|axis| line.start[axis].min(line.end[axis]));
        let max = std::array::from_fn(|axis| line.start[axis].max(line.end[axis]));
        self.add_object(
            line.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "line",
                layer_id: line.layer_id,
                appearance: line.appearance,
                visible: line.visible,
                geometry: EntityGeometry::Line {
                    start: line.start,
                    end: line.end,
                },
                min,
                max,
            },
        )
    }

    pub fn add_geometric_entity(
        &mut self,
        entity: GeometricEntityDefinition,
    ) -> Result<u64, DrawingBuildError> {
        let kind = match &entity.geometry {
            EntityGeometry::Line { .. } => "line",
            EntityGeometry::Point { .. } => "point",
            EntityGeometry::Circle { .. } => "circle",
            EntityGeometry::Arc { .. } => "arc",
            EntityGeometry::Ellipse { arc: None, .. } => "ellipse",
            EntityGeometry::Ellipse { arc: Some(_), .. } => "ellipseArc",
            EntityGeometry::PlanarPolyline { .. } => "planarPolyline",
            EntityGeometry::SpatialPolyline { .. } => "spatialPolyline",
            EntityGeometry::BlockInstance { .. } => "blockInstance",
        };
        let (min, max) = match &entity.geometry {
            EntityGeometry::BlockInstance { transform, .. } => {
                let origin = transform.placement().origin().components();
                (origin, origin)
            }
            geometry => super::logical::enclosure(geometry)
                .ok_or_else(|| DrawingBuildError::Invalid("invalid entity geometry".into()))?,
        };
        self.add_object(
            entity.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind,
                layer_id: entity.layer_id,
                appearance: entity.appearance,
                visible: entity.visible,
                geometry: entity.geometry,
                min,
                max,
            },
        )
    }

    pub fn add_point(&mut self, point: PointDefinition) -> Result<u64, DrawingBuildError> {
        let placement = frame_at(point.position)?;
        self.add_object(
            point.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "point",
                layer_id: point.layer_id,
                appearance: point.appearance,
                visible: point.visible,
                geometry: EntityGeometry::Point { placement },
                min: point.position,
                max: point.position,
            },
        )
    }

    pub fn add_circle(&mut self, circle: CircleDefinition) -> Result<u64, DrawingBuildError> {
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(circle.center[0], circle.center[1], circle.center[2]),
            crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
            crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
        )
        .map_err(|error| DrawingBuildError::Invalid(error.to_string()))?;
        let enclosure =
            crate::ocdraw::geometry::circular_bounds(frame.components(), circle.radius, None)
                .ok_or_else(|| {
                    DrawingBuildError::Invalid(
                        "circle requires finite positive radius and valid placement".into(),
                    )
                })?;
        let min = enclosure.min().components();
        let max = enclosure.max().components();
        self.add_object(
            circle.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "circle",
                layer_id: circle.layer_id,
                appearance: circle.appearance,
                visible: circle.visible,
                geometry: EntityGeometry::Circle {
                    placement: frame,
                    radius: circle.radius,
                },
                min,
                max,
            },
        )
    }

    pub fn add_arc(&mut self, arc: ArcDefinition) -> Result<u64, DrawingBuildError> {
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(arc.center[0], arc.center[1], arc.center[2]),
            crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
            crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
        )
        .map_err(|error| DrawingBuildError::Invalid(error.to_string()))?;
        let enclosure = crate::ocdraw::geometry::circular_bounds(
            frame.components(),
            arc.radius,
            Some((arc.start_parameter, arc.sweep_parameter)),
        )
        .ok_or_else(|| DrawingBuildError::Invalid("invalid arc geometry".into()))?;
        self.add_object(
            arc.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "arc",
                layer_id: arc.layer_id,
                appearance: arc.appearance,
                visible: arc.visible,
                geometry: EntityGeometry::Arc {
                    placement: frame,
                    radius: arc.radius,
                    start_parameter: arc.start_parameter,
                    sweep_parameter: arc.sweep_parameter,
                },
                min: enclosure.min().components(),
                max: enclosure.max().components(),
            },
        )
    }

    pub fn add_ellipse(&mut self, ellipse: EllipseDefinition) -> Result<u64, DrawingBuildError> {
        self.add_elliptic(ellipse, false)
    }

    pub fn add_ellipse_arc(
        &mut self,
        ellipse: EllipseDefinition,
    ) -> Result<u64, DrawingBuildError> {
        self.add_elliptic(ellipse, true)
    }

    fn add_elliptic(
        &mut self,
        ellipse: EllipseDefinition,
        partial: bool,
    ) -> Result<u64, DrawingBuildError> {
        if partial != ellipse.arc.is_some() {
            return Err(DrawingBuildError::Invalid(
                "ellipse arc requires a sweep; full ellipse forbids one".into(),
            ));
        }
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(ellipse.center[0], ellipse.center[1], ellipse.center[2]),
            crate::ocdraw::Vector3::new(ellipse.x_axis[0], ellipse.x_axis[1], ellipse.x_axis[2]),
            crate::ocdraw::Vector3::new(ellipse.y_axis[0], ellipse.y_axis[1], ellipse.y_axis[2]),
        )
        .map_err(|error| DrawingBuildError::Invalid(error.to_string()))?;
        let enclosure = crate::ocdraw::geometry::elliptic_bounds(
            frame.components(),
            ellipse.semi_major_radius,
            ellipse.semi_minor_radius,
            ellipse.arc,
        )
        .ok_or_else(|| DrawingBuildError::Invalid("invalid ellipse geometry".into()))?;
        self.add_object(
            ellipse.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: if partial { "ellipseArc" } else { "ellipse" },
                layer_id: ellipse.layer_id,
                appearance: ellipse.appearance,
                visible: ellipse.visible,
                geometry: EntityGeometry::Ellipse {
                    placement: frame,
                    semi_major_radius: ellipse.semi_major_radius,
                    semi_minor_radius: ellipse.semi_minor_radius,
                    arc: ellipse.arc,
                },
                min: enclosure.min().components(),
                max: enclosure.max().components(),
            },
        )
    }

    pub fn add_planar_polyline(
        &mut self,
        polyline: PlanarPolylineDefinition,
    ) -> Result<u64, DrawingBuildError> {
        if polyline.vertices.len() < 2 {
            return Err(DrawingBuildError::Invalid(
                "planar polyline needs two vertices".into(),
            ));
        }
        let mut min = [f64::INFINITY, f64::INFINITY, 0.0];
        let mut max = [f64::NEG_INFINITY, f64::NEG_INFINITY, 0.0];
        let segments = if polyline.closed {
            polyline.vertices.len()
        } else {
            polyline.vertices.len() - 1
        };
        for index in 0..segments {
            let from = polyline.vertices[index];
            let to = polyline.vertices[(index + 1) % polyline.vertices.len()];
            let bounds = crate::ocdraw::geometry::bulge_segment_bounds(
                crate::ocdraw::Point2::new(from[0], from[1]),
                crate::ocdraw::Point2::new(to[0], to[1]),
                from[2],
            )
            .ok_or_else(|| DrawingBuildError::Invalid("invalid polyline segment".into()))?;
            min[0] = min[0].min(bounds.min().x());
            min[1] = min[1].min(bounds.min().y());
            max[0] = max[0].max(bounds.max().x());
            max[1] = max[1].max(bounds.max().y());
        }
        self.add_object(
            polyline.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "planarPolyline",
                layer_id: polyline.layer_id,
                appearance: polyline.appearance,
                visible: polyline.visible,
                geometry: EntityGeometry::PlanarPolyline {
                    placement: crate::ocdraw::CoordinateFrame3::default(),
                    vertices: polyline.vertices,
                    closed: polyline.closed,
                },
                min,
                max,
            },
        )
    }

    pub fn add_spatial_polyline(
        &mut self,
        polyline: SpatialPolylineDefinition,
    ) -> Result<u64, DrawingBuildError> {
        if polyline.vertices.len() < 2
            || polyline
                .vertices
                .iter()
                .flatten()
                .any(|value| !value.is_finite())
        {
            return Err(DrawingBuildError::Invalid(
                "invalid spatial polyline vertices".into(),
            ));
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for vertex in &polyline.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex[axis]);
                max[axis] = max[axis].max(vertex[axis]);
            }
        }
        self.add_object(
            polyline.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "spatialPolyline",
                layer_id: polyline.layer_id,
                appearance: polyline.appearance,
                visible: polyline.visible,
                geometry: EntityGeometry::SpatialPolyline {
                    vertices: polyline.vertices,
                    closed: polyline.closed,
                },
                min,
                max,
            },
        )
    }

    pub fn add_block_instance(
        &mut self,
        instance: BlockInstanceDefinition,
    ) -> Result<u64, DrawingBuildError> {
        let transform = crate::ocdraw::BlockTransform::try_new(
            frame_at(instance.origin)?,
            0.0,
            crate::ocdraw::Scale3::default(),
        )
        .map_err(|error| DrawingBuildError::Invalid(error.to_string()))?;
        self.add_object(
            instance.scope_id,
            DrawingEntityRecord {
                id: 0,
                kind: "blockInstance",
                layer_id: instance.layer_id,
                appearance: instance.appearance,
                visible: instance.visible,
                geometry: EntityGeometry::BlockInstance {
                    definition_scope_id: instance.definition_scope_id,
                    transform,
                },
                min: instance.origin,
                max: instance.origin,
            },
        )
    }

    fn add_object(
        &mut self,
        scope_id: u32,
        mut row: DrawingEntityRecord,
    ) -> Result<u64, DrawingBuildError> {
        let id = self.next_entity_id;
        self.next_entity_id = id.checked_add(1).ok_or(DrawingBuildError::IdExhausted)?;
        row.id = id;
        self.scope_entities.entry(scope_id).or_default().push(id);
        self.objects.push(row);
        Ok(id)
    }

    pub fn finish(self) -> Result<EncodedDrawing, DrawingBuildError> {
        let mut logical = self.logical_model()?;
        if let Some(error) = logical.validate().into_iter().next() {
            return Err(DrawingBuildError::Invalid(format!(
                "{}: {}",
                error.location, error.message
            )));
        }
        let total_scopes = self.paper_layouts.len() + self.block_definitions.len() + 1;
        let first_block_scope = self.paper_layouts.len() + 1;
        let mut cache = vec![None; total_scopes];
        let mut visiting = vec![false; total_scopes];
        let objects_by_id = self
            .objects
            .iter()
            .map(|row| (row.id, row))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut bounds = (0..total_scopes)
            .map(|scope| {
                resolve_scope_bounds(
                    scope,
                    first_block_scope,
                    &self.block_definitions,
                    &objects_by_id,
                    &self.scope_entities,
                    &mut cache,
                    &mut visiting,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let owners = self
            .scope_entities
            .iter()
            .flat_map(|(scope, ids)| ids.iter().map(move |id| (*id, *scope)))
            .collect::<std::collections::BTreeMap<_, _>>();
        for viewport in &self.viewports {
            let index = owners[&viewport.id] as usize;
            let Some(scope) = bounds.get_mut(index) else {
                return Err(DrawingBuildError::Invalid(
                    "viewport owner scope does not exist".into(),
                ));
            };
            let frame = super::logical::viewport_bounds(viewport.frame)
                .ok_or_else(|| DrawingBuildError::Invalid("invalid viewport frame".into()))?;
            let min = frame.min().components();
            let max = frame.max().components();
            match scope {
                Some((lower, upper)) => {
                    for i in 0..3 {
                        lower[i] = lower[i].min(min[i]);
                        upper[i] = upper[i].max(max[i]);
                    }
                }
                None => *scope = Some((min, max)),
            }
        }
        for (scope, bounds) in logical.scopes.iter_mut().zip(&bounds) {
            scope.has_bounds = Some(bounds.is_some());
            scope.bounds = bounds.map(|(min, max)| crate::ocdraw::Bounds3d {
                min: crate::ocdraw::Point3::new(min[0], min[1], min[2]),
                max: crate::ocdraw::Point3::new(max[0], max[1], max[2]),
            });
        }
        if let Some(error) = logical.validate().into_iter().next() {
            return Err(DrawingBuildError::Invalid(format!(
                "{}: {}",
                error.location, error.message
            )));
        }
        let geometric_entities = self
            .objects
            .iter()
            .map(|row| super::logical::DrawingGeometricEntity {
                id: row.id,
                layer_id: row.layer_id,
                visible: row.visible,
                appearance: row.appearance.clone(),
                geometry: row.geometry.clone(),
            })
            .collect::<Vec<_>>();
        let scopes = logical
            .scopes
            .iter()
            .map(|scope| super::logical::DrawingScope {
                id: scope.id,
                kind: match scope.kind {
                    super::logical::ScopeKind::Model => super::logical::DrawingScopeKind::Model,
                    super::logical::ScopeKind::Paper => super::logical::DrawingScopeKind::Paper,
                    super::logical::ScopeKind::Block => super::logical::DrawingScopeKind::Block,
                },
                bounds: scope.bounds,
                entities: scope.entities.clone(),
            })
            .collect::<Vec<_>>();
        if let Some(error) = super::logical::validate_geometry_bounds(&geometric_entities, &scopes)
            .into_iter()
            .next()
        {
            return Err(DrawingBuildError::Invalid(format!(
                "{}: {}",
                error.location, error.message
            )));
        }
        let bytes = super::codec::json::encode_document_bytes(&self, &bounds)?;
        let read = load_drawing_bytes(&bytes);
        if read.status() != DrawingLoadStatus::Valid {
            return Err(DrawingBuildError::Invalid(format!(
                "{:?}",
                read.diagnostics()
            )));
        }
        Ok(EncodedDrawing { bytes })
    }
}

type ScopeBounds = Option<([f64; 3], [f64; 3])>;

fn resolve_scope_bounds(
    scope: usize,
    first_block_scope: usize,
    definitions: &[BlockDefinition],
    objects: &std::collections::BTreeMap<u64, &DrawingEntityRecord>,
    scope_entities: &std::collections::BTreeMap<u32, Vec<u64>>,
    cache: &mut [Option<ScopeBounds>],
    visiting: &mut [bool],
) -> Result<ScopeBounds, DrawingBuildError> {
    if scope >= cache.len() {
        return Err(DrawingBuildError::Invalid(format!("unknown scope {scope}")));
    }
    if let Some(bounds) = cache[scope] {
        return Ok(bounds);
    }
    if visiting[scope] {
        return Err(DrawingBuildError::Invalid("cyclic block instances".into()));
    }
    visiting[scope] = true;
    let mut bounds: ScopeBounds = None;
    for object in scope_entities
        .get(&(scope as u32))
        .into_iter()
        .flatten()
        .filter_map(|id| objects.get(id))
    {
        let (min, max) = if object.kind == "blockInstance" {
            let definition = match object.geometry {
                EntityGeometry::BlockInstance {
                    definition_scope_id,
                    ..
                } => definition_scope_id as usize,
                _ => unreachable!("block instance kind and geometry agree"),
            };
            if definition < first_block_scope || definition >= cache.len() {
                return Err(DrawingBuildError::Invalid(
                    "block instance references a non-block scope".into(),
                ));
            }
            let definition_bounds = resolve_scope_bounds(
                definition,
                first_block_scope,
                definitions,
                objects,
                scope_entities,
                cache,
                visiting,
            )?;
            if let Some((min, max)) = definition_bounds {
                let transform = match object.geometry {
                    EntityGeometry::BlockInstance { transform, .. } => transform,
                    _ => unreachable!("block instance geometry"),
                };
                let base = definitions[definition - first_block_scope].base_point;
                let prepared = super::geometry::PreparedBlockTransform::new(
                    transform,
                    super::Point3::new(base[0], base[1], base[2]),
                )
                .ok_or_else(|| {
                    DrawingBuildError::Invalid("block transform cannot be evaluated".into())
                })?;
                let local = std::array::from_fn(|axis| super::geometry::numeric::Interval {
                    lower: min[axis],
                    upper: max[axis],
                });
                let projected = prepared.apply_intervals(local).ok_or_else(|| {
                    DrawingBuildError::Invalid("block bounds are out of range".into())
                })?;
                (projected.map(|v| v.lower), projected.map(|v| v.upper))
            } else {
                (object.min, object.max)
            }
        } else {
            (object.min, object.max)
        };
        if !min.into_iter().chain(max).all(f64::is_finite) {
            return Err(DrawingBuildError::Invalid(
                "non-finite entity bounds".into(),
            ));
        }
        match &mut bounds {
            Some((scope_min, scope_max)) => {
                for axis in 0..3 {
                    scope_min[axis] = scope_min[axis].min(min[axis]);
                    scope_max[axis] = scope_max[axis].max(max[axis]);
                }
            }
            None => bounds = Some((min, max)),
        }
    }
    visiting[scope] = false;
    cache[scope] = Some(bounds);
    Ok(bounds)
}

fn logical_pair<T>(selection: &AppearanceSelection<T>) -> AppearancePair {
    match selection {
        AppearanceSelection::ByLayer => AppearancePair {
            mode: LogicalAppearanceMode::ByLayer,
            has_value: false,
        },
        AppearanceSelection::ByBlock => AppearancePair {
            mode: LogicalAppearanceMode::ByBlock,
            has_value: false,
        },
        AppearanceSelection::Explicit(_) => AppearancePair {
            mode: LogicalAppearanceMode::Explicit,
            has_value: true,
        },
    }
}

fn frame_at(origin: [f64; 3]) -> Result<crate::ocdraw::CoordinateFrame3, DrawingBuildError> {
    crate::ocdraw::CoordinateFrame3::try_new(
        crate::ocdraw::Point3::new(origin[0], origin[1], origin[2]),
        crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
        crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
    )
    .map_err(|error| DrawingBuildError::Invalid(error.to_string()))
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    fn model() -> DrawingModel {
        let mut builder = DrawingBuilder::new(DrawingOptions::new("owners", "mm")).unwrap();
        let layer = builder
            .add_layer(LayerDefinition::new(
                "0",
                super::super::RgbColor::new(255, 255, 255),
            ))
            .unwrap();
        builder.add_paper_layout("Sheet").unwrap();
        builder
            .add_line(LineDefinition::new(layer, [0.; 3], [1.; 3]))
            .unwrap();
        builder.logical_model().unwrap()
    }
    #[test]
    fn shared_validation_checks_membership_without_an_encoding_backing() {
        assert!(model().validate().is_empty());
        let mut duplicate = model();
        duplicate.scopes[0].entities.push(1);
        assert!(duplicate
            .validate()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP" && e.location == "/scopes/0/entities/1"));
        let mut multiple = model();
        multiple.scopes[1].entities.push(1);
        assert!(multiple
            .validate()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP"));
        let mut missing = model();
        missing.scopes[0].entities.push(99);
        assert!(missing
            .validate()
            .iter()
            .any(|e| e.code == "ENTITY_REFERENCE"));
        let mut orphan = model();
        orphan.scopes[0].entities.clear();
        assert!(orphan
            .validate()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP"));
    }
}
