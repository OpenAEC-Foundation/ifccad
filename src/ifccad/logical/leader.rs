//! Leader-family values. Annotation identity is separate from owned presentations.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadLeaderPath {
    StraightSegments(Vec<[f64; 3]>),
    /// Interpolation intent only; no particular spline evaluator is prescribed.
    SplineThroughVertices(Vec<[f64; 3]>),
}
impl IfccadLeaderPath {
    pub fn vertices(&self) -> &[[f64; 3]] {
        match self {
            Self::StraightSegments(points) | Self::SplineThroughVertices(points) => points,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadLeaderAnnotationKind {
    None,
    MText,
    Block,
    Tolerance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadLeaderHookDirection {
    OppositeHorizontal,
    AlongHorizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadLeaderArrow {
    #[default]
    DefaultClosedFilled,
    None,
    Symbol {
        definition_id: u64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLeader {
    pub placement: IfccadPlacement,
    pub path: IfccadLeaderPath,
    pub annotation_kind: IfccadLeaderAnnotationKind,
    /// Ordinary same-owner entity, never an owned text or graphic command.
    pub annotation_entity: Option<u64>,
    pub dimension_style: IfccadDimensionStyleId,
    pub overrides: IfccadDimensionOverrides,
    /// Independent leader-arrow override; does not replace DIMBLK/DIMBLK1/DIMBLK2.
    pub arrow: Option<IfccadLeaderArrow>,
    pub arrow_enabled: bool,
    pub hook_enabled: bool,
    pub hook_direction: IfccadLeaderHookDirection,
    /// Stored [width, height]; absence is distinct from explicit zero dimensions.
    pub annotation_box: Option<[f64; 2]>,
    pub horizontal_direction: [f64; 3],
    pub block_offset: [f64; 3],
    pub annotation_offset: [f64; 3],
    pub origin: [f64; 3],
    pub annotation: Option<IfccadAnnotation<IfccadLeaderAnnotationContext>>,
    pub graphics: Option<IfccadLeaderGraphics>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLeaderAnnotationContext {
    pub points: Vec<[f64; 3]>,
    pub horizontal_direction: [f64; 3],
    pub annotation_enabled: bool,
    pub insertion_offset: [f64; 3],
    pub endpoint_projection: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadToleranceNotation {
    /// Literal CAD feature-control-frame controls, not parsed MText or a field cache.
    CadFeatureControlTextV1 { code: String },
}
impl IfccadToleranceNotation {
    pub fn code(&self) -> &str {
        match self {
            Self::CadFeatureControlTextV1 { code } => code,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadTolerance {
    pub placement: IfccadPlacement,
    pub dimension_style: IfccadDimensionStyleId,
    pub overrides: IfccadDimensionOverrides,
    pub notation: IfccadToleranceNotation,
    pub annotation: Option<IfccadAnnotation<IfccadToleranceAnnotationContext>>,
    pub graphics: Option<IfccadLeaderGraphics>,
}

/// A stored FCF scale presentation; base placement and notation stay independent.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadToleranceAnnotationContext {
    pub location: [f64; 3],
    pub horizontal_direction: [f64; 3],
}

fn invalid(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-LEADER-001", path, detail)
}
fn point(value: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    if value.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(invalid(path, "coordinates must be finite"))
    }
}
fn direction(value: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    point(value, path)?;
    if value == [0.; 3] {
        Err(invalid(path, "direction must be nonzero"))
    } else {
        Ok(())
    }
}
pub(super) fn style(
    drawing: &IfccadDocument,
    id: IfccadDimensionStyleId,
    overrides: &IfccadDimensionOverrides,
    path: &str,
) -> Result<(), IfccadReport> {
    let source = drawing
        .dimension_styles
        .iter()
        .find(|style| style.id == id)
        .ok_or_else(|| invalid(path, "unresolved DimensionStyle"))?;
    // Dimension subtype overrides have no Leader/Tolerance subtype. Retain all
    // explicit entity overrides, including values equal to the style settings.
    let mut effective = source.settings.clone();
    overrides.apply(&mut effective);
    super::dimension_validation::settings(drawing, path, &source.settings)?;
    super::dimension_validation::settings(drawing, path, &effective)
}

impl IfccadLeader {
    /// Validate values and resource closure. Entity ownership/target kind is a
    /// separate drawing-graph check; this method does not resolve that relationship.
    pub fn validate_values(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        self.placement.coordinate_frame()?;
        style(
            drawing,
            self.dimension_style,
            &self.overrides,
            "/leader/style",
        )?;
        for (index, value) in self.path.vertices().iter().enumerate() {
            point(*value, &format!("/leader/path/{index}"))?;
        }
        direction(self.horizontal_direction, "/leader/horizontalDirection")?;
        point(self.block_offset, "/leader/blockOffset")?;
        point(self.annotation_offset, "/leader/annotationOffset")?;
        point(self.origin, "/leader/origin")?;
        if self.annotation_box.is_some_and(|dimensions| {
            dimensions
                .into_iter()
                .any(|value| !value.is_finite() || value < 0.)
        }) {
            return Err(invalid(
                "/leader/annotationBox",
                "dimensions must be finite and nonnegative",
            ));
        }
        if let Some(IfccadLeaderArrow::Symbol { definition_id }) = self.arrow {
            if !drawing.blocks.iter().any(|block| block.id == definition_id) {
                return Err(invalid("/leader/arrow", "unresolved arrow block"));
            }
        }
        if let Some(annotation) = &self.annotation {
            annotation.validate_memberships(&drawing.annotation_scales, false)?;
            for (index, context) in annotation.contexts.iter().enumerate() {
                if let Some(context) = &context.presentation {
                    let path = format!("/leader/annotation/contexts/{index}");
                    for (index, value) in context.points.iter().enumerate() {
                        point(*value, &format!("{path}/points/{index}"))?;
                    }
                    direction(
                        context.horizontal_direction,
                        &format!("{path}/horizontalDirection"),
                    )?;
                    point(context.insertion_offset, &format!("{path}/insertionOffset"))?;
                    point(
                        context.endpoint_projection,
                        &format!("{path}/endpointProjection"),
                    )?;
                }
            }
        }
        if let Some(graphics) = &self.graphics {
            graphics.validate(drawing)?;
        }
        Ok(())
    }
}

impl IfccadToleranceNotation {
    pub fn validate(&self) -> Result<(), IfccadReport> {
        if self.code().contains('\0') || self.code().contains("%<") {
            return Err(invalid(
                "/tolerance/notation/code",
                "NUL or an unbound embedded field is unsupported",
            ));
        }
        Ok(())
    }
}
impl IfccadTolerance {
    /// Validate the literal notation, resources and independently stored contexts.
    pub fn validate_values(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        self.placement.coordinate_frame()?;
        style(
            drawing,
            self.dimension_style,
            &self.overrides,
            "/tolerance/style",
        )?;
        self.notation.validate()?;
        if let Some(annotation) = &self.annotation {
            annotation.validate_memberships(&drawing.annotation_scales, false)?;
            for (index, context) in annotation.contexts.iter().enumerate() {
                if let Some(context) = &context.presentation {
                    let path = format!("/tolerance/annotation/contexts/{index}");
                    point(context.location, &format!("{path}/location"))?;
                    direction(
                        context.horizontal_direction,
                        &format!("{path}/horizontalDirection"),
                    )?;
                }
            }
        }
        if let Some(graphics) = &self.graphics {
            graphics.validate(drawing)?;
        }
        Ok(())
    }
}
