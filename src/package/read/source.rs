use super::PackageDiagnostic;

/// Physical source identity, distinct from logical ResourceId.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ResourceSourceKey {
    External(String),
    Inline {
        document_uri: String,
        pointer: String,
    },
}

impl ResourceSourceKey {
    pub(crate) fn origin(&self) -> ResourceOrigin {
        match self {
            Self::External(uri) => ResourceOrigin {
                document_uri: uri.clone(),
                pointer: String::new(),
            },
            Self::Inline {
                document_uri,
                pointer,
            } => ResourceOrigin {
                document_uri: document_uri.clone(),
                pointer: pointer.clone(),
            },
        }
    }
    pub(crate) fn external_uri(&self) -> Option<&str> {
        match self {
            Self::External(uri) => Some(uri),
            Self::Inline { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceOrigin {
    pub(crate) document_uri: String,
    pub(crate) pointer: String,
}
impl ResourceOrigin {
    pub(crate) fn locate(&self, local: Option<&str>) -> String {
        format!("{}{}", self.pointer, local.unwrap_or(""))
    }
    pub(crate) fn apply(&self, diagnostic: &mut PackageDiagnostic) {
        diagnostic.resource_uri = Some(self.document_uri.clone());
        if !self.pointer.is_empty() {
            diagnostic.location = Some(self.locate(diagnostic.location.as_deref()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origin_preserves_external_and_prefixes_inline_locations() {
        let external = ResourceSourceKey::External("drawing.json".into()).origin();
        assert_eq!(
            external.locate(Some("/header/resourceId")),
            "/header/resourceId"
        );
        let inline = ResourceSourceKey::Inline {
            document_uri: "package.ifcx.json".into(),
            pointer: "/data/3/attributes/resource/content".into(),
        }
        .origin();
        assert_eq!(inline.locate(None), "/data/3/attributes/resource/content");
        assert_eq!(
            inline.locate(Some("/header/resourceId")),
            "/data/3/attributes/resource/content/header/resourceId"
        );
    }
}
