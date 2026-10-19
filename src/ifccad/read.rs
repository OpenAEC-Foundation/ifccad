use super::*;
mod cbor;
pub use cbor::*;
#[cfg(feature = "ifccad-cbor-zstd")]
mod cbor_zstd;
#[cfg(feature = "ifccad-cbor-zstd")]
pub use cbor_zstd::*;
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

/// Read the explicitly selected xformop placement experiment.
///
/// Supported matrices replace geometry placement only. They must map exactly
/// to the native origin/X/Y frame, including its derived Z direction. The
/// immutable source graph retains the matrix, while the editable document and
/// ordinary encoder use native placements. This is not general USD scene loading.
///
/// ```
/// use ifccad::{load_ifccad_xformop_bytes, encode_ifccad_document};
/// # fn project(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
/// let source = load_ifccad_xformop_bytes(bytes, Default::default())?;
/// let native = encode_ifccad_document(source.document())?;
/// // Original source bytes stay available independently of native re-encoding.
/// let original = source.graph().source_bytes();
/// # let _ = (native, original);
/// # Ok(())
/// # }
/// ```
pub fn load_ifccad_xformop_bytes(
    bytes: &[u8],
    options: IfccadReadOptions,
) -> Result<ValidatedIfccad, IfccadReadError> {
    let graph = LoadedIfccadGraph::load(bytes, options.composition_policy)?;
    let document = super::codec::json::project_xformop(graph.composed_ifcx())?;
    Ok(ValidatedIfccad { graph, document })
}
