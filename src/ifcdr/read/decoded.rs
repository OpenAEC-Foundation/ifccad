//! Typed column backing. JSON construction belongs to the codec.
use crate::ifcdr::logical::*;
use crate::ifcdr::{Bounds2d, IfcdrLengthUnit, Point2};
use crate::ResourceId;

#[derive(Debug, Default)]
pub(crate) struct EntityColumns {
    pub ids: Vec<u64>,
    pub scopes: Vec<u32>,
    pub layers: Vec<u32>,
    pub appearances: Vec<u32>,
    pub visible: Vec<bool>,
}
impl EntityColumns {
    pub(crate) fn get(&self, row: usize) -> Option<IfcdrEntityRow> {
        Some(IfcdrEntityRow {
            entity_id: *self.ids.get(row)?,
            scope_id: *self.scopes.get(row)?,
            layer_id: *self.layers.get(row)?,
            appearance_id: *self.appearances.get(row)?,
            visible: *self.visible.get(row)?,
        })
    }
}
#[derive(Debug, Default)]
pub(crate) struct LineColumns {
    pub entity: EntityColumns,
    pub x1: Vec<f64>,
    pub y1: Vec<f64>,
    pub x2: Vec<f64>,
    pub y2: Vec<f64>,
}
#[derive(Debug, Default)]
pub(crate) struct PolylineColumns {
    pub entity: EntityColumns,
    pub offsets: Vec<usize>,
    pub counts: Vec<usize>,
    pub closed: Vec<bool>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}
#[derive(Debug)]
pub(crate) struct DecodedIfcdrResource {
    pub id: ResourceId,
    pub unit: IfcdrLengthUnit,
    pub next: u64,
    pub bounds: Option<Bounds2d>,
    pub scopes: Vec<IfcdrScope>,
    pub layers: Vec<IfcdrLayerBinding>,
    pub appearances: Vec<IfcdrAppearanceBinding>,
    pub overrides: Vec<IfcdrAppearanceOverride>,
    pub orders: Vec<IfcdrScopeOrder>,
    pub lines: LineColumns,
    pub polylines: PolylineColumns,
}
#[derive(Clone, Copy)]
pub(crate) struct DecodedLines<'a>(pub &'a LineColumns);
#[derive(Clone, Copy)]
pub(crate) struct DecodedPolylines<'a>(pub &'a PolylineColumns);
#[derive(Clone, Copy)]
pub(crate) struct DecodedPolyline<'a> {
    pub columns: &'a PolylineColumns,
    pub row: usize,
}
impl IfcdrLinesAccess for DecodedLines<'_> {
    fn len(&self) -> usize {
        self.0.entity.ids.len()
    }
    fn get(&self, row: usize) -> Option<IfcdrLineRow> {
        Some(IfcdrLineRow {
            entity: self.0.entity.get(row)?,
            start: Point2::new(*self.0.x1.get(row)?, *self.0.y1.get(row)?),
            end: Point2::new(*self.0.x2.get(row)?, *self.0.y2.get(row)?),
        })
    }
}
impl IfcdrPolylinesAccess for DecodedPolylines<'_> {
    type Polyline<'a>
        = DecodedPolyline<'a>
    where
        Self: 'a;
    fn len(&self) -> usize {
        self.0.entity.ids.len()
    }
    fn get(&self, row: usize) -> Option<DecodedPolyline<'_>> {
        self.0.entity.get(row)?;
        self.0.offsets.get(row)?;
        self.0.counts.get(row)?;
        self.0.closed.get(row)?;
        Some(DecodedPolyline {
            columns: self.0,
            row,
        })
    }
}
impl IfcdrPolylineAccess for DecodedPolyline<'_> {
    fn entity(&self) -> IfcdrEntityRow {
        self.columns.entity.get(self.row).expect("checked row view")
    }
    fn closed(&self) -> bool {
        self.columns.closed[self.row]
    }
    fn vertex_count(&self) -> usize {
        self.columns.counts[self.row]
    }
    fn vertex(&self, index: usize) -> Option<Point2> {
        if index >= self.vertex_count() {
            return None;
        }
        let i = self.columns.offsets[self.row].checked_add(index)?;
        Some(Point2::new(
            *self.columns.x.get(i)?,
            *self.columns.y.get(i)?,
        ))
    }
}
impl IfcdrResourceAccess for DecodedIfcdrResource {
    type Lines<'a> = DecodedLines<'a>;
    type Polylines<'a> = DecodedPolylines<'a>;
    fn resource_id(&self) -> &ResourceId {
        &self.id
    }
    fn unit(&self) -> IfcdrLengthUnit {
        self.unit
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
    fn lines(&self) -> DecodedLines<'_> {
        DecodedLines(&self.lines)
    }
    fn polylines(&self) -> DecodedPolylines<'_> {
        DecodedPolylines(&self.polylines)
    }
    fn orders(&self) -> &[IfcdrScopeOrder] {
        &self.orders
    }
}
