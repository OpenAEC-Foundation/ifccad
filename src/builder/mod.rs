mod error;
mod state;
mod types;

pub use error::BuildError;
pub use types::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, EntityAppearance, IndexedColor,
    LayerDefinition, LayerKey, LineDefinition, LinePatternDefinition, NamedColor, PackageOptions,
    PolylineDefinition,
};

use crate::package::canonical_rfc3339_utc;
use state::{AppearanceEntry, BuilderState, LayerEntry};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_BUILDER_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct IfccadPackageBuilder {
    pub(crate) options: PackageOptions,
    pub(crate) token: u64,
    pub(crate) state: BuilderState,
}

impl IfccadPackageBuilder {
    pub fn new(mut options: PackageOptions) -> Result<Self, BuildError> {
        for (field, value) in [
            ("data_version", options.data_version.as_str()),
            ("author", options.author.as_str()),
            ("model_layout_name", options.model_layout_name.as_str()),
        ] {
            if value.is_empty() {
                return Err(BuildError::EmptyValue { field });
            }
        }

        options.timestamp =
            canonical_rfc3339_utc(&options.timestamp).ok_or(BuildError::InvalidTimestamp)?;
        let token = NEXT_BUILDER_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| BuildError::RangeExhausted {
                kind: "builder token",
            })?;

        Ok(Self {
            options,
            token,
            state: BuilderState::default(),
        })
    }

    pub fn appearances(&mut self) -> Appearances<'_> {
        Appearances { builder: self }
    }

    pub fn layers(&mut self) -> Layers<'_> {
        Layers { builder: self }
    }
}

pub struct Appearances<'a> {
    builder: &'a mut IfccadPackageBuilder,
}

impl Appearances<'_> {
    pub fn add(&mut self, definition: AppearanceDefinition) -> Result<AppearanceKey, BuildError> {
        validate_appearance(&definition)?;
        let offset = u32::try_from(self.builder.state.appearances.len())
            .map_err(|_| BuildError::RangeExhausted { kind: "appearance" })?;
        let local_id = 2_u32
            .checked_add(offset)
            .ok_or(BuildError::RangeExhausted { kind: "appearance" })?;
        self.builder.state.appearances.push(AppearanceEntry {
            local_id,
            definition,
        });
        Ok(AppearanceKey {
            builder_token: self.builder.token,
            local_id,
        })
    }
}

pub struct Layers<'a> {
    builder: &'a mut IfccadPackageBuilder,
}

impl Layers<'_> {
    pub fn add(&mut self, definition: LayerDefinition) -> Result<LayerKey, BuildError> {
        if definition.name.is_empty() {
            return Err(BuildError::EmptyValue {
                field: "layer_name",
            });
        }
        self.builder
            .validate_appearance_key(definition.appearance)?;
        let normalized_name = definition.name.to_ascii_lowercase();
        if self
            .builder
            .state
            .layer_names
            .contains_key(&normalized_name)
        {
            return Err(BuildError::DuplicateLayerName {
                name: definition.name,
            });
        }
        let local_id = u32::try_from(self.builder.state.layers.len())
            .map_err(|_| BuildError::RangeExhausted { kind: "layer" })?;
        let index = self.builder.state.layers.len();
        self.builder.state.layers.push(LayerEntry {
            local_id,
            definition,
        });
        self.builder
            .state
            .layer_names
            .insert(normalized_name, index);
        Ok(LayerKey {
            builder_token: self.builder.token,
            local_id,
        })
    }

    pub fn by_name(&self, name: &str) -> Option<LayerKey> {
        let index = *self
            .builder
            .state
            .layer_names
            .get(&name.to_ascii_lowercase())?;
        Some(LayerKey {
            builder_token: self.builder.token,
            local_id: self.builder.state.layers[index].local_id,
        })
    }
}

impl IfccadPackageBuilder {
    fn validate_appearance_key(&self, key: AppearanceKey) -> Result<(), BuildError> {
        let index = key
            .local_id
            .checked_sub(2)
            .and_then(|value| usize::try_from(value).ok());
        if key.builder_token != self.token
            || index.is_none_or(|index| index >= self.state.appearances.len())
        {
            return Err(BuildError::ForeignAppearanceKey);
        }
        Ok(())
    }
}

fn validate_appearance(definition: &AppearanceDefinition) -> Result<(), BuildError> {
    for (field, value) in [
        ("appearance_name", definition.name.as_str()),
        ("line_pattern_name", definition.line_pattern.name.as_str()),
    ] {
        if value.is_empty() {
            return Err(BuildError::EmptyValue { field });
        }
    }
    if !definition.opacity.is_finite() || !(0.0..=1.0).contains(&definition.opacity) {
        return Err(BuildError::InvalidOpacity);
    }
    if !definition.line_weight.is_finite() || definition.line_weight < 0.0 {
        return Err(BuildError::InvalidLineWeight);
    }
    if definition
        .color
        .indexed
        .as_ref()
        .is_some_and(|color| color.system.is_empty())
    {
        return Err(BuildError::EmptyValue {
            field: "indexed_color_system",
        });
    }
    if let Some(color) = &definition.color.named {
        if color.catalog.is_empty() {
            return Err(BuildError::EmptyValue {
                field: "named_color_catalog",
            });
        }
        if color.name.is_empty() {
            return Err(BuildError::EmptyValue {
                field: "named_color_name",
            });
        }
    }
    Ok(())
}
