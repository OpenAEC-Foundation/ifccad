//! Independent saved leader pictures. Commands retain state and transform order.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadLeaderGraphicsOrigin {
    SourceSnapshot,
    Generated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLeaderGraphics {
    pub origin: IfccadLeaderGraphicsOrigin,
    pub placement: IfccadPlacement,
    pub commands: Vec<IfccadLeaderGraphicCommand>,
}

/// Initial owner state is distinct from an authored ByBlock or ByLayer command.
#[derive(Clone, Debug, PartialEq)]
pub enum IfccadLeaderGraphicInheritance<T> {
    Owner,
    ByLayer,
    ByBlock,
    Explicit(T),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadLeaderGraphicLineWeight {
    Owner,
    ByLayer,
    ByBlock,
    DrawingDefault,
    Explicit(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadLeaderGraphicCommand {
    Draw(IfccadLeaderGraphicPrimitive),
    StoredExtents {
        min: [f64; 3],
        max: [f64; 3],
    },
    SetColor(IfccadLeaderGraphicInheritance<IfccadColor>),
    /// None selects the owner layer; Some is a native layer ID, never a table index.
    SetLayer(Option<u64>),
    SetLinePattern(IfccadLeaderGraphicInheritance<IfccadLinePatternId>),
    SetSelectionMarker(u32),
    SetFill(bool),
    SetLineWeight(IfccadLeaderGraphicLineWeight),
    SetLinePatternScale(f64),
    SetThickness(f64),
    /// Column-major matrix, independent of the picture's placement.
    PushTransform([f64; 16]),
    PopTransform,
    /// Exactly the stored CAD record51/null-word control; evaluation is unassessed.
    CadCompatibility51Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadLeaderGraphicArcClosure {
    Simple,
    Sector,
    Chord,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadLeaderGraphicPrimitive {
    Point([f64; 3]),
    Polyline {
        vertices: Vec<[f64; 3]>,
        normal: Option<[f64; 3]>,
    },
    Polygon {
        vertices: Vec<[f64; 3]>,
    },
    Circle {
        center: [f64; 3],
        radius: f64,
        normal: [f64; 3],
    },
    Circle3 {
        points: [[f64; 3]; 3],
    },
    Arc {
        center: [f64; 3],
        radius: f64,
        normal: [f64; 3],
        start_direction: [f64; 3],
        sweep_angle: f64,
        closure: IfccadLeaderGraphicArcClosure,
    },
    Arc3 {
        points: [[f64; 3]; 3],
        closure: IfccadLeaderGraphicArcClosure,
    },
    Text(Box<IfccadLeaderGraphicText>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadLeaderGraphicTextProcessing {
    Interpreted,
    Raw,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLeaderGraphicFont {
    pub family: String,
    pub font_file: String,
    pub big_font_file: String,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub charset: Option<u32>,
    pub pitch_family: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadLeaderGraphicText {
    pub position: [f64; 3],
    pub normal: [f64; 3],
    pub direction: [f64; 3],
    pub text: String,
    pub processing: IfccadLeaderGraphicTextProcessing,
    pub height: f64,
    pub width_factor: f64,
    pub oblique_angle: f64,
    /// Stored CAD tracking scalar. IO does not reinterpret it as an MTEXT percentage.
    pub tracking: f64,
    pub backwards: bool,
    pub upside_down: bool,
    pub vertical: bool,
    pub underline: bool,
    pub overline: bool,
    /// None stores no explicit font request. Older file-only requests leave
    /// descriptor properties absent rather than inventing false/default values.
    pub font: Option<IfccadLeaderGraphicFont>,
}

fn invalid(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-LEADER-GRAPHICS-001", path, detail)
}
fn finite(value: f64, path: &str) -> Result<(), IfccadReport> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(invalid(path, "nonfinite graphics value"))
    }
}
fn point(value: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    for value in value {
        finite(value, path)?;
    }
    Ok(())
}
fn positive(value: f64, path: &str) -> Result<(), IfccadReport> {
    finite(value, path)?;
    if value > 0. {
        Ok(())
    } else {
        Err(invalid(path, "graphics value must be positive"))
    }
}
fn nonnegative(value: f64, path: &str) -> Result<(), IfccadReport> {
    finite(value, path)?;
    if value >= 0. {
        Ok(())
    } else {
        Err(invalid(path, "graphics value must be nonnegative"))
    }
}
fn direction(value: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    point(value, path)?;
    if value.iter().any(|value| *value != 0.) {
        Ok(())
    } else {
        Err(invalid(path, "zero graphics direction"))
    }
}
fn independent(a: [num_rational::BigRational; 3], b: [num_rational::BigRational; 3]) -> bool {
    (0..3).any(|i| &a[(i + 1) % 3] * &b[(i + 2) % 3] != &a[(i + 2) % 3] * &b[(i + 1) % 3])
}
fn exact(value: [f64; 3]) -> [num_rational::BigRational; 3] {
    value.map(|value| {
        num_rational::BigRational::from_float(value).expect("finite graphics components checked")
    })
}
fn three_points(points: &[[f64; 3]; 3], path: &str) -> Result<(), IfccadReport> {
    for value in points {
        point(*value, path)?;
    }
    let [a, b, c] = points.map(exact);
    let u = std::array::from_fn(|i| &b[i] - &a[i]);
    let v = std::array::from_fn(|i| &c[i] - &a[i]);
    if independent(u, v) {
        Ok(())
    } else {
        Err(invalid(
            path,
            "three-point curve requires noncollinear points",
        ))
    }
}
fn frame(normal: [f64; 3], axis: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    direction(normal, path)?;
    direction(axis, path)?;
    if independent(exact(normal), exact(axis)) {
        Ok(())
    } else {
        Err(invalid(path, "parallel graphics normal and direction"))
    }
}

impl IfccadLeaderGraphicPrimitive {
    fn validate(&self, path: &str) -> Result<(), IfccadReport> {
        match self {
            Self::Point(value) => point(*value, path),
            Self::Polyline { vertices, normal } => {
                if vertices.len() < 2 {
                    return Err(invalid(path, "polyline needs two vertices"));
                }
                for value in vertices {
                    point(*value, path)?;
                }
                if let Some(normal) = normal {
                    direction(*normal, path)?;
                }
                Ok(())
            }
            Self::Polygon { vertices } => {
                if vertices.len() < 3 {
                    return Err(invalid(path, "polygon needs three vertices"));
                }
                for value in vertices {
                    point(*value, path)?;
                }
                Ok(())
            }
            Self::Circle {
                center,
                radius,
                normal,
            } => {
                point(*center, path)?;
                positive(*radius, path)?;
                direction(*normal, path)
            }
            Self::Circle3 { points } | Self::Arc3 { points, .. } => three_points(points, path),
            Self::Arc {
                center,
                radius,
                normal,
                start_direction,
                sweep_angle,
                ..
            } => {
                point(*center, path)?;
                positive(*radius, path)?;
                frame(*normal, *start_direction, path)?;
                finite(*sweep_angle, path)
            }
            Self::Text(text) => {
                point(text.position, path)?;
                frame(text.normal, text.direction, path)?;
                nonnegative(text.height, path)?;
                positive(text.width_factor, path)?;
                finite(text.oblique_angle, path)?;
                nonnegative(text.tracking, path)?;
                if text.text.contains('\0') {
                    return Err(invalid(path, "graphics string contains NUL"));
                }
                if let Some(font) = &text.font {
                    for value in [&font.family, &font.font_file, &font.big_font_file] {
                        if value.contains('\0') {
                            return Err(invalid(path, "graphics string contains NUL"));
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

impl IfccadLeaderGraphics {
    /// Validate this snapshot's values and native resource references. Does not
    /// validate unrelated document contents, evaluate commands, or check freshness.
    pub fn validate(&self, document: &IfccadDocument) -> Result<(), IfccadReport> {
        use IfccadLeaderGraphicCommand as C;
        self.placement.coordinate_frame()?;
        let mut depth = 0usize;
        for (index, command) in self.commands.iter().enumerate() {
            let path = format!("/leaderGraphics/commands/{index}");
            match command {
                C::Draw(value) => value.validate(&path)?,
                C::StoredExtents { min, max } => {
                    point(*min, &path)?;
                    point(*max, &path)?;
                    if (0..3).any(|i| min[i] > max[i]) {
                        return Err(invalid(&path, "reversed stored extents"));
                    }
                }
                C::SetColor(IfccadLeaderGraphicInheritance::Explicit(color)) => {
                    if !color.is_valid() {
                        return Err(invalid(&path, "invalid graphics color metadata"));
                    }
                }
                C::SetLayer(Some(id)) => {
                    if !document.layers.iter().any(|layer| layer.id == *id) {
                        return Err(invalid(&path, "unresolved graphics layer"));
                    }
                }
                C::SetLinePattern(IfccadLeaderGraphicInheritance::Explicit(id)) => {
                    if !document
                        .line_patterns
                        .iter()
                        .any(|pattern| pattern.id == *id)
                    {
                        return Err(invalid(&path, "unresolved graphics line pattern"));
                    }
                }
                C::SetLineWeight(IfccadLeaderGraphicLineWeight::Explicit(weight)) => {
                    nonnegative(*weight, &path)?
                }
                C::SetLinePatternScale(scale) => positive(*scale, &path)?,
                C::SetThickness(thickness) => finite(*thickness, &path)?,
                C::PushTransform(matrix) => {
                    for value in matrix {
                        finite(*value, &path)?;
                    }
                    depth = depth
                        .checked_add(1)
                        .ok_or_else(|| invalid(&path, "graphics stack overflow"))?;
                }
                C::PopTransform => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or_else(|| invalid(&path, "graphics stack underflow"))?;
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(invalid(
                "/leaderGraphics/commands",
                "unbalanced graphics transform stack",
            ));
        }
        Ok(())
    }
}
