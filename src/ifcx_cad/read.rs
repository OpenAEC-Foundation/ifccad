use super::*;
/// A validated CAD projection together with its immutable full source context.
#[derive(Clone, Debug)]
pub struct ValidatedIfcxCad {
    pub(crate) graph: LoadedIfcxGraph,
    pub(crate) document: IfcxCadDocument,
}

impl ValidatedIfcxCad {
    pub fn document(&self) -> &IfcxCadDocument {
        &self.document
    }
    /// Borrow the complete source snapshot, including non-CAD information.
    pub fn graph(&self) -> &LoadedIfcxGraph {
        &self.graph
    }
    /// Extract an editable CAD projection, discarding its source context.
    pub fn into_document(self) -> IfcxCadDocument {
        self.document
    }
    /// Extract the independent source snapshot and editable CAD projection.
    /// Changes to the document are not merged into the snapshot.
    pub fn into_parts(self) -> (LoadedIfcxGraph, IfcxCadDocument) {
        (self.graph, self.document)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfcxCadReadOptions {
    pub composition_policy: IfcxCompositionPolicy,
}
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0}")]
pub struct IfcxCadReadError(#[from] IfcxCadReport);
impl IfcxCadReadError {
    pub fn report(&self) -> &IfcxCadReport {
        &self.0
    }
}
pub fn load_ifcx_cad_bytes(
    bytes: &[u8],
    options: IfcxCadReadOptions,
) -> Result<ValidatedIfcxCad, IfcxCadReadError> {
    let graph = LoadedIfcxGraph::load(bytes, options.composition_policy)?;
    let document = super::codec::ifcx_json::project(graph.composed_ifcx())?;
    Ok(ValidatedIfcxCad { graph, document })
}
