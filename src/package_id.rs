use serde::Serialize;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct PackageId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("package ID must not be empty")]
pub struct InvalidPackageId;

impl PackageId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidPackageId> {
        let value = value.into();
        if value.is_empty() {
            return Err(InvalidPackageId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_only_the_empty_identity() {
        assert_eq!(PackageId::new(""), Err(InvalidPackageId));
        assert_eq!(PackageId::new(" ").unwrap().as_str(), " ");
        assert_eq!(
            PackageId::new("package:main").unwrap().as_str(),
            "package:main"
        );
    }

    #[test]
    fn serializes_as_a_json_string() {
        let id = PackageId::new("building-main").unwrap();
        assert_eq!(
            serde_json::to_value(id).unwrap(),
            serde_json::json!("building-main")
        );
    }
}
