use super::*;
/// A validated CAD projection together with its immutable full source context.
#[derive(Clone, Debug)]
pub struct ValidatedIfccad {
    pub(crate) graph: LoadedIfccadGraph,
    pub(crate) document: IfccadDocument,
}

impl ValidatedIfccad {
    pub fn document(&self) -> &IfccadDocument {
        &self.document
    }
    /// Borrow the complete source snapshot, including non-CAD information.
    pub fn graph(&self) -> &LoadedIfccadGraph {
        &self.graph
    }
    /// Extract an editable CAD projection, discarding its source context.
    pub fn into_document(self) -> IfccadDocument {
        self.document
    }
    /// Extract the independent source snapshot and editable CAD projection.
    /// Changes to the document are not merged into the snapshot.
    pub fn into_parts(self) -> (LoadedIfccadGraph, IfccadDocument) {
        (self.graph, self.document)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfccadReadOptions {
    pub composition_policy: IfccadCompositionPolicy,
}
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0}")]
pub struct IfccadReadError(#[from] IfccadReport);
impl IfccadReadError {
    pub fn report(&self) -> &IfccadReport {
        &self.0
    }
}
pub fn load_ifccad_bytes(
    bytes: &[u8],
    options: IfccadReadOptions,
) -> Result<ValidatedIfccad, IfccadReadError> {
    let graph = LoadedIfccadGraph::load(bytes, options.composition_policy)?;
    let document = super::codec::json::project(graph.composed_ifcx())?;
    Ok(ValidatedIfccad { graph, document })
}
