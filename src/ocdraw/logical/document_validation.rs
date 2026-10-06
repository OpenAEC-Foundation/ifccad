use super::*;
use crate::ocdraw::OcdrawDiagnostic;

/// Structured failures for invalid authored or decoded drawing content.
#[derive(Clone, Debug, thiserror::Error)]
#[error("drawing content is invalid")]
pub struct OcdrawValidationError {
    diagnostics: Vec<OcdrawDiagnostic>,
}
impl OcdrawValidationError {
    pub fn diagnostics(&self) -> &[OcdrawDiagnostic] {
        &self.diagnostics
    }
    pub fn into_diagnostics(self) -> Vec<OcdrawDiagnostic> {
        self.diagnostics
    }
    pub(crate) fn from_logical_errors(errors: Vec<LogicalError>) -> Self {
        Self {
            diagnostics: errors
                .into_iter()
                .map(|e| OcdrawDiagnostic {
                    code: e.code,
                    location: e.location,
                    message: e.message,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValidationPhase {
    BeforeBounds,
    Complete,
}

/// Validates current content without encoding, repairing or mutating it.
pub fn validate_ocdraw_document(doc: &OcdrawDocument) -> Result<(), OcdrawValidationError> {
    let errors = validate_logical_document(doc, ValidationPhase::Complete);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(OcdrawValidationError::from_logical_errors(errors))
    }
}

pub(crate) fn validate_logical_document(
    doc: &OcdrawDocument,
    phase: ValidationPhase,
) -> Vec<LogicalError> {
    let mut errors = super::field_validation::validate_fields(doc);
    errors.extend(super::preservation_validation::validate_preservation(doc));
    errors.extend(super::document_projection::project_validation_model(doc, phase).validate());
    // Index-based evaluation is safe only after unique identities and ownership.
    if !errors.is_empty() {
        return errors;
    }
    errors.extend(validate_state(doc));
    if phase == ValidationPhase::Complete {
        errors.extend(validate_geometry_bounds(
            &doc.geometric_entities,
            &doc.scopes,
        ));
        errors.extend(validate_blocks(
            &doc.geometric_entities,
            &doc.block_definitions,
            &doc.scopes,
        ));
        errors.extend(validate_viewport_bounds(&doc.viewports, &doc.scopes));
    }
    errors
}
