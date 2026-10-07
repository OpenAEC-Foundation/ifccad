mod preservation;
mod state;
use super::logical::{DrawingColor, DrawingEntityRecord, EntityAppearance, EntityGeometry};
use super::EncodedOcdraw;
use super::{LayoutSettings, PointDisplay, UcsDefinition};
pub use preservation::OpaqueEntityDefinition;
pub use state::{DrawingSavedState, ViewportDefinition};

#[derive(Clone, Debug)]
pub struct OcdrawBuildOptions {
    pub drawing_id: String,
    pub unit: String,
}

pub use crate::plot_kernel::PlotStyleMode;

impl OcdrawBuildOptions {
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
    pub line_pattern_id: super::LinePatternId,
    pub line_weight: f64,
}

impl LayerDefinition {
    pub fn new(
        name: impl Into<String>,
        color: impl Into<DrawingColor>,
        line_pattern_id: super::LinePatternId,
    ) -> Self {
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
            line_pattern_id,
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
    pub line_pattern_generation: super::LinePatternGeneration,
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
            line_pattern_generation: super::LinePatternGeneration::PerSegment,
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
    pub line_pattern_generation: super::LinePatternGeneration,
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
            line_pattern_generation: super::LinePatternGeneration::PerSegment,
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
pub enum OcdrawBuildError {
    #[error("drawing content is invalid: {0}")]
    Invalid(String),
    #[error("drawing ID space is exhausted")]
    IdExhausted,
    #[error("drawing serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct OcdrawBuilder {
    pub(crate) preservation: Option<super::OcdrawPreservation>,
    pub(crate) opaque_entities: Vec<super::DrawingOpaqueEntity>,
    pub(crate) options: OcdrawBuildOptions,
    pub(crate) model_layout_name: String,
    pub(crate) line_patterns: Vec<super::LinePatternDefinition>,
    pub(crate) line_pattern_scale: f64,
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

impl OcdrawBuilder {
    pub fn new(options: OcdrawBuildOptions) -> Result<Self, OcdrawBuildError> {
        if options.drawing_id.is_empty() || options.unit.is_empty() {
            return Err(OcdrawBuildError::Invalid(
                "drawing identity and unit must be nonempty".into(),
            ));
        }
        Ok(Self {
            preservation: None,
            opaque_entities: Vec::new(),
            options,
            model_layout_name: "Model".into(),
            line_patterns: Vec::new(),
            line_pattern_scale: 1.0,
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

    pub fn add_paper_layout(&mut self, name: impl Into<String>) -> Result<u32, OcdrawBuildError> {
        if !self.block_definitions.is_empty() {
            return Err(OcdrawBuildError::Invalid(
                "paper layouts must be added before block definitions".into(),
            ));
        }
        let id = u32::try_from(self.paper_layouts.len())
            .map_err(|_| OcdrawBuildError::IdExhausted)?
            .checked_add(1)
            .ok_or(OcdrawBuildError::IdExhausted)?;
        self.paper_layouts.push(name.into());
        self.layout_settings.push(LayoutSettings::default());
        Ok(id)
    }

    pub fn add_block_definition(
        &mut self,
        definition: impl Into<BlockDefinition>,
    ) -> Result<u32, OcdrawBuildError> {
        let id = self
            .paper_layouts
            .len()
            .checked_add(self.block_definitions.len())
            .and_then(|count| count.checked_add(1))
            .and_then(|count| u32::try_from(count).ok())
            .ok_or(OcdrawBuildError::IdExhausted)?;
        self.block_definitions.push(definition.into());
        Ok(id)
    }

    pub fn set_model_layout_name(&mut self, name: impl Into<String>) {
        self.model_layout_name = name.into();
    }

    pub fn add_ucs_definition(
        &mut self,
        definition: UcsDefinition,
    ) -> Result<u32, OcdrawBuildError> {
        if definition.name.is_empty() || !definition.elevation.is_finite() {
            return Err(OcdrawBuildError::Invalid("invalid UCS definition".into()));
        }
        let id =
            u32::try_from(self.ucs_definitions.len()).map_err(|_| OcdrawBuildError::IdExhausted)?;
        self.ucs_definitions.push(definition);
        Ok(id)
    }

    pub fn set_layout_settings(
        &mut self,
        layout_id: u32,
        settings: LayoutSettings,
    ) -> Result<(), OcdrawBuildError> {
        crate::plot_kernel::validate_layout_output(
            &settings,
            if layout_id == 0 {
                crate::plot_kernel::LayoutOutputKind::Model
            } else {
                crate::plot_kernel::LayoutOutputKind::Paper
            },
        )
        .map_err(|e| OcdrawBuildError::Invalid(format!("invalid layout output: {e:?}")))?;
        let slot = self
            .layout_settings
            .get_mut(layout_id as usize)
            .ok_or_else(|| OcdrawBuildError::Invalid("unknown layout ID".into()))?;
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

    pub fn set_point_display(&mut self, display: PointDisplay) -> Result<(), OcdrawBuildError> {
        if !display.is_valid() {
            return Err(OcdrawBuildError::Invalid(
                "invalid point display size".into(),
            ));
        }
        self.point_display = Some(display);
        Ok(())
    }

    /// Allocates a local ID after validating the definition and folded name.
    pub fn add_line_pattern(
        &mut self,
        definition: super::LinePatternDefinition,
    ) -> Result<super::LinePatternId, OcdrawBuildError> {
        let id =
            u32::try_from(self.line_patterns.len()).map_err(|_| OcdrawBuildError::IdExhausted)?;
        id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        let row = super::DrawingLinePattern {
            id: super::LinePatternId(id),
            name: definition.name.clone(),
            description: definition.description.clone(),
            pattern: definition.pattern.clone(),
        };
        let errors = super::logical::line_pattern_validation::validate_line_patterns(
            &[row],
            id + 1,
            &[],
            &[],
        );
        if !errors.is_empty()
            || self.line_patterns.iter().any(|p| {
                super::names::name_key(&p.name) == super::names::name_key(&definition.name)
            })
        {
            return Err(OcdrawBuildError::Invalid(
                "invalid line pattern definition".into(),
            ));
        }
        self.line_patterns.push(definition);
        Ok(super::LinePatternId(id))
    }
    /// Reuses or creates the named empty Continuous definition; its ID is not fixed.
    pub fn ensure_continuous_line_pattern(
        &mut self,
    ) -> Result<super::LinePatternId, OcdrawBuildError> {
        if let Some(i) = self
            .line_patterns
            .iter()
            .position(|p| super::names::name_key(&p.name) == super::names::name_key("Continuous"))
        {
            return Ok(super::LinePatternId(i as u32));
        }
        self.add_line_pattern(super::LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
    }
    /// Sets a finite positive drawing scale without changing definition lengths.
    pub fn set_line_pattern_scale(&mut self, scale: f64) -> Result<(), OcdrawBuildError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(OcdrawBuildError::Invalid(
                "invalid line pattern scale".into(),
            ));
        }
        self.line_pattern_scale = scale;
        Ok(())
    }
    pub fn add_layer(&mut self, layer: LayerDefinition) -> Result<u32, OcdrawBuildError> {
        let id = u32::try_from(self.layers.len()).map_err(|_| OcdrawBuildError::IdExhausted)?;
        self.layers.push(layer);
        Ok(id)
    }

    pub fn add_line(&mut self, line: LineDefinition) -> Result<u64, OcdrawBuildError> {
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
    ) -> Result<u64, OcdrawBuildError> {
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
                .ok_or_else(|| OcdrawBuildError::Invalid("invalid entity geometry".into()))?,
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

    pub fn add_point(&mut self, point: PointDefinition) -> Result<u64, OcdrawBuildError> {
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

    pub fn add_circle(&mut self, circle: CircleDefinition) -> Result<u64, OcdrawBuildError> {
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(circle.center[0], circle.center[1], circle.center[2]),
            crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
            crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
        )
        .map_err(|error| OcdrawBuildError::Invalid(error.to_string()))?;
        let enclosure =
            crate::ocdraw::geometry::circular_bounds(frame.components(), circle.radius, None)
                .ok_or_else(|| {
                    OcdrawBuildError::Invalid(
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

    pub fn add_arc(&mut self, arc: ArcDefinition) -> Result<u64, OcdrawBuildError> {
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(arc.center[0], arc.center[1], arc.center[2]),
            crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
            crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
        )
        .map_err(|error| OcdrawBuildError::Invalid(error.to_string()))?;
        let enclosure = crate::ocdraw::geometry::circular_bounds(
            frame.components(),
            arc.radius,
            Some((arc.start_parameter, arc.sweep_parameter)),
        )
        .ok_or_else(|| OcdrawBuildError::Invalid("invalid arc geometry".into()))?;
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

    pub fn add_ellipse(&mut self, ellipse: EllipseDefinition) -> Result<u64, OcdrawBuildError> {
        self.add_elliptic(ellipse, false)
    }

    pub fn add_ellipse_arc(&mut self, ellipse: EllipseDefinition) -> Result<u64, OcdrawBuildError> {
        self.add_elliptic(ellipse, true)
    }

    fn add_elliptic(
        &mut self,
        ellipse: EllipseDefinition,
        partial: bool,
    ) -> Result<u64, OcdrawBuildError> {
        if partial != ellipse.arc.is_some() {
            return Err(OcdrawBuildError::Invalid(
                "ellipse arc requires a sweep; full ellipse forbids one".into(),
            ));
        }
        let frame = crate::ocdraw::CoordinateFrame3::try_new(
            crate::ocdraw::Point3::new(ellipse.center[0], ellipse.center[1], ellipse.center[2]),
            crate::ocdraw::Vector3::new(ellipse.x_axis[0], ellipse.x_axis[1], ellipse.x_axis[2]),
            crate::ocdraw::Vector3::new(ellipse.y_axis[0], ellipse.y_axis[1], ellipse.y_axis[2]),
        )
        .map_err(|error| OcdrawBuildError::Invalid(error.to_string()))?;
        let enclosure = crate::ocdraw::geometry::elliptic_bounds(
            frame.components(),
            ellipse.semi_major_radius,
            ellipse.semi_minor_radius,
            ellipse.arc,
        )
        .ok_or_else(|| OcdrawBuildError::Invalid("invalid ellipse geometry".into()))?;
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
    ) -> Result<u64, OcdrawBuildError> {
        if polyline.vertices.len() < 2 {
            return Err(OcdrawBuildError::Invalid(
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
            .ok_or_else(|| OcdrawBuildError::Invalid("invalid polyline segment".into()))?;
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
                    line_pattern_generation: polyline.line_pattern_generation,
                },
                min,
                max,
            },
        )
    }

    pub fn add_spatial_polyline(
        &mut self,
        polyline: SpatialPolylineDefinition,
    ) -> Result<u64, OcdrawBuildError> {
        if polyline.vertices.len() < 2
            || polyline
                .vertices
                .iter()
                .flatten()
                .any(|value| !value.is_finite())
        {
            return Err(OcdrawBuildError::Invalid(
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
                    line_pattern_generation: polyline.line_pattern_generation,
                },
                min,
                max,
            },
        )
    }

    pub fn add_block_instance(
        &mut self,
        instance: BlockInstanceDefinition,
    ) -> Result<u64, OcdrawBuildError> {
        let transform = crate::ocdraw::BlockTransform::try_new(
            frame_at(instance.origin)?,
            0.0,
            crate::ocdraw::Scale3::default(),
        )
        .map_err(|error| OcdrawBuildError::Invalid(error.to_string()))?;
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
    ) -> Result<u64, OcdrawBuildError> {
        let id = self.next_entity_id;
        self.next_entity_id = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        row.id = id;
        self.scope_entities.entry(scope_id).or_default().push(id);
        self.objects.push(row);
        Ok(id)
    }

    /// Builds complete typed content without encoding it. Bounds are prepared automatically.
    pub fn build_document(self) -> Result<super::OcdrawDocument, OcdrawBuildError> {
        use super::logical::*;
        for row in &self.objects {
            if !row.min.into_iter().chain(row.max).all(f64::is_finite) {
                return Err(OcdrawBuildError::Invalid(format!(
                    "non-finite {} bounds",
                    row.kind
                )));
            }
        }
        let count = |n| u32::try_from(n).map_err(|_| OcdrawBuildError::IdExhausted);
        let next_layer_id = count(self.layers.len())?;
        let next_line_pattern_id = count(self.line_patterns.len())?;
        let next_layout_id = count(self.paper_layouts.len() + 1)?;
        let first_block_scope = self.paper_layouts.len() + 1;
        count(first_block_scope + self.block_definitions.len())?;
        let mut names = vec![self.model_layout_name];
        names.extend(self.paper_layouts);
        let layouts = names
            .into_iter()
            .zip(self.layout_settings)
            .enumerate()
            .map(|(i, (name, settings))| DrawingLayout {
                id: i as u32,
                scope_id: i as u32,
                kind: if i == 0 {
                    DrawingLayoutKind::Model
                } else {
                    DrawingLayoutKind::Paper
                },
                name,
                tab_index: i as u32,
                settings,
            })
            .collect::<Vec<_>>();
        let mut scope_entities = self.scope_entities;
        let mut scopes = layouts
            .iter()
            .map(|l| DrawingScope {
                id: l.scope_id,
                kind: if l.kind == DrawingLayoutKind::Model {
                    DrawingScopeKind::Model
                } else {
                    DrawingScopeKind::Paper
                },
                bounds: None,
                entities: scope_entities.remove(&l.scope_id).unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        let block_definitions = self
            .block_definitions
            .into_iter()
            .enumerate()
            .map(|(i, b)| {
                let scope_id = (first_block_scope + i) as u32;
                scopes.push(DrawingScope {
                    id: scope_id,
                    kind: DrawingScopeKind::Block,
                    bounds: None,
                    entities: scope_entities.remove(&scope_id).unwrap_or_default(),
                });
                DrawingBlockDefinition {
                    scope_id,
                    name: b.name,
                    base_point: b.base_point,
                    description: b.description,
                    anonymous: b.anonymous,
                    insertion_unit: b.insertion_unit,
                    explodable: b.explodable,
                    uniform_scaling: b.uniform_scaling,
                }
            })
            .collect();
        if !scope_entities.is_empty() {
            return Err(OcdrawBuildError::Invalid(
                "entity owner scope does not exist".into(),
            ));
        }
        let mut doc = OcdrawDocument {
            preservation: self.preservation,
            opaque_entities: self.opaque_entities,
            next_entity_id: self.next_entity_id,
            next_layer_id,
            next_layout_id,
            next_line_pattern_id,
            drawing_id: self.options.drawing_id,
            unit: self.options.unit,
            plot_style_mode: self.plot_style_mode,
            line_pattern_scale: self.line_pattern_scale,
            point_display: self.point_display,
            geometric_entities: self
                .objects
                .into_iter()
                .map(|e| DrawingGeometricEntity {
                    id: e.id,
                    layer_id: e.layer_id,
                    visible: e.visible,
                    appearance: e.appearance,
                    geometry: e.geometry,
                })
                .collect(),
            viewports: self.viewports,
            layers: self
                .layers
                .into_iter()
                .enumerate()
                .map(|(i, l)| DrawingLayer {
                    id: i as u32,
                    name: l.name,
                    description: l.description,
                    visible: l.visible,
                    frozen: l.frozen,
                    locked: l.locked,
                    plottable: l.plottable,
                    frozen_in_new_viewports: l.frozen_in_new_viewports,
                    color: l.color,
                    opacity: l.opacity,
                    line_pattern_id: l.line_pattern_id,
                    line_weight: l.line_weight,
                })
                .collect(),
            line_patterns: self
                .line_patterns
                .into_iter()
                .enumerate()
                .map(|(i, p)| DrawingLinePattern {
                    id: super::LinePatternId(i as u32),
                    name: p.name,
                    description: p.description,
                    pattern: p.pattern,
                })
                .collect(),
            layouts,
            scopes,
            block_definitions,
            ucs_definitions: self
                .ucs_definitions
                .into_iter()
                .enumerate()
                .map(|(i, definition)| DrawingUcsDefinition {
                    id: i as u32,
                    definition,
                })
                .collect(),
            workspace_state: if self.current_layer.is_some() || self.active_layout.is_some() {
                Some(DrawingWorkspaceState {
                    current_layer_id: self.current_layer,
                    active_layout_id: self.active_layout,
                })
            } else {
                None
            },
            view_state: self.saved_state.view_state,
            model_windows: self.saved_state.model_windows,
            paper_canvases: self.saved_state.paper_canvases,
            viewport_workspaces: self.saved_state.viewport_workspaces,
        };
        super::recompute_ocdraw_document_bounds(&mut doc)
            .map_err(|e| OcdrawBuildError::Invalid(format!("{:?}", e.diagnostics())))?;
        super::validate_ocdraw_document(&doc)
            .map_err(|e| OcdrawBuildError::Invalid(format!("{:?}", e.diagnostics())))?;
        Ok(doc)
    }

    /// Convenience route: build a complete document, then use the shared encoder.
    pub fn finish(self) -> Result<EncodedOcdraw, OcdrawBuildError> {
        super::encode_ocdraw_document(&self.build_document()?).map_err(OcdrawBuildError::from)
    }
}

fn frame_at(origin: [f64; 3]) -> Result<crate::ocdraw::CoordinateFrame3, OcdrawBuildError> {
    crate::ocdraw::CoordinateFrame3::try_new(
        crate::ocdraw::Point3::new(origin[0], origin[1], origin[2]),
        crate::ocdraw::Vector3::new(1.0, 0.0, 0.0),
        crate::ocdraw::Vector3::new(0.0, 1.0, 0.0),
    )
    .map_err(|error| OcdrawBuildError::Invalid(error.to_string()))
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    fn model() -> super::super::OcdrawDocument {
        let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("owners", "mm")).unwrap();
        builder.ensure_continuous_line_pattern().unwrap();
        let layer = builder
            .add_layer(LayerDefinition::new(
                "0",
                super::super::RgbColor::new(255, 255, 255),
                crate::ocdraw::LinePatternId(0),
            ))
            .unwrap();
        builder.add_paper_layout("Sheet").unwrap();
        builder
            .add_line(LineDefinition::new(layer, [0.; 3], [1.; 3]))
            .unwrap();
        builder.build_document().unwrap()
    }
    #[test]
    fn shared_validation_checks_membership_without_an_encoding_backing() {
        assert!(super::super::validate_ocdraw_document(&model()).is_ok());
        let mut duplicate = model();
        duplicate.scopes[0].entities.push(1);
        assert!(super::super::validate_ocdraw_document(&duplicate)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP" && e.location == "/scopes/0/entities/1"));
        let mut multiple = model();
        multiple.scopes[1].entities.push(1);
        assert!(super::super::validate_ocdraw_document(&multiple)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP"));
        let mut missing = model();
        missing.scopes[0].entities.push(99);
        assert!(super::super::validate_ocdraw_document(&missing)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|e| e.code == "ENTITY_REFERENCE"));
        let mut orphan = model();
        orphan.scopes[0].entities.clear();
        assert!(super::super::validate_ocdraw_document(&orphan)
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|e| e.code == "ENTITY_OWNERSHIP"));
    }
}
