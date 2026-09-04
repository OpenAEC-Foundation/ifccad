use super::artifact::EncodedPackage;
use super::error::PackageBuildError;
use super::ifcx::{assemble_ifcx, NodePaths, MODEL_SPACE_RESOURCE_URI};
use super::state::{AppearanceEntry, DrawingState, LayerEntry, PackageState, PendingEntity};
use super::types::{
    AppearanceDefinition, AppearanceKey, DrawingOptions, EntityAppearance, LayerDefinition,
    LayerKey, LineDefinition, PackageOptions, PolylineDefinition,
};

use crate::ifcdr::write::{
    encode, IfcdrAppearanceBindingInput, IfcdrEncodeError, IfcdrEncodeInput, IfcdrEntityInput,
    IfcdrLayerBindingInput, IfcdrScopeInput,
};
use crate::ifcdr::{AppearanceId, EntityId, LayerId, Point2, ScopeId};
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
            options,
            token,
            appearances: Vec::new(),
            layers: Vec::new(),
            layer_names: Default::default(),
            entities: Vec::new(),
        });
        Ok(DrawingBuilder {
            state: self.state.drawing.as_mut().expect("drawing inserted"),
        })
    }

    /// Validates and encodes the completed package in memory.
    pub fn finish(self) -> Result<EncodedPackage, PackageBuildError> {
        let drawing = self
            .state
            .drawing
            .ok_or(PackageBuildError::DrawingMissing)?;
        let paths = NodePaths::for_drawing(&drawing)?;
        let layers = drawing
            .layers
            .iter()
            .zip(&paths.layers)
            .map(|(layer, path)| IfcdrLayerBindingInput {
                id: LayerId::from(layer.local_id),
                ifcx_path: path,
            })
            .collect::<Vec<_>>();
        let appearances = drawing
            .appearances
            .iter()
            .zip(&paths.appearances)
            .map(|(appearance, path)| IfcdrAppearanceBindingInput {
                id: AppearanceId::from(appearance.local_id),
                ifcx_path: path,
            })
            .collect::<Vec<_>>();
        let entities = drawing
            .entities
            .iter()
            .map(|entity| match entity {
                PendingEntity::Line {
                    entity_id,
                    definition,
                } => IfcdrEntityInput::Line {
                    entity_id: *entity_id,
                    start: definition.start,
                    end: definition.end,
                    layer_id: LayerId::from(definition.layer.local_id),
                    appearance_id: appearance_id(definition.appearance),
                    visible: definition.visible,
                },
                PendingEntity::Polyline {
                    entity_id,
                    definition,
                } => IfcdrEntityInput::Polyline {
                    entity_id: *entity_id,
                    points: &definition.points,
                    closed: definition.closed,
                    layer_id: LayerId::from(definition.layer.local_id),
                    appearance_id: appearance_id(definition.appearance),
                    visible: definition.visible,
                },
            })
            .collect::<Vec<_>>();
        let input = IfcdrEncodeInput {
            resource_id: &drawing.options.representation_resource_id,
            unit: drawing.options.length_unit,
            scope: IfcdrScopeInput {
                id: ScopeId::new(0),
                kind: 0,
                name: "ModelSpace",
                base: Point2::new(0.0, 0.0),
                flags: 0,
            },
            layers: &layers,
            appearances: &appearances,
            entities: &entities,
        };
        let resource = encode(&input).map_err(map_ifcdr_encode_error)?;
        debug_assert!(resource.bounds.min().x().is_finite());
        debug_assert!(resource.bounds.min().y().is_finite());
        debug_assert!(resource.bounds.max().x().is_finite());
        debug_assert!(resource.bounds.max().y().is_finite());
        let entrypoint = assemble_ifcx(&self.options, &drawing, &paths, &resource)?;
        Ok(EncodedPackage::new([
            (
                crate::package::DIRECTORY_PACKAGE_ENTRYPOINT.to_owned(),
                entrypoint,
            ),
            (MODEL_SPACE_RESOURCE_URI.to_owned(), resource.bytes),
        ]))
    }
}

fn appearance_id(appearance: EntityAppearance) -> AppearanceId {
    AppearanceId::from(match appearance {
        EntityAppearance::ByLayer => 0,
        EntityAppearance::ByBlock => 1,
        EntityAppearance::Explicit(key) => key.local_id,
    })
}

fn map_ifcdr_encode_error(error: IfcdrEncodeError) -> PackageBuildError {
    match error {
        IfcdrEncodeError::RangeExhausted { kind } => PackageBuildError::RangeExhausted { kind },
        IfcdrEncodeError::InvalidInput { message }
        | IfcdrEncodeError::Serialization { message } => PackageBuildError::Encoding {
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
        ModelSpaceBuilder { state: self.state }
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
        self.state.appearances.push(AppearanceEntry {
            local_id,
            definition,
        });
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
        let normalized_name = definition.name.to_ascii_lowercase();
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
        let index = *self.state.layer_names.get(&name.to_ascii_lowercase())?;
        Some(LayerKey {
            builder_token: self.state.token,
            local_id: self.state.layers[index].local_id,
        })
    }
}

impl DrawingState {
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

    fn validate_entity_appearance(
        &self,
        appearance: EntityAppearance,
    ) -> Result<(), PackageBuildError> {
        match appearance {
            EntityAppearance::ByLayer | EntityAppearance::ByBlock => Ok(()),
            EntityAppearance::Explicit(key) => self.validate_appearance_key(key),
        }
    }

    fn next_entity_id(&self) -> Result<EntityId, PackageBuildError> {
        let count = u64::try_from(self.entities.len())
            .map_err(|_| PackageBuildError::RangeExhausted { kind: "entity" })?;
        let value = count
            .checked_add(1)
            .ok_or(PackageBuildError::RangeExhausted { kind: "entity" })?;
        EntityId::new(value).ok_or(PackageBuildError::RangeExhausted { kind: "entity" })
    }
}

pub struct ModelSpaceBuilder<'a> {
    state: &'a mut DrawingState,
}

impl ModelSpaceBuilder<'_> {
    pub fn add_line(&mut self, definition: LineDefinition) -> Result<EntityId, PackageBuildError> {
        self.state.validate_layer_key(definition.layer)?;
        self.state
            .validate_entity_appearance(definition.appearance)?;
        validate_points([definition.start, definition.end])?;
        let entity_id = self.state.next_entity_id()?;
        self.state.entities.push(PendingEntity::Line {
            entity_id,
            definition,
        });
        Ok(entity_id)
    }

    pub fn add_polyline(
        &mut self,
        definition: PolylineDefinition,
    ) -> Result<EntityId, PackageBuildError> {
        if definition.points.len() < 2 {
            return Err(PackageBuildError::PolylineTooShort);
        }
        self.state.validate_layer_key(definition.layer)?;
        self.state
            .validate_entity_appearance(definition.appearance)?;
        validate_points(definition.points.iter().copied())?;
        let entity_id = self.state.next_entity_id()?;
        self.state.entities.push(PendingEntity::Polyline {
            entity_id,
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
        .any(|point| !point.x().is_finite() || !point.y().is_finite())
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
    if !definition.opacity.is_finite() || !(0.0..=1.0).contains(&definition.opacity) {
        return Err(PackageBuildError::InvalidOpacity);
    }
    if !definition.line_weight.is_finite() || definition.line_weight < 0.0 {
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
