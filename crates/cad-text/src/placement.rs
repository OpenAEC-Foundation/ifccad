//! Source/target parameter preparation, using the shared exact/paired kernels.
//! No font geometry, document IDs, target ownership, tolerance verdict or file IO.
use crate::*;
use cad_geometry_convert::{
    geometry::{
        self,
        blocks::{EvaluatedBlock, PairedPoint},
        numeric::{exact, round_nearest, sqrt_interval},
    },
    GeometryPair,
};
use num_rational::BigRational as Q;
use num_traits::Zero;
use ocdraw::geometry_kernel::{
    BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3 as NativeVector,
};
use ocdraw::text::*;
use opencadcodec::{
    entities::{TextHorizontalAlignment as H, TextVerticalAlignment as V},
    EntityType, MText, Text, Vector3,
};

pub struct PreparedText {
    pub style_name: String,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub layout: TextLayout,
    pub oblique_angle: f64,
    pub thickness: f64,
    pub content: Vec<TextRun>,
    pub pair: GeometryPair,
    pub normal_normalized: bool,
    pub glyph_geometry_unassessed: bool,
}
pub struct PreparedMText {
    pub style_name: String,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub height: f64,
    pub attachment: MTextAttachment,
    pub flow: MTextFlow,
    pub wrap_width: Option<f64>,
    pub columns: Option<MTextColumns>,
    pub background: Option<MTextBackground<MarkupColor>>,
    pub content: ParsedMTextContent,
    pub pair: GeometryPair,
    pub issues: Vec<CadTextIssue>,
    pub normal_normalized: bool,
    pub glyph_geometry_unassessed: bool,
}
pub struct TextCadInput<'a> {
    pub style_name: &'a str,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub layout: TextLayout,
    pub oblique_angle: f64,
    pub thickness: f64,
    pub content: &'a [TextRun],
}
pub struct PreparedCadText {
    pub entity: EntityType,
    pub pair: GeometryPair,
    pub parameterization_changed: bool,
    pub glyph_geometry_unassessed: bool,
    pub issues: Vec<CadTextIssue>,
}

pub struct MTextCadInput<'a> {
    pub style_name: &'a str,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub height: f64,
    pub attachment: MTextAttachment,
    pub flow: MTextFlow,
    pub wrap_width: Option<f64>,
    pub columns: Option<&'a MTextColumns>,
    pub background: Option<&'a MTextBackground<MarkupColor>>,
    pub content: &'a [MTextParagraph<MarkupColor>],
    pub character_format: &'a CharacterFormat<MarkupColor>,
    pub paragraph_format: &'a ParagraphFormat,
}

pub fn prepare_mtext_to_cad(input: &MTextCadInput<'_>) -> Result<PreparedCadText, CadTextError> {
    crate::styles::name(input.style_name)?;
    positive(input.height)?;
    finite(input.rotation)?;
    if input.backward || input.upside_down {
        return Err(CadTextError::Unsupported(
            "independent MText mirror flags have no qualified direction/normal mapping",
        ));
    }
    validate_character_format(input.character_format, crate::content::valid_color)?;
    validate_mtext_content_with_paragraph_format(
        input.content,
        MTextValueContext {
            nominal_height: input.height,
            wrap_width: input.wrap_width,
            columns: input.columns,
        },
        input.paragraph_format,
        crate::content::valid_color,
    )?;
    let normal = geometry::stored_normal(input.placement)
        .ok_or(CadTextError::InvalidSource("invalid MText normal"))?;
    let (s, c) = input.rotation.sin_cos();
    let x = input.placement.x_axis().components();
    let y = input.placement.y_axis().components();
    let direction = cv(std::array::from_fn(|i| c * x[i] + s * y[i]));
    vec_finite(direction)?;
    let mut target = MText::new();
    target.style = input.style_name.into();
    target.height = input.height;
    target.insertion_point = cv(input.placement.origin().components());
    target.normal = normal;
    target.rotation = direction.y.atan2(direction.x);
    target.dwg_x_direction = Some(direction);
    target.attachment_point = crate::layout::cad_attachment(input.attachment);
    target.drawing_direction = crate::layout::cad_flow(input.flow);
    let mut issues = Vec::new();
    crate::layout::columns_to_cad(input, &mut target)?;
    crate::layout::background_to_cad(input, &mut target, &mut issues)?;
    let mut paragraph_format = input.paragraph_format.clone();
    if let Some(spacing) = paragraph_format.line_spacing.take() {
        crate::layout::spacing_to_cad(spacing, &mut target, &mut issues)?;
    }
    if paragraph_format != ParagraphFormat::default() {
        issues.push(CadTextIssue::ParagraphBasisExpanded);
    }
    target.value = emit_mtext(
        MTextMarkupInput {
            content: input.content,
            character_format: input.character_format,
            paragraph_format: &paragraph_format,
        },
        Default::default(),
    )?;
    let decoded = parse_mtext(&target.value, Default::default())?;
    if decoded.character_format != *input.character_format
        || decoded.paragraph_format != paragraph_format
        || decoded.content != input.content
    {
        issues.push(CadTextIssue::AuthoredFormattingNormalized);
    }
    let origin = input.placement.origin().components();
    Ok(PreparedCadText {
        entity: EntityType::MText(target),
        pair: pair(origin.map(exact), origin.map(exact)),
        parameterization_changed: input.rotation != 0.,
        glyph_geometry_unassessed: true,
        issues,
    })
}

pub fn prepare_text_from_cad(source: &Text) -> Result<PreparedText, CadTextError> {
    crate::styles::name(&source.style)?;
    positive(source.height)?;
    positive(source.width_factor)?;
    finite(source.rotation)?;
    finite(source.thickness)?;
    if source.generation_flags & !6 != 0 {
        return Err(CadTextError::Unsupported("unknown text generation flags"));
    }
    let matrix = matrix(source.normal)?;
    let dual = matches!(source.horizontal_alignment, H::Aligned | H::Fit);
    let active = if dual
        || (source.horizontal_alignment == H::Left && source.vertical_alignment == V::Baseline)
    {
        source.insertion_point
    } else {
        source.alignment_point.ok_or(CadTextError::InvalidSource(
            "active alignment point missing",
        ))?
    };
    let origin_exact = ocs(matrix, active)?;
    let origin = point(round(origin_exact.clone())?);
    let mut frame =
        CoordinateFrame3::try_new(origin, nv(column(matrix, 0)), nv(column(matrix, 1)))?;
    let mut rotation = source.rotation;
    let layout = match source.horizontal_alignment {
        H::Aligned | H::Fit => {
            if source.vertical_alignment != V::Baseline {
                return Err(CadTextError::Unsupported(
                    "aligned/fit vertical alignment combination",
                ));
            }
            let end = source
                .alignment_point
                .ok_or(CadTextError::InvalidSource("aligned/fit end missing"))?;
            vec_finite(end)?;
            if active.z != end.z {
                return Err(CadTextError::InvalidSource(
                    "aligned/fit endpoints are not coplanar",
                ));
            }
            let end_exact = ocs(matrix, end)?;
            let delta: [Q; 3] = std::array::from_fn(|i| &end_exact[i] - &origin_exact[i]);
            let square: Q = delta.iter().map(|v| v * v).sum();
            if square.is_zero() {
                return Err(CadTextError::InvalidSource(
                    "aligned/fit baseline length is zero",
                ));
            }
            let length = sqrt_interval(&square).ok_or(CadTextError::OutOfRange)?.1;
            let direction = cv(round(delta)?);
            let normal = cv(column(matrix, 2));
            let (x, y) = geometry::orthonormal_pair(direction, normal.cross(&direction)).ok_or(
                CadTextError::InvalidSource("baseline direction cannot define text frame"),
            )?;
            frame = CoordinateFrame3::try_new(origin, nv(xyz(x)), nv(xyz(y)))?;
            rotation = 0.;
            if source.horizontal_alignment == H::Aligned {
                TextLayout::Aligned {
                    length,
                    width_factor: source.width_factor,
                }
            } else {
                TextLayout::Fit {
                    length,
                    height: source.height,
                }
            }
        }
        H::Middle => {
            if source.vertical_alignment != V::Baseline {
                return Err(CadTextError::Unsupported(
                    "whole-middle vertical alignment combination",
                ));
            }
            TextLayout::WholeTextMiddle {
                height: source.height,
                width_factor: source.width_factor,
            }
        }
        h => TextLayout::Anchored {
            height: source.height,
            width_factor: source.width_factor,
            horizontal: match h {
                H::Left => TextHorizontalAlignment::Left,
                H::Center => TextHorizontalAlignment::Center,
                _ => TextHorizontalAlignment::Right,
            },
            vertical: match source.vertical_alignment {
                V::Baseline => TextVerticalAlignment::Baseline,
                V::Bottom => TextVerticalAlignment::Bottom,
                V::Middle => TextVerticalAlignment::Middle,
                V::Top => TextVerticalAlignment::Top,
            },
        },
    };
    validate_text_layout(&layout)?;
    validate_character_format(
        &CharacterFormat::<MarkupColor> {
            oblique_angle: Some(source.oblique_angle),
            ..Default::default()
        },
        |_| true,
    )?;
    let mut pair = pair(origin_exact.clone(), origin.components().map(exact));
    let native = evaluated(frame, rotation)?;
    if dual {
        let length = match layout {
            TextLayout::Aligned { length, .. } | TextLayout::Fit { length, .. } => length,
            _ => unreachable!(),
        };
        let source_end = ocs(matrix, source.alignment_point.unwrap())?;
        let target = evaluated_from_axes(
            origin.components().map(exact),
            frame.x_axis().components(),
            frame.y_axis().components(),
            normal(frame),
            0.,
        );
        let identity = evaluated(CoordinateFrame3::default(), 0.)?;
        let mut p = PairedPoint::new(source_end, [exact(length), Q::zero(), Q::zero()]);
        p.apply(&identity, &target);
        add_pair(&mut pair, p);
    }
    if source.thickness != 0. {
        let source_eval = evaluated_from_axes(
            origin_exact,
            column(matrix, 0),
            column(matrix, 1),
            column(matrix, 2),
            source.rotation,
        );
        let mut p = PairedPoint::exact([0., 0., source.thickness]);
        p.apply(&source_eval, &native);
        add_pair(&mut pair, p);
    }
    Ok(PreparedText {
        style_name: source.style.clone(),
        placement: frame,
        rotation,
        backward: source.generation_flags & 2 != 0,
        upside_down: source.generation_flags & 4 != 0,
        layout,
        oblique_angle: source.oblique_angle,
        thickness: source.thickness,
        content: parse_text(&source.value, Default::default())?,
        pair,
        normal_normalized: geometry::stored_normal(frame) != Some(source.normal),
        glyph_geometry_unassessed: true,
    })
}

pub fn prepare_mtext_from_cad(source: &MText) -> Result<PreparedMText, CadTextError> {
    crate::styles::name(&source.style)?;
    positive(source.height)?;
    finite(source.rotation)?;
    vec_finite(source.insertion_point)?;
    if source.is_annotative {
        return Err(CadTextError::Unsupported("annotative MText contexts"));
    }
    let n = geometry::cad_plane(source.normal)
        .ok_or(CadTextError::InvalidSource("invalid MText normal"))?
        .n;
    let direction = source
        .dwg_x_direction
        .filter(|dir| dir.y.atan2(dir.x) == source.rotation)
        .unwrap_or_else(|| Vector3::new(source.rotation.cos(), source.rotation.sin(), 0.));
    vec_finite(direction)?;
    let (x, unit_normal) = geometry::orthonormal_pair(direction, cv(n)).ok_or(
        CadTextError::InvalidSource("invalid MText direction/normal relationship"),
    )?;
    let unit_x = geometry::cad_plane(direction)
        .ok_or(CadTextError::InvalidSource("zero MText direction"))?
        .n;
    CoordinateFrame3::try_new(Point3::new(0., 0., 0.), nv(unit_x), nv(n))
        .map_err(|_| CadTextError::InvalidSource("MText direction is outside its plane"))?;
    let y = unit_normal.cross(&x);
    let frame =
        CoordinateFrame3::try_new(point(xyz(source.insertion_point)), nv(xyz(x)), nv(xyz(y)))?;
    let mut content = parse_mtext(&source.value, Default::default())?;
    let mut issues = Vec::new();
    if content.character_basis_normalized {
        issues.push(CadTextIssue::CharacterBasisNormalized);
    }
    let columns = crate::layout::columns_from_cad(source, &mut issues)?;
    let wrap_width = if columns.is_some() {
        None
    } else {
        finite(source.rectangle_width)?;
        if source.rectangle_width < 0. {
            return Err(CadTextError::InvalidSource("negative wrap width"));
        }
        if source.rectangle_width == 0. {
            None
        } else {
            Some(source.rectangle_width)
        }
    };
    crate::layout::spacing_from_cad(source, &mut content.paragraph_format, &mut issues)?;
    let background = crate::layout::background_from_cad(source, &mut issues)?;
    validate_mtext_content_with_paragraph_format(
        &content.content,
        MTextValueContext {
            nominal_height: source.height,
            wrap_width,
            columns: columns.as_ref(),
        },
        &content.paragraph_format,
        crate::content::valid_color,
    )?;
    let origin = xyz(source.insertion_point);
    Ok(PreparedMText {
        style_name: source.style.clone(),
        placement: frame,
        rotation: 0.,
        height: source.height,
        attachment: crate::layout::attachment(source.attachment_point),
        flow: crate::layout::flow(source.drawing_direction),
        wrap_width,
        columns,
        background,
        content,
        pair: pair(origin.map(exact), origin.map(exact)),
        issues,
        normal_normalized: geometry::stored_normal(frame) != Some(source.normal),
        glyph_geometry_unassessed: true,
    })
}

pub fn prepare_text_to_cad(input: &TextCadInput<'_>) -> Result<PreparedCadText, CadTextError> {
    crate::styles::name(input.style_name)?;
    validate_text_layout(&input.layout)?;
    finite(input.rotation)?;
    finite(input.thickness)?;
    validate_character_format(
        &CharacterFormat::<MarkupColor> {
            oblique_angle: Some(input.oblique_angle),
            ..Default::default()
        },
        |_| true,
    )?;
    let native = evaluated(input.placement, input.rotation)?;
    let normal = geometry::stored_normal(input.placement)
        .ok_or(CadTextError::InvalidSource("invalid stored text normal"))?;
    let matrix = matrix(normal)?;
    let origin = input.placement.origin().components();
    let local = project(matrix, origin)?;
    let mut target = Text::new();
    target.style = input.style_name.into();
    target.value = emit_text(input.content)?;
    target.normal = normal;
    target.insertion_point = cv(local);
    target.oblique_angle = input.oblique_angle;
    target.thickness = input.thickness;
    target.generation_flags =
        (if input.backward { 2 } else { 0 }) | (if input.upside_down { 4 } else { 0 });
    let compatible = input.placement.x_axis().components() == column(matrix, 0)
        && input.placement.y_axis().components() == column(matrix, 1);
    target.rotation = if compatible {
        input.rotation
    } else {
        let (s, c) = input.rotation.sin_cos();
        let x = input.placement.x_axis().components();
        let y = input.placement.y_axis().components();
        let direction: [f64; 3] = std::array::from_fn(|i| c * x[i] + s * y[i]);
        let u = dot(direction, column(matrix, 0));
        let v = dot(direction, column(matrix, 1));
        v.atan2(u)
    };
    match input.layout {
        TextLayout::Anchored {
            horizontal,
            vertical,
            height,
            width_factor,
        } => {
            target.height = height;
            target.width_factor = width_factor;
            target.horizontal_alignment = match horizontal {
                TextHorizontalAlignment::Left => H::Left,
                TextHorizontalAlignment::Center => H::Center,
                TextHorizontalAlignment::Right => H::Right,
            };
            target.vertical_alignment = match vertical {
                TextVerticalAlignment::Baseline => V::Baseline,
                TextVerticalAlignment::Bottom => V::Bottom,
                TextVerticalAlignment::Middle => V::Middle,
                TextVerticalAlignment::Top => V::Top,
            };
            if target.horizontal_alignment != H::Left || target.vertical_alignment != V::Baseline {
                target.alignment_point = Some(cv(local));
            }
        }
        TextLayout::WholeTextMiddle {
            height,
            width_factor,
        } => {
            target.height = height;
            target.width_factor = width_factor;
            target.horizontal_alignment = H::Middle;
            target.alignment_point = Some(cv(local));
        }
        TextLayout::Aligned { width_factor, .. } => {
            target.height = 1.;
            target.width_factor = width_factor;
            target.horizontal_alignment = H::Aligned;
        }
        TextLayout::Fit { height, .. } => {
            target.height = height;
            target.width_factor = 1.;
            target.horizontal_alignment = H::Fit;
        }
    }
    let target_origin = ocs(matrix, target.insertion_point)?;
    let target_eval = evaluated_from_axes(
        target_origin.clone(),
        column(matrix, 0),
        column(matrix, 1),
        column(matrix, 2),
        target.rotation,
    );
    let mut proofs = pair(origin.map(exact), target_origin);
    if let TextLayout::Aligned { length, .. } | TextLayout::Fit { length, .. } = input.layout {
        let mut end = PairedPoint::exact([length, 0., 0.]);
        end.apply(&native, &target_eval);
        add_pair(&mut proofs, end);
        target.alignment_point = Some(Vector3::new(
            finite_sum(local[0], length * target.rotation.cos())?,
            finite_sum(local[1], length * target.rotation.sin())?,
            local[2],
        ));
        // Actual constructed CAD endpoints, not the idealized rotated pair.
        let actual_end = ocs(matrix, target.alignment_point.unwrap())?;
        let identity = evaluated(CoordinateFrame3::default(), 0.)?;
        let mut end = PairedPoint::new([exact(length), Q::zero(), Q::zero()], actual_end);
        end.apply(&native, &identity);
        proofs.points.pop();
        add_pair(&mut proofs, end);
    }
    if input.thickness != 0. {
        let mut p = PairedPoint::exact([0., 0., input.thickness]);
        p.apply(&native, &target_eval);
        add_pair(&mut proofs, p);
    }
    Ok(PreparedCadText {
        entity: EntityType::Text(target),
        pair: proofs,
        parameterization_changed: !compatible,
        glyph_geometry_unassessed: true,
        issues: Vec::new(),
    })
}

pub(crate) fn finite(value: f64) -> Result<(), CadTextError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(CadTextError::InvalidSource("nonfinite text parameter"))
    }
}
pub(crate) fn positive(value: f64) -> Result<(), CadTextError> {
    finite(value)?;
    if value > 0. {
        Ok(())
    } else {
        Err(CadTextError::InvalidSource(
            "text parameter must be positive",
        ))
    }
}
fn vec_finite(v: Vector3) -> Result<(), CadTextError> {
    for n in xyz(v) {
        finite(n)?;
    }
    Ok(())
}
fn finite_sum(a: f64, b: f64) -> Result<f64, CadTextError> {
    let value = a + b;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CadTextError::OutOfRange)
    }
}
fn xyz(v: Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn point(v: [f64; 3]) -> Point3 {
    Point3::new(v[0], v[1], v[2])
}
fn nv(v: [f64; 3]) -> NativeVector {
    NativeVector::new(v[0], v[1], v[2])
}
fn cv(v: [f64; 3]) -> Vector3 {
    Vector3::new(v[0], v[1], v[2])
}
fn column(m: [[f64; 3]; 3], i: usize) -> [f64; 3] {
    [m[0][i], m[1][i], m[2][i]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn normal(frame: CoordinateFrame3) -> [f64; 3] {
    let x = frame.x_axis().components();
    let y = frame.y_axis().components();
    [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ]
}
fn matrix(normal: Vector3) -> Result<[[f64; 3]; 3], CadTextError> {
    vec_finite(normal)?;
    if normal == Vector3::ZERO {
        return Err(CadTextError::InvalidSource("zero text normal"));
    }
    let m = opencadcodec::types::Matrix3::arbitrary_axis(normal).m;
    if m.iter().flatten().any(|v| !v.is_finite()) {
        return Err(CadTextError::OutOfRange);
    }
    CoordinateFrame3::try_new(Point3::new(0., 0., 0.), nv(column(m, 0)), nv(column(m, 1)))?;
    Ok(m)
}
fn ocs(m: [[f64; 3]; 3], p: Vector3) -> Result<[Q; 3], CadTextError> {
    vec_finite(p)?;
    let p = xyz(p).map(exact);
    Ok(std::array::from_fn(|i| {
        (0..3).map(|j| exact(m[i][j]) * &p[j]).sum()
    }))
}
fn round(p: [Q; 3]) -> Result<[f64; 3], CadTextError> {
    let mut out = [0.; 3];
    for (i, v) in p.iter().enumerate() {
        out[i] = round_nearest(v).map_err(|_| CadTextError::OutOfRange)?;
    }
    Ok(out)
}
fn project(m: [[f64; 3]; 3], p: [f64; 3]) -> Result<[f64; 3], CadTextError> {
    round(std::array::from_fn(|i| {
        (0..3).map(|j| exact(m[j][i]) * exact(p[j])).sum()
    }))
}
fn pair(source: [Q; 3], target: [Q; 3]) -> GeometryPair {
    let mut pair = GeometryPair {
        points: Vec::new(),
        curves: Vec::new(),
    };
    add_pair(&mut pair, PairedPoint::new(source, target));
    pair
}
fn add_pair(pair: &mut GeometryPair, p: PairedPoint) {
    let squared = p.squared_deviation().1;
    pair.points.push((p, squared));
}
fn evaluated(frame: CoordinateFrame3, rotation: f64) -> Result<EvaluatedBlock, CadTextError> {
    let t = BlockTransform::try_new(frame, rotation, Scale3::default())
        .map_err(|_| CadTextError::InvalidSource("invalid text rotation"))?;
    Ok(EvaluatedBlock::native(t, [0.; 3]))
}
fn evaluated_from_axes(
    origin: [Q; 3],
    x: [f64; 3],
    y: [f64; 3],
    n: [f64; 3],
    rotation: f64,
) -> EvaluatedBlock {
    EvaluatedBlock {
        origin,
        base: std::array::from_fn(|_| Q::zero()),
        rotation,
        cosine: std::array::from_fn(|i| [exact(x[i]), exact(y[i]), Q::zero()]),
        sine: std::array::from_fn(|i| [exact(y[i]), -exact(x[i]), Q::zero()]),
        constant: std::array::from_fn(|i| [Q::zero(), Q::zero(), exact(n[i])]),
    }
}
