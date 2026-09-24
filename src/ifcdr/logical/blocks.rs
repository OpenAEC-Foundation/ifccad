use super::validation::diagnostic;
use super::*;
use crate::ifcdr::{BlockScaling, BlockTransform};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Default)]
pub(crate) struct ScopeGeometry {
    pub points: Vec<usize>,
    pub circles: Vec<usize>,
    pub arcs: Vec<usize>,
    pub ellipses: Vec<usize>,
    pub ellipse_arcs: Vec<usize>,
    pub lines: Vec<usize>,
    pub polylines: Vec<usize>,
    pub spatial_polylines: Vec<usize>,
    pub instances: Vec<usize>,
    pub viewports: Vec<usize>,
}

/// All resource scopes, including unused definitions. Topological order places
/// every dependency before its owners; traversal never uses the Rust call stack.
pub(crate) struct BlockGraph {
    pub scopes: BTreeMap<u32, ScopeGeometry>,
    pub definitions: BTreeMap<u32, usize>,
    pub order: Vec<u32>,
    pub nonempty: BTreeSet<u32>,
}

impl BlockGraph {
    pub(crate) fn build<R: IfcdrResourceAccess>(r: &R) -> Result<Self, Vec<IfcdrDiagnostic>> {
        let mut errors = Vec::new();
        let mut scopes = BTreeMap::new();
        let mut kinds = BTreeMap::new();
        for (row, s) in r.scopes().iter().enumerate() {
            if scopes.insert(s.id, ScopeGeometry::default()).is_some() {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_SCOPE_INVALID,
                    "scope",
                    Some(row),
                    "id",
                    "duplicate scope ID",
                ));
            }
            kinds.insert(s.id, s.kind);
        }
        if r.scopes()
            .iter()
            .filter(|s| s.kind == IfcdrScopeKind::ModelSpace)
            .count()
            != 1
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_SCOPE_INVALID,
                "scope",
                None,
                "kind",
                "exactly one ModelSpace is required",
            ));
        }
        let mut definitions = BTreeMap::new();
        let mut names = BTreeSet::new();
        for (row, d) in r.block_definitions().iter().enumerate() {
            if definitions.insert(d.scope_id, row).is_some()
                || kinds.get(&d.scope_id) != Some(&IfcdrScopeKind::BlockDefinition)
            {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockDefinition",
                    Some(row),
                    "scopeId",
                    "definition requires one distinct BlockDefinition scope",
                ));
            }
            if d.name.is_empty() || !names.insert(crate::ifcdr::names::name_key(&d.name)) {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockDefinition",
                    Some(row),
                    "name",
                    "definition names must be nonempty and unique under full default case folding",
                ));
            }
            if !valid_point3(d.base_point) {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockDefinition",
                    Some(row),
                    "basePoint",
                    "base point must be finite",
                ));
            }
        }
        for (row, s) in r.scopes().iter().enumerate() {
            if s.kind == IfcdrScopeKind::BlockDefinition && !definitions.contains_key(&s.id) {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "scope",
                    Some(row),
                    "id",
                    "block scope has no definition row",
                ));
            }
        }
        for (row, point) in r.points().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&point.entity.scope_id) {
                scope.points.push(row);
            }
        }
        for (row, circle) in r.circles().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&circle.entity.scope_id) {
                scope.circles.push(row);
            }
        }
        for (row, arc) in r.arcs().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&arc.entity.scope_id) {
                scope.arcs.push(row);
            }
        }
        for (row, ellipse) in r.ellipses().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&ellipse.entity.scope_id) {
                scope.ellipses.push(row);
            }
        }
        for (row, arc) in r.ellipse_arcs().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&arc.entity.scope_id) {
                scope.ellipse_arcs.push(row);
            }
        }
        let lines = r.lines();
        for row in 0..lines.len() {
            if let Some(line) = lines.get(row) {
                if let Some(scope) = scopes.get_mut(&line.entity.scope_id) {
                    scope.lines.push(row);
                }
            }
        }
        let polylines = r.polylines();
        for row in 0..polylines.len() {
            if let Some(polyline) = polylines.get(row) {
                if let Some(scope) = scopes.get_mut(&polyline.entity().scope_id) {
                    scope.polylines.push(row);
                }
            }
        }
        for (row, polyline) in r.spatial_polylines().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&polyline.entity.scope_id) {
                scope.spatial_polylines.push(row);
            }
        }
        for (row, viewport) in r.viewports().iter().enumerate() {
            if let Some(scope) = scopes.get_mut(&viewport.entity.scope_id) {
                scope.viewports.push(row);
            }
        }
        let mut remaining: BTreeMap<u32, usize> = scopes.keys().map(|id| (*id, 0)).collect();
        let mut owners: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let instances = r.block_instances();
        for row in 0..instances.len() {
            let Some(instance) = instances.get(row) else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockInstance",
                    Some(row),
                    "row",
                    "instance row is inaccessible",
                ));
                continue;
            };
            if let Err(error) = instance.transform.validate() {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockInstance",
                    Some(row),
                    "transform",
                    error.to_string(),
                ));
            }
            let Some(&definition) = definitions.get(&instance.definition_scope_id) else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockInstance",
                    Some(row),
                    "definitionScopeId",
                    "target is not a local block definition",
                ));
                continue;
            };
            if r.block_definitions()[definition].scaling == BlockScaling::Uniform
                && !instance.transform.scale.is_uniform()
            {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockInstance",
                    Some(row),
                    "transform",
                    "definition requires exact signed uniform scale",
                ));
            }
            let Some(scope) = scopes.get_mut(&instance.entity.scope_id) else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BLOCK_INVALID,
                    "blockInstance",
                    Some(row),
                    "scopeId",
                    "owning scope is missing",
                ));
                continue;
            };
            scope.instances.push(row);
            *remaining.get_mut(&instance.entity.scope_id).unwrap() += 1;
            owners
                .entry(instance.definition_scope_id)
                .or_default()
                .push(instance.entity.scope_id);
        }
        let mut ready: VecDeque<_> = remaining
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_front() {
            order.push(id);
            for owner in owners.get(&id).into_iter().flatten() {
                let n = remaining.get_mut(owner).unwrap();
                *n -= 1;
                if *n == 0 {
                    ready.push_back(*owner);
                }
            }
        }
        if order.len() != scopes.len() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_BLOCK_CYCLE,
                "blockInstance",
                None,
                "definitionScopeId",
                "block dependency graph contains a cycle",
            ));
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let mut nonempty = BTreeSet::new();
        for id in &order {
            let scope = &scopes[id];
            if !scope.lines.is_empty()
                || !scope.polylines.is_empty()
                || !scope.spatial_polylines.is_empty()
                || !scope.viewports.is_empty()
                || scope
                    .instances
                    .iter()
                    .any(|row| nonempty.contains(&instances.get(*row).unwrap().definition_scope_id))
            {
                nonempty.insert(*id);
            }
        }
        Ok(Self {
            scopes,
            definitions,
            order,
            nonempty,
        })
    }
    pub(crate) fn transform<R: IfcdrResourceAccess>(
        &self,
        r: &R,
        row: usize,
    ) -> Option<crate::ifcdr::geometry::PreparedBlockTransform> {
        let instance = r.block_instances().get(row)?;
        let base =
            r.block_definitions()[self.definitions[&instance.definition_scope_id]].base_point;
        crate::ifcdr::geometry::PreparedBlockTransform::new(
            BlockTransform::from_validated_components(instance.transform),
            base,
        )
    }
}
