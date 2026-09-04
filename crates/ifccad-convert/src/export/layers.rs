use super::appearance::LayerAppearanceError;
use super::conversion::ExportContext;
use super::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::{CadDocument, Handle, Layer};
use ifccad::package::{DrawingBuilder, LayerDefinition, PackageBuildError};

pub(crate) fn add_layers(
    document: &CadDocument,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<(), PackageBuildError> {
    for source in document.layers.iter() {
        let (appearance, mut reasons) =
            match context.appearances.add_layer_appearance(drawing, source) {
                Ok(converted) => converted,
                Err(LayerAppearanceError::Loss(reasons)) => {
                    context.diagnostics.push(ExportDiagnostic::loss(
                        ExportDiagnosticSource::Layer {
                            name: source.name.clone(),
                        },
                        ExportAction::Skipped,
                        reasons,
                    ));
                    continue;
                }
                Err(LayerAppearanceError::Build(error)) => return Err(error),
            };

        append_auxiliary_losses(source, &mut reasons);
        let key = drawing.layers().add(LayerDefinition {
            name: source.name.clone(),
            visible: !source.flags.off && !source.flags.frozen,
            appearance,
        })?;
        context.layer_keys.insert(source.name.to_lowercase(), key);
        if !reasons.is_empty() {
            context.diagnostics.push(ExportDiagnostic::loss(
                ExportDiagnosticSource::Layer {
                    name: source.name.clone(),
                },
                ExportAction::PartiallyExported,
                reasons,
            ));
        }
    }
    Ok(())
}

fn append_auxiliary_losses(layer: &Layer, reasons: &mut Vec<ExportLossReason>) {
    if layer.flags.locked {
        reasons.push(ExportLossReason::LayerLocked);
    }
    if layer.flags.frozen_in_new_viewport {
        reasons.push(ExportLossReason::LayerFrozenInNewViewport);
    }
    if layer.flags.xref_dependent {
        reasons.push(ExportLossReason::LayerXrefDependent);
    }
    if !layer.is_plottable {
        reasons.push(ExportLossReason::LayerNotPlottable);
    }
    if !layer.plot_style.is_empty() {
        reasons.push(ExportLossReason::LayerPlotStyle {
            name: layer.plot_style.clone(),
        });
    }
    if layer.material != Handle::NULL {
        reasons.push(ExportLossReason::MaterialReference {
            handle: layer.material,
        });
    }
    if layer.plotstyle_handle != Handle::NULL {
        reasons.push(ExportLossReason::PlotStyleReference {
            handle: layer.plotstyle_handle,
        });
    }
    if layer.xref_block_record_handle != Handle::NULL {
        reasons.push(ExportLossReason::XrefBlockRecordReference {
            handle: layer.xref_block_record_handle,
        });
    }
}
