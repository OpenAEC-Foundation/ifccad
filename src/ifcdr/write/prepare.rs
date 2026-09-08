use crate::ifcdr::logical::*;
use crate::ifcdr::{Bounds2d, IfcdrLengthUnit, Point2};
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
    pub layers: Vec<IfcdrLayerBinding>,
    pub appearances: Vec<IfcdrAppearanceBinding>,
    pub overrides: Vec<IfcdrAppearanceOverride>,
    pub entities: Vec<IfcdrWriteEntity>,
}

#[derive(Debug)]
pub(crate) enum IfcdrWriteEntity {
    Line(IfcdrLineRow),
    Polyline {
        entity: IfcdrEntityRow,
        closed: bool,
        points: Vec<Point2>,
    },
}

/// Writer backing with per-kind indexes over owned entities. This is a
/// candidate until shared validation succeeds; vertices are not copied.
#[derive(Debug)]
pub(crate) struct PreparedIfcdrResource {
    input: IfcdrWriteInput,
    bounds: Option<Bounds2d>,
    lines: Vec<usize>,
    polylines: Vec<usize>,
    orders: Vec<IfcdrScopeOrder>,
}

pub(crate) fn prepare_resource(
    input: IfcdrWriteInput,
) -> Result<PreparedIfcdrResource, Vec<IfcdrDiagnostic>> {
    let mut lines = Vec::new();
    let mut polylines = Vec::new();
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
            IfcdrWriteEntity::Line(line) => {
                lines.push(index);
                line.entity
            }
            IfcdrWriteEntity::Polyline { entity, .. } => {
                polylines.push(index);
                *entity
            }
        };
        if let Some(&row) = scope_rows.get(&entity.scope_id) {
            orders[row].entities.push(entity.entity_id);
        }
        // Missing/duplicate scope definitions are diagnosed by shared validation.
    }
    let mut prepared = PreparedIfcdrResource {
        input,
        bounds: None,
        lines,
        polylines,
        orders,
    };
    prepared.bounds = geometric_bounds(&prepared)?;
    Ok(prepared)
}

pub(crate) struct PreparedLines<'a>(&'a PreparedIfcdrResource);
pub(crate) struct PreparedPolylines<'a>(&'a PreparedIfcdrResource);
pub(crate) struct PreparedPolyline<'a> {
    entity: IfcdrEntityRow,
    closed: bool,
    points: &'a [Point2],
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
            IfcdrWriteEntity::Polyline {
                entity,
                closed,
                points,
            } => Some(PreparedPolyline {
                entity: *entity,
                closed: *closed,
                points,
            }),
            _ => None,
        }
    }
}
impl IfcdrPolylineAccess for PreparedPolyline<'_> {
    fn entity(&self) -> IfcdrEntityRow {
        self.entity
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
}
impl IfcdrResourceAccess for PreparedIfcdrResource {
    type Lines<'a> = PreparedLines<'a>;
    type Polylines<'a> = PreparedPolylines<'a>;
    fn resource_id(&self) -> &ResourceId {
        &self.input.resource_id
    }
    fn unit(&self) -> IfcdrLengthUnit {
        self.input.unit
    }
    fn next_entity_id(&self) -> u64 {
        self.input.next_entity_id
    }
    fn bounds(&self) -> Option<Bounds2d> {
        self.bounds
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
    fn polylines(&self) -> PreparedPolylines<'_> {
        PreparedPolylines(self)
    }
    fn orders(&self) -> &[IfcdrScopeOrder] {
        &self.orders
    }
}
