use super::{ExportError, ExportOptions, ExportOutcome, SourceStructureProblem};
use cadcodec::{CadDocument, Handle};
use ifccad::package::{PackageBuilder, PackageOptions};

pub fn cad_document_to_package(
    document: &CadDocument,
    package_options: PackageOptions,
    _export_options: ExportOptions,
) -> Result<ExportOutcome, ExportError> {
    let _builder = PackageBuilder::new(package_options)?;
    if document.header.model_space_block_handle == Handle::NULL {
        return Err(ExportError::InvalidSourceStructure {
            problems: vec![SourceStructureProblem::ModelSpaceBlockMissing],
        });
    }
    Err(ExportError::InternalInvariant {
        message: "CadDocument export is not implemented yet".to_owned(),
    })
}
