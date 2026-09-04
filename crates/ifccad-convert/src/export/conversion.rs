use super::appearance::AppearanceRegistry;
use super::layers::add_layers;
use super::structure::inspect_model_space;
use super::units::map_length_unit;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportEntityMapping, ExportError,
    ExportOptions, ExportOutcome,
};
use cadcodec::CadDocument;
use ifccad::package::{DrawingOptions, LayerKey, PackageBuilder, PackageOptions};
use ifccad::ResourceId;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct ExportContext {
    pub(crate) diagnostics: Vec<ExportDiagnostic>,
    pub(crate) layer_keys: BTreeMap<String, LayerKey>,
    pub(crate) appearances: AppearanceRegistry,
}

pub fn cad_document_to_package(
    document: &CadDocument,
    package_options: PackageOptions,
    _export_options: ExportOptions,
) -> Result<ExportOutcome, ExportError> {
    let mut builder = PackageBuilder::new(package_options)?;
    let model_space = inspect_model_space(document)
        .map_err(|problems| ExportError::InvalidSourceStructure { problems })?;
    debug_assert_eq!(
        model_space.block_handle,
        document.header.model_space_block_handle
    );

    let (length_unit, unit_loss) = map_length_unit(document.header.insertion_units);
    let mut context = ExportContext::default();
    if let Some(reason) = unit_loss {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.insertion_units".to_owned(),
            },
            ExportAction::PartiallyExported,
            vec![reason],
        ));
    }

    {
        let mut drawing = builder.add_drawing(DrawingOptions {
            model_layout_name: model_space.layout_name.to_owned(),
            representation_resource_id: ResourceId::new("drawing-main")
                .expect("constant resource ID is non-empty"),
            length_unit,
        })?;
        add_layers(document, &mut drawing, &mut context)?;
    }

    let package = builder.finish()?;
    Ok(ExportOutcome::new(
        package,
        context.diagnostics,
        ExportEntityMapping::default(),
    ))
}
