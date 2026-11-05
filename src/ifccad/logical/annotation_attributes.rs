use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum IfccadAttributeAnnotationContext {
    Text {
        text: IfccadTextAnnotationContext,
    },
    MText {
        text: IfccadTextAnnotationContext,
        #[serde(rename = "mTextEnabled")]
        m_text_enabled: bool,
        embedded: Option<IfccadAttributeEmbeddedContext>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadAttributeEmbeddedContext {
    pub scale_id: IfccadAnnotationScaleId,
    pub is_default: bool,
    pub presentation: IfccadMTextAnnotationContext,
}

impl IfccadAttributeAnnotationContext {
    pub(super) fn validate(
        &self,
        d: &IfccadDocument,
        multiline: bool,
        path: &str,
    ) -> Result<(), IfccadReport> {
        match self {
            Self::Text { text } if !multiline => text.validate(),
            Self::MText { text, embedded, .. } if multiline => {
                text.validate()?;
                if let Some(e) = embedded {
                    if !d.annotation_scales.iter().any(|s| s.id == e.scale_id) {
                        return Err(super::attributes::fail(
                            path,
                            "unresolved embedded attribute scale",
                        ));
                    }
                    e.presentation.validate()?;
                }
                Ok(())
            }
            _ => Err(super::attributes::fail(
                path,
                "attribute context kind does not match text kind",
            )),
        }
    }
}
