use super::*;
use crate::ifcdr::{Bounds2d, Point2};
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
    Line,
    Polyline,
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
fn diagnostic<R: IfcdrResourceAccess>(
    r: &R,
    code: &'static str,
    collection: &'static str,
    row: Option<usize>,
    property: &'static str,
    message: impl Into<String>,
) -> IfcdrDiagnostic {
    IfcdrDiagnostic {
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
    for (row, scope) in r.scopes().iter().enumerate() {
        if !valid_point(scope.base) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_GEOMETRY_INVALID,
                "scope",
                Some(row),
                "base",
                "scope base must be finite",
            ));
        }
    }
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
                IfcdrEntityKind::Polyline,
                row,
                &mut evidence,
                &mut errors,
            );
            if !valid_polyline_vertex_count(polyline.vertex_count()) {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_POLYLINE_INVALID,
                    "polyline",
                    Some(row),
                    "vertices",
                    "polyline requires at least two vertices",
                ));
            }
        } else {
            identity_valid = false;
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_STRUCTURE_INVALID,
                "polyline",
                Some(row),
                "row",
                "polyline row is inaccessible",
            ));
        }
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
    match geometric_bounds(r) {
        Err(mut geometry_errors) => errors.append(&mut geometry_errors),
        Ok(actual) => {
            let empty = lines.is_empty() && polylines.is_empty();
            let valid = match (r.bounds(), actual) {
                (None, None) => empty,
                (Some(b), Some(a)) => {
                    valid_point(b.min())
                        && valid_point(b.max())
                        && b.min().x() <= a.min().x()
                        && b.min().y() <= a.min().y()
                        && b.max().x() >= a.max().x()
                        && b.max().y() >= a.max().y()
                }
                _ => false,
            };
            if !valid {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_BOUNDS_INVALID,
                    "resource",
                    None,
                    "bounds",
                    "bounds must enclose all geometry, and be absent exactly for empty resources",
                ));
            }
        }
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
        IfcdrEntityKind::Line => "line",
        IfcdrEntityKind::Polyline => "polyline",
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
) -> Result<Option<Bounds2d>, Vec<IfcdrDiagnostic>> {
    let mut bounds: Option<Bounds2d> = None;
    let mut errors = Vec::new();
    let mut point =
        |value: Option<Point2>, collection: &'static str, row: usize, property: &'static str| {
            if let Some(p) = value.filter(|p| valid_point(*p)) {
                match &mut bounds {
                    None => bounds = Some(Bounds2d { min: p, max: p }),
                    Some(b) => {
                        b.min = Point2::new(b.min.x().min(p.x()), b.min.y().min(p.y()));
                        b.max = Point2::new(b.max.x().max(p.x()), b.max.y().max(p.y()));
                    }
                }
            } else {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_GEOMETRY_INVALID,
                    collection,
                    Some(row),
                    property,
                    "coordinate is non-finite or inaccessible",
                ));
            }
        };
    let lines = r.lines();
    for row in 0..lines.len() {
        match lines.get(row) {
            Some(line) => {
                point(Some(line.start), "line", row, "start");
                point(Some(line.end), "line", row, "end");
            }
            None => point(None, "line", row, "row"),
        }
    }
    let polylines = r.polylines();
    for row in 0..polylines.len() {
        match polylines.get(row) {
            Some(polyline) => {
                for i in 0..polyline.vertex_count() {
                    point(polyline.vertex(i), "polyline", row, "vertices");
                }
            }
            None => point(None, "polyline", row, "row"),
        }
    }
    if errors.is_empty() {
        Ok(bounds)
    } else {
        Err(errors)
    }
}
