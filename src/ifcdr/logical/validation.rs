use super::viewport::{check_viewport, check_viewport_boundaries, frame_bounds};
use super::*;
use crate::ifcdr::{Bounds3d, PlanePlacement, Point2, Point3};
use crate::validated::{EvidenceOutcome, Validated, ValidationOutcome, ValidationTarget};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(crate) struct IfcdrCandidate<R>(R);
impl<R> IfcdrCandidate<R> {
    pub(crate) fn resource(&self) -> &R {
        &self.0
    }
}
/// Encoding-neutral resource proof over reader or writer storage `R`.
/// Shared semantic validation supplies indexes and immutable access. This
/// does not prove external IFCX links or the containing package valid.
pub(crate) type ValidatedIfcdr<R> = Validated<IfcdrCandidate<R>>;
#[derive(Clone, Copy, Debug)]
pub(crate) enum IfcdrEntityKind {
    Point,
    Circle,
    Arc,
    Ellipse,
    EllipseArc,
    Line,
    PlanarPolyline,
    SpatialPolyline,
    BlockInstance,
    Viewport,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct IfcdrEntityLocation {
    pub kind: IfcdrEntityKind,
    pub row: usize,
    pub scope: u32,
}
#[derive(Debug, Default)]
pub(crate) struct IfcdrEvidence {
    pub entities: BTreeMap<u64, IfcdrEntityLocation>,
    pub scopes: BTreeMap<u32, usize>,
    pub layers: BTreeMap<u32, usize>,
    pub appearances: BTreeMap<u32, usize>,
    pub overrides: BTreeMap<u32, usize>,
}
impl<R: IfcdrResourceAccess> ValidationTarget for IfcdrCandidate<R> {
    type Context = ();
    type Evidence = IfcdrEvidence;
    type Diagnostic = IfcdrDiagnostic;
    fn build_evidence(&self, _: &()) -> EvidenceOutcome<IfcdrEvidence, IfcdrDiagnostic> {
        let (evidence, errors) = check(&self.0);
        if errors.is_empty() {
            EvidenceOutcome::success(evidence, errors)
        } else {
            EvidenceOutcome::failure(errors)
        }
    }
}
pub(crate) fn validate_resource<R: IfcdrResourceAccess>(
    resource: R,
) -> ValidationOutcome<IfcdrCandidate<R>> {
    Validated::validate(IfcdrCandidate(resource), &())
}
pub(super) fn diagnostic<R: IfcdrResourceAccess>(
    r: &R,
    code: &'static str,
    collection: &'static str,
    row: Option<usize>,
    property: &'static str,
    message: impl Into<String>,
) -> IfcdrDiagnostic {
    IfcdrDiagnostic {
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code,
        resource_id: r.resource_id().clone(),
        collection,
        row,
        property,
        message: message.into(),
    }
}
fn table<R: IfcdrResourceAccess>(
    r: &R,
    name: &'static str,
    ids: impl Iterator<Item = u32>,
    errors: &mut Vec<IfcdrDiagnostic>,
) -> BTreeMap<u32, usize> {
    let mut index = BTreeMap::new();
    for (row, id) in ids.enumerate() {
        if index.insert(id, row).is_some() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                name,
                Some(row),
                "id",
                format!("duplicate table ID {id}"),
            ));
        }
    }
    index
}
fn check<R: IfcdrResourceAccess>(r: &R) -> (IfcdrEvidence, Vec<IfcdrDiagnostic>) {
    let mut errors = Vec::new();
    let mut evidence = IfcdrEvidence {
        scopes: table(r, "scope", r.scopes().iter().map(|s| s.id), &mut errors),
        layers: table(
            r,
            "layerBinding",
            r.layers().iter().map(|s| s.id),
            &mut errors,
        ),
        appearances: table(
            r,
            "appearanceBinding",
            r.appearances().iter().map(|s| s.id),
            &mut errors,
        ),
        overrides: table(
            r,
            "appearanceOverride",
            r.overrides().iter().map(|s| s.id),
            &mut errors,
        ),
        ..Default::default()
    };
    for (row, layer) in r.layers().iter().enumerate() {
        if layer.ifcx_layer.is_empty() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                "layerBinding",
                Some(row),
                "ifcxLayer",
                "IFCX identity must not be empty",
            ));
        }
    }
    for (row, binding) in r.appearances().iter().enumerate() {
        for (property, mode) in [
            "colorMode",
            "opacityMode",
            "linePatternMode",
            "lineWeightMode",
        ]
        .into_iter()
        .zip(binding.modes)
        {
            if appearance_mode(mode).is_none() {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_APPEARANCE_INVALID,
                    "appearanceBinding",
                    Some(row),
                    property,
                    "unsupported appearance mode",
                ));
            }
        }
        if binding
            .override_id
            .is_some_and(|id| !evidence.overrides.contains_key(&id))
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                "appearanceBinding",
                Some(row),
                "overrideId",
                "appearance override is missing",
            ));
        }
        if binding
            .ifcx_appearance
            .as_ref()
            .is_some_and(|s| s.is_empty())
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                "appearanceBinding",
                Some(row),
                "ifcxAppearance",
                "IFCX identity must not be empty",
            ));
        }
    }
    for (row, value) in r.overrides().iter().enumerate() {
        for (property, invalid) in [
            (
                "color",
                value.color.as_ref().is_some_and(|c| !valid_color(c)),
            ),
            ("opacity", value.opacity.is_some_and(|v| !valid_opacity(v))),
            (
                "lineWeight",
                value.line_weight.is_some_and(|v| !valid_line_weight(v)),
            ),
            (
                "ifcxLinePattern",
                value
                    .ifcx_line_pattern
                    .as_ref()
                    .is_some_and(|v| v.is_empty()),
            ),
        ] {
            if invalid {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_APPEARANCE_INVALID,
                    "appearanceOverride",
                    Some(row),
                    property,
                    "invalid stored appearance value",
                ));
            }
        }
    }
    let mut identity_valid = true;
    for (row, point) in r.points().iter().enumerate() {
        identity_valid &= entity(
            r,
            point.entity,
            IfcdrEntityKind::Point,
            row,
            &mut evidence,
            &mut errors,
        );
    }
    for (row, circle) in r.circles().iter().enumerate() {
        identity_valid &= entity(
            r,
            circle.entity,
            IfcdrEntityKind::Circle,
            row,
            &mut evidence,
            &mut errors,
        );
    }
    for (row, arc) in r.arcs().iter().enumerate() {
        identity_valid &= entity(
            r,
            arc.entity,
            IfcdrEntityKind::Arc,
            row,
            &mut evidence,
            &mut errors,
        );
    }
    for (row, ellipse) in r.ellipses().iter().enumerate() {
        identity_valid &= entity(
            r,
            ellipse.entity,
            IfcdrEntityKind::Ellipse,
            row,
            &mut evidence,
            &mut errors,
        );
    }
    for (row, arc) in r.ellipse_arcs().iter().enumerate() {
        identity_valid &= entity(
            r,
            arc.entity,
            IfcdrEntityKind::EllipseArc,
            row,
            &mut evidence,
            &mut errors,
        );
    }
    let lines = r.lines();
    for row in 0..lines.len() {
        if let Some(line) = lines.get(row) {
            identity_valid &= entity(
                r,
                line.entity,
                IfcdrEntityKind::Line,
                row,
                &mut evidence,
                &mut errors,
            );
        } else {
            identity_valid = false;
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                "line",
                Some(row),
                "row",
                "line row is inaccessible",
            ));
        }
    }
    let polylines = r.polylines();
    for row in 0..polylines.len() {
        if let Some(polyline) = polylines.get(row) {
            identity_valid &= entity(
                r,
                polyline.entity(),
                IfcdrEntityKind::PlanarPolyline,
                row,
                &mut evidence,
                &mut errors,
            );
            if !valid_polyline_vertex_count(polyline.vertex_count()) {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_POLYLINE_INVALID,
                    "planarPolyline",
                    Some(row),
                    "vertices",
                    "polyline requires at least two vertices",
                ));
            }
            for index in 0..polyline.vertex_count() {
                let bulge = polyline.bulge(index);
                let invalid = bulge.is_none_or(|value| !value.is_finite())
                    || (bulge.is_some_and(|value| value != 0.0)
                        && (index + 1 < polyline.vertex_count() || polyline.closed())
                        && polyline.vertex(index)
                            == polyline.vertex((index + 1) % polyline.vertex_count()));
                if invalid {
                    errors.push(diagnostic(
                        r,
                        IFCCAD_IFCDR_POLYLINE_INVALID,
                        "planarPolyline",
                        Some(row),
                        "vertices",
                        "bulge must be finite and a curved segment needs distinct endpoints",
                    ));
                }
            }
        } else {
            identity_valid = false;
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                "planarPolyline",
                Some(row),
                "row",
                "polyline row is inaccessible",
            ));
        }
    }
    for (row, polyline) in r.spatial_polylines().iter().enumerate() {
        identity_valid &= entity(
            r,
            polyline.entity,
            IfcdrEntityKind::SpatialPolyline,
            row,
            &mut evidence,
            &mut errors,
        );
        if !valid_polyline_vertex_count(polyline.points.len()) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_POLYLINE_INVALID,
                "spatialPolyline",
                Some(row),
                "vertices",
                "spatial polyline requires at least two vertices",
            ));
        }
    }
    let instances = r.block_instances();
    for row in 0..instances.len() {
        if let Some(instance) = instances.get(row) {
            identity_valid &= entity(
                r,
                instance.entity,
                IfcdrEntityKind::BlockInstance,
                row,
                &mut evidence,
                &mut errors,
            );
        } else {
            identity_valid = false;
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                "blockInstance",
                Some(row),
                "row",
                "instance is inaccessible",
            ));
        }
    }
    for (row, viewport) in r.viewports().iter().enumerate() {
        identity_valid &= entity(
            r,
            viewport.entity,
            IfcdrEntityKind::Viewport,
            row,
            &mut evidence,
            &mut errors,
        );
        check_viewport(r, viewport, row, &evidence, &mut errors);
    }
    if r.next_entity_id() == 0
        || evidence
            .entities
            .keys()
            .next_back()
            .is_some_and(|id| *id >= r.next_entity_id())
    {
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_ENTITY_ID_INVALID,
            "resource",
            None,
            "nextEntityId",
            "next entity ID must be positive and exceed all entity IDs",
        ));
    }
    if identity_valid {
        check_order(r, &evidence, &mut errors);
    }
    check_viewport_boundaries(r, &evidence, &mut errors);
    if let Err(mut geometry_errors) = collect_geometry(r, true) {
        errors.append(&mut geometry_errors);
    }
    (evidence, errors)
}
fn entity<R: IfcdrResourceAccess>(
    r: &R,
    value: IfcdrEntityRow,
    kind: IfcdrEntityKind,
    row: usize,
    evidence: &mut IfcdrEvidence,
    errors: &mut Vec<IfcdrDiagnostic>,
) -> bool {
    let name = match kind {
        IfcdrEntityKind::Point => "point",
        IfcdrEntityKind::Circle => "circle",
        IfcdrEntityKind::Arc => "arc",
        IfcdrEntityKind::Ellipse => "ellipse",
        IfcdrEntityKind::EllipseArc => "ellipseArc",
        IfcdrEntityKind::Line => "line",
        IfcdrEntityKind::PlanarPolyline => "planarPolyline",
        IfcdrEntityKind::SpatialPolyline => "spatialPolyline",
        IfcdrEntityKind::BlockInstance => "blockInstance",
        IfcdrEntityKind::Viewport => "viewport",
    };
    let mut valid = true;
    if value.entity_id == 0 {
        valid = false;
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_ENTITY_ID_INVALID,
            name,
            Some(row),
            "entityId",
            "entity ID must be nonzero",
        ));
    } else if evidence
        .entities
        .insert(
            value.entity_id,
            IfcdrEntityLocation {
                kind,
                row,
                scope: value.scope_id,
            },
        )
        .is_some()
    {
        valid = false;
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_ENTITY_ID_DUPLICATE,
            name,
            Some(row),
            "entityId",
            "duplicate entity ID",
        ));
    }
    for (property, present) in [
        ("scopeId", evidence.scopes.contains_key(&value.scope_id)),
        ("layerId", evidence.layers.contains_key(&value.layer_id)),
        (
            "appearanceId",
            evidence.appearances.contains_key(&value.appearance_id),
        ),
    ] {
        if !present {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                name,
                Some(row),
                property,
                "referenced table row is missing",
            ));
        }
    }
    valid
}
fn check_order<R: IfcdrResourceAccess>(
    r: &R,
    evidence: &IfcdrEvidence,
    errors: &mut Vec<IfcdrDiagnostic>,
) {
    let mut scopes = BTreeSet::new();
    let mut ordered = BTreeSet::new();
    let mut entry_row = 0usize;
    for (row, order) in r.orders().iter().enumerate() {
        let mut valid =
            evidence.scopes.contains_key(&order.scope_id) && scopes.insert(order.scope_id);
        for id in &order.entities {
            let entry_valid = evidence
                .entities
                .get(id)
                .is_some_and(|e| e.scope == order.scope_id)
                && ordered.insert(*id);
            if !entry_valid {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
                    "entityOrderEntry",
                    Some(entry_row),
                    "entityId",
                    "order references a missing, repeated or wrong-scope entity",
                ));
            }
            valid &= entry_valid;
            entry_row += 1;
        }
        if !valid {
            errors.push(diagnostic(r, IFCCAD_IFCDR_ENTITY_ORDER_INVALID, "entityOrder", Some(row), "entries", "order has an unknown/duplicate scope, missing entity, duplicate entity or wrong-scope entity"));
        }
    }
    if scopes.len() != evidence.scopes.len() || ordered.len() != evidence.entities.len() {
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_ENTITY_ORDER_INVALID,
            "entityOrder",
            None,
            "entries",
            "every scope and entity must occur exactly once in scope order",
        ));
    }
}
pub(crate) fn geometric_bounds<R: IfcdrResourceAccess>(
    r: &R,
) -> Result<BTreeMap<u32, Option<Bounds3d>>, Vec<IfcdrDiagnostic>> {
    collect_geometry(r, false)
}
pub(super) fn valid_bounds(b: Bounds3d) -> bool {
    valid_point3(b.min)
        && valid_point3(b.max)
        && b.min
            .components()
            .into_iter()
            .zip(b.max.components())
            .all(|(a, b)| a <= b)
}
fn contains(outer: Bounds3d, inner: Bounds3d) -> bool {
    outer
        .min
        .components()
        .into_iter()
        .zip(inner.min.components())
        .all(|(a, b)| a <= b)
        && outer
            .max
            .components()
            .into_iter()
            .zip(inner.max.components())
            .all(|(a, b)| a >= b)
}
fn collect_geometry<R: IfcdrResourceAccess>(
    r: &R,
    verify_bounds: bool,
) -> Result<BTreeMap<u32, Option<Bounds3d>>, Vec<IfcdrDiagnostic>> {
    let graph = BlockGraph::build(r);
    let direct = collect_direct_geometry(r, false);
    match (graph, direct) {
        (Err(mut graph), Err(mut direct)) => {
            graph.append(&mut direct);
            Err(graph)
        }
        (Err(errors), _) | (_, Err(errors)) => Err(errors),
        (Ok(graph), Ok(_)) => {
            if r.block_instances().is_empty() {
                collect_direct_geometry(r, verify_bounds)
            } else {
                super::block_bounds::collect(r, &graph, verify_bounds)
            }
        }
    }
}
fn collect_direct_geometry<R: IfcdrResourceAccess>(
    r: &R,
    verify_bounds: bool,
) -> Result<BTreeMap<u32, Option<Bounds3d>>, Vec<IfcdrDiagnostic>> {
    let mut bounds: BTreeMap<u32, Option<Bounds3d>> =
        r.scopes().iter().map(|s| (s.id, None)).collect();
    let scope_bounds: BTreeMap<_, _> = r.scopes().iter().map(|s| (s.id, s.bounds)).collect();
    let mut errors = Vec::new();
    let mut failed_scopes = BTreeSet::new();
    let mut add = |scope: u32, enclosure: Bounds3d, placed: Option<(PlanePlacement, Point2)>| {
        if verify_bounds {
            let enclosing = scope_bounds
                .get(&scope)
                .copied()
                .flatten()
                .filter(|b| valid_bounds(*b));
            let fits = enclosing.is_some_and(|b| {
                contains(b, enclosure) || placed.is_some_and(|(plane, p)| plane.enclosed_by(p, b))
            });
            if !fits {
                failed_scopes.insert(scope);
            }
        }
        let b = bounds.entry(scope).or_default();
        match b {
            None => *b = Some(enclosure),
            Some(b) => {
                let lo: [f64; 3] = std::array::from_fn(|i| {
                    b.min.components()[i].min(enclosure.min.components()[i])
                });
                let hi: [f64; 3] = std::array::from_fn(|i| {
                    b.max.components()[i].max(enclosure.max.components()[i])
                });
                b.min = Point3::new(lo[0], lo[1], lo[2]);
                b.max = Point3::new(hi[0], hi[1], hi[2]);
            }
        }
    };
    for (row, point) in r.points().iter().enumerate() {
        if let Err(error) = point.placement.validate() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "point",
                Some(row),
                "placement",
                error.to_string(),
            ));
            continue;
        }
        let position = point.placement.origin;
        add(
            point.entity.scope_id,
            Bounds3d {
                min: position,
                max: position,
            },
            None,
        );
    }
    for (row, circle) in r.circles().iter().enumerate() {
        match crate::ifcdr::geometry::circular_bounds(circle.placement, circle.radius, None) {
            Some(enclosure) => add(circle.entity.scope_id, enclosure, None),
            None => errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "circle",
                Some(row),
                "geometry",
                "circle requires a valid placement and finite positive radius",
            )),
        }
    }
    for (row, arc) in r.arcs().iter().enumerate() {
        match crate::ifcdr::geometry::circular_bounds(arc.placement, arc.radius, Some((arc.start_parameter, arc.sweep_parameter))) {
            Some(enclosure) => add(arc.entity.scope_id, enclosure, None),
            None => errors.push(diagnostic(r, IFCCAD_IFCDR_GEOMETRY_INVALID, "arc", Some(row), "geometry", "arc requires a valid placement, finite positive radius, finite start and nonzero sweep below a full turn")),
        }
    }
    for (row, ellipse) in r.ellipses().iter().enumerate() {
        match crate::ifcdr::geometry::elliptic_bounds(
            ellipse.placement,
            ellipse.semi_major_radius,
            ellipse.semi_minor_radius,
            None,
        ) {
            Some(enclosure) => add(ellipse.entity.scope_id, enclosure, None),
            None => errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "ellipse",
                Some(row),
                "geometry",
                "ellipse requires a valid placement and finite ordered positive semiaxes",
            )),
        }
    }
    for (row, arc) in r.ellipse_arcs().iter().enumerate() {
        match crate::ifcdr::geometry::elliptic_bounds(arc.placement, arc.semi_major_radius,
            arc.semi_minor_radius, Some((arc.start_parameter, arc.sweep_parameter))) {
            Some(enclosure) => add(arc.entity.scope_id, enclosure, None),
            None => errors.push(diagnostic(r, IFCCAD_IFCDR_GEOMETRY_INVALID, "ellipseArc", Some(row), "geometry",
                "elliptic arc requires valid ordered semiaxes, finite start and nonzero sweep below a full turn")),
        }
    }
    let lines = r.lines();
    for row in 0..lines.len() {
        if let Some(line) = lines.get(row) {
            for (property, p) in [("start", line.start), ("end", line.end)] {
                if valid_point3(p) {
                    add(line.entity.scope_id, Bounds3d { min: p, max: p }, None);
                } else {
                    errors.push(diagnostic(
                        r,
                        IFCCAD_IFCDR_GEOMETRY_INVALID,
                        "line",
                        Some(row),
                        property,
                        "coordinate is non-finite",
                    ));
                }
            }
        } else {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "line",
                Some(row),
                "row",
                "line is inaccessible",
            ));
        }
    }
    let polylines = r.polylines();
    for row in 0..polylines.len() {
        let Some(p) = polylines.get(row) else {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "planarPolyline",
                Some(row),
                "row",
                "polyline is inaccessible",
            ));
            continue;
        };
        if let Err(error) = p.placement().validate() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "planarPolyline",
                Some(row),
                "placement",
                error.to_string(),
            ));
            continue;
        }
        let plane = PlanePlacement::from_validated_components(p.placement());
        for i in 0..p.vertex_count() {
            match p
                .vertex(i)
                .and_then(|point| plane.enclose_point(point).ok().map(|b| (point, b)))
            {
                Some((point, b)) => add(p.entity().scope_id, b, Some((plane, point))),
                None => errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_GEOMETRY_INVALID,
                    "planarPolyline",
                    Some(row),
                    "vertices",
                    "placed coordinate is non-finite, out of range or inaccessible",
                )),
            }
        }
        for index in 0..p.vertex_count() {
            if index + 1 == p.vertex_count() && !p.closed() {
                continue;
            }
            let Some((start, end, bulge)) = p
                .vertex(index)
                .zip(p.vertex((index + 1) % p.vertex_count()))
                .and_then(|(a, b)| p.bulge(index).map(|bulge| (a, b, bulge)))
            else {
                continue;
            };
            if bulge == 0.0 {
                continue;
            }
            let Some(local) = crate::ifcdr::geometry::bulge_segment_bounds(start, end, bulge)
            else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_GEOMETRY_INVALID,
                    "planarPolyline",
                    Some(row),
                    "vertices",
                    "curved segment bounds cannot be evaluated",
                ));
                continue;
            };
            for x in [local.min().x(), local.max().x()] {
                for y in [local.min().y(), local.max().y()] {
                    let point = Point2::new(x, y);
                    match plane.enclose_point(point) {
                        Ok(bounds) => add(p.entity().scope_id, bounds, Some((plane, point))),
                        Err(_) => errors.push(diagnostic(
                            r,
                            IFCCAD_IFCDR_GEOMETRY_INVALID,
                            "planarPolyline",
                            Some(row),
                            "vertices",
                            "curved segment is outside finite range",
                        )),
                    }
                }
            }
        }
    }
    for (row, polyline) in r.spatial_polylines().iter().enumerate() {
        for point in &polyline.points {
            if valid_point3(*point) {
                add(
                    polyline.entity.scope_id,
                    Bounds3d {
                        min: *point,
                        max: *point,
                    },
                    None,
                );
            } else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_GEOMETRY_INVALID,
                    "spatialPolyline",
                    Some(row),
                    "vertices",
                    "coordinate is non-finite",
                ));
            }
        }
    }
    for (row, viewport) in r.viewports().iter().enumerate() {
        match frame_bounds(viewport.frame) {
            Some(frame) => add(viewport.entity.scope_id, frame, None),
            None => errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "frame",
                "viewport frame is invalid or outside finite coordinate range",
            )),
        }
    }
    if errors.is_empty() && verify_bounds {
        for (row, scope) in r.scopes().iter().enumerate() {
            let empty = bounds.get(&scope.id).is_none_or(Option::is_none);
            if failed_scopes.contains(&scope.id)
                || empty != scope.bounds.is_none()
                || scope.bounds.is_some_and(|b| !valid_bounds(b))
            {
                errors.push(diagnostic(r,IFCCAD_IFCDR_BOUNDS_INVALID,"scope",Some(row),"bounds","scope bounds must enclose exact geometry and be null exactly for an empty scope"));
            }
        }
    }
    if errors.is_empty() {
        Ok(bounds)
    } else {
        Err(errors)
    }
}
