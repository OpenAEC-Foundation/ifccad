//! Optional supplied bounds validation and explicit atomic preparation.
use super::*;
use crate::geometry_kernel::{
    geometry_bounds, numeric::Interval, paper_frame_bounds, BlockTransform, PaperFrame, Point3,
    PreparedBlockTransform, Scale3,
};
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
#[derive(Clone, Copy)]
enum Enclosure {
    Empty,
    Complete(IfccadBounds3d),
    Unavailable,
}
impl Enclosure {
    fn bounds(self) -> Option<IfccadBounds3d> {
        match self {
            Self::Complete(b) => Some(b),
            _ => None,
        }
    }
    fn completeness(self) -> IfccadGeometryCompleteness {
        match self {
            Self::Empty => IfccadGeometryCompleteness::Empty,
            Self::Complete(_) => IfccadGeometryCompleteness::Complete,
            Self::Unavailable => IfccadGeometryCompleteness::Unavailable,
        }
    }
}
fn error(owner: Owner, message: &str) -> IfccadReport {
    IfccadReport::one(format!("{owner:?}.bounds: {message}"))
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
struct Evaluation<'a> {
    contents: BTreeMap<Owner, &'a [IfccadEntity]>,
    blocks: BTreeMap<u64, &'a IfccadBlockDefinition>,
    completed: BTreeMap<Owner, Enclosure>,
}
impl<'a> Evaluation<'a> {
    fn new(d: &'a IfccadDocument) -> Self {
        let contents = std::iter::once((Owner::Layout(d.model.id), d.model.entities.as_slice()))
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
            .collect();
        Self {
            contents,
            blocks: d.blocks.iter().map(|b| (b.id, b)).collect(),
            completed: BTreeMap::new(),
        }
    }
    fn scope(&mut self, owner: Owner) -> Result<Enclosure, IfccadReport> {
        if let Some(b) = self.completed.get(&owner) {
            return Ok(*b);
        }
        let mut combined = None;
        let mut unavailable = false;
        for entity in self.contents[&owner] {
            let Some(entity) = entity.as_native() else {
                unavailable = true;
                continue;
            };
            let value = match &entity.kind {
                IfccadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } => {
                    let block = self.blocks[definition_id];
                    let local = self.scope(Owner::BlockDefinition(*definition_id))?;
                    let base = block.base_point;
                    let transform_value = BlockTransform::try_new(
                        transform.placement.coordinate_frame()?,
                        transform.rotation,
                        Scale3::new(transform.scale[0], transform.scale[1], transform.scale[2]),
                    )
                    .map_err(|e| error(owner, &e.to_string()))?;
                    let prepared = PreparedBlockTransform::new(
                        transform_value,
                        Point3::new(base[0], base[1], base[2]),
                    )
                    .ok_or_else(|| error(owner, "cannot evaluate block transform"))?;
                    match local {
                        Enclosure::Unavailable => {
                            unavailable = true;
                            continue;
                        }
                        Enclosure::Empty => IfccadBounds3d {
                            min: transform.placement.origin,
                            max: transform.placement.origin,
                        },
                        Enclosure::Complete(local) => {
                            let result = prepared
                                .apply_intervals(std::array::from_fn(|i| Interval {
                                    lower: local.min[i],
                                    upper: local.max[i],
                                }))
                                .ok_or_else(|| {
                                    error(owner, "transformed block enclosure exceeds finite range")
                                })?;
                            IfccadBounds3d {
                                min: result.map(|v| v.lower),
                                max: result.map(|v| v.upper),
                            }
                        }
                    }
                }
                IfccadEntityKind::Viewport(v) => {
                    let b = paper_frame_bounds(PaperFrame {
                        center: v.frame.center,
                        width: v.frame.width,
                        height: v.frame.height,
                    })
                    .map_err(|e| error(owner, &e.to_string()))?;
                    IfccadBounds3d {
                        min: b.min().components(),
                        max: b.max().components(),
                    }
                }
                kind => {
                    let primitive = kind.as_shared_geometry()?.expect("primitive branch");
                    let b = geometry_bounds(primitive).map_err(|e| error(owner, &e.to_string()))?;
                    IfccadBounds3d {
                        min: b.min().components(),
                        max: b.max().components(),
                    }
                }
            };
            union(&mut combined, value);
        }
        let result = if unavailable {
            Enclosure::Unavailable
        } else {
            combined
                .map(Enclosure::Complete)
                .unwrap_or(Enclosure::Empty)
        };
        self.completed.insert(owner, result);
        Ok(result)
    }
}
pub(super) fn validate_supplied(d: &IfccadDocument) -> Result<(), IfccadReport> {
    let scopes = std::iter::once((
        Owner::Layout(d.model.id),
        d.model.bounds,
        d.model.entities.is_empty(),
    ))
    .chain(
        d.paper_layouts
            .iter()
            .map(|p| (Owner::Layout(p.id), p.bounds, p.entities.is_empty())),
    )
    .chain(d.blocks.iter().map(|b| {
        (
            Owner::BlockDefinition(b.id),
            b.bounds,
            b.entities.is_empty(),
        )
    }));
    let mut evaluation = Evaluation::new(d);
    for (owner, supplied, empty) in scopes {
        let Some(b) = supplied else { continue };
        if empty {
            return Err(error(owner, "empty owner requires absent bounds"));
        }
        if b.min.into_iter().chain(b.max).any(|v| !v.is_finite())
            || (0..3).any(|i| b.min[i] > b.max[i])
        {
            return Err(error(owner, "bounds need finite ordered XYZ values"));
        }
        let actual = match evaluation.scope(owner)? {
            Enclosure::Complete(b) => b,
            Enclosure::Unavailable => {
                return Err(error(
                    owner,
                    "opaque geometry requires absent complete bounds",
                ))
            }
            Enclosure::Empty => return Err(error(owner, "empty owner requires absent bounds")),
        };
        if (0..3).any(|i| b.min[i] > actual.min[i] || b.max[i] < actual.max[i]) {
            return Err(error(
                owner,
                "supplied bounds do not enclose the conservative geometry envelope",
            ));
        }
    }
    Ok(())
}
/// Replaces all scope bounds explicitly; failure leaves every document field intact.
pub fn recompute_ifccad_document_bounds(d: &mut IfccadDocument) -> Result<(), IfccadReport> {
    super::document_validation::validate_document(
        d,
        super::document_validation::ValidationPhase::BeforeBounds,
    )?;
    let mut evaluation = Evaluation::new(d);
    let model = evaluation.scope(Owner::Layout(d.model.id))?;
    let papers = d
        .paper_layouts
        .iter()
        .map(|p| evaluation.scope(Owner::Layout(p.id)))
        .collect::<Result<Vec<_>, _>>()?;
    let blocks = d
        .blocks
        .iter()
        .map(|b| evaluation.scope(Owner::BlockDefinition(b.id)))
        .collect::<Result<Vec<_>, _>>()?;
    d.model.bounds = model.bounds();
    for (p, b) in d.paper_layouts.iter_mut().zip(papers) {
        p.bounds = b.bounds();
    }
    for (p, b) in d.blocks.iter_mut().zip(blocks) {
        p.bounds = b.bounds();
    }
    Ok(())
}

/// Derive geometric completeness for all scopes, including unused definitions.
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
        .map(|owner| {
            evaluation
                .scope(owner)
                .map(|state| (owner, state.completeness()))
        })
        .collect()
}
