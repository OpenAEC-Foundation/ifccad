use crate::ifcdr::{Bounds3d, Point2, Point3, Vector3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IfcdrScopeKind {
    ModelSpace,
    PaperSpace,
    BlockDefinition,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrScope {
    pub id: u32,
    pub kind: IfcdrScopeKind,
    pub bounds: Option<Bounds3d>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrBlockDefinition {
    pub scope_id: u32,
    pub name: String,
    pub base_point: Point3,
    pub description: String,
    pub anonymous: bool,
    pub insertion_unit: crate::ifcdr::IfcdrLengthUnit,
    pub explodable: bool,
    pub scaling: crate::ifcdr::BlockScaling,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrBlockInstanceRow {
    pub entity: IfcdrEntityRow,
    pub definition_scope_id: u32,
    pub transform: crate::ifcdr::geometry::BlockTransformComponents,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportFrame {
    pub center: Point2,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionMode {
    Orthographic,
    Perspective,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontClipMode {
    Disabled,
    AtCamera,
    AtDistance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrontClip {
    pub mode: FrontClipMode,
    pub distance: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackClipMode {
    Disabled,
    AtDistance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackClip {
    pub mode: BackClipMode,
    pub distance: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewDefinition {
    pub center: Point2,
    pub target: Point3,
    pub direction: Vector3,
    pub height: f64,
    pub twist: f64,
    pub projection: ProjectionMode,
    pub lens_length: Option<f64>,
    pub front_clip: FrontClip,
    pub back_clip: BackClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewportRenderMode {
    TwoDimensional,
    Wireframe,
    HiddenLine,
    FlatShadedWithoutEdges,
    FlatShadedWithEdges,
    SmoothShadedWithoutEdges,
    SmoothShadedWithEdges,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperClip {
    pub enabled: bool,
    pub boundary_entity_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadedPlotMode {
    AsDisplayed,
    Wireframe,
    Hidden,
    Rendered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadedPlotQualityMode {
    Draft,
    Preview,
    Normal,
    Presentation,
    Maximum,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadedPlotQuality {
    pub mode: ShadedPlotQualityMode,
    pub dpi: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadedPlot {
    pub mode: ShadedPlotMode,
    pub quality: ShadedPlotQuality,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportLayerOverride {
    pub layer_id: u32,
    pub frozen: bool,
    pub appearance_override_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrViewportRow {
    pub entity: IfcdrEntityRow,
    pub view_scope_id: u32,
    pub frame: ViewportFrame,
    pub view: ViewDefinition,
    pub render_mode: ViewportRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub paper_clip: PaperClip,
    pub plot_shading_override: Option<ShadedPlot>,
    pub layer_overrides: Vec<ViewportLayerOverride>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrLayerBinding {
    pub id: u32,
    pub ifcx_layer: String,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrAppearanceBinding {
    pub id: u32,
    pub ifcx_appearance: Option<String>,
    pub modes: [u32; 4],
    pub override_id: Option<u32>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrAppearanceOverride {
    pub id: u32,
    pub color: Option<IfcdrColor>,
    pub opacity: Option<f64>,
    pub ifcx_line_pattern: Option<String>,
    pub line_weight: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrScopeOrder {
    pub scope_id: u32,
    pub entities: Vec<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrColor {
    pub(crate) rgb: [u8; 3],
    pub(crate) indexed: Option<IfcdrIndexedColor>,
    pub(crate) named: Option<IfcdrNamedColor>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrIndexedColor {
    pub system: String,
    pub index: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrNamedColor {
    pub catalog: String,
    pub name: String,
}

impl IfcdrColor {
    pub fn rgb_components(&self) -> [u8; 3] {
        self.rgb
    }
    pub fn indexed(&self) -> Option<&IfcdrIndexedColor> {
        self.indexed.as_ref()
    }
    pub fn named(&self) -> Option<&IfcdrNamedColor> {
        self.named.as_ref()
    }
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: [red, green, blue],
            indexed: None,
            named: None,
        }
    }
    pub fn with_indexed(mut self, system: impl Into<String>, index: u64) -> Self {
        self.indexed = Some(IfcdrIndexedColor {
            system: system.into(),
            index,
        });
        self
    }
    pub fn with_named(mut self, catalog: impl Into<String>, name: impl Into<String>) -> Self {
        self.named = Some(IfcdrNamedColor {
            catalog: catalog.into(),
            name: name.into(),
        });
        self
    }
}

pub(crate) fn valid_polyline_vertex_count(count: usize) -> bool {
    count >= 2
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrEntityRow {
    pub entity_id: u64,
    pub scope_id: u32,
    pub layer_id: u32,
    pub appearance_id: u32,
    pub visible: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrLineRow {
    pub entity: IfcdrEntityRow,
    pub start: Point3,
    pub end: Point3,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrSpatialPolylineRow {
    pub entity: IfcdrEntityRow,
    pub closed: bool,
    pub points: Vec<Point3>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrPointRow {
    pub entity: IfcdrEntityRow,
    pub placement: crate::ifcdr::geometry::PlanePlacementComponents,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrCircleRow {
    pub entity: IfcdrEntityRow,
    pub placement: crate::ifcdr::geometry::PlanePlacementComponents,
    pub radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrArcRow {
    pub entity: IfcdrEntityRow,
    pub placement: crate::ifcdr::geometry::PlanePlacementComponents,
    pub radius: f64,
    pub start_parameter: f64,
    pub sweep_parameter: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrEllipseRow {
    pub entity: IfcdrEntityRow,
    pub placement: crate::ifcdr::geometry::PlanePlacementComponents,
    pub semi_major_radius: f64,
    pub semi_minor_radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrEllipseArcRow {
    pub entity: IfcdrEntityRow,
    pub placement: crate::ifcdr::geometry::PlanePlacementComponents,
    pub semi_major_radius: f64,
    pub semi_minor_radius: f64,
    pub start_parameter: f64,
    pub sweep_parameter: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceMode {
    ByLayer,
    ByBlock,
    Explicit,
}

pub(crate) fn appearance_mode(value: u32) -> Option<AppearanceMode> {
    match value {
        0 => Some(AppearanceMode::ByLayer),
        1 => Some(AppearanceMode::Explicit),
        2 => Some(AppearanceMode::ByBlock),
        _ => None,
    }
}
pub(crate) fn length_unit(value: &str) -> Option<crate::ifcdr::IfcdrLengthUnit> {
    use crate::ifcdr::IfcdrLengthUnit::*;
    Some(match value {
        "unitless" => Unitless,
        "mm" => Millimetre,
        "cm" => Centimetre,
        "m" => Metre,
        "km" => Kilometre,
        "in" => Inch,
        "ft" => Foot,
        "mi" => Mile,
        "microin" => Microinch,
        "mil" => Mil,
        "yd" => Yard,
        "angstrom" => Angstrom,
        "nm" => Nanometre,
        "um" => Micrometre,
        "dm" => Decimetre,
        "dam" => Decametre,
        "hm" => Hectometre,
        "Gm" => Gigametre,
        "au" => AstronomicalUnit,
        "ly" => LightYear,
        "pc" => Parsec,
        "usSurveyFoot" => UsSurveyFoot,
        "usSurveyInch" => UsSurveyInch,
        "usSurveyYard" => UsSurveyYard,
        "usSurveyMile" => UsSurveyMile,
        _ => return None,
    })
}
pub(crate) fn rgb_channel(value: u64) -> Option<u8> {
    u8::try_from(value).ok()
}
pub(crate) fn valid_opacity(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
pub(crate) fn valid_line_weight(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}
pub(crate) fn valid_point3(point: Point3) -> bool {
    point.components().into_iter().all(f64::is_finite)
}
pub(crate) fn valid_point(point: Point2) -> bool {
    point.x().is_finite() && point.y().is_finite()
}
pub(crate) fn valid_color(color: &IfcdrColor) -> bool {
    color.indexed.as_ref().is_none_or(|v| !v.system.is_empty())
        && color
            .named
            .as_ref()
            .is_none_or(|v| !v.catalog.is_empty() && !v.name.is_empty())
}
