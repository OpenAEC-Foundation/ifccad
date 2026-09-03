mod error;
mod types;

pub use error::BuildError;
pub use types::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, EntityAppearance, IndexedColor,
    LayerDefinition, LayerKey, LineDefinition, LinePatternDefinition, NamedColor, PackageOptions,
    PolylineDefinition,
};

use crate::package::canonical_rfc3339_utc;

#[derive(Debug)]
pub struct IfccadPackageBuilder {
    pub(crate) options: PackageOptions,
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
        Ok(Self { options })
    }
}
