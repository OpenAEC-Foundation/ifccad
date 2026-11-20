//! Drawing-local external identities. Validation never resolves a source.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadExternalSourceId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum IfccadExternalSourceLocation {
    UriReference {
        uri: String,
    },
    SearchFileName {
        #[serde(rename = "fileName")]
        file_name: String,
    },
}

impl IfccadExternalSourceLocation {
    /// Check intrinsic syntax without filesystem or network access.
    pub fn validate(&self) -> Result<(), IfccadReport> {
        let valid = match self {
            Self::UriReference { uri } => {
                !uri.is_empty() && uriparse::URIReference::try_from(uri.as_str()).is_ok()
            }
            Self::SearchFileName { file_name } => {
                !file_name.is_empty()
                    && file_name != "."
                    && file_name != ".."
                    && !file_name
                        .chars()
                        .any(|c| matches!(c, '/' | '\\' | ':' | '\0'))
            }
        };
        if valid {
            Ok(())
        } else {
            Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-EXTERNAL-001",
                "/location",
                "invalid external source location",
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadExternalSource {
    pub id: IfccadExternalSourceId,
    pub name: String,
    pub location: IfccadExternalSourceLocation,
    pub source: IfccadExternalSourceContent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "camelCase", deny_unknown_fields)]
pub enum IfccadExternalSourceContent {
    Dwg {
        #[serde(rename = "basePoint")]
        base_point: [f64; 3],
        #[serde(
            rename = "insertionUnit",
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "crate::dimension::present_option"
        )]
        insertion_unit: Option<String>,
        #[serde(
            rename = "fingerprintGuid",
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "crate::dimension::present_option"
        )]
        fingerprint_guid: Option<String>,
    },
    Dxf {
        #[serde(rename = "basePoint")]
        base_point: [f64; 3],
        #[serde(
            rename = "insertionUnit",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        insertion_unit: Option<String>,
        #[serde(
            rename = "fingerprintGuid",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        fingerprint_guid: Option<String>,
    },
    Ifccad {
        #[serde(rename = "basePoint")]
        base_point: [f64; 3],
        #[serde(
            rename = "insertionUnit",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        insertion_unit: Option<String>,
    },
}

/// Declared source format, independent of the locator's filename extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadExternalSourceFormat {
    Dwg,
    Dxf,
    Ifccad,
}

impl IfccadExternalSourceContent {
    pub fn format(&self) -> IfccadExternalSourceFormat {
        match self {
            Self::Dwg { .. } => IfccadExternalSourceFormat::Dwg,
            Self::Dxf { .. } => IfccadExternalSourceFormat::Dxf,
            Self::Ifccad { .. } => IfccadExternalSourceFormat::Ifccad,
        }
    }

    pub fn base_point(&self) -> [f64; 3] {
        match self {
            Self::Dwg { base_point, .. }
            | Self::Dxf { base_point, .. }
            | Self::Ifccad { base_point, .. } => *base_point,
        }
    }

    pub fn insertion_unit(&self) -> Option<&str> {
        match self {
            Self::Dwg { insertion_unit, .. }
            | Self::Dxf { insertion_unit, .. }
            | Self::Ifccad { insertion_unit, .. } => insertion_unit.as_deref(),
        }
    }

    pub fn fingerprint_guid(&self) -> Option<&str> {
        match self {
            Self::Dwg {
                fingerprint_guid, ..
            }
            | Self::Dxf {
                fingerprint_guid, ..
            } => fingerprint_guid.as_deref(),
            Self::Ifccad { .. } => None,
        }
    }
}

fn canonical_guid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}

/// Normalize a valid DWG UUID spelling for native storage; absent/invalid text
/// has no inferred identity. This does not generate a new source identity.
pub fn normalize_ifccad_fingerprint_guid(value: &str) -> Option<String> {
    let value = value
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .unwrap_or(value);
    let value = value.to_ascii_lowercase();
    canonical_guid(&value).then_some(value)
}

impl IfccadExternalSource {
    /// Check one source intrinsically without resolving external bytes.
    pub fn validate(&self) -> Result<(), IfccadReport> {
        self.location.validate()?;
        if self.name.is_empty()
            || self.name.contains('\0')
            || !self.source.base_point().iter().all(|v| v.is_finite())
            || self
                .source
                .insertion_unit()
                .is_some_and(|v| !super::document_validation::unit(v))
            || self
                .source
                .fingerprint_guid()
                .is_some_and(|v| !canonical_guid(v))
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-EXTERNAL-001",
                "/externalSource",
                "invalid external source",
            ));
        }
        Ok(())
    }
}

pub(super) fn validate(document: &IfccadDocument) -> Result<(), IfccadReport> {
    let mut ids = BTreeSet::new();
    for source in &document.external_sources {
        let path = format!(
            "/cad/d{}/externalSource/{}",
            document.drawing_id, source.id.0
        );
        source
            .validate()
            .map_err(|r| crate::ifccad::diagnostics::context(r, "IFCCAD-EXTERNAL-001", &path))?;
        if !ids.insert(source.id) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-EXTERNAL-001",
                &path,
                "duplicate external source identity",
            ));
        }
    }
    Ok(())
}
