//! Intrinsic typed-record constraints shared by decoded and authored documents.
use super::*;
use crate::ocdraw::*;
use std::collections::BTreeSet;

pub(crate) fn logical_error(
    code: &'static str,
    location: impl Into<String>,
    message: &str,
) -> LogicalError {
    LogicalError {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn unit(value: &str) -> bool {
    matches!(
        value,
        "unitless"
            | "in"
            | "ft"
            | "mi"
            | "mm"
            | "cm"
            | "m"
            | "km"
            | "microin"
            | "mil"
            | "yd"
            | "angstrom"
            | "nm"
            | "um"
            | "dm"
            | "dam"
            | "hm"
            | "Gm"
            | "au"
            | "ly"
            | "pc"
            | "usSurveyFoot"
            | "usSurveyInch"
            | "usSurveyYard"
            | "usSurveyMile"
    )
}
fn color(c: &DrawingColor) -> bool {
    c.indexed.as_ref().is_none_or(|(s, _)| !s.is_empty())
        && c.named
            .as_ref()
            .is_none_or(|(c, n)| !c.is_empty() && !n.is_empty())
}
fn opacity(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}
fn weight(v: f64) -> bool {
    v.is_finite() && v >= 0.
}
fn appearance(a: &EntityAppearance) -> bool {
    (match &a.color {
        AppearanceSelection::Explicit(c) => color(c),
        _ => true,
    }) && (match a.opacity {
        AppearanceSelection::Explicit(v) => opacity(v),
        _ => true,
    }) && (match a.line_weight {
        AppearanceSelection::Explicit(v) => weight(v),
        _ => true,
    })
}

pub(crate) fn validate_fields(doc: &OcdrawDocument) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut check = |valid: bool, code, location: String, message| {
        if !valid {
            errors.push(logical_error(code, location, message));
        }
    };
    check(
        !doc.drawing_id.is_empty() && unit(&doc.unit),
        "DOCUMENT_FIELD",
        "/header".into(),
        "drawing identity and unit must be valid",
    );
    check(
        doc.next_entity_id > 0 && doc.next_layout_id > 0,
        "ID_WATERMARK",
        "/header".into(),
        "entity and layout counters must be positive",
    );
    check(
        doc.point_display.is_none_or(PointDisplay::is_valid),
        "POINT_DISPLAY",
        "/pointDisplay".into(),
        "invalid point display size",
    );
    check(
        doc.workspace_state
            .is_none_or(|s| s.current_layer_id.is_some() || s.active_layout_id.is_some()),
        "WORKSPACE_STATE",
        "/drawingWorkspaceState".into(),
        "authored workspace needs a selection",
    );
    for (i, l) in doc.layers.iter().enumerate() {
        check(
            !l.name.is_empty() && color(&l.color) && opacity(l.opacity) && weight(l.line_weight),
            "LAYER_FIELD",
            format!("/layers/{i}"),
            "invalid layer name or appearance",
        );
    }
    for (i, l) in doc.layouts.iter().enumerate() {
        check(
            !l.name.is_empty(),
            "LAYOUT_FIELD",
            format!("/layouts/{i}/name"),
            "layout name must be nonempty",
        );
        check(
            crate::plot_kernel::validate_layout_output(
                &l.settings,
                match l.kind {
                    DrawingLayoutKind::Model => crate::plot_kernel::LayoutOutputKind::Model,
                    DrawingLayoutKind::Paper => crate::plot_kernel::LayoutOutputKind::Paper,
                },
            )
            .is_ok(),
            "PLOT_RECT",
            format!("/layouts/{i}"),
            "invalid layout medium or plot settings",
        );
    }
    for (i, u) in doc.ucs_definitions.iter().enumerate() {
        check(
            !u.definition.name.is_empty() && u.definition.elevation.is_finite(),
            "UCS_FRAME",
            format!("/ucsDefinitions/{i}"),
            "invalid UCS name or elevation",
        );
    }
    for (i, b) in doc.block_definitions.iter().enumerate() {
        check(
            !b.name.is_empty()
                && unit(&b.insertion_unit)
                && b.base_point.into_iter().all(f64::is_finite),
            "BLOCK_FIELD",
            format!("/blockDefinitions/{i}"),
            "invalid block name, units or base point",
        );
    }
    for (i, entity) in doc.opaque_entities.iter().enumerate() {
        check(
            entity.appearance.as_ref().is_none_or(appearance),
            "APPEARANCE_VALUE",
            format!("/opaqueEntities/{i}/appearance"),
            "invalid explicit appearance",
        );
    }
    for e in &doc.geometric_entities {
        if let EntityGeometry::BlockInstance {
            definition_scope_id,
            transform,
        } = &e.geometry
        {
            check(
                doc.block_definitions
                    .iter()
                    .find(|b| b.scope_id == *definition_scope_id)
                    .is_none_or(|b| !b.uniform_scaling || transform.scale().is_uniform()),
                "BLOCK_SCALE",
                format!("/entities/{}", e.id),
                "block definition requires uniform scaling",
            );
        }
        check(
            appearance(&e.appearance),
            "APPEARANCE_VALUE",
            format!("/entities/{}/appearance", e.id),
            "invalid explicit appearance",
        );
        let valid = geometry_is_valid(&e.geometry);
        check(
            valid,
            "ENTITY_GEOMETRY",
            format!("/entities/{}", e.id),
            "invalid geometry, curve parameters or block scaling",
        );
    }
    let layers = doc.layers.iter().map(|l| l.id).collect::<BTreeSet<_>>();
    let owners = owner_index(&doc.scopes);
    let kinds = doc
        .scopes
        .iter()
        .map(|s| (s.id, s.kind))
        .collect::<std::collections::BTreeMap<_, _>>();
    let model = doc
        .scopes
        .iter()
        .find(|s| s.kind == DrawingScopeKind::Model)
        .map(|s| s.id);
    for (i, v) in doc.viewports.iter().enumerate() {
        check(
            appearance(&v.appearance),
            "APPEARANCE_VALUE",
            format!("/viewports/{i}/appearance"),
            "invalid explicit appearance",
        );
        check(
            owners.get(&v.id).and_then(|id| kinds.get(id)) == Some(&DrawingScopeKind::Paper)
                && Some(v.view_scope_id) == model,
            "VIEWPORT_SCOPE",
            format!("/viewports/{i}/viewScopeId"),
            "viewport must view model scope",
        );
        check(
            viewport_bounds(v.frame).is_some(),
            "VIEWPORT_BOUNDS",
            format!("/viewports/{i}/frame"),
            "invalid viewport frame",
        );
        let mut seen = BTreeSet::new();
        for (j, o) in v.layer_overrides.iter().enumerate() {
            check(
                layers.contains(&o.layer_id)
                    && seen.insert(o.layer_id)
                    && o.color.as_ref().is_none_or(color)
                    && o.opacity.is_none_or(opacity)
                    && o.line_weight.is_none_or(weight),
                "VIEWPORT_LAYER",
                format!("/viewports/{i}/layerOverrides/{j}"),
                "invalid viewport layer override",
            );
        }
    }
    errors
}

pub(crate) fn geometry_is_valid(geometry: &EntityGeometry) -> bool {
    match super::geometry_validation::geometry_ref(geometry) {
        Some(g) => crate::geometry_kernel::validate_geometry(g).is_ok(),
        None => matches!(geometry, EntityGeometry::BlockInstance { .. }),
    }
}
