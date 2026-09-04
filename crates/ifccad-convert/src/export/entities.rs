use super::appearance::EntityAppearanceError;
use super::conversion::ExportContext;
use super::structure::ModelSpaceInfo;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason,
    SourceStructureProblem,
};
use cadcodec::entities::EntityCommon;
use cadcodec::{CadDocument, EntityType, Handle, Line, LwPolyline, Vector3};
use ifccad::ifcdr::Point2;
use ifccad::package::{DrawingBuilder, LineDefinition, PackageBuildError, PolylineDefinition};

pub(crate) fn add_entities(
    document: &CadDocument,
    model_space: &ModelSpaceInfo<'_>,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<Vec<SourceStructureProblem>, PackageBuildError> {
    let mut structural_problems = Vec::new();
    for source in document.entities() {
        let common = source.common();
        let common_losses = common_semantic_losses(common);
        if !classify_owner(
            document,
            model_space,
            source,
            common,
            context,
            &mut structural_problems,
            &common_losses,
        ) {
            continue;
        }

        let mut geometry_losses = match source {
            EntityType::Line(line) => line_losses(line),
            EntityType::LwPolyline(polyline) => polyline_losses(polyline),
            _ => vec![ExportLossReason::UnsupportedEntityType {
                kind: source.as_entity().entity_type().to_owned(),
            }],
        };
        if !geometry_losses.is_empty() {
            geometry_losses.extend(common_losses);
            record_skipped(source, geometry_losses, context);
            continue;
        }

        let Some(&layer) = context.layer_keys.get(&common.layer.to_lowercase()) else {
            let mut reasons = vec![ExportLossReason::MissingEntityLayer {
                name: common.layer.clone(),
            }];
            reasons.extend(common_losses);
            record_skipped(source, reasons, context);
            continue;
        };
        let appearance = match context.appearances.add_entity_appearance(drawing, common) {
            Ok(appearance) => appearance,
            Err(EntityAppearanceError::Loss(reasons)) => {
                let mut reasons = reasons;
                reasons.extend(common_losses);
                record_skipped(source, reasons, context);
                continue;
            }
            Err(EntityAppearanceError::Build(error)) => return Err(error),
        };

        let entity_id = match source {
            EntityType::Line(line) => drawing.model_space().add_line(LineDefinition {
                start: Point2::new(line.start.x, line.start.y),
                end: Point2::new(line.end.x, line.end.y),
                layer,
                appearance,
                visible: !common.invisible,
            })?,
            EntityType::LwPolyline(polyline) => {
                drawing.model_space().add_polyline(PolylineDefinition {
                    points: polyline
                        .vertices
                        .iter()
                        .map(|vertex| Point2::new(vertex.location.x, vertex.location.y))
                        .collect(),
                    closed: polyline.is_closed,
                    layer,
                    appearance,
                    visible: !common.invisible,
                })?
            }
            _ => unreachable!("unsupported entity was classified as loss"),
        };
        context.entity_mapping.insert(common.handle, entity_id);
        if !common_losses.is_empty() {
            record_diagnostic(
                source,
                ExportAction::PartiallyExported,
                common_losses,
                context,
            );
        }
    }
    Ok(structural_problems)
}

fn classify_owner(
    document: &CadDocument,
    model_space: &ModelSpaceInfo<'_>,
    source: &EntityType,
    common: &EntityCommon,
    context: &mut ExportContext,
    problems: &mut Vec<SourceStructureProblem>,
    common_losses: &[ExportLossReason],
) -> bool {
    if common.owner_handle == model_space.block_handle {
        return true;
    }
    if common.owner_handle == Handle::NULL {
        problems.push(SourceStructureProblem::EntityOwnerMissing {
            entity: common.handle,
        });
    } else if common.owner_handle == document.header.paper_space_block_handle {
        let mut reasons = vec![ExportLossReason::PaperSpaceEntity];
        reasons.extend_from_slice(common_losses);
        record_skipped(source, reasons, context);
    } else if document
        .block_records
        .iter()
        .any(|record| record.handle == common.owner_handle)
    {
        let mut reasons = vec![ExportLossReason::BlockOwnedEntity {
            owner: common.owner_handle,
        }];
        reasons.extend_from_slice(common_losses);
        record_skipped(source, reasons, context);
    } else {
        problems.push(SourceStructureProblem::EntityOwnerUnknown {
            entity: common.handle,
            owner: common.owner_handle,
        });
    }
    false
}

fn record_skipped(
    source: &EntityType,
    reasons: Vec<ExportLossReason>,
    context: &mut ExportContext,
) {
    record_diagnostic(source, ExportAction::Skipped, reasons, context);
}

fn record_diagnostic(
    source: &EntityType,
    action: ExportAction,
    reasons: Vec<ExportLossReason>,
    context: &mut ExportContext,
) {
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Entity {
            handle: source.common().handle,
            kind: source.as_entity().entity_type().to_owned(),
        },
        action,
        reasons,
    ));
}

fn common_semantic_losses(common: &EntityCommon) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    if common.linetype_scale != 1.0 {
        reasons.push(ExportLossReason::EntityLinetypeScale);
    }
    if common.linetype_handle.is_some() {
        reasons.push(ExportLossReason::EntityLinetypeHandle);
    }
    if !common.extended_data.is_empty() {
        reasons.push(ExportLossReason::EntityExtendedData);
    }
    if common.graphic_data.is_some() {
        reasons.push(ExportLossReason::EntityGraphicData);
    }
    if !common.reactors.is_empty() {
        reasons.push(ExportLossReason::EntityReactors);
    }
    if common.xdictionary_handle.is_some() {
        reasons.push(ExportLossReason::EntityExtensionDictionary);
    }
    if common.color_book_handle.is_some() {
        reasons.push(ExportLossReason::EntityColorBookReference);
    }
    if common.full_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityFullVisualStyle);
    }
    if common.face_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityFaceVisualStyle);
    }
    if common.edge_visual_style_handle.is_some() {
        reasons.push(ExportLossReason::EntityEdgeVisualStyle);
    }
    if common.material_flags != 0 || common.material_handle.is_some() {
        reasons.push(ExportLossReason::EntityMaterial);
    }
    if common.shadow_flags != 0 {
        reasons.push(ExportLossReason::EntityShadowFlags);
    }
    if common.plotstyle_flags != 0 || common.plotstyle_handle.is_some() {
        reasons.push(ExportLossReason::EntityPlotStyle);
    }
    reasons
}

fn line_losses(line: &Line) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    if ![
        line.start.x,
        line.start.y,
        line.start.z,
        line.end.x,
        line.end.y,
        line.end.z,
        line.thickness,
        line.normal.x,
        line.normal.y,
        line.normal.z,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if line.start.z != 0.0 || line.end.z != 0.0 {
        reasons.push(ExportLossReason::NonPlanarZ);
    }
    if line.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if line.normal != Vector3::UNIT_Z {
        reasons.push(ExportLossReason::UnsupportedNormal);
    }
    reasons
}

fn polyline_losses(polyline: &LwPolyline) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    let finite = [
        polyline.constant_width,
        polyline.elevation,
        polyline.thickness,
        polyline.normal.x,
        polyline.normal.y,
        polyline.normal.z,
    ]
    .into_iter()
    .chain(polyline.vertices.iter().flat_map(|vertex| {
        [
            vertex.location.x,
            vertex.location.y,
            vertex.bulge,
            vertex.start_width,
            vertex.end_width,
        ]
    }))
    .all(f64::is_finite);
    if !finite {
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    if polyline.vertices.len() < 2 {
        reasons.push(ExportLossReason::PolylineTooFewVertices {
            count: polyline.vertices.len(),
        });
    }
    if polyline.elevation != 0.0 {
        reasons.push(ExportLossReason::NonZeroElevation);
    }
    if polyline.thickness != 0.0 {
        reasons.push(ExportLossReason::NonZeroThickness);
    }
    if polyline.normal != Vector3::UNIT_Z {
        reasons.push(ExportLossReason::UnsupportedNormal);
    }
    if polyline.vertices.iter().any(|vertex| vertex.bulge != 0.0) {
        reasons.push(ExportLossReason::PolylineBulge);
    }
    if polyline.constant_width != 0.0
        || polyline
            .vertices
            .iter()
            .any(|vertex| vertex.start_width != 0.0 || vertex.end_width != 0.0)
    {
        reasons.push(ExportLossReason::PolylineWidth);
    }
    if polyline.plinegen {
        reasons.push(ExportLossReason::PolylinePlinegen);
    }
    reasons
}
