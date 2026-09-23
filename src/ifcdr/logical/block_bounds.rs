use super::validation::diagnostic;
use super::*;
use crate::diagnostic::PackageDiagnosticCategory;
use crate::ifcdr::geometry::{numeric::Interval, PreparedBlockTransform};
use crate::ifcdr::{BlockTransform, Bounds3d, PlanePlacement, Point2, Point3};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
enum Leaf {
    Point(Point3),
    Placed(PlanePlacement, Point2),
}
impl Leaf {
    fn enclosure(self) -> Option<[Interval; 3]> {
        match self {
            Self::Point(p) => Some(p.components().map(Interval::point)),
            Self::Placed(plane, p) => plane.enclose_point(p).ok().map(box_intervals),
        }
    }
    fn exactly_inside(self, bounds: Bounds3d) -> bool {
        match self {
            Self::Point(p) => contains(bounds, p.components().map(Interval::point)),
            Self::Placed(plane, p) => plane.enclosed_by(p, bounds),
        }
    }
}
fn box_intervals(b: Bounds3d) -> [Interval; 3] {
    std::array::from_fn(|i| Interval {
        lower: b.min.components()[i],
        upper: b.max.components()[i],
    })
}
fn contains(b: Bounds3d, p: [Interval; 3]) -> bool {
    p.into_iter()
        .enumerate()
        .all(|(i, p)| b.min.components()[i] <= p.lower && p.upper <= b.max.components()[i])
}
fn outside(b: Bounds3d, p: [Interval; 3]) -> bool {
    p.into_iter()
        .enumerate()
        .any(|(i, p)| p.upper < b.min.components()[i] || b.max.components()[i] < p.lower)
}
fn extend(bounds: &mut Option<Bounds3d>, p: [Interval; 3]) {
    let lo = p.map(|p| p.lower);
    let hi = p.map(|p| p.upper);
    let next = Bounds3d {
        min: Point3::new(lo[0], lo[1], lo[2]),
        max: Point3::new(hi[0], hi[1], hi[2]),
    };
    *bounds = Some(match *bounds {
        None => next,
        Some(b) => {
            let lo: [f64; 3] = std::array::from_fn(|i| b.min.components()[i].min(lo[i]));
            let hi: [f64; 3] = std::array::from_fn(|i| b.max.components()[i].max(hi[i]));
            Bounds3d {
                min: Point3::new(lo[0], lo[1], lo[2]),
                max: Point3::new(hi[0], hi[1], hi[2]),
            }
        }
    });
}

/// Points and prepared transforms are shared across roots; path state is an
/// index arena (not recursive boxes) and exact identical occurrences are reused.
struct Geometry<'a, R> {
    resource: &'a R,
    graph: &'a BlockGraph,
    leaves: BTreeMap<u32, Vec<Leaf>>,
    prepared: Vec<Option<PreparedBlockTransform>>,
    transform_ids: Vec<usize>,
    identities: Vec<bool>,
}
impl<'a, R: IfcdrResourceAccess> Geometry<'a, R> {
    fn new(r: &'a R, graph: &'a BlockGraph) -> Self {
        let lines = r.lines();
        let polylines = r.polylines();
        let mut leaves = BTreeMap::new();
        for (id, scope) in &graph.scopes {
            let mut points = Vec::new();
            for row in &scope.lines {
                let line = lines.get(*row).unwrap();
                points.extend([Leaf::Point(line.start), Leaf::Point(line.end)]);
            }
            for row in &scope.polylines {
                let p = polylines.get(*row).unwrap();
                let plane = PlanePlacement::from_validated_components(p.placement());
                points.extend(
                    (0..p.vertex_count()).map(|i| Leaf::Placed(plane, p.vertex(i).unwrap())),
                );
            }
            for row in &scope.viewports {
                if let Some(frame) = super::viewport::frame_bounds(r.viewports()[*row].frame) {
                    points.extend([Leaf::Point(frame.min), Leaf::Point(frame.max)]);
                }
            }
            leaves.insert(*id, points);
        }
        let instances = r.block_instances();
        let mut signatures = BTreeMap::new();
        let mut transform_ids = Vec::new();
        let mut identities = Vec::new();
        let mut prepared = Vec::new();
        for row in 0..instances.len() {
            let instance = instances.get(row).unwrap();
            let base =
                r.block_definitions()[graph.definitions[&instance.definition_scope_id]].base_point;
            let t = instance.transform;
            let signature: Vec<_> = [
                t.placement.origin.components(),
                t.placement.x.components(),
                t.placement.y.components(),
                t.scale.components(),
                base.components(),
            ]
            .into_iter()
            .flatten()
            .chain([t.rotation])
            .map(f64::to_bits)
            .collect();
            let next = signatures.len();
            transform_ids.push(*signatures.entry(signature).or_insert(next));
            identities.push(
                t == BlockTransform::default().components() && base == Point3::new(0., 0., 0.),
            );
            prepared.push(if graph.nonempty.contains(&instance.definition_scope_id) {
                graph.transform(r, row)
            } else {
                None
            });
        }
        Self {
            resource: r,
            graph,
            leaves,
            prepared,
            transform_ids,
            identities,
        }
    }
    fn evaluate(&self, root: u32, verify: bool) -> Evaluation {
        let supplied = self
            .resource
            .scopes()
            .iter()
            .find(|s| s.id == root)
            .unwrap()
            .bounds;
        let valid_box = supplied.filter(|b| super::validation::valid_bounds(*b));
        let mut result = Evaluation {
            bounds: None,
            outside: verify
                && (supplied.is_some() && valid_box.is_none()
                    || self.graph.nonempty.contains(&root) == supplied.is_none()),
            unresolved: false,
        };
        if !self.graph.nonempty.contains(&root) {
            return result;
        }
        let instances = self.resource.block_instances();
        let mut paths: Vec<(usize, Option<usize>)> = Vec::new();
        let mut path_ids = BTreeMap::new();
        let mut pending: Vec<(u32, Option<usize>)> = vec![(root, None)];
        let mut visited = BTreeSet::new();
        while let Some((scope, path)) = pending.pop() {
            if !visited.insert((scope, path)) {
                continue;
            }
            for leaf in &self.leaves[&scope] {
                let mut evaluated = leaf.enclosure();
                let mut cursor = path;
                while let Some(index) = cursor {
                    let (row, parent) = paths[index];
                    evaluated =
                        evaluated.and_then(|p| self.prepared[row].as_ref()?.apply_intervals(p));
                    cursor = parent;
                }
                let Some(point) = evaluated else {
                    result.unresolved = true;
                    continue;
                };
                extend(&mut result.bounds, point);
                if verify {
                    if let Some(b) = valid_box {
                        if !contains(b, point) {
                            if path.is_none() {
                                result.outside |= !leaf.exactly_inside(b);
                            } else if outside(b, point) {
                                result.outside = true;
                            } else {
                                result.unresolved = true;
                            }
                        }
                    }
                }
            }
            for row in &self.graph.scopes[&scope].instances {
                let instance = instances.get(*row).unwrap();
                if !self.graph.nonempty.contains(&instance.definition_scope_id) {
                    continue;
                }
                let next_path = if self.identities[*row] {
                    path
                } else {
                    let next = paths.len();
                    let index = *path_ids
                        .entry((self.transform_ids[*row], path))
                        .or_insert_with(|| {
                            paths.push((*row, path));
                            next
                        });
                    Some(index)
                };
                pending.push((instance.definition_scope_id, next_path));
            }
        }
        result
    }
}
struct Evaluation {
    bounds: Option<Bounds3d>,
    outside: bool,
    unresolved: bool,
}

pub(super) fn collect<R: IfcdrResourceAccess>(
    r: &R,
    graph: &BlockGraph,
    verify: bool,
) -> Result<BTreeMap<u32, Option<Bounds3d>>, Vec<IfcdrDiagnostic>> {
    let geometry = Geometry::new(r, graph);
    let mut bounds = BTreeMap::new();
    let mut errors = Vec::new();
    let rows: BTreeMap<_, _> = r
        .scopes()
        .iter()
        .enumerate()
        .map(|(row, s)| (s.id, row))
        .collect();
    for id in &graph.order {
        let result = geometry.evaluate(*id, verify);
        bounds.insert(*id, result.bounds);
        if result.outside {
            errors.push(diagnostic(r,IFCCAD_IFCDR_BOUNDS_INVALID,"scope",Some(rows[id]),"bounds","bounds exclude proved geometry, are malformed, or disagree with empty scope geometry"));
        }
        if result.unresolved {
            let mut d = diagnostic(
                r,
                IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE,
                "scope",
                Some(rows[id]),
                "bounds",
                "fixed numerical evaluation cannot certify these scope bounds",
            );
            d.category = PackageDiagnosticCategory::ExecutionBlocked;
            errors.push(d);
        }
    }
    if errors.is_empty() {
        Ok(bounds)
    } else {
        Err(errors)
    }
}
