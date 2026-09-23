use super::artifact::EncodedPackage;
use super::error::PackageBuildError;
use super::ifcx::{assemble_ifcx, NodePaths};
use super::state::{
    AppearanceBindingEntry, AppearanceEntry, DrawingState, LayerEntry, PackageState, PendingEntity,
};
use super::types::{
    AppearanceDefinition, AppearanceKey, AppearanceMode, BlockDefinitionKey,
    BlockDefinitionOptions, BlockInstanceDefinition, DrawingOptions, EntityAppearance,
    LayerDefinition, LayerKey, LineDefinition, PackageOptions, PaperSpaceKey, PolylineDefinition,
    ViewportDefinition,
};

use super::prepare::prepare_drawing;
use crate::ifcdr::codec::json::{encode_json, logical_diagnostic, IfcdrEncodeError};
use crate::ifcdr::logical::validate_resource;
use crate::ifcdr::{AppearanceId, EntityId};
use crate::package::canonical_rfc3339_utc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_BUILDER_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
/// Builds one complete IFCCAD package.
///
/// The current writer requires exactly one drawing. Add it with
/// [`PackageBuilder::add_drawing`] and finish the package only after the
/// returned [`DrawingBuilder`] is no longer borrowed.
pub struct PackageBuilder {
    pub(crate) options: PackageOptions,
    pub(crate) state: PackageState,
}

impl PackageBuilder {
    /// Starts a package using its package-level identity and provenance.
    pub fn new(mut options: PackageOptions) -> Result<Self, PackageBuildError> {
        for (field, value) in [
            ("data_version", options.data_version.as_str()),
            ("author", options.author.as_str()),
        ] {
            if value.is_empty() {
                return Err(PackageBuildError::EmptyValue { field });
            }
        }

        options.timestamp =
            canonical_rfc3339_utc(&options.timestamp).ok_or(PackageBuildError::InvalidTimestamp)?;

        Ok(Self {
            options,
            state: PackageState::default(),
        })
    }

    /// Adds the package's single drawing and returns its drawing-scoped builder.
    pub fn add_drawing(
        &mut self,
        options: DrawingOptions,
    ) -> Result<DrawingBuilder<'_>, PackageBuildError> {
        if self.state.drawing.is_some() {
            return Err(PackageBuildError::DrawingAlreadyDefined);
        }
        if options.model_layout_name.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "model_layout_name",
            });
        }
        let token = NEXT_BUILDER_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| PackageBuildError::RangeExhausted {
                kind: "drawing token",
            })?;
        self.state.drawing = Some(DrawingState {
            scopes: vec![crate::ifcdr::logical::IfcdrScope {
                id: 0,
                kind: crate::ifcdr::logical::IfcdrScopeKind::ModelSpace,
                bounds: None,
            }],
            block_definitions: Vec::new(),
            block_names: Default::default(),
            paper_layouts: Vec::new(),
            model_layout_settings: super::LayoutSettings::default(),
            paper_layout_settings: Default::default(),
            plot_style_mode: super::PlotStyleMode::ColorDependent,
            options,
            storage: super::DrawingResourceStorage::default(),
            token,
            appearances: Vec::new(),
            appearance_bindings: Vec::new(),
            layers: Vec::new(),
            layer_names: Default::default(),
            entities: Vec::new(),
            next_entity_id: 1,
            assigned_entity_ids: Default::default(),
        });
        Ok(DrawingBuilder {
            state: self.state.drawing.as_mut().expect("drawing inserted"),
        })
    }

    /// Validates and encodes the completed package in memory.
    pub fn finish(self) -> Result<EncodedPackage, PackageBuildError> {
        let mut drawing = self
            .state
            .drawing
            .ok_or(PackageBuildError::DrawingMissing)?;
        let paths = NodePaths::for_drawing(&drawing)?;
        let uri = super::ifcx::DRAWING_RESOURCE_URI;
        let inline = drawing.storage == super::DrawingResourceStorage::Inline;
        let report_logical = |d| {
            let mut diagnostic = logical_diagnostic(
                if inline {
                    crate::package::DIRECTORY_PACKAGE_ENTRYPOINT
                } else {
                    uri
                },
                d,
            );
            if inline {
                diagnostic.location = Some(format!(
                    "/data/{}/attributes/resource/content{}",
                    paths.representation_index,
                    diagnostic.location.as_deref().unwrap_or("")
                ));
            }
            diagnostic
        };
        let prepared = prepare_drawing(&mut drawing, &paths).map_err(|errors| {
            PackageBuildError::Validation {
                diagnostics: errors.into_iter().map(report_logical).collect(),
            }
        })?;
        let (proof, errors) = validate_resource(prepared).into_parts();
        let proof = proof.ok_or_else(|| PackageBuildError::Validation {
            diagnostics: errors.into_iter().map(report_logical).collect(),
        })?;
        let resource = encode_json(&proof).map_err(map_ifcdr_encode_error)?;
        let entrypoint = assemble_ifcx(&self.options, &drawing, &paths, &resource)?;
        let mut files = vec![(
            crate::package::DIRECTORY_PACKAGE_ENTRYPOINT.to_owned(),
            entrypoint,
        )];
        if drawing.storage == super::DrawingResourceStorage::External {
            files.push((uri.to_owned(), resource.bytes));
        }
        let package = EncodedPackage::new(files);
        let diagnostics = crate::package::read::validate_encoded_package(&package);
        if diagnostics.is_empty() {
            Ok(package)
        } else {
            Err(PackageBuildError::Validation { diagnostics })
        }
    }
}

fn map_ifcdr_encode_error(error: IfcdrEncodeError) -> PackageBuildError {
    match error {
        IfcdrEncodeError::RangeExhausted { kind } => PackageBuildError::RangeExhausted { kind },
        IfcdrEncodeError::Serialization { message } => PackageBuildError::Encoding {
            stage: "IFCDR",
            message,
        },
    }
}

/// Drawing-scoped access to appearances, layers, and model-space entities.
pub struct DrawingBuilder<'a> {
    state: &'a mut DrawingState,
}

impl DrawingBuilder<'_> {
    pub fn set_plot_style_mode(&mut self, mode: super::PlotStyleMode) {
        self.state.plot_style_mode = mode;
    }
    pub fn set_model_layout_settings(&mut self, settings: super::LayoutSettings) {
        self.state.model_layout_settings = settings;
    }
    pub fn set_paper_layout_settings(
        &mut self,
        key: PaperSpaceKey,
        settings: super::LayoutSettings,
    ) -> Result<(), PackageBuildError> {
        self.state.validate_paper_key(key)?;
        self.state
            .paper_layout_settings
            .insert(key.local_id, settings);
        Ok(())
    }
    /// Selects inline or external storage; new drawings default to external.
    /// This affects package encoding, not the drawing's identity or semantics.
    pub fn set_resource_storage(&mut self, storage: super::DrawingResourceStorage) {
        self.state.storage = storage;
    }

    /// Opens the appearance collection for this drawing.
    pub fn appearances(&mut self) -> DrawingAppearances<'_> {
        DrawingAppearances { state: self.state }
    }

    /// Opens the layer collection for this drawing.
    pub fn layers(&mut self) -> DrawingLayers<'_> {
        DrawingLayers { state: self.state }
    }

    /// Opens the drawing's model-space entity collection.
    pub fn model_space(&mut self) -> ModelSpaceBuilder<'_> {
        ScopeEntitiesBuilder {
            state: self.state,
            scope_id: 0,
        }
    }
    /// Adds metadata and an initially empty local definition scope.
    pub fn add_block_definition(
        &mut self,
        options: BlockDefinitionOptions,
    ) -> Result<BlockDefinitionKey, PackageBuildError> {
        use crate::ifcdr::logical::{IfcdrBlockDefinition, IfcdrScope, IfcdrScopeKind};
        if options.name.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "block_name",
            });
        }
        if !crate::ifcdr::logical::valid_point3(options.base_point) {
            return Err(PackageBuildError::NonFiniteCoordinate);
        }
        let name_key = crate::ifcdr::names::name_key(&options.name);
        if self.state.block_names.contains_key(&name_key) {
            return Err(PackageBuildError::DuplicateBlockName { name: options.name });
        }
        let id = u32::try_from(self.state.scopes.len())
            .map_err(|_| PackageBuildError::RangeExhausted { kind: "scope" })?;
        self.state.block_definitions.push(IfcdrBlockDefinition {
            scope_id: id,
            name: options.name,
            base_point: options.base_point,
            description: options.description,
            anonymous: options.anonymous,
            insertion_unit: options.insertion_unit,
            explodable: options.explodable,
            scaling: options.scaling,
        });
        self.state.scopes.push(IfcdrScope {
            id,
            kind: IfcdrScopeKind::BlockDefinition,
            bounds: None,
        });
        self.state.block_names.insert(name_key, id);
        Ok(BlockDefinitionKey {
            builder_token: self.state.token,
            local_id: id,
        })
    }
    pub fn block_definition(
        &mut self,
        key: BlockDefinitionKey,
    ) -> Result<ScopeEntitiesBuilder<'_>, PackageBuildError> {
        self.state.validate_block_key(key)?;
        Ok(ScopeEntitiesBuilder {
            state: self.state,
            scope_id: key.local_id,
        })
    }
    /// Creates a native paper coordinate scope and minimal IFCX layout binding.
    /// This does not add viewports or plot settings.
    pub fn add_paper_space(
        &mut self,
        layout_name: String,
    ) -> Result<PaperSpaceKey, PackageBuildError> {
        if layout_name.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "layout_name",
            });
        }
        let id = u32::try_from(self.state.scopes.len())
            .map_err(|_| PackageBuildError::RangeExhausted { kind: "scope" })?;
        self.state.scopes.push(crate::ifcdr::logical::IfcdrScope {
            id,
            kind: crate::ifcdr::logical::IfcdrScopeKind::PaperSpace,
            bounds: None,
        });
        self.state.paper_layouts.push((id, layout_name));
        Ok(PaperSpaceKey {
            builder_token: self.state.token,
            local_id: id,
        })
    }
    pub fn paper_space(
        &mut self,
        key: PaperSpaceKey,
    ) -> Result<ScopeEntitiesBuilder<'_>, PackageBuildError> {
        self.state.validate_paper_key(key)?;
        Ok(ScopeEntitiesBuilder {
            state: self.state,
            scope_id: key.local_id,
        })
    }
}

pub struct DrawingAppearances<'a> {
    state: &'a mut DrawingState,
}

impl DrawingAppearances<'_> {
    pub fn add(
        &mut self,
        definition: AppearanceDefinition,
    ) -> Result<AppearanceKey, PackageBuildError> {
        validate_appearance(&definition)?;
        let offset = u32::try_from(self.state.appearances.len())
            .map_err(|_| PackageBuildError::RangeExhausted { kind: "appearance" })?;
        let local_id = 2_u32
            .checked_add(offset)
            .ok_or(PackageBuildError::RangeExhausted { kind: "appearance" })?;
        self.state.appearances.push(AppearanceEntry { definition });
        Ok(AppearanceKey {
            builder_token: self.state.token,
            local_id,
        })
    }
}

pub struct DrawingLayers<'a> {
    state: &'a mut DrawingState,
}

impl DrawingLayers<'_> {
    pub fn add(&mut self, definition: LayerDefinition) -> Result<LayerKey, PackageBuildError> {
        if definition.name.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "layer_name",
            });
        }
        self.state.validate_appearance_key(definition.appearance)?;
        let normalized_name = crate::ifcdr::names::name_key(&definition.name);
        if self.state.layer_names.contains_key(&normalized_name) {
            return Err(PackageBuildError::DuplicateLayerName {
                name: definition.name,
            });
        }
        let local_id = u32::try_from(self.state.layers.len())
            .map_err(|_| PackageBuildError::RangeExhausted { kind: "layer" })?;
        let index = self.state.layers.len();
        self.state.layers.push(LayerEntry {
            local_id,
            definition,
        });
        self.state.layer_names.insert(normalized_name, index);
        Ok(LayerKey {
            builder_token: self.state.token,
            local_id,
        })
    }

    pub fn by_name(&self, name: &str) -> Option<LayerKey> {
        let index = *self
            .state
            .layer_names
            .get(&crate::ifcdr::names::name_key(name))?;
        Some(LayerKey {
            builder_token: self.state.token,
            local_id: self.state.layers[index].local_id,
        })
    }
}

impl DrawingState {
    fn validate_paper_key(&self, key: PaperSpaceKey) -> Result<(), PackageBuildError> {
        if key.builder_token != self.token
            || self
                .scopes
                .get(key.local_id as usize)
                .is_none_or(|scope| scope.kind != crate::ifcdr::logical::IfcdrScopeKind::PaperSpace)
        {
            return Err(PackageBuildError::ForeignPaperSpaceKey);
        }
        Ok(())
    }
    fn validate_block_key(&self, key: BlockDefinitionKey) -> Result<(), PackageBuildError> {
        if key.builder_token != self.token
            || self
                .scopes
                .get(key.local_id as usize)
                .is_none_or(|s| s.kind != crate::ifcdr::logical::IfcdrScopeKind::BlockDefinition)
        {
            return Err(PackageBuildError::ForeignBlockDefinitionKey);
        }
        Ok(())
    }
    fn validate_appearance_key(&self, key: AppearanceKey) -> Result<(), PackageBuildError> {
        let index = key
            .local_id
            .checked_sub(2)
            .and_then(|value| usize::try_from(value).ok());
        if key.builder_token != self.token
            || index.is_none_or(|index| index >= self.appearances.len())
        {
            return Err(PackageBuildError::ForeignAppearanceKey);
        }
        Ok(())
    }

    fn validate_layer_key(&self, key: LayerKey) -> Result<(), PackageBuildError> {
        let index = usize::try_from(key.local_id).ok();
        if key.builder_token != self.token || index.is_none_or(|index| index >= self.layers.len()) {
            return Err(PackageBuildError::ForeignLayerKey);
        }
        Ok(())
    }

    fn resolve_entity_appearance(
        &mut self,
        appearance: EntityAppearance,
    ) -> Result<AppearanceId, PackageBuildError> {
        let uses_explicit = [
            appearance.color_mode,
            appearance.opacity_mode,
            appearance.line_pattern_mode,
            appearance.line_weight_mode,
        ]
        .contains(&AppearanceMode::Explicit);
        if uses_explicit && appearance.appearance.is_none() {
            return Err(PackageBuildError::AppearanceDefinitionMissing);
        }
        if let Some(key) = appearance.appearance {
            self.validate_appearance_key(key)?;
        }
        if appearance == EntityAppearance::by_layer() {
            return Ok(AppearanceId::from(0));
        }
        if appearance == EntityAppearance::by_block() {
            return Ok(AppearanceId::from(1));
        }
        if let Some(existing) = self
            .appearance_bindings
            .iter()
            .find(|entry| entry.definition == appearance)
        {
            return Ok(existing.id);
        }
        let offset = u32::try_from(self.appearance_bindings.len()).map_err(|_| {
            PackageBuildError::RangeExhausted {
                kind: "appearance binding",
            }
        })?;
        let local_id = 2_u32
            .checked_add(offset)
            .ok_or(PackageBuildError::RangeExhausted {
                kind: "appearance binding",
            })?;
        let id = AppearanceId::from(local_id);
        self.appearance_bindings.push(AppearanceBindingEntry {
            id,
            definition: appearance,
        });
        Ok(id)
    }

    fn candidate_entity_id(
        &self,
        supplied: Option<EntityId>,
    ) -> Result<EntityId, PackageBuildError> {
        let id = supplied
            .or_else(|| EntityId::new(self.next_entity_id))
            .ok_or(PackageBuildError::RangeExhausted { kind: "entity" })?;
        if self.assigned_entity_ids.contains(&id) {
            return Err(PackageBuildError::DuplicateEntityId { id });
        }
        id.get()
            .checked_add(1)
            .ok_or(PackageBuildError::RangeExhausted { kind: "entity" })?;
        Ok(id)
    }

    fn record_entity_id(&mut self, id: EntityId) {
        self.next_entity_id = self.next_entity_id.max(id.get() + 1);
        self.assigned_entity_ids.insert(id);
    }
}

pub struct ScopeEntitiesBuilder<'a> {
    state: &'a mut DrawingState,
    scope_id: u32,
}
/// Model-space convenience using the same allocation and insertion rules.
pub type ModelSpaceBuilder<'a> = ScopeEntitiesBuilder<'a>;

impl ScopeEntitiesBuilder<'_> {
    pub fn add_viewport(
        &mut self,
        definition: ViewportDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        if self
            .state
            .scopes
            .get(self.scope_id as usize)
            .is_none_or(|scope| scope.kind != crate::ifcdr::logical::IfcdrScopeKind::PaperSpace)
        {
            return Err(PackageBuildError::ViewportRequiresPaperSpace);
        }
        self.state.validate_layer_key(definition.layer)?;
        for override_row in &definition.layer_overrides {
            self.state.validate_layer_key(override_row.layer)?;
            if let Some(patch) = &override_row.appearance {
                if patch
                    .opacity
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                {
                    return Err(PackageBuildError::InvalidOpacity);
                }
                if patch
                    .line_weight
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                {
                    return Err(PackageBuildError::InvalidLineWeight);
                }
            }
        }
        let entity_id = self.state.candidate_entity_id(None)?;
        let appearance_id = self
            .state
            .resolve_entity_appearance(definition.appearance)?;
        self.state.record_entity_id(entity_id);
        self.state.entities.push(PendingEntity::Viewport {
            scope_id: self.scope_id,
            entity_id,
            appearance_id,
            definition,
        });
        Ok(entity_id)
    }
    pub fn add_block_instance(
        &mut self,
        definition: BlockInstanceDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.insert_block_instance(None, definition)
    }
    pub fn add_block_instance_with_id(
        &mut self,
        id: EntityId,
        definition: BlockInstanceDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.insert_block_instance(Some(id), definition)
    }
    fn insert_block_instance(
        &mut self,
        supplied: Option<EntityId>,
        definition: BlockInstanceDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.state.validate_block_key(definition.definition)?;
        self.state.validate_layer_key(definition.layer)?;
        let target = self
            .state
            .block_definitions
            .iter()
            .find(|d| d.scope_id == definition.definition.local_id)
            .expect("builder definition key");
        if target.scaling == crate::ifcdr::BlockScaling::Uniform
            && !definition.transform.scale().is_uniform()
        {
            return Err(PackageBuildError::NonUniformBlockScale);
        }
        let entity_id = self.state.candidate_entity_id(supplied)?;
        let appearance_id = self
            .state
            .resolve_entity_appearance(definition.appearance)?;
        self.state.record_entity_id(entity_id);
        self.state.entities.push(PendingEntity::BlockInstance {
            scope_id: self.scope_id,
            entity_id,
            appearance_id,
            definition,
        });
        Ok(entity_id)
    }
    pub fn add_line(&mut self, definition: LineDefinition) -> Result<EntityId, PackageBuildError> {
        self.insert_line(None, definition)
    }

    /// Adds a line with a caller-supplied ID, advancing automatic allocation as needed.
    pub fn add_line_with_id(
        &mut self,
        id: EntityId,
        definition: LineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.insert_line(Some(id), definition)
    }

    fn insert_line(
        &mut self,
        supplied: Option<EntityId>,
        definition: LineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.state.validate_layer_key(definition.layer)?;
        if !crate::ifcdr::logical::valid_point3(definition.start)
            || !crate::ifcdr::logical::valid_point3(definition.end)
        {
            return Err(PackageBuildError::NonFiniteCoordinate);
        }
        let entity_id = self.state.candidate_entity_id(supplied)?;
        let appearance_id = self
            .state
            .resolve_entity_appearance(definition.appearance)?;
        self.state.record_entity_id(entity_id);
        self.state.entities.push(PendingEntity::Line {
            scope_id: self.scope_id,
            entity_id,
            appearance_id,
            definition,
        });
        Ok(entity_id)
    }

    pub fn add_polyline(
        &mut self,
        definition: PolylineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.insert_polyline(None, definition)
    }

    /// Adds a polyline with a caller-supplied ID shared with the other entity kinds.
    pub fn add_polyline_with_id(
        &mut self,
        id: EntityId,
        definition: PolylineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        self.insert_polyline(Some(id), definition)
    }

    fn insert_polyline(
        &mut self,
        supplied: Option<EntityId>,
        definition: PolylineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        if !crate::ifcdr::logical::valid_polyline_vertex_count(definition.points.len()) {
            return Err(PackageBuildError::PolylineTooShort);
        }
        self.state.validate_layer_key(definition.layer)?;
        validate_points(definition.points.iter().copied())?;
        for point in &definition.points {
            definition
                .placement
                .enclose_point(*point)
                .map_err(|_| PackageBuildError::PlacedCoordinateOutOfRange)?;
        }
        let entity_id = self.state.candidate_entity_id(supplied)?;
        let appearance_id = self
            .state
            .resolve_entity_appearance(definition.appearance)?;
        self.state.record_entity_id(entity_id);
        self.state.entities.push(PendingEntity::Polyline {
            scope_id: self.scope_id,
            entity_id,
            appearance_id,
            definition,
        });
        Ok(entity_id)
    }
}

fn validate_points(
    points: impl IntoIterator<Item = crate::ifcdr::Point2>,
) -> Result<(), PackageBuildError> {
    if points
        .into_iter()
        .any(|point| !crate::ifcdr::logical::valid_point(point))
    {
        return Err(PackageBuildError::NonFiniteCoordinate);
    }
    Ok(())
}

fn validate_appearance(definition: &AppearanceDefinition) -> Result<(), PackageBuildError> {
    for (field, value) in [
        ("appearance_name", definition.name.as_str()),
        ("line_pattern_name", definition.line_pattern.name.as_str()),
    ] {
        if value.is_empty() {
            return Err(PackageBuildError::EmptyValue { field });
        }
    }
    if !crate::ifcdr::logical::valid_opacity(definition.opacity) {
        return Err(PackageBuildError::InvalidOpacity);
    }
    if !crate::ifcdr::logical::valid_line_weight(definition.line_weight) {
        return Err(PackageBuildError::InvalidLineWeight);
    }
    if definition
        .color
        .indexed
        .as_ref()
        .is_some_and(|color| color.system.is_empty())
    {
        return Err(PackageBuildError::EmptyValue {
            field: "indexed_color_system",
        });
    }
    if let Some(color) = &definition.color.named {
        if color.catalog.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "named_color_catalog",
            });
        }
        if color.name.is_empty() {
            return Err(PackageBuildError::EmptyValue {
                field: "named_color_name",
            });
        }
    }
    Ok(())
}
