use crate::ifcdr::geometry::PlanePlacementComponents;
use crate::ifcdr::logical::*;
use crate::ifcdr::{IfcdrLengthUnit, Point2};
use crate::ResourceId;
use std::collections::BTreeMap;

/// Owned resource input. The caller supplies resolved external identities and
/// entities in draw order; preparation neither allocates IDs nor resolves IFCX.
#[derive(Debug)]
pub(crate) struct IfcdrWriteInput {
    pub resource_id: ResourceId,
    pub unit: IfcdrLengthUnit,
    pub next_entity_id: u64,
    pub scopes: Vec<IfcdrScope>,
    pub block_definitions: Vec<IfcdrBlockDefinition>,
    pub layers: Vec<IfcdrLayerBinding>,
    pub appearances: Vec<IfcdrAppearanceBinding>,
    pub overrides: Vec<IfcdrAppearanceOverride>,
    pub entities: Vec<IfcdrWriteEntity>,
}

#[derive(Debug)]
pub(crate) enum IfcdrWriteEntity {
    Point(IfcdrPointRow),
    Circle(IfcdrCircleRow),
    Arc(IfcdrArcRow),
    Ellipse(IfcdrEllipseRow),
    EllipseArc(IfcdrEllipseArcRow),
    BlockInstance(IfcdrBlockInstanceRow),
    Viewport(IfcdrViewportRow),
    Line(IfcdrLineRow),
    PlanarPolyline {
        entity: IfcdrEntityRow,
        closed: bool,
        points: Vec<Point2>,
        bulges: Vec<f64>,
        placement: PlanePlacementComponents,
    },
    SpatialPolyline(IfcdrSpatialPolylineRow),
}

/// Writer backing with per-kind indexes over owned entities. This is a
/// candidate until shared validation succeeds; vertices are not copied.
#[derive(Debug)]
pub(crate) struct PreparedIfcdrResource {
    input: IfcdrWriteInput,
    points: Vec<IfcdrPointRow>,
    circles: Vec<IfcdrCircleRow>,
    arcs: Vec<IfcdrArcRow>,
    ellipses: Vec<IfcdrEllipseRow>,
    ellipse_arcs: Vec<IfcdrEllipseArcRow>,
    lines: Vec<usize>,
    polylines: Vec<usize>,
    spatial_polylines: Vec<IfcdrSpatialPolylineRow>,
    block_instances: Vec<usize>,
    viewports: Vec<IfcdrViewportRow>,
    orders: Vec<IfcdrScopeOrder>,
}

pub(crate) fn prepare_resource(
    input: IfcdrWriteInput,
) -> Result<PreparedIfcdrResource, Vec<IfcdrDiagnostic>> {
    let mut points = Vec::new();
    let mut circles = Vec::new();
    let mut arcs = Vec::new();
    let mut ellipses = Vec::new();
    let mut ellipse_arcs = Vec::new();
    let mut lines = Vec::new();
    let mut polylines = Vec::new();
    let mut spatial_polylines = Vec::new();
    let mut block_instances = Vec::new();
    let mut viewports = Vec::new();
    let mut orders: Vec<_> = input
        .scopes
        .iter()
        .map(|s| IfcdrScopeOrder {
            scope_id: s.id,
            entities: Vec::new(),
        })
        .collect();
    let scope_rows: BTreeMap<_, _> = input
        .scopes
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id, i))
        .collect();
    for (index, value) in input.entities.iter().enumerate() {
        let entity = match value {
            IfcdrWriteEntity::Point(point) => {
                points.push(*point);
                point.entity
            }
            IfcdrWriteEntity::Circle(circle) => {
                circles.push(*circle);
                circle.entity
            }
            IfcdrWriteEntity::Arc(arc) => {
                arcs.push(*arc);
                arc.entity
            }
            IfcdrWriteEntity::Ellipse(ellipse) => {
                ellipses.push(*ellipse);
                ellipse.entity
            }
            IfcdrWriteEntity::EllipseArc(arc) => {
                ellipse_arcs.push(*arc);
                arc.entity
            }
            IfcdrWriteEntity::Viewport(viewport) => {
                viewports.push(viewport.clone());
                viewport.entity
            }
            IfcdrWriteEntity::BlockInstance(instance) => {
                block_instances.push(index);
                instance.entity
            }
            IfcdrWriteEntity::Line(line) => {
                lines.push(index);
                line.entity
            }
            IfcdrWriteEntity::PlanarPolyline { entity, .. } => {
                polylines.push(index);
                *entity
            }
            IfcdrWriteEntity::SpatialPolyline(polyline) => {
                spatial_polylines.push(polyline.clone());
                polyline.entity
            }
        };
        if let Some(&row) = scope_rows.get(&entity.scope_id) {
            orders[row].entities.push(entity.entity_id);
        }
        // Missing/duplicate scope definitions are diagnosed by shared validation.
    }
    let mut prepared = PreparedIfcdrResource {
        input,
        points,
        circles,
        arcs,
        ellipses,
        ellipse_arcs,
        lines,
        polylines,
        spatial_polylines,
        block_instances,
        viewports,
        orders,
    };
    let bounds = geometric_bounds(&prepared)?;
    for scope in &mut prepared.input.scopes {
        scope.bounds = bounds.get(&scope.id).copied().flatten();
    }
    Ok(prepared)
}

pub(crate) struct PreparedLines<'a>(&'a PreparedIfcdrResource);
pub(crate) struct PreparedBlockInstances<'a>(&'a PreparedIfcdrResource);
impl IfcdrBlockInstancesAccess for PreparedBlockInstances<'_> {
    fn len(&self) -> usize {
        self.0.block_instances.len()
    }
    fn get(&self, row: usize) -> Option<IfcdrBlockInstanceRow> {
        match self
            .0
            .input
            .entities
            .get(*self.0.block_instances.get(row)?)?
        {
            IfcdrWriteEntity::BlockInstance(instance) => Some(*instance),
            _ => None,
        }
    }
}
pub(crate) struct PreparedPolylines<'a>(&'a PreparedIfcdrResource);
pub(crate) struct PreparedPolyline<'a> {
    entity: IfcdrEntityRow,
    closed: bool,
    points: &'a [Point2],
    bulges: &'a [f64],
    placement: PlanePlacementComponents,
}

impl IfcdrLinesAccess for PreparedLines<'_> {
    fn len(&self) -> usize {
        self.0.lines.len()
    }
    fn get(&self, row: usize) -> Option<IfcdrLineRow> {
        match self.0.input.entities.get(*self.0.lines.get(row)?)? {
            IfcdrWriteEntity::Line(line) => Some(*line),
            _ => None,
        }
    }
}
impl IfcdrPolylinesAccess for PreparedPolylines<'_> {
    type Polyline<'a>
        = PreparedPolyline<'a>
    where
        Self: 'a;
    fn len(&self) -> usize {
        self.0.polylines.len()
    }
    fn get(&self, row: usize) -> Option<PreparedPolyline<'_>> {
        match self.0.input.entities.get(*self.0.polylines.get(row)?)? {
            IfcdrWriteEntity::PlanarPolyline {
                entity,
                closed,
                points,
                bulges,
                placement,
            } => Some(PreparedPolyline {
                entity: *entity,
                closed: *closed,
                points,
                bulges,
                placement: *placement,
            }),
            _ => None,
        }
    }
}
impl IfcdrPolylineAccess for PreparedPolyline<'_> {
    fn entity(&self) -> IfcdrEntityRow {
        self.entity
    }
    fn placement(&self) -> PlanePlacementComponents {
        self.placement
    }
    fn closed(&self) -> bool {
        self.closed
    }
    fn vertex_count(&self) -> usize {
        self.points.len()
    }
    fn vertex(&self, index: usize) -> Option<Point2> {
        self.points.get(index).copied()
    }
    fn bulge(&self, index: usize) -> Option<f64> {
        self.bulges.get(index).copied()
    }
}
impl IfcdrResourceAccess for PreparedIfcdrResource {
    type Lines<'a> = PreparedLines<'a>;
    type Polylines<'a> = PreparedPolylines<'a>;
    type BlockInstances<'a> = PreparedBlockInstances<'a>;
    fn block_definitions(&self) -> &[IfcdrBlockDefinition] {
        &self.input.block_definitions
    }
    fn block_instances(&self) -> Self::BlockInstances<'_> {
        PreparedBlockInstances(self)
    }
    fn viewports(&self) -> &[IfcdrViewportRow] {
        &self.viewports
    }
    fn resource_id(&self) -> &ResourceId {
        &self.input.resource_id
    }
    fn unit(&self) -> IfcdrLengthUnit {
        self.input.unit
    }
    fn next_entity_id(&self) -> u64 {
        self.input.next_entity_id
    }
    fn scopes(&self) -> &[IfcdrScope] {
        &self.input.scopes
    }
    fn layers(&self) -> &[IfcdrLayerBinding] {
        &self.input.layers
    }
    fn appearances(&self) -> &[IfcdrAppearanceBinding] {
        &self.input.appearances
    }
    fn overrides(&self) -> &[IfcdrAppearanceOverride] {
        &self.input.overrides
    }
    fn lines(&self) -> PreparedLines<'_> {
        PreparedLines(self)
    }
    fn points(&self) -> &[IfcdrPointRow] {
        &self.points
    }
    fn circles(&self) -> &[IfcdrCircleRow] {
        &self.circles
    }
    fn arcs(&self) -> &[IfcdrArcRow] {
        &self.arcs
    }
    fn ellipses(&self) -> &[IfcdrEllipseRow] {
        &self.ellipses
    }
    fn ellipse_arcs(&self) -> &[IfcdrEllipseArcRow] {
        &self.ellipse_arcs
    }
    fn polylines(&self) -> PreparedPolylines<'_> {
        PreparedPolylines(self)
    }
    fn spatial_polylines(&self) -> &[IfcdrSpatialPolylineRow] {
        &self.spatial_polylines
    }
    fn orders(&self) -> &[IfcdrScopeOrder] {
        &self.orders
    }
}
