use super::model::*;
use crate::ocdraw::names::name_key;
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
        let mut errors = super::line_pattern_validation::validate_line_patterns(
            &self.line_patterns,
            self.next_line_pattern_id,
            &self.line_pattern_refs,
            &self.line_pattern_scales,
        );
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
            if let Some(bounds) = scope.bounds {
                let min = bounds.min().components();
                let max = bounds.max().components();
                if min
                    .into_iter()
                    .zip(max)
                    .any(|(min, max)| !min.is_finite() || !max.is_finite() || min > max)
                {
                    errors.push(error(
                        "SCOPE_BOUNDS",
                        format!("/scopes/{index}/bounds"),
                        "scope minimum exceeds maximum or is non-finite",
                    ));
                }
            }
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
        let ucs_names = self
            .ucs_definitions
            .iter()
            .map(|definition| NamedId {
                id: definition.id,
                name: definition.name.clone(),
            })
            .collect::<Vec<_>>();
        let ucs_ids = named_ids(&ucs_names, "ucsDefinitions", &mut errors);
        for (index, definition) in self.ucs_definitions.iter().enumerate() {
            if definition.frame.is_none() {
                errors.push(error(
                    "UCS_FRAME",
                    format!("/ucsDefinitions/{index}/frame"),
                    "UCS frame is invalid",
                ));
            }
        }
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
        let mut entity_ids = BTreeSet::new();
        for entity in &self.entities {
            if entity.id == 0 || !entity_ids.insert(entity.id) {
                errors.push(error(
                    "ENTITY_ID",
                    &entity.location,
                    "entity ID must be nonzero and unique",
                ));
            }
            if entity.layer_id.is_some_and(|id| !layer_ids.contains(&id)) {
                errors.push(error(
                    "ENTITY_REF",
                    &entity.location,
                    "entity Layer does not resolve",
                ));
            }
            for pair in entity.appearance.iter().flatten() {
                if (pair.mode == AppearanceMode::Explicit) != pair.has_value {
                    errors.push(error(
                        "APPEARANCE_PAIR",
                        &entity.location,
                        "appearance mode/value pair is inconsistent",
                    ));
                }
            }
        }
        let mut owners = BTreeMap::new();
        for (index, scope) in self.scopes.iter().enumerate() {
            for (position, id) in scope.entities.iter().enumerate() {
                let location = format!("/scopes/{index}/entities/{position}");
                if !entity_ids.contains(id) {
                    errors.push(error(
                        "ENTITY_REFERENCE",
                        &location,
                        "scope entry does not resolve to an entity",
                    ));
                }
                if owners.insert(*id, scope.id).is_some() {
                    errors.push(error(
                        "ENTITY_OWNERSHIP",
                        &location,
                        "entity must occur exactly once in exactly one scope list",
                    ));
                }
            }
            if scope.has_bounds.is_some_and(|has_bounds| {
                has_bounds
                    != (scope.geometry_completeness == super::ScopeGeometryCompleteness::Complete)
            }) {
                errors.push(error(
                    "SCOPE_BOUNDS",
                    format!("/scopes/{index}/bounds"),
                    "empty or incomplete scope needs null bounds; complete nonempty scope needs finite bounds",
                ));
            }
        }
        let mut block_edges = BTreeMap::<u32, Vec<u32>>::new();
        for entity in &self.entities {
            if !owners.contains_key(&entity.id) {
                errors.push(error(
                    "ENTITY_OWNERSHIP",
                    &entity.location,
                    "entity has no owner in the scope lists",
                ));
            }
            if let Some(target) = entity.definition_scope_id {
                if scope_kinds.get(&target) != Some(&ScopeKind::Block) {
                    errors.push(error(
                        "BLOCK_REF",
                        &entity.location,
                        "block instance must reference a block-definition scope",
                    ));
                } else if let Some(owner) = owners.get(&entity.id) {
                    block_edges.entry(*owner).or_default().push(target);
                }
            }
        }
        if entity_ids
            .iter()
            .next_back()
            .is_some_and(|id| self.next_entity_id <= *id)
        {
            errors.push(error(
                "ID_WATERMARK",
                "/header/nextEntityId",
                "ID watermark must exceed allocated entities",
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
