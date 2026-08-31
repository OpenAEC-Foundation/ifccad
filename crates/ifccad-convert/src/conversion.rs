use crate::appearance::{map_explicit_color, map_layer_opacity, map_line_pattern, map_line_weight};
use crate::diagnostic::DiagnosticAccumulator;
use crate::units::apply_units;
use crate::{ConversionError, ConversionOutcome, EntityMapping};
use cadcodec::{CadDocument, Layer, LineType};
use ifccad::package::{AppearanceProperty, DrawingLayoutKind, DrawingRef, LayerRef};

pub fn convert_drawing(drawing: DrawingRef<'_>) -> Result<ConversionOutcome, ConversionError> {
    let layouts = drawing.layouts().collect::<Vec<_>>();
    let model_layouts = layouts
        .iter()
        .filter(|layout| layout.kind() == DrawingLayoutKind::Model)
        .count();
    if layouts.len() != 1 || model_layouts != 1 {
        return Err(ConversionError::UnsupportedDrawingStructure {
            total_layouts: layouts.len(),
            model_layouts,
        });
    }

    let representation = drawing.representation();
    let mut document = CadDocument::new();
    let mut diagnostics = DiagnosticAccumulator::default();
    apply_units(&mut document, representation.resource().unit());

    for source in representation.layers() {
        let target = convert_layer(&mut document, source, &mut diagnostics)?;
        if source.name() == "0" {
            let standard =
                document
                    .layers
                    .get_mut("0")
                    .ok_or_else(|| ConversionError::InternalInvariant {
                        message: "fresh CadDocument has no standard layer 0".to_owned(),
                    })?;
            standard.flags = target.flags;
            standard.color = target.color;
            standard.line_type = target.line_type;
            standard.line_weight = target.line_weight;
            standard.transparency = target.transparency;
        } else {
            let name = target.name.clone();
            document
                .layers
                .add(target)
                .map_err(|reason| ConversionError::LayerInsertion {
                    layer: name,
                    reason,
                })?;
        }
    }

    Ok(ConversionOutcome::new(
        document,
        diagnostics.finish(),
        EntityMapping::default(),
    ))
}

fn convert_layer(
    document: &mut CadDocument,
    source: LayerRef<'_>,
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<Layer, ConversionError> {
    let mut target = Layer::new(source.name());
    target.flags.off = !source.visible();

    if let Some(appearance) = source.appearance() {
        let mapped_color = map_explicit_color(appearance.color());
        target.color = mapped_color.color;
        if let Some(named) = appearance.color().named() {
            diagnostics.record_named_layer_color(source.name(), named.catalog(), named.name());
        }
        let _layer_color_name_is_intentionally_diagnostic_only = mapped_color.name;
        target.line_type = map_line_pattern(
            AppearanceProperty::Explicit(appearance.line_pattern()),
            diagnostics,
        );
        ensure_linetype(document, &target.line_type)?;
        target.line_weight = map_line_weight(
            AppearanceProperty::Explicit(appearance.line_weight()),
            diagnostics,
        );
        target.transparency = map_layer_opacity(appearance.opacity());
    }

    Ok(target)
}

pub(crate) fn ensure_linetype(
    document: &mut CadDocument,
    name: &str,
) -> Result<(), ConversionError> {
    if name == "Dashed" && !document.line_types.contains(name) {
        document
            .line_types
            .add(LineType::dashed())
            .map_err(|reason| ConversionError::InternalInvariant {
                message: format!("could not insert standard Dashed linetype: {reason}"),
            })?;
    }
    Ok(())
}
