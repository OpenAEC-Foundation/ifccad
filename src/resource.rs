use serde::Serialize;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ResourceId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("resource ID must not be empty")]
pub struct InvalidResourceId;

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidResourceId> {
        let value = value.into();
        if value.is_empty() {
            return Err(InvalidResourceId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_non_empty_identity_without_uri_rules() {
        let id = ResourceId::new("geometry:model/main").expect("resource ID");
        assert_eq!(id.as_str(), "geometry:model/main");
    }

    #[test]
    fn rejects_only_the_empty_identity() {
        assert_eq!(ResourceId::new(""), Err(InvalidResourceId));
        assert!(ResourceId::new("drawing.ifcdr.json").is_ok());
    }

    #[test]
    fn serializes_as_a_json_string() {
        let id = ResourceId::new("geometry-main").unwrap();
        assert_eq!(
            serde_json::to_value(id).unwrap(),
            serde_json::json!("geometry-main")
        );
    }
}
