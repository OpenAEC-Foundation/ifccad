use super::*;

/// Placement maps source coordinates relative to its base point, scaled once.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadXref {
    pub source_id: IfccadExternalSourceId,
    pub placement: IfccadPlacement,
    pub scale: f64,
}

impl IfccadXref {
    pub(super) fn validate_at(
        &self,
        document: &IfccadDocument,
        path: &str,
    ) -> Result<(), IfccadReport> {
        self.placement.coordinate_frame()?;
        if !self.scale.is_finite()
            || self.scale <= 0.
            || !document
                .external_sources
                .iter()
                .any(|s| s.id == self.source_id)
            || self.placement.x_axis[2].abs() > 1e-12
            || self.placement.y_axis[2].abs() > 1e-12
            || self.placement.x_axis[0] * self.placement.y_axis[1]
                - self.placement.x_axis[1] * self.placement.y_axis[0]
                <= 0.
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-XREF-001",
                path,
                "XREF requires a local source, positive finite scale and rotation about Z",
            ));
        }
        Ok(())
    }
}

pub(super) fn validate_owners(document: &IfccadDocument) -> Result<(), IfccadReport> {
    for entity in document
        .paper_layouts
        .iter()
        .flat_map(|p| &p.entities)
        .chain(document.blocks.iter().flat_map(|b| &b.entities))
    {
        if entity
            .as_native()
            .is_some_and(|e| matches!(e.kind, IfccadEntityKind::Xref(_)))
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-XREF-002",
                &format!("/cad/d{}/e{}", document.drawing_id, entity.id()),
                "first-slice XREF must belong to Model",
            ));
        }
    }
    Ok(())
}
