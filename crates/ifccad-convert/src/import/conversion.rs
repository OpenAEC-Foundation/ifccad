use super::appearance::{
    map_entity_opacity, map_explicit_color, map_layer_opacity, map_line_pattern, map_line_weight,
    MappedColor,
};
use super::diagnostic::DiagnosticAccumulator;
use super::units::apply_units;
use crate::{ImportEntityMapping, ImportError, ImportOutcome};
use cadcodec::entities::EntityCommon;
use cadcodec::{CadDocument, Color, EntityType, Layer, Line, LineType, LwPolyline, Vector2};
use ifccad::ifcdr::{AppearanceId, EntityId, IfcdrEntityRef, LayerId};
use ifccad::package::{
    AppearanceProperty, DrawingLayoutKind, DrawingRef, GeometryRepresentationRef, LayerRef,
};

/// Imports one validated IFCCAD drawing into a cadcodec [`CadDocument`].
///
/// `Import` is named from the `CadDocument` boundary: IFCCAD is the source and
/// the returned CAD document is the destination.
pub fn drawing_to_cad_document(drawing: DrawingRef<'_>) -> Result<ImportOutcome, ImportError> {
    let layouts = drawing.layouts().collect::<Vec<_>>();
    let model_layouts = layouts
        .iter()
        .filter(|layout| layout.kind() == DrawingLayoutKind::Model)
        .count();
    if layouts.len() != 1 || model_layouts != 1 {
        return Err(ImportError::UnsupportedDrawingStructure {
            total_layouts: layouts.len(),
            model_layouts,
        });
    }

    let layout = layouts[0];
    let representation = layout.representation();
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
                    .ok_or_else(|| ImportError::InternalInvariant {
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
                .map_err(|reason| ImportError::LayerInsertion {
                    layer: name,
                    reason,
                })?;
        }
    }

    let scope_id = layout.scope().id();
    let mut entity_mapping = ImportEntityMapping::default();
    for source in representation.resource().entities(scope_id) {
        match source {
            IfcdrEntityRef::Line(source) => {
                let start = source.start();
                let end = source.end();
                let mut target =
                    Line::from_coords(start.x(), start.y(), 0.0, end.x(), end.y(), 0.0);
                apply_entity_common(
                    &mut document,
                    representation,
                    &mut target.common,
                    source.entity_id(),
                    source.layer_id(),
                    source.appearance_id(),
                    source.visible(),
                    &mut diagnostics,
                )?;
                add_and_map(
                    &mut document,
                    &mut entity_mapping,
                    source.entity_id(),
                    EntityType::Line(target),
                )?;
            }
            IfcdrEntityRef::Polyline(source) => {
                let points = source
                    .points()
                    .map(|point| Vector2::new(point.x(), point.y()))
                    .collect();
                let mut target = LwPolyline::from_points(points);
                target.is_closed = source.closed();
                apply_entity_common(
                    &mut document,
                    representation,
                    &mut target.common,
                    source.entity_id(),
                    source.layer_id(),
                    source.appearance_id(),
                    source.visible(),
                    &mut diagnostics,
                )?;
                add_and_map(
                    &mut document,
                    &mut entity_mapping,
                    source.entity_id(),
                    EntityType::LwPolyline(target),
                )?;
            }
            IfcdrEntityRef::Unmodeled(source) => {
                diagnostics.record_unmodeled(source.schema_id());
            }
        }
    }

    Ok(ImportOutcome::new(
        document,
        diagnostics.finish(),
        entity_mapping,
    ))
}

#[allow(clippy::too_many_arguments)]
fn apply_entity_common(
    document: &mut CadDocument,
    representation: GeometryRepresentationRef<'_>,
    common: &mut EntityCommon,
    entity_id: EntityId,
    layer_id: LayerId,
    appearance_id: AppearanceId,
    visible: bool,
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<(), ImportError> {
    let layer = representation
        .layer(layer_id)
        .ok_or(ImportError::MissingEntityLayer {
            entity_id,
            layer_id,
        })?;
    let appearance =
        representation
            .appearance(appearance_id)
            .ok_or(ImportError::MissingEntityAppearance {
                entity_id,
                appearance_id,
            })?;

    common.layer = layer.name().to_owned();
    common.invisible = !visible;
    match appearance.color() {
        AppearanceProperty::ByLayer => {
            common.color = Color::ByLayer;
            common.color_name = None;
        }
        AppearanceProperty::ByBlock => {
            common.color = Color::ByBlock;
            common.color_name = None;
        }
        AppearanceProperty::Explicit(color) => {
            let mapped = map_explicit_color(color);
            common.color = mapped.color;
            common.color_name = mapped.name;
        }
    }
    common.linetype = map_line_pattern(appearance.line_pattern(), diagnostics);
    ensure_linetype(document, &common.linetype)?;
    common.line_weight = map_line_weight(appearance.line_weight(), diagnostics);
    common.transparency = map_entity_opacity(appearance.opacity());
    Ok(())
}

fn add_and_map(
    document: &mut CadDocument,
    mapping: &mut ImportEntityMapping,
    source_id: EntityId,
    target: EntityType,
) -> Result<(), ImportError> {
    let handle =
        document
            .add_entity(target)
            .map_err(|source| ImportError::CadcodecEntityInsertion {
                entity_id: source_id,
                source,
            })?;
    mapping.insert(source_id, handle);
    Ok(())
}

fn convert_layer(
    document: &mut CadDocument,
    source: LayerRef<'_>,
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<Layer, ImportError> {
    let mut target = Layer::new(source.name());
    target.flags.off = !source.visible();

    if let Some(appearance) = source.appearance() {
        let source_color = appearance.color();
        let MappedColor { color, name: _ } = map_explicit_color(source_color);
        target.color = color;
        if let Some(named) = source_color.named() {
            target.color_name = Some(named.name().to_owned());
            target.book_name = Some(named.catalog().to_owned());
        }
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

pub(crate) fn ensure_linetype(document: &mut CadDocument, name: &str) -> Result<(), ImportError> {
    if name == "Dashed" && !document.line_types.contains(name) {
        document
            .line_types
            .add(LineType::dashed())
            .map_err(|reason| ImportError::InternalInvariant {
                message: format!("could not insert standard Dashed linetype: {reason}"),
            })?;
    }
    Ok(())
}
