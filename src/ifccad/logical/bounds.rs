//! Optional supplied bounds validation and explicit atomic preparation.
use super::*;
use crate::geometry_kernel::{
    geometry_bounds, numeric::Interval, paper_frame_bounds, BlockTransform, PaperFrame, Point3,
    PreparedBlockTransform, Scale3,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    Layout(u64),
    Block(u64),
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
    completed: BTreeMap<Owner, Option<IfccadBounds3d>>,
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
                    .map(|b| (Owner::Block(b.id), b.entities.as_slice())),
            )
            .collect();
        Self {
            contents,
            blocks: d.blocks.iter().map(|b| (b.id, b)).collect(),
            completed: BTreeMap::new(),
        }
    }
    fn scope(&mut self, owner: Owner) -> Result<Option<IfccadBounds3d>, IfccadReport> {
        if let Some(b) = self.completed.get(&owner) {
            return Ok(*b);
        }
        let mut combined = None;
        for entity in self.contents[&owner] {
            let value = match &entity.kind {
                IfccadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } => {
                    let block = self.blocks[definition_id];
                    if let Some(local) = self.scope(Owner::Block(*definition_id))? {
                        let base = block.base_point;
                        let transform = BlockTransform::try_new(
                            transform.placement.coordinate_frame()?,
                            transform.rotation,
                            Scale3::new(transform.scale[0], transform.scale[1], transform.scale[2]),
                        )
                        .map_err(|e| error(owner, &e.to_string()))?;
                        let prepared = PreparedBlockTransform::new(
                            transform,
                            Point3::new(base[0], base[1], base[2]),
                        )
                        .ok_or_else(|| error(owner, "cannot evaluate block transform"))?;
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
                    } else {
                        IfccadBounds3d {
                            min: transform.placement.origin,
                            max: transform.placement.origin,
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
        self.completed.insert(owner, combined);
        Ok(combined)
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
    .chain(
        d.blocks
            .iter()
            .map(|b| (Owner::Block(b.id), b.bounds, b.entities.is_empty())),
    );
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
        let actual = evaluation.scope(owner)?.expect("nonempty owner enclosure");
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
        .map(|b| evaluation.scope(Owner::Block(b.id)))
        .collect::<Result<Vec<_>, _>>()?;
    d.model.bounds = model;
    for (p, b) in d.paper_layouts.iter_mut().zip(papers) {
        p.bounds = b;
    }
    for (p, b) in d.blocks.iter_mut().zip(blocks) {
        p.bounds = b;
    }
    Ok(())
}
