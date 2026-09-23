use super::*;
use crate::ifcdr::{Bounds3d, IfcdrLengthUnit, Point2, Point3};
use crate::ResourceId;

#[derive(Debug)]
struct TestResource {
    id: ResourceId,
    next: u64,
    scopes: Vec<IfcdrScope>,
    definitions: Vec<IfcdrBlockDefinition>,
    instances: Vec<IfcdrBlockInstanceRow>,
    layers: Vec<IfcdrLayerBinding>,
    appearances: Vec<IfcdrAppearanceBinding>,
    overrides: Vec<IfcdrAppearanceOverride>,
    lines: Vec<IfcdrLineRow>,
    polylines: Vec<TestPolyline>,
    viewports: Vec<IfcdrViewportRow>,
    orders: Vec<IfcdrScopeOrder>,
}
#[derive(Debug)]
struct TestPolyline {
    entity: IfcdrEntityRow,
    points: Vec<Point2>,
    closed: bool,
}
struct Lines<'a>(&'a [IfcdrLineRow]);
struct Polylines<'a>(&'a [TestPolyline]);
struct Polyline<'a>(&'a TestPolyline);
impl IfcdrLinesAccess for Lines<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, row: usize) -> Option<IfcdrLineRow> {
        self.0.get(row).copied()
    }
}
impl IfcdrPolylinesAccess for Polylines<'_> {
    type Polyline<'a>
        = Polyline<'a>
    where
        Self: 'a;
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, row: usize) -> Option<Polyline<'_>> {
        self.0.get(row).map(Polyline)
    }
}
impl IfcdrPolylineAccess for Polyline<'_> {
    fn entity(&self) -> IfcdrEntityRow {
        self.0.entity
    }
    fn placement(&self) -> crate::ifcdr::geometry::PlanePlacementComponents {
        crate::ifcdr::PlanePlacement::default().components()
    }
    fn closed(&self) -> bool {
        self.0.closed
    }
    fn vertex_count(&self) -> usize {
        self.0.points.len()
    }
    fn vertex(&self, i: usize) -> Option<Point2> {
        self.0.points.get(i).copied()
    }
}
impl IfcdrResourceAccess for TestResource {
    fn viewports(&self) -> &[IfcdrViewportRow] {
        &self.viewports
    }
    type BlockInstances<'a> = &'a [IfcdrBlockInstanceRow];
    fn block_instances(&self) -> Self::BlockInstances<'_> {
        &self.instances
    }
    fn block_definitions(&self) -> &[IfcdrBlockDefinition] {
        &self.definitions
    }
    type Lines<'a> = Lines<'a>;
    type Polylines<'a> = Polylines<'a>;
    fn resource_id(&self) -> &ResourceId {
        &self.id
    }
    fn unit(&self) -> IfcdrLengthUnit {
        IfcdrLengthUnit::Millimetre
    }
    fn next_entity_id(&self) -> u64 {
        self.next
    }
    fn scopes(&self) -> &[IfcdrScope] {
        &self.scopes
    }
    fn layers(&self) -> &[IfcdrLayerBinding] {
        &self.layers
    }
    fn appearances(&self) -> &[IfcdrAppearanceBinding] {
        &self.appearances
    }
    fn overrides(&self) -> &[IfcdrAppearanceOverride] {
        &self.overrides
    }
    fn lines(&self) -> Lines<'_> {
        Lines(&self.lines)
    }
    fn polylines(&self) -> Polylines<'_> {
        Polylines(&self.polylines)
    }
    fn orders(&self) -> &[IfcdrScopeOrder] {
        &self.orders
    }
}
fn line_candidate() -> TestResource {
    TestResource {
        definitions: vec![],
        instances: vec![],
        id: ResourceId::new("drawing").unwrap(),
        next: 2,
        scopes: vec![IfcdrScope {
            bounds: Some(Bounds3d {
                min: Point3::new(0., 0., 0.),
                max: Point3::new(1., 1., 0.),
            }),
            id: 0,
            kind: IfcdrScopeKind::ModelSpace,
        }],
        layers: vec![IfcdrLayerBinding {
            id: 0,
            ifcx_layer: "layer-0".into(),
        }],
        appearances: vec![IfcdrAppearanceBinding {
            id: 0,
            ifcx_appearance: None,
            modes: [0; 4],
            override_id: None,
        }],
        overrides: vec![],
        lines: vec![IfcdrLineRow {
            entity: IfcdrEntityRow {
                entity_id: 1,
                scope_id: 0,
                layer_id: 0,
                appearance_id: 0,
                visible: true,
            },
            start: crate::ifcdr::Point3::new(0., 0., 0.0),
            end: crate::ifcdr::Point3::new(1., 1., 0.0),
        }],
        polylines: vec![],
        viewports: vec![],
        orders: vec![IfcdrScopeOrder {
            scope_id: 0,
            entities: vec![1],
        }],
    }
}

fn block_candidate() -> TestResource {
    let mut r = line_candidate();
    let mut definition_scope = r.scopes[0].clone();
    definition_scope.id = 21;
    definition_scope.kind = IfcdrScopeKind::BlockDefinition;
    r.scopes.push(definition_scope);
    r.lines[0].entity.scope_id = 21;
    r.definitions.push(IfcdrBlockDefinition {
        scope_id: 21,
        name: "Door".into(),
        base_point: Point3::new(0., 0., 0.),
        description: String::new(),
        anonymous: false,
        insertion_unit: IfcdrLengthUnit::Unitless,
        explodable: true,
        scaling: crate::ifcdr::BlockScaling::Any,
    });
    r.instances.push(IfcdrBlockInstanceRow {
        entity: IfcdrEntityRow {
            entity_id: 2,
            scope_id: 0,
            layer_id: 0,
            appearance_id: 0,
            visible: false,
        },
        definition_scope_id: 21,
        transform: crate::ifcdr::BlockTransform::default().components(),
    });
    r.orders[0].entities = vec![2];
    r.orders.push(IfcdrScopeOrder {
        scope_id: 21,
        entities: vec![1],
    });
    r.next = 3;
    r
}

fn viewport_candidate() -> TestResource {
    let mut r = line_candidate();
    r.scopes.push(IfcdrScope {
        id: 7,
        kind: IfcdrScopeKind::PaperSpace,
        bounds: Some(Bounds3d {
            min: Point3::new(8.0, 19.0, 0.0),
            max: Point3::new(12.0, 21.0, 0.0),
        }),
    });
    r.viewports.push(IfcdrViewportRow {
        entity: IfcdrEntityRow {
            entity_id: 2,
            scope_id: 7,
            layer_id: 0,
            appearance_id: 0,
            visible: true,
        },
        view_scope_id: 0,
        frame: ViewportFrame {
            center: Point2::new(10.0, 20.0),
            width: 4.0,
            height: 2.0,
        },
        view: ViewDefinition {
            center: Point2::new(0.0, 0.0),
            target: Point3::new(0.0, 0.0, 0.0),
            direction: crate::ifcdr::Vector3::new(0.0, 0.0, 1.0),
            height: 100.0,
            twist: 0.0,
            projection: ProjectionMode::Orthographic,
            lens_length: None,
            front_clip: FrontClip {
                mode: FrontClipMode::Disabled,
                distance: None,
            },
            back_clip: BackClip {
                mode: BackClipMode::Disabled,
                distance: None,
            },
        },
        render_mode: ViewportRenderMode::TwoDimensional,
        view_enabled: true,
        view_locked: false,
        paper_clip: PaperClip {
            enabled: false,
            boundary_entity_id: None,
        },
        plot_shading_override: None,
        layer_overrides: vec![ViewportLayerOverride {
            layer_id: 0,
            frozen: true,
            appearance_override_id: None,
        }],
    });
    r.orders.push(IfcdrScopeOrder {
        scope_id: 7,
        entities: vec![2],
    });
    r.next = 3;
    r
}

#[test]
fn paper_viewport_uses_its_frame_for_bounds_and_checks_ownership() {
    assert!(codes(viewport_candidate()).is_empty());
    let mut invalid = viewport_candidate();
    invalid.viewports[0].entity.scope_id = 0;
    assert!(codes(invalid).contains(&IFCCAD_IFCDR_VIEWPORT_INVALID));
    let mut invalid = viewport_candidate();
    invalid.viewports[0].view.direction = crate::ifcdr::Vector3::new(0.0, 0.0, 0.0);
    assert!(codes(invalid).contains(&IFCCAD_IFCDR_VIEWPORT_INVALID));
    let mut invalid = viewport_candidate();
    invalid.viewports[0].layer_overrides[0].frozen = false;
    assert!(codes(invalid).contains(&IFCCAD_IFCDR_VIEWPORT_INVALID));
    let mut empty_patch = viewport_candidate();
    empty_patch.viewports[0].layer_overrides[0].frozen = false;
    empty_patch.viewports[0].layer_overrides[0].appearance_override_id = Some(3);
    empty_patch.overrides.push(IfcdrAppearanceOverride {
        id: 3,
        color: None,
        opacity: None,
        ifcx_line_pattern: None,
        line_weight: None,
    });
    assert!(codes(empty_patch).contains(&IFCCAD_IFCDR_VIEWPORT_INVALID));
    let mut invalid = viewport_candidate();
    invalid.scopes[1].bounds = Some(Bounds3d {
        min: Point3::new(9.0, 19.0, 0.0),
        max: Point3::new(12.0, 21.0, 0.0),
    });
    assert!(codes(invalid).contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
}

#[test]
fn active_viewport_clip_rejects_duplicate_geometric_vertices_even_with_signed_zero() {
    let mut resource = viewport_candidate();
    resource.viewports[0].frame.center = Point2::new(0.0, 0.0);
    resource.scopes[1].bounds = Some(Bounds3d {
        min: Point3::new(-2.0, -1.0, 0.0),
        max: Point3::new(2.0, 1.0, 0.0),
    });
    resource.polylines.push(TestPolyline {
        entity: IfcdrEntityRow {
            entity_id: 3,
            scope_id: 7,
            layer_id: 0,
            appearance_id: 0,
            visible: false,
        },
        points: vec![
            Point2::new(0.0, 0.0),
            Point2::new(-0.0, 0.0),
            Point2::new(1.0, 1.0),
        ],
        closed: true,
    });
    resource.viewports[0].paper_clip = PaperClip {
        enabled: true,
        boundary_entity_id: Some(3),
    };
    resource.orders[1].entities.push(3);
    resource.next = 4;
    assert!(codes(resource).contains(&IFCCAD_IFCDR_VIEWPORT_INVALID));
}

#[test]
fn block_graph_validates_every_definition_and_signed_uniformity() {
    assert!(codes(block_candidate()).is_empty());
    for change in [
        "zero_model",
        "two_models",
        "missing_definition",
        "duplicate_definition",
        "wrong_definition_kind",
        "missing_target",
        "duplicate_name",
        "uniform_reflection",
        "unused_cycle",
    ] {
        let mut r = block_candidate();
        match change {
            "zero_model" => r.scopes[0].kind = IfcdrScopeKind::PaperSpace,
            "two_models" => r.scopes.push(IfcdrScope {
                id: 99,
                kind: IfcdrScopeKind::ModelSpace,
                bounds: None,
            }),
            "missing_definition" => r.definitions.clear(),
            "duplicate_definition" => r.definitions.push(r.definitions[0].clone()),
            "wrong_definition_kind" => r.scopes[1].kind = IfcdrScopeKind::PaperSpace,
            "missing_target" => r.instances[0].definition_scope_id = 99,
            "duplicate_name" => {
                let mut d = r.definitions[0].clone();
                d.scope_id = 22;
                d.name = "DOOR".into();
                r.definitions.push(d);
                r.scopes.push(IfcdrScope {
                    id: 22,
                    kind: IfcdrScopeKind::BlockDefinition,
                    bounds: None,
                });
            }
            "uniform_reflection" => {
                r.definitions[0].scaling = crate::ifcdr::BlockScaling::Uniform;
                r.instances[0].transform.scale = crate::ifcdr::Scale3::new(-1., 1., 1.);
            }
            "unused_cycle" => {
                r.instances[0].entity.scope_id = 21;
                r.orders[0].entities.clear();
                r.scopes[0].bounds = None;
                r.orders[1].entities.push(2);
            }
            _ => unreachable!(),
        }
        assert!(codes(r).iter().any(|c| c.starts_with("IFCCAD_IFCDR_BLOCK_") || *c == "IFCCAD_IFCDR_SCOPE_INVALID"),"{change}");
    }
}

#[test]
fn block_instance_identity_is_global_and_order_is_owned_by_inserting_scope() {
    let mut r = block_candidate();
    r.instances[0].entity.entity_id = 1;
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ID_DUPLICATE));
    let mut r = block_candidate();
    r.orders[0].entities = vec![1];
    r.orders[1].entities = vec![2];
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ORDER_INVALID));
}

#[test]
fn block_bounds_empty_instances_and_nonzero_base_follow_evaluated_geometry() {
    let mut empty = block_candidate();
    empty.lines.clear();
    empty.orders[1].entities.clear();
    for scope in &mut empty.scopes {
        scope.bounds = None;
    }
    empty.instances[0].transform.placement.origin = Point3::new(100., 100., 100.);
    assert!(codes(empty).is_empty());
    let mut r = block_candidate();
    let base = Point3::new(2., 0., 0.);
    r.definitions[0].base_point = base;
    r.lines[0].start = base;
    r.lines[0].end = base;
    r.scopes[1].bounds = Some(Bounds3d {
        min: base,
        max: base,
    });
    let insert = Point3::new(0., 1., 0.);
    r.instances[0].transform.placement.origin = insert;
    r.scopes[0].bounds = Some(Bounds3d {
        min: insert,
        max: insert,
    });
    assert!(codes(r).is_empty());
}

#[test]
fn block_bounds_loose_definition_box_does_not_reject_fitting_leaf_geometry() {
    let mut r = block_candidate();
    r.instances[0].transform.rotation = 0.7;
    let derived = geometric_bounds(&r).unwrap();
    r.scopes[0].bounds = derived[&0];
    let huge = Bounds3d {
        min: Point3::new(-100., -100., -100.),
        max: Point3::new(100., 100., 100.),
    };
    r.scopes[1].bounds = Some(huge);
    let graph = BlockGraph::build(&r).unwrap();
    let corner = graph.transform(&r, 0).unwrap().apply(huge.max).unwrap();
    assert!(corner[2].lower > r.scopes[0].bounds.unwrap().max.z());
    assert!(codes(r).is_empty());
}

#[test]
fn block_bounds_distinguish_unresolved_intervals_from_proved_outside_geometry() {
    use crate::diagnostic::PackageDiagnosticCategory;
    let mut r = block_candidate();
    let point = Point3::new(0., 1., 0.);
    r.lines[0].start = point;
    r.lines[0].end = point;
    r.scopes[1].bounds = Some(Bounds3d {
        min: point,
        max: point,
    });
    r.instances[0].transform.rotation = 0.7;
    let enclosure = BlockGraph::build(&r)
        .unwrap()
        .transform(&r, 0)
        .unwrap()
        .apply(point)
        .unwrap();
    assert!(enclosure[0].lower < enclosure[0].upper.next_down());
    r.scopes[0].bounds = Some(Bounds3d {
        min: Point3::new(enclosure[0].lower, enclosure[1].lower, enclosure[2].lower),
        max: Point3::new(
            enclosure[0].upper.next_down(),
            enclosure[1].upper,
            enclosure[2].upper,
        ),
    });
    let (_, errors) = validate_resource(r).into_parts();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE);
    assert_eq!(
        errors[0].category,
        PackageDiagnosticCategory::ExecutionBlocked
    );
    let mut outside = block_candidate();
    outside.scopes[0].bounds = Some(Bounds3d {
        min: Point3::new(-2., -2., -2.),
        max: Point3::new(-1., -1., -1.),
    });
    let (_, errors) = validate_resource(outside).into_parts();
    assert!(errors.iter().any(|d| d.code == IFCCAD_IFCDR_BOUNDS_INVALID
        && d.category == PackageDiagnosticCategory::ContractViolation));
    assert!(!errors
        .iter()
        .any(|d| d.category == PackageDiagnosticCategory::ExecutionBlocked));
}

#[test]
fn block_graph_ten_thousand_empty_levels_are_stack_safe() {
    let mut r = block_candidate();
    let definition = r.definitions[0].clone();
    let instance = r.instances[0];
    r.definitions.clear();
    r.instances.clear();
    r.lines.clear();
    r.scopes.truncate(1);
    r.orders.clear();
    r.scopes[0].bounds = None;
    const DEPTH: u32 = 10_000;
    for id in 1..=DEPTH {
        let mut d = definition.clone();
        d.scope_id = id;
        d.name = format!("D{id}");
        r.definitions.push(d);
        r.scopes.push(IfcdrScope {
            id,
            kind: IfcdrScopeKind::BlockDefinition,
            bounds: None,
        });
        let mut i = instance;
        i.entity.entity_id = u64::from(id);
        i.entity.scope_id = id - 1;
        i.definition_scope_id = id;
        r.instances.push(i);
        r.orders.push(IfcdrScopeOrder {
            scope_id: id - 1,
            entities: vec![u64::from(id)],
        });
    }
    r.orders.push(IfcdrScopeOrder {
        scope_id: DEPTH,
        entities: vec![],
    });
    r.next = u64::from(DEPTH) + 1;
    assert!(codes(r).is_empty());
}

#[test]
fn block_graph_shared_dag_and_full_unicode_name_rules() {
    let mut r = block_candidate();
    let mut other = r.instances[0];
    other.entity.entity_id = 3;
    r.instances.push(other);
    r.orders[0].entities.push(3);
    r.next = 4;
    assert!(codes(r).is_empty());
    let mut r = block_candidate();
    r.definitions[0].name = "Straße".into();
    let mut other = r.definitions[0].clone();
    other.scope_id = 22;
    other.name = "STRASSE".into();
    r.definitions.push(other);
    r.scopes.push(IfcdrScope {
        id: 22,
        kind: IfcdrScopeKind::BlockDefinition,
        bounds: None,
    });
    r.orders.push(IfcdrScopeOrder {
        scope_id: 22,
        entities: vec![],
    });
    assert!(codes(r).contains(&IFCCAD_IFCDR_BLOCK_INVALID));
}
fn codes(resource: TestResource) -> Vec<&'static str> {
    let (proof, diagnostics) = validate_resource(resource).into_parts();
    assert_eq!(proof.is_some(), diagnostics.is_empty());
    diagnostics.iter().map(|d| d.code).collect()
}
#[test]
fn conservative_bounds_high_water_id_and_degenerate_line_are_valid() {
    let mut r = line_candidate();
    r.next = 40;
    r.lines[0].end = r.lines[0].start;
    assert!(codes(r).is_empty());
}
#[test]
fn independent_identity_and_bounds_failures_are_collected() {
    let mut r = line_candidate();
    r.lines[0].entity.entity_id = 0;
    r.scopes[0].bounds = None;
    let errors = codes(r);
    assert!(errors.contains(&IFCCAD_IFCDR_ENTITY_ID_INVALID));
    assert!(errors.contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
    assert!(!errors.contains(&IFCCAD_IFCDR_ENTITY_ORDER_INVALID));
}
#[test]
fn local_reference_failures_are_shared_and_order_is_scope_specific() {
    let mut r = line_candidate();
    r.lines[0].entity.layer_id = 9;
    assert!(codes(r).contains(&IFCCAD_IFCDR_REFERENCE_MISSING));
    let mut r = line_candidate();
    r.orders[0].entities = vec![];
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ORDER_INVALID));
    let mut r = line_candidate();
    r.orders[0].entities = vec![1, 1];
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ORDER_INVALID));
}
#[test]
fn polyline_minimum_preserves_duplicate_vertices_and_closed_flag() {
    for closed in [false, true] {
        for count in [0, 1, 2, 3] {
            let mut r = line_candidate();
            r.polylines.push(TestPolyline {
                entity: r.lines.remove(0).entity,
                closed,
                points: vec![Point2::new(0., 0.); count],
            });
            let errors = codes(r);
            assert_eq!(errors.contains(&IFCCAD_IFCDR_POLYLINE_INVALID), count < 2);
            if count >= 2 {
                assert!(errors.is_empty(), "{errors:?}");
            }
        }
    }
}
#[test]
fn invisible_geometry_must_fit_bounds_and_empty_geometry_has_none() {
    let mut r = line_candidate();
    r.lines[0].entity.visible = false;
    r.lines[0].end = Point3::new(2., 0., 0.);
    assert!(codes(r).contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
    let mut r = line_candidate();
    r.lines.clear();
    r.orders[0].entities.clear();
    r.scopes[0].bounds = None;
    assert!(codes(r).is_empty());
    let mut r = line_candidate();
    r.lines.clear();
    r.orders[0].entities.clear();
    assert!(codes(r).contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
}
#[test]
fn invalid_geometry_does_not_cascade_to_bounds() {
    let mut r = line_candidate();
    r.lines[0].end = Point3::new(f64::NAN, 0., 0.);
    let errors = codes(r);
    assert!(errors.contains(&IFCCAD_IFCDR_GEOMETRY_INVALID));
    assert!(!errors.contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
}
#[test]
fn unused_override_values_are_validated_and_metadata_can_coexist() {
    let mut r = line_candidate();
    r.overrides.push(IfcdrAppearanceOverride {
        id: 7,
        color: Some(IfcdrColor {
            rgb: [1, 2, 3],
            indexed: Some(IfcdrIndexedColor {
                system: "ACI".into(),
                index: u64::MAX,
            }),
            named: Some(IfcdrNamedColor {
                catalog: "catalog".into(),
                name: "name".into(),
            }),
        }),
        opacity: Some(-1.),
        ifcx_line_pattern: None,
        line_weight: None,
    });
    assert!(codes(r).contains(&IFCCAD_IFCDR_APPEARANCE_INVALID));
    let mut r = line_candidate();
    r.appearances[0].modes[0] = 8;
    assert!(codes(r).contains(&IFCCAD_IFCDR_APPEARANCE_INVALID));
}
#[test]
fn duplicate_ids_and_next_id_exhaustion_are_rejected() {
    let mut r = line_candidate();
    r.lines.push(r.lines[0]);
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ID_DUPLICATE));
    let mut r = line_candidate();
    r.next = 1;
    assert!(codes(r).contains(&IFCCAD_IFCDR_ENTITY_ID_INVALID));
}

#[test]
fn model_and_paper_scope_bounds_are_independent() {
    for incorrect_second in [false, true] {
        let mut r = line_candidate();
        r.next = 3;
        let mut second = r.scopes[0].clone();
        second.id = 7;
        second.kind = IfcdrScopeKind::PaperSpace;
        second.bounds = Some(Bounds3d {
            min: Point3::new(100., 0., 5.),
            max: Point3::new(101., 0., 5.),
        });
        if incorrect_second {
            second.bounds = r.scopes[0].bounds;
        }
        r.scopes.push(second);
        let mut line = r.lines[0];
        line.entity.entity_id = 2;
        line.entity.scope_id = 7;
        line.start = Point3::new(100., 0., 5.);
        line.end = Point3::new(101., 0., 5.);
        r.lines.push(line);
        r.orders.push(IfcdrScopeOrder {
            scope_id: 7,
            entities: vec![2],
        });
        let errors = codes(r);
        assert_eq!(
            errors.contains(&IFCCAD_IFCDR_BOUNDS_INVALID),
            incorrect_second
        );
        if !incorrect_second {
            assert!(errors.is_empty(), "{errors:?}");
        }
    }
}
