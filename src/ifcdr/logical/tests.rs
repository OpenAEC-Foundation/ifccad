use super::*;
use crate::ifcdr::{Bounds2d, IfcdrLengthUnit, Point2};
use crate::ResourceId;

#[derive(Debug)]
struct TestResource {
    id: ResourceId,
    next: u64,
    bounds: Option<Bounds2d>,
    scopes: Vec<IfcdrScope>,
    layers: Vec<IfcdrLayerBinding>,
    appearances: Vec<IfcdrAppearanceBinding>,
    overrides: Vec<IfcdrAppearanceOverride>,
    lines: Vec<IfcdrLineRow>,
    polylines: Vec<TestPolyline>,
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
    fn bounds(&self) -> Option<Bounds2d> {
        self.bounds
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
        id: ResourceId::new("drawing").unwrap(),
        next: 2,
        bounds: Some(Bounds2d {
            min: Point2::new(0., 0.),
            max: Point2::new(1., 1.),
        }),
        scopes: vec![IfcdrScope {
            id: 0,
            kind: 0,
            name: "Model".into(),
            base: Point2::new(0., 0.),
            flags: 0,
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
            start: Point2::new(0., 0.),
            end: Point2::new(1., 1.),
        }],
        polylines: vec![],
        orders: vec![IfcdrScopeOrder {
            scope_id: 0,
            entities: vec![1],
        }],
    }
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
    r.bounds = None;
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
    r.lines[0].end = Point2::new(2., 0.);
    assert!(codes(r).contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
    let mut r = line_candidate();
    r.lines.clear();
    r.orders[0].entities.clear();
    r.bounds = None;
    assert!(codes(r).is_empty());
    let mut r = line_candidate();
    r.lines.clear();
    r.orders[0].entities.clear();
    assert!(codes(r).contains(&IFCCAD_IFCDR_BOUNDS_INVALID));
}
#[test]
fn invalid_geometry_does_not_cascade_to_bounds() {
    let mut r = line_candidate();
    r.lines[0].end = Point2::new(f64::NAN, 0.);
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
