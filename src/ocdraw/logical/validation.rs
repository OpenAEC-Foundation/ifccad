use super::model::*;
use crate::drawing::names::name_key;
use std::collections::{BTreeMap, BTreeSet};

fn error(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> LogicalError {
    LogicalError {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn named_ids(
    rows: &[NamedId],
    label: &'static str,
    errors: &mut Vec<LogicalError>,
) -> BTreeSet<u32> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        if !ids.insert(row.id) {
            errors.push(error(
                "DUPLICATE_ID",
                format!("/{label}/{index}/id"),
                format!("duplicate {label} ID {}", row.id),
            ));
        }
        if !names.insert(name_key(&row.name)) {
            errors.push(error(
                "DUPLICATE_NAME",
                format!("/{label}/{index}/name"),
                format!("duplicate {label} name {}", row.name),
            ));
        }
    }
    ids
}

impl DrawingModel {
    pub(crate) fn validate(&self) -> Vec<LogicalError> {
        let mut errors = Vec::new();
        let layer_ids = named_ids(&self.layers, "layers", &mut errors);
        let layout_names = self
            .layouts
            .iter()
            .map(|layout| NamedId {
                id: layout.id,
                name: layout.name.clone(),
            })
            .collect::<Vec<_>>();
        let layout_ids = named_ids(&layout_names, "layouts", &mut errors);
        let mut scope_kinds = BTreeMap::new();
        for (index, scope) in self.scopes.iter().enumerate() {
            if scope_kinds.insert(scope.id, scope.kind).is_some() {
                errors.push(error(
                    "DUPLICATE_ID",
                    format!("/scopes/{index}/id"),
                    "duplicate scope ID",
                ));
            }
        }
        if scope_kinds
            .values()
            .filter(|kind| **kind == ScopeKind::Model)
            .count()
            != 1
        {
            errors.push(error(
                "MODEL_SCOPE",
                "/scopes",
                "exactly one model scope is required",
            ));
        }
        let mut layout_scopes = BTreeMap::<u32, usize>::new();
        let mut tabs = BTreeSet::new();
        let mut model_layouts = 0;
        for (index, layout) in self.layouts.iter().enumerate() {
            if let Some(plot) = layout.plot_rectangles {
                for (label, rect) in [
                    ("printableArea", Some(plot.printable_area)),
                    ("window", plot.window),
                ] {
                    if rect.is_some_and(|rect| {
                        ![rect.min_x, rect.min_y, rect.max_x, rect.max_y]
                            .into_iter()
                            .all(f64::is_finite)
                            || rect.min_x >= rect.max_x
                            || rect.min_y >= rect.max_y
                    }) {
                        errors.push(error(
                            "PLOT_RECT",
                            format!("/layouts/{index}/plotSettings/{label}"),
                            "plot rectangle must have positive width and height",
                        ));
                    }
                }
            }
            if layout.limits.is_some_and(|rect| {
                ![rect.min_x, rect.min_y, rect.max_x, rect.max_y]
                    .into_iter()
                    .all(f64::is_finite)
                    || rect.min_x > rect.max_x
                    || rect.min_y > rect.max_y
            }) {
                errors.push(error(
                    "LAYOUT_LIMITS",
                    format!("/layouts/{index}/limits"),
                    "invalid layout limits",
                ));
            }
            if layout.kind == ScopeKind::Model {
                model_layouts += 1;
            }
            if scope_kinds.get(&layout.scope_id) != Some(&layout.kind) {
                errors.push(error(
                    "LAYOUT_SCOPE",
                    format!("/layouts/{index}/scopeId"),
                    "layout targets wrong or missing scope kind",
                ));
            }
            *layout_scopes.entry(layout.scope_id).or_default() += 1;
            if !tabs.insert(layout.tab_index)
                || (layout.kind == ScopeKind::Model && layout.tab_index != 0)
            {
                errors.push(error(
                    "LAYOUT_TAB",
                    format!("/layouts/{index}/tabIndex"),
                    "invalid layout tab index",
                ));
            }
        }
        if model_layouts != 1 || tabs != (0..self.layouts.len() as u32).collect() {
            errors.push(error(
                "LAYOUT_CLOSURE",
                "/layouts",
                "one model layout and contiguous tabs are required",
            ));
        }
        for (id, kind) in &scope_kinds {
            if *kind != ScopeKind::Block && layout_scopes.get(id) != Some(&1) {
                errors.push(error(
                    "LAYOUT_CLOSURE",
                    "/layouts",
                    format!("scope {id} requires one layout"),
                ));
            }
        }
        let mut block_scopes = BTreeMap::<u32, usize>::new();
        let mut block_names = BTreeSet::new();
        for (index, block) in self.blocks.iter().enumerate() {
            if scope_kinds.get(&block.scope_id) != Some(&ScopeKind::Block) {
                errors.push(error(
                    "BLOCK_SCOPE",
                    format!("/blockDefinitions/{index}/scopeId"),
                    "block definition must target a block scope",
                ));
            }
            *block_scopes.entry(block.scope_id).or_default() += 1;
            if !block_names.insert(name_key(&block.name)) {
                errors.push(error(
                    "DUPLICATE_NAME",
                    format!("/blockDefinitions/{index}/name"),
                    "duplicate block definition name",
                ));
            }
        }
        for (id, kind) in &scope_kinds {
            if *kind == ScopeKind::Block && block_scopes.get(id) != Some(&1) {
                errors.push(error(
                    "BLOCK_SCOPE",
                    "/blockDefinitions",
                    format!("block scope {id} requires one definition"),
                ));
            }
        }
        for (field, next, ids) in [
            ("nextLayerId", self.next_layer_id, &layer_ids),
            ("nextLayoutId", self.next_layout_id, &layout_ids),
        ] {
            if ids.iter().next_back().is_some_and(|max| next <= *max) {
                errors.push(error(
                    "ID_WATERMARK",
                    format!("/header/{field}"),
                    "ID watermark must exceed allocated IDs",
                ));
            }
        }
        for (field, selected, ids) in [
            ("currentLayerId", self.current_layer_id, &layer_ids),
            ("activeLayoutId", self.active_layout_id, &layout_ids),
        ] {
            if selected.is_some_and(|id| !ids.contains(&id)) {
                errors.push(error(
                    "WORKSPACE_REF",
                    format!("/drawingWorkspaceState/{field}"),
                    "selection does not resolve",
                ));
            }
        }
        let ucs_ids = named_ids(&self.ucs_definitions, "ucsDefinitions", &mut errors);
        let mut model_window_ids = BTreeSet::new();
        for (index, id) in self.model_window_ids.iter().enumerate() {
            if !model_window_ids.insert(*id) {
                errors.push(error(
                    "DUPLICATE_ID",
                    format!("/modelWindows/{index}/modelWindowId"),
                    "duplicate model window ID",
                ));
            }
        }
        if self
            .active_model_window_id
            .is_some_and(|id| !model_window_ids.contains(&id))
        {
            errors.push(error(
                "MODEL_WINDOW_REF",
                "/drawingViewState/activeModelWindowId",
                "active model window does not resolve",
            ));
        }
        for (id, location) in &self.named_ucs_refs {
            if !id.is_some_and(|id| ucs_ids.contains(&id)) {
                errors.push(error("UCS_REF", location, "named UCS does not resolve"));
            }
        }
        for choice in &self.ucs_choices {
            let valid = match choice.kind.as_str() {
                "World" => !choice.has_id && !choice.has_frame,
                "Named" => choice.has_id && !choice.has_frame,
                "Unnamed" => !choice.has_id && choice.has_frame && choice.frame_valid,
                _ => false,
            };
            if !valid {
                errors.push(error(
                    "UCS_CHOICE",
                    &choice.location,
                    "UCS choice must contain exactly its selected value and a valid frame",
                ));
            }
        }
        let mut entity_scopes = BTreeMap::new();
        let mut block_edges = BTreeMap::<u32, Vec<u32>>::new();
        for entity in &self.entities {
            if entity.id == 0 || entity_scopes.insert(entity.id, entity.scope_id).is_some() {
                errors.push(error(
                    "ENTITY_ID",
                    &entity.location,
                    "entity ID must be nonzero and unique",
                ));
            }
            if !layer_ids.contains(&entity.layer_id) || !scope_kinds.contains_key(&entity.scope_id)
            {
                errors.push(error(
                    "ENTITY_REF",
                    &entity.location,
                    "entity Layer or scope does not resolve",
                ));
            }
            for pair in &entity.appearance {
                if (pair.mode == AppearanceMode::Explicit) != pair.has_value {
                    errors.push(error(
                        "APPEARANCE_PAIR",
                        &entity.location,
                        "appearance mode/value pair is inconsistent",
                    ));
                }
            }
            if let Some(target) = entity.definition_scope_id {
                if scope_kinds.get(&target) != Some(&ScopeKind::Block) {
                    errors.push(error(
                        "BLOCK_REF",
                        &entity.location,
                        "block instance must reference a block-definition scope",
                    ));
                } else {
                    block_edges.entry(entity.scope_id).or_default().push(target);
                }
            }
        }
        if entity_scopes
            .keys()
            .next_back()
            .is_some_and(|id| self.next_entity_id <= *id)
        {
            errors.push(error(
                "ID_WATERMARK",
                "/header/nextEntityId",
                "ID watermark must exceed allocated entities",
            ));
        }
        for (index, scope) in self.scopes.iter().enumerate() {
            if let Some(has_bounds) = scope.has_bounds {
                let has_entities = entity_scopes.values().any(|owner| *owner == scope.id);
                if has_bounds != has_entities {
                    errors.push(error(
                        "SCOPE_BOUNDS",
                        format!("/scopes/{index}/bounds"),
                        "empty scope needs null bounds; nonempty scope needs bounds",
                    ));
                }
            }
        }
        let mut seen_scopes = BTreeSet::new();
        let mut ordered = BTreeSet::new();
        for order in &self.orders {
            if !scope_kinds.contains_key(&order.scope_id) || !seen_scopes.insert(order.scope_id) {
                errors.push(error(
                    "ENTITY_ORDER",
                    &order.location,
                    "order scope must resolve uniquely",
                ));
            }
            for id in &order.entities {
                if !ordered.insert(*id) || entity_scopes.get(id) != Some(&order.scope_id) {
                    errors.push(error(
                        "ENTITY_ORDER",
                        &order.location,
                        "order contains a duplicate or an entity from another scope",
                    ));
                }
            }
        }
        if ordered.len() != entity_scopes.len() {
            errors.push(error(
                "ENTITY_ORDER",
                "/streams/entityOrderEntryStream",
                "every entity must occur in draw order",
            ));
        }
        let mut indegree = scope_kinds
            .keys()
            .map(|id| (*id, 0usize))
            .collect::<BTreeMap<_, _>>();
        for targets in block_edges.values() {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree += 1;
                }
            }
        }
        let mut queue = indegree
            .iter()
            .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
            .collect::<std::collections::VecDeque<_>>();
        let mut visited = 0;
        while let Some(id) = queue.pop_front() {
            visited += 1;
            if let Some(targets) = block_edges.get(&id) {
                for target in targets {
                    if let Some(degree) = indegree.get_mut(target) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(*target);
                        }
                    }
                }
            }
        }
        if visited != indegree.len() {
            errors.push(error(
                "BLOCK_CYCLE",
                "/streams/blockInstanceStream",
                "block instance dependency graph is cyclic",
            ));
        }
        errors
    }
}
