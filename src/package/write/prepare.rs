//! Adapter from package construction to resource-only writer input.
//! IFCX identities and package builder keys are resolved here. Entity geometry
//! is transferred by ownership; IFCDR preparation never sees package state.
use super::ifcx::NodePaths;
use super::state::{DrawingState, PendingEntity};
use crate::ifcdr::logical::*;
use crate::ifcdr::write::{
    prepare_resource, IfcdrWriteEntity, IfcdrWriteInput, PreparedIfcdrResource,
};
#[cfg(test)]
use crate::ifcdr::Point2;

pub(crate) fn prepare_drawing(
    drawing: &mut DrawingState,
    paths: &NodePaths,
) -> Result<PreparedIfcdrResource, Vec<IfcdrDiagnostic>> {
    let fail = || {
        vec![IfcdrDiagnostic {
            category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
            code: IFCCAD_IFCDR_REFERENCE_MISSING,
            resource_id: drawing.options.representation_resource_id.clone(),
            collection: "appearanceBinding",
            row: None,
            property: "ifcxAppearance",
            message: "prepared binding has no IFCX definition".into(),
        }]
    };
    if paths.layers.len() != drawing.layers.len()
        || paths.appearances.len() != drawing.appearances.len()
    {
        return Err(fail());
    }
    // These are construction defaults, not implicit rows in the format or codec.
    let mut appearances = vec![
        IfcdrAppearanceBinding {
            id: 0,
            ifcx_appearance: None,
            modes: [0; 4],
            override_id: None,
        },
        IfcdrAppearanceBinding {
            id: 1,
            ifcx_appearance: None,
            modes: [2; 4],
            override_id: None,
        },
    ];
    for binding in &drawing.appearance_bindings {
        let target = match binding.definition.appearance {
            Some(key) => Some(
                paths
                    .appearances
                    .get(key.local_id.checked_sub(2).ok_or_else(fail)? as usize)
                    .ok_or_else(fail)?
                    .clone(),
            ),
            None => None,
        };
        let d = binding.definition;
        let modes = [
            d.color_mode,
            d.opacity_mode,
            d.line_pattern_mode,
            d.line_weight_mode,
        ]
        .map(|m| match m {
            AppearanceMode::ByLayer => 0,
            AppearanceMode::Explicit => 1,
            AppearanceMode::ByBlock => 2,
        });
        appearances.push(IfcdrAppearanceBinding {
            id: binding.id.get(),
            ifcx_appearance: target,
            modes,
            override_id: None,
        });
    }
    let layers = drawing
        .layers
        .iter()
        .zip(&paths.layers)
        .map(|(l, p)| IfcdrLayerBinding {
            id: l.local_id,
            ifcx_layer: p.clone(),
        })
        .collect();
    let mut overrides = Vec::new();
    let entities = std::mem::take(&mut drawing.entities)
        .into_iter()
        .map(|pending| match pending {
            PendingEntity::Viewport {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => {
                let layer_overrides = d
                    .layer_overrides
                    .into_iter()
                    .map(|entry| {
                        let appearance_override_id = entry.appearance.map(|patch| {
                            let id = u32::try_from(overrides.len() + 1)
                                .expect("viewport override count fits u32");
                            overrides.push(IfcdrAppearanceOverride {
                                id,
                                color: patch.color,
                                opacity: patch.opacity,
                                ifcx_line_pattern: patch.line_pattern.map(|line| line.name),
                                line_weight: patch.line_weight,
                            });
                            id
                        });
                        ViewportLayerOverride {
                            layer_id: entry.layer.local_id,
                            frozen: entry.frozen,
                            appearance_override_id,
                        }
                    })
                    .collect();
                IfcdrWriteEntity::Viewport(IfcdrViewportRow {
                    entity: IfcdrEntityRow {
                        entity_id: entity_id.get(),
                        scope_id,
                        layer_id: d.layer.local_id,
                        appearance_id: appearance_id.get(),
                        visible: d.visible,
                    },
                    view_scope_id: 0,
                    frame: d.frame,
                    view: d.view,
                    render_mode: d.render_mode,
                    view_enabled: d.view_enabled,
                    view_locked: d.view_locked,
                    paper_clip: d.paper_clip,
                    plot_shading_override: d.plot_shading_override,
                    layer_overrides,
                })
            }
            PendingEntity::BlockInstance {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::BlockInstance(IfcdrBlockInstanceRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                definition_scope_id: d.definition.local_id,
                transform: d.transform.components(),
            }),
            PendingEntity::Point {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::Point(IfcdrPointRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                placement: d.placement.components(),
            }),
            PendingEntity::Circle {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::Circle(IfcdrCircleRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                placement: d.placement.components(),
                radius: d.radius,
            }),
            PendingEntity::Arc {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::Arc(IfcdrArcRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                placement: d.placement.components(),
                radius: d.radius,
                start_parameter: d.start_parameter,
                sweep_parameter: d.sweep_parameter,
            }),
            PendingEntity::Ellipse {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::Ellipse(IfcdrEllipseRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                placement: d.placement.components(),
                semi_major_radius: d.semi_major_radius,
                semi_minor_radius: d.semi_minor_radius,
            }),
            PendingEntity::EllipseArc {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::EllipseArc(IfcdrEllipseArcRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                placement: d.placement.components(),
                semi_major_radius: d.semi_major_radius,
                semi_minor_radius: d.semi_minor_radius,
                start_parameter: d.start_parameter,
                sweep_parameter: d.sweep_parameter,
            }),
            PendingEntity::Line {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::Line(IfcdrLineRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                start: d.start,
                end: d.end,
            }),
            PendingEntity::PlanarPolyline {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::PlanarPolyline {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                closed: d.closed,
                points: d.points,
                bulges: d.bulges,
                placement: d.placement.components(),
            },
            PendingEntity::SpatialPolyline {
                scope_id,
                entity_id,
                appearance_id,
                definition: d,
            } => IfcdrWriteEntity::SpatialPolyline(IfcdrSpatialPolylineRow {
                entity: IfcdrEntityRow {
                    entity_id: entity_id.get(),
                    scope_id,
                    layer_id: d.layer.local_id,
                    appearance_id: appearance_id.get(),
                    visible: d.visible,
                },
                closed: d.closed,
                points: d.points,
            }),
        })
        .collect();
    prepare_resource(IfcdrWriteInput {
        resource_id: drawing.options.representation_resource_id.clone(),
        unit: drawing.options.length_unit,
        next_entity_id: drawing.next_entity_id,
        block_definitions: drawing.block_definitions.clone(),
        scopes: drawing.scopes.clone(),
        layers,
        appearances,
        overrides,
        entities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifcdr::logical::{validate_resource, IfcdrLinesAccess, IfcdrResourceAccess};
    use crate::ifcdr::IfcdrLengthUnit;
    use crate::package::{DrawingOptions, PackageBuilder, PackageOptions};
    use crate::{PackageId, ResourceId};

    fn mixed_builder() -> PackageBuilder {
        use crate::ifcdr::EntityId;
        use crate::package::*;
        let mut builder = PackageBuilder::new(PackageOptions {
            package_id: PackageId::new("package").unwrap(),
            data_version: "1".into(),
            author: "test".into(),
            timestamp: "2026-09-08T10:00:00Z".into(),
        })
        .unwrap();
        let mut drawing = builder
            .add_drawing(DrawingOptions {
                model_layout_name: "Model".into(),
                representation_resource_id: ResourceId::new("resource").unwrap(),
                length_unit: IfcdrLengthUnit::Millimetre,
            })
            .unwrap();
        let style = drawing
            .appearances()
            .add(AppearanceDefinition {
                name: "style".into(),
                color: AppearanceColor::rgb(1, 2, 3),
                opacity: 1.,
                line_pattern: LinePatternDefinition::named("continuous"),
                line_weight: 0.,
            })
            .unwrap();
        let layer = drawing
            .layers()
            .add(LayerDefinition {
                name: "0".into(),
                visible: true,
                frozen: false,
                locked: false,
                plottable: true,
                frozen_in_new_viewports: false,
                description: None,
                appearance: style,
            })
            .unwrap();
        let mut model = drawing.model_space();
        model
            .add_line_with_id(
                EntityId::new(10).unwrap(),
                LineDefinition {
                    start: crate::ifcdr::Point3::new(1., 1., 0.0),
                    end: crate::ifcdr::Point3::new(1., 1., 0.0),
                    layer,
                    appearance: EntityAppearance::by_layer(),
                    visible: false,
                },
            )
            .unwrap();
        model
            .add_planar_polyline_with_id(
                EntityId::new(5).unwrap(),
                PlanarPolylineDefinition {
                    placement: crate::ifcdr::PlanePlacement::default(),
                    bulges: Vec::new(),
                    points: vec![Point2::new(0., 0.), Point2::new(0., 0.)],
                    closed: true,
                    layer,
                    appearance: EntityAppearance::explicit(style),
                    visible: true,
                },
            )
            .unwrap();
        model
            .add_line(LineDefinition {
                start: crate::ifcdr::Point3::new(-2., 3., 0.0),
                end: crate::ifcdr::Point3::new(4., 5., 0.0),
                layer,
                appearance: EntityAppearance::by_block(),
                visible: true,
            })
            .unwrap();
        builder
    }

    #[test]
    fn prepared_and_decoded_backings_share_the_resource_contract() {
        use crate::ifcdr::codec::json::{decode_json, encode_json};
        let mut drawing = mixed_builder().state.drawing.unwrap();
        let paths = NodePaths::for_drawing(&drawing).unwrap();
        let prepared = prepare_drawing(&mut drawing, &paths).unwrap();
        assert_eq!(prepared.order(0), Some([10, 5, 11].as_slice()));
        assert_eq!(prepared.next_entity_id(), 12);
        let (proof, errors) = validate_resource(prepared).into_parts();
        let proof = proof.unwrap_or_else(|| panic!("{errors:?}"));
        let encoded = encode_json(&proof).unwrap();
        let decoded = decode_json(
            "resource.json",
            &serde_json::from_slice(&encoded.bytes).unwrap(),
        )
        .unwrap();
        crate::ifcdr::logical::test_support::assert_resource_eq(
            proof.loaded().resource(),
            &decoded,
        );
        assert!(validate_resource(decoded).validated().is_some());
    }

    #[test]
    fn completion_collects_independent_missing_references() {
        let mut builder = mixed_builder();
        let state = builder.state.drawing.as_mut().unwrap();
        if let PendingEntity::Line { definition, .. } = &mut state.entities[0] {
            definition.layer.local_id = 998;
        }
        if let PendingEntity::PlanarPolyline { appearance_id, .. } = &mut state.entities[1] {
            *appearance_id = crate::ifcdr::AppearanceId::new(999);
        }
        let Err(crate::package::PackageBuildError::Validation { diagnostics }) = builder.finish()
        else {
            panic!("invalid completion must return diagnostics without an artifact");
        };
        assert!(diagnostics
            .iter()
            .any(|d| d.location.as_deref() == Some("/streams/lineStream/layerId/0")));
        assert!(
            diagnostics
                .iter()
                .any(|d| d.location.as_deref()
                    == Some("/streams/planarPolylineStream/appearanceId/0"))
        );
    }

    #[test]
    fn empty_prepared_drawing_has_no_bounds_and_passes_shared_validation() {
        let mut builder = PackageBuilder::new(PackageOptions {
            package_id: PackageId::new("package").unwrap(),
            data_version: "1".into(),
            author: "test".into(),
            timestamp: "2026-09-08T10:00:00Z".into(),
        })
        .unwrap();
        builder
            .add_drawing(DrawingOptions {
                model_layout_name: "Model".into(),
                representation_resource_id: ResourceId::new("resource").unwrap(),
                length_unit: IfcdrLengthUnit::Millimetre,
            })
            .unwrap();
        let mut drawing = builder.state.drawing.unwrap();
        let paths = NodePaths::for_drawing(&drawing).unwrap();
        let prepared = prepare_drawing(&mut drawing, &paths).unwrap();
        assert!(prepared.lines().is_empty());
        assert!(prepared.scopes()[0].bounds.is_none());
        assert_eq!(prepared.next_entity_id(), 1);
        let (proof, errors) = validate_resource(prepared).into_parts();
        assert!(proof.is_some(), "{errors:?}");
    }
}
