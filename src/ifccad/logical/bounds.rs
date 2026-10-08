//! Independent enclosure evidence, estimates and atomic preparation.
use super::*;
use crate::geometry_kernel::{
    geometry_bounds, numeric::Interval, paper_frame_bounds, BlockTransform, PaperFrame, Point3,
    PreparedBlockTransform, Scale3,
};
use crate::text::*;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IfccadScopeId {
    Layout(u64),
    BlockDefinition(u64),
}
type Owner = IfccadScopeId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadGeometryCompleteness {
    Empty,
    Complete,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadBoundsQuality {
    Enclosing,
    Estimated,
    Partial,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadScopeBoundsAssessment {
    pub scope_id: IfccadScopeId,
    pub bounds: Option<IfccadBounds3d>,
    pub quality: Option<IfccadBoundsQuality>,
    pub enclosure_verified: bool,
    pub text_reasons: Vec<TextExtentReason>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadBoundsAssessment {
    pub scopes: BTreeMap<IfccadScopeId, IfccadScopeBoundsAssessment>,
}
#[derive(Clone, Default)]
struct Extents {
    all: Option<IfccadBounds3d>,
    proven: Option<IfccadBounds3d>,
    unavailable: bool,
    estimated: bool,
    reasons: Vec<TextExtentReason>,
}
impl Extents {
    fn bounds(&self) -> Option<IfccadBounds3d> {
        if self.unavailable {
            None
        } else {
            self.all
        }
    }
    fn quality(&self) -> Option<IfccadBoundsQuality> {
        if self.unavailable {
            Some(IfccadBoundsQuality::Partial)
        } else if self.all.is_none() {
            None
        } else if self.estimated {
            Some(IfccadBoundsQuality::Estimated)
        } else {
            Some(IfccadBoundsQuality::Enclosing)
        }
    }
    fn completeness(&self) -> IfccadGeometryCompleteness {
        if self.unavailable {
            IfccadGeometryCompleteness::Unavailable
        } else if self.all.is_some() {
            IfccadGeometryCompleteness::Complete
        } else {
            IfccadGeometryCompleteness::Empty
        }
    }
    fn merge(&mut self, other: Self) {
        if let Some(b) = other.all {
            union(&mut self.all, b);
        }
        if let Some(b) = other.proven {
            union(&mut self.proven, b);
        }
        self.unavailable |= other.unavailable;
        self.estimated |= other.estimated;
        for r in other.reasons {
            if !self.reasons.contains(&r) {
                self.reasons.push(r);
            }
        }
    }
}
fn error(owner: Owner, message: &str) -> IfccadReport {
    let path = match owner {
        Owner::Layout(id) => format!("/layout/{id}/ifccad::layout/bounds"),
        Owner::BlockDefinition(id) => format!("/block/{id}/ifccad::blockDefinition/bounds"),
    };
    crate::ifccad::diagnostics::failure("IFCCAD-BOUNDS-002", &path, message)
}
fn union(target: &mut Option<IfccadBounds3d>, value: IfccadBounds3d) {
    match target {
        Some(b) => {
            for i in 0..3 {
                b.min[i] = b.min[i].min(value.min[i]);
                b.max[i] = b.max[i].max(value.max[i]);
            }
        }
        None => *target = Some(value),
    }
}
fn native(b: crate::geometry_kernel::Bounds3d) -> IfccadBounds3d {
    IfccadBounds3d {
        min: b.min().components(),
        max: b.max().components(),
    }
}
fn frame_stroke(
    d: &IfccadDocument,
    owner: Owner,
    weight: Option<f64>,
) -> Result<Option<f64>, TextExtentError> {
    use num_rational::BigRational as Q;
    use num_traits::ToPrimitive;
    let Some(weight) = weight else {
        return Ok(None);
    };
    if weight == 0. {
        return Ok(Some(0.));
    }
    let q = |n: i64, den: i64| Q::new(n.into(), den.into());
    let paper = match owner {
        Owner::Layout(id) => d.paper_layouts.iter().find(|p| p.id == id),
        _ => None,
    };
    let scale = if let Some(paper) = paper {
        let Some(plot) = paper.settings.plot_settings.as_ref() else {
            return Ok(None);
        };
        let crate::plot_kernel::PlotScale::Fixed {
            output_length,
            scope_length,
        } = plot.mapping.scale
        else {
            return Ok(None);
        };
        if !output_length.is_finite()
            || output_length <= 0.
            || !scope_length.is_finite()
            || scope_length <= 0.
        {
            return Err(TextExtentError::OutOfRange);
        }
        let metres = match plot.plot_unit {
            crate::plot_kernel::PlotUnit::Millimetre => q(1, 1000),
            crate::plot_kernel::PlotUnit::Inch => q(127, 5000),
            crate::plot_kernel::PlotUnit::Pixel => return Ok(None),
        };
        Q::from_float(scope_length).ok_or(TextExtentError::OutOfRange)?
            / (metres * Q::from_float(output_length).ok_or(TextExtentError::OutOfRange)?)
    } else {
        let Some((_, upper)) =
            crate::geometry_kernel::CoordinateLengthUnit::from_token(&d.length_unit)
                .and_then(|u| u.coordinates_per_metre())
        else {
            return Ok(None);
        };
        upper
    };
    let exact = Q::from_float(weight).ok_or(TextExtentError::OutOfRange)? * q(1, 1000) * scale;
    let mut value = exact
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or(TextExtentError::OutOfRange)?;
    if Q::from_float(value).ok_or(TextExtentError::OutOfRange)? < exact {
        value = value.next_up();
    }
    if !value.is_finite() {
        return Err(TextExtentError::OutOfRange);
    }
    Ok(Some(value))
}
struct Evaluation<'a> {
    drawing: &'a IfccadDocument,
    contents: BTreeMap<Owner, &'a [IfccadEntity]>,
    blocks: BTreeMap<u64, &'a IfccadBlockDefinition>,
    completed: BTreeMap<Owner, Extents>,
}
impl<'a> Evaluation<'a> {
    fn new(d: &'a IfccadDocument) -> Self {
        Self {
            drawing: d,
            contents: std::iter::once((Owner::Layout(d.model.id), d.model.entities.as_slice()))
                .chain(
                    d.paper_layouts
                        .iter()
                        .map(|p| (Owner::Layout(p.id), p.entities.as_slice())),
                )
                .chain(
                    d.blocks
                        .iter()
                        .map(|b| (Owner::BlockDefinition(b.id), b.entities.as_slice())),
                )
                .collect(),
            blocks: d.blocks.iter().map(|b| (b.id, b)).collect(),
            completed: BTreeMap::new(),
        }
    }
    fn scope(&mut self, owner: Owner) -> Result<Extents, IfccadReport> {
        if let Some(b) = self.completed.get(&owner) {
            return Ok(b.clone());
        }
        let mut combined = Extents::default();
        for e in self.contents[&owner] {
            let Some(e) = e.as_native() else {
                combined.unavailable = true;
                continue;
            };
            let value = match &e.kind {
                IfccadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } => {
                    let block = self.blocks[definition_id];
                    let mut local = self.scope(Owner::BlockDefinition(*definition_id))?;
                    let t = BlockTransform::try_new(
                        transform.placement.coordinate_frame()?,
                        transform.rotation,
                        Scale3::new(transform.scale[0], transform.scale[1], transform.scale[2]),
                    )
                    .map_err(|e| error(owner, &e.to_string()))?;
                    let base = block.base_point;
                    let prepared =
                        PreparedBlockTransform::new(t, Point3::new(base[0], base[1], base[2]))
                            .ok_or_else(|| error(owner, "cannot evaluate block transform"))?;
                    let apply = |b: IfccadBounds3d| -> Result<IfccadBounds3d, IfccadReport> {
                        let result = prepared
                            .apply_intervals(std::array::from_fn(|i| Interval {
                                lower: b.min[i],
                                upper: b.max[i],
                            }))
                            .ok_or_else(|| {
                                error(owner, "transformed block enclosure exceeds finite range")
                            })?;
                        Ok(IfccadBounds3d {
                            min: result.map(|v| v.lower),
                            max: result.map(|v| v.upper),
                        })
                    };
                    if block.entities.is_empty() && local.all.is_none() && !local.unavailable {
                        let b = IfccadBounds3d {
                            min: transform.placement.origin,
                            max: transform.placement.origin,
                        };
                        local.all = Some(b);
                        local.proven = Some(b);
                    } else {
                        local.all = local.all.map(apply).transpose()?;
                        local.proven = local.proven.map(apply).transpose()?;
                    }
                    local
                }
                IfccadEntityKind::Text(t) => {
                    let style = self
                        .drawing
                        .text_styles
                        .iter()
                        .find(|s| s.id == t.style_id)
                        .expect("validated style");
                    let estimate = estimate_text_extent(&ResolvedTextExtentInput {
                        content: &t.content,
                        layout: t.layout,
                        placement: t.placement.coordinate_frame()?,
                        rotation: t.rotation,
                        backward: t.backward,
                        upside_down: t.upside_down,
                        oblique_angle: t.oblique_angle,
                        thickness: t.thickness,
                        vertical: style.properties.vertical,
                    })
                    .map_err(|e| error(owner, &e.to_string()))?;
                    Self::estimate(estimate)
                }
                IfccadEntityKind::MText(t) => {
                    let style = self
                        .drawing
                        .text_styles
                        .iter()
                        .find(|s| s.id == t.style_id)
                        .expect("validated style");
                    let weight = match e.appearance.line_weight {
                        IfccadMode::Explicit(w) => Some(w),
                        IfccadMode::ByLayer => self
                            .drawing
                            .layers
                            .iter()
                            .find(|l| l.id == e.layer_id)
                            .map(|l| l.appearance.line_weight),
                        IfccadMode::ByBlock => None,
                    };
                    let estimate = estimate_mtext_extent(&ResolvedMTextExtentInput {
                        content: &t.content,
                        style: &style.properties,
                        character_format: &t.character_format,
                        paragraph_format: &t.paragraph_format,
                        height: t.height,
                        attachment: t.attachment,
                        flow: t.flow,
                        wrap_width: t.wrap_width,
                        columns: t.columns.as_ref(),
                        background: t.background.as_ref(),
                        frame_stroke_width: frame_stroke(self.drawing, owner, weight)
                            .map_err(|e| error(owner, &e.to_string()))?,
                        placement: t.placement.coordinate_frame()?,
                        rotation: t.rotation,
                        backward: t.backward,
                        upside_down: t.upside_down,
                    })
                    .map_err(|e| error(owner, &e.to_string()))?;
                    Self::estimate(estimate)
                }
                IfccadEntityKind::Viewport(v) => {
                    let b = native(
                        paper_frame_bounds(PaperFrame {
                            center: v.frame.center,
                            width: v.frame.width,
                            height: v.frame.height,
                        })
                        .map_err(|e| error(owner, &e.to_string()))?,
                    );
                    Extents {
                        all: Some(b),
                        proven: Some(b),
                        ..Default::default()
                    }
                }
                kind => {
                    let b = native(
                        geometry_bounds(kind.as_shared_geometry()?.expect("primitive branch"))
                            .map_err(|e| error(owner, &e.to_string()))?,
                    );
                    Extents {
                        all: Some(b),
                        proven: Some(b),
                        ..Default::default()
                    }
                }
            };
            combined.merge(value);
        }
        self.completed.insert(owner, combined.clone());
        Ok(combined)
    }
    fn estimate(e: TextExtentEstimate) -> Extents {
        Extents {
            all: e.bounds.map(native),
            proven: None,
            unavailable: e.status == TextExtentStatus::Unavailable,
            estimated: e.status != TextExtentStatus::Empty,
            reasons: e.reasons,
        }
    }
}
pub(super) fn validate_supplied(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let scopes = std::iter::once((
        Owner::Layout(d.model.id),
        d.model.bounds,
        d.model.bounds_quality,
    ))
    .chain(
        d.paper_layouts
            .iter()
            .map(|p| (Owner::Layout(p.id), p.bounds, p.bounds_quality)),
    )
    .chain(
        d.blocks
            .iter()
            .map(|b| (Owner::BlockDefinition(b.id), b.bounds, b.bounds_quality)),
    );
    let mut evaluation = Evaluation::new(d);
    for (owner, supplied, quality) in scopes {
        if quality.is_some() && supplied.is_none() {
            return Err(error(owner, "boundsQuality requires bounds"));
        }
        if quality == Some(IfccadBoundsQuality::Partial) {
            return Err(error(
                owner,
                "partial is derived, never a stored box quality",
            ));
        }
        // Missing extents do not disable arithmetic checks for known content.
        let actual = evaluation.scope(owner)?;
        let Some(b) = supplied else {
            continue;
        };
        if b.min.into_iter().chain(b.max).any(|v| !v.is_finite())
            || (0..3).any(|i| b.min[i] > b.max[i])
        {
            return Err(error(owner, "bounds need finite ordered XYZ values"));
        }
        if actual.unavailable {
            return Err(error(
                owner,
                "opaque or unavailable geometry requires absent complete bounds",
            ));
        }
        if actual.all.is_none() {
            return Err(error(owner, "empty owner requires absent bounds"));
        }
        if actual
            .proven
            .is_some_and(|p| (0..3).any(|i| b.min[i] > p.min[i] || b.max[i] < p.max[i]))
        {
            return Err(error(
                owner,
                "supplied bounds do not enclose the conservative geometry envelope",
            ));
        }
    }
    Ok(())
}
/// Replace boxes and qualities only after all scopes evaluate successfully.
pub fn recompute_ifccad_document_bounds(d: &mut IfccadDocument) -> Result<(), IfccadReport> {
    let a = assess_ifccad_document_bounds(d)?;
    let set =
        |id, bounds: &mut Option<IfccadBounds3d>, quality: &mut Option<IfccadBoundsQuality>| {
            let s = &a.scopes[&id];
            *bounds = s.bounds;
            *quality = if s.bounds.is_some() { s.quality } else { None };
        };
    set(
        Owner::Layout(d.model.id),
        &mut d.model.bounds,
        &mut d.model.bounds_quality,
    );
    for p in &mut d.paper_layouts {
        set(Owner::Layout(p.id), &mut p.bounds, &mut p.bounds_quality);
    }
    for b in &mut d.blocks {
        set(
            Owner::BlockDefinition(b.id),
            &mut b.bounds,
            &mut b.bounds_quality,
        );
    }
    Ok(())
}
/// Evidence is independent of supplied producer declarations.
pub fn assess_ifccad_document_bounds(
    d: &IfccadDocument,
) -> Result<IfccadBoundsAssessment, IfccadReport> {
    super::document_validation::validate_document(
        d,
        super::document_validation::ValidationPhase::BeforeBounds,
    )?;
    let mut evaluation = Evaluation::new(d);
    let owners: Vec<_> = evaluation.contents.keys().copied().collect();
    let scopes = owners
        .into_iter()
        .map(|owner| {
            let e = evaluation.scope(owner)?;
            Ok((
                owner,
                IfccadScopeBoundsAssessment {
                    scope_id: owner,
                    bounds: e.bounds(),
                    quality: e.quality(),
                    enclosure_verified: !e.unavailable && !e.estimated,
                    text_reasons: e.reasons,
                },
            ))
        })
        .collect::<Result<_, IfccadReport>>()
        .map_err(|report| {
            crate::ifccad::diagnostics::context(
                report,
                "IFCCAD-BOUNDS-002",
                &format!("/cad/d{}", d.drawing_id),
            )
        })?;
    Ok(IfccadBoundsAssessment { scopes })
}
/// Complete availability can include estimates; it is not a glyph certificate.
pub fn derive_ifccad_geometry_completeness(
    d: &IfccadDocument,
) -> Result<BTreeMap<IfccadScopeId, IfccadGeometryCompleteness>, IfccadReport> {
    super::document_validation::validate_document(
        d,
        super::document_validation::ValidationPhase::BeforeBounds,
    )?;
    let mut evaluation = Evaluation::new(d);
    let owners: Vec<_> = evaluation.contents.keys().copied().collect();
    owners
        .into_iter()
        .map(|owner| evaluation.scope(owner).map(|e| (owner, e.completeness())))
        .collect::<Result<_, IfccadReport>>()
        .map_err(|report| {
            crate::ifccad::diagnostics::context(
                report,
                "IFCCAD-BOUNDS-002",
                &format!("/cad/d{}", d.drawing_id),
            )
        })
}
