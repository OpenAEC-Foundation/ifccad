//! Authored CAD editing preferences, independent of the active literal fill.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HatchClassicFill {
    Solid,
    LinePattern,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HatchCreationType {
    UserDefined,
    Predefined,
    Custom,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HatchClassicSettings {
    pub preferred_fill: Option<HatchClassicFill>,
    pub creation_type: Option<HatchCreationType>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub origin: Option<[f64; 2]>,
    pub rotation: f64,
    pub scale: f64,
    pub double: bool,
    pub definition_lines: Option<Vec<HatchPatternDefinitionLine>>,
}
impl Default for HatchClassicSettings {
    fn default() -> Self {
        Self {
            preferred_fill: None,
            creation_type: None,
            name: None,
            description: None,
            origin: None,
            rotation: 0.,
            scale: 1.,
            double: false,
            definition_lines: None,
        }
    }
}
impl HatchClassicSettings {
    pub fn is_empty(&self) -> bool {
        self.preferred_fill.is_none()
            && self.creation_type.is_none()
            && self.name.is_none()
            && self.description.is_none()
            && self.origin.is_none()
            && self.definition_lines.is_none()
            && self.rotation == 0.
            && self.scale == 1.
            && !self.double
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HatchBoundaryConstruction {
    pub seed_points: Vec<[f64; 2]>,
    pub pixel_size: f64,
}
impl HatchBoundaryConstruction {
    pub fn is_empty(&self) -> bool {
        self.seed_points.is_empty() && self.pixel_size == 0.
    }
}
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[error("Hatch editing {field}: {reason}")]
pub struct HatchEditingValidationError {
    pub field: &'static str,
    pub reason: String,
}
fn invalid(field: &'static str, reason: &str) -> HatchEditingValidationError {
    HatchEditingValidationError {
        field,
        reason: reason.into(),
    }
}
pub fn validate_hatch_classic_settings(
    c: &HatchClassicSettings,
) -> Result<(), HatchEditingValidationError> {
    if c.is_empty() {
        return Err(invalid("classicSettings", "empty record must be omitted"));
    }
    if !c.rotation.is_finite() {
        return Err(invalid("rotation", "must be finite"));
    }
    if !c.scale.is_finite() || c.scale <= 0. {
        return Err(invalid("scale", "must be finite and positive"));
    }
    if c.origin.is_some_and(|p| !p.into_iter().all(f64::is_finite)) {
        return Err(invalid("origin", "must be finite"));
    }
    if let Some(lines) = &c.definition_lines {
        for line in lines {
            validate_hatch_pattern_definition_line(line)?;
        }
    }
    Ok(())
}
pub fn validate_hatch_boundary_construction(
    c: &HatchBoundaryConstruction,
) -> Result<(), HatchEditingValidationError> {
    if c.is_empty() {
        return Err(invalid(
            "boundaryConstruction",
            "empty record must be omitted",
        ));
    }
    if !c.seed_points.iter().flatten().copied().all(f64::is_finite) {
        return Err(invalid("seedPoints", "must be finite"));
    }
    if !c.pixel_size.is_finite() || c.pixel_size < 0. {
        return Err(invalid("pixelSize", "must be finite and nonnegative"));
    }
    Ok(())
}

/// Direct stored definition in local Hatch XY; not an unscaled PAT family.
#[derive(Clone, Debug, PartialEq)]
pub struct HatchPatternDefinitionLine {
    pub angle: f64,
    pub base_point: [f64; 2],
    pub offset_vector: [f64; 2],
    pub dash_lengths: Vec<f64>,
}
pub fn validate_hatch_pattern_definition_line(
    line: &HatchPatternDefinitionLine,
) -> Result<(), HatchEditingValidationError> {
    if ![
        line.angle,
        line.base_point[0],
        line.base_point[1],
        line.offset_vector[0],
        line.offset_vector[1],
    ]
    .into_iter()
    .chain(line.dash_lengths.iter().copied())
    .all(f64::is_finite)
    {
        return Err(invalid("definitionLines", "values must be finite"));
    }
    if line.offset_vector == [0., 0.] {
        return Err(invalid("offsetVector", "must be nonzero"));
    }
    if !line.dash_lengths.is_empty() {
        use num_traits::Zero;
        let period = line
            .dash_lengths
            .iter()
            .map(|d| super::super::numeric::exact(d.abs()))
            .sum::<num_rational::BigRational>();
        if period.is_zero() || period > super::super::numeric::exact(f64::MAX) {
            return Err(invalid(
                "dashLengths",
                "total absolute period must be positive and finite",
            ));
        }
    }
    Ok(())
}
