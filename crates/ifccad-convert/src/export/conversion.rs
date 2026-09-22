use super::appearance::AppearanceRegistry;
use super::coverage::scan_document_semantics;
use super::entities::add_entities;
use super::layers::add_layers;
use super::structure::inspect_model_space;
use super::units::map_length_unit;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportEntityMapping, ExportError,
    ExportLossPolicy, ExportOptions, ExportOutcome,
};
use cadcodec::CadDocument;
use ifccad::package::{DrawingOptions, LayerKey, PackageBuilder, PackageOptions};
use ifccad::ResourceId;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct ExportContext {
    pub(crate) block_instances: BTreeMap<cadcodec::Handle, super::blocks::ConvertedInstance>,
    pub(crate) block_points: BTreeMap<cadcodec::Handle, Vec<crate::geometry::blocks::PairedPoint>>,
    pub(crate) blocks: BTreeMap<cadcodec::Handle, ifccad::package::BlockDefinitionKey>,
    pub(crate) geometry: Option<crate::ConversionGeometryAssessment>,
    pub(crate) diagnostics: Vec<ExportDiagnostic>,
    pub(crate) layer_keys: BTreeMap<String, LayerKey>,
    pub(crate) appearances: AppearanceRegistry,
    pub(crate) entity_mapping: ExportEntityMapping,
}

pub fn cad_document_to_package(
    document: &CadDocument,
    package_options: PackageOptions,
    export_options: ExportOptions,
) -> Result<ExportOutcome, ExportError> {
    let mut builder = PackageBuilder::new(package_options)?;
    let model_space = inspect_model_space(document)
        .map_err(|problems| ExportError::InvalidSourceStructure { problems })?;
    let mut problems = super::blocks::inspect_markers(document);
    problems.extend(super::blocks::inspect_references(document));
    if !problems.is_empty() {
        return Err(ExportError::InvalidSourceStructure { problems });
    }
    debug_assert_eq!(
        model_space.block_handle,
        document.header.model_space_block_handle
    );

    let (length_unit, unit_loss) = map_length_unit(document.header.insertion_units);
    let mut context = ExportContext {
        geometry: Some(crate::ConversionGeometryAssessment::new(
            export_options.geometry_tolerance,
            length_unit,
        )?),
        ..Default::default()
    };
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
        super::blocks::add_definitions(document, &mut drawing, &mut context)?;
        let structural_problems = add_entities(document, &model_space, &mut drawing, &mut context)?;
        if !structural_problems.is_empty() {
            return Err(ExportError::InvalidSourceStructure {
                problems: structural_problems,
            });
        }
        super::blocks::assess_occurrences(document, &mut context)?;
        super::blocks::propagate_losses(document, &mut context);
    }

    scan_document_semantics(document, &mut context);

    if export_options.loss_policy == ExportLossPolicy::Reject
        && context
            .diagnostics
            .iter()
            .any(ExportDiagnostic::blocks_reject)
    {
        return Err(ExportError::LossRejected {
            diagnostics: context.diagnostics,
        });
    }

    let package = builder.finish()?;
    Ok(
        ExportOutcome::new(package, context.diagnostics, context.entity_mapping)
            .with_geometry_assessment(context.geometry.unwrap()),
    )
}
