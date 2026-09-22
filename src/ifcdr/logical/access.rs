use super::types::*;
use crate::ifcdr::{IfcdrLengthUnit, Point2};
use crate::ResourceId;

pub(crate) trait IfcdrResourceAccess {
    type Lines<'a>: IfcdrLinesAccess
    where
        Self: 'a;
    type Polylines<'a>: IfcdrPolylinesAccess
    where
        Self: 'a;
    type BlockInstances<'a>: IfcdrBlockInstancesAccess
    where
        Self: 'a;
    fn resource_id(&self) -> &ResourceId;
    fn unit(&self) -> IfcdrLengthUnit;
    fn next_entity_id(&self) -> u64;
    fn scopes(&self) -> &[IfcdrScope];
    fn block_definitions(&self) -> &[IfcdrBlockDefinition];
    fn layers(&self) -> &[IfcdrLayerBinding];
    fn appearances(&self) -> &[IfcdrAppearanceBinding];
    fn overrides(&self) -> &[IfcdrAppearanceOverride];
    fn lines(&self) -> Self::Lines<'_>;
    fn polylines(&self) -> Self::Polylines<'_>;
    fn block_instances(&self) -> Self::BlockInstances<'_>;
    fn orders(&self) -> &[IfcdrScopeOrder];
    fn order(&self, scope: u32) -> Option<&[u64]> {
        self.orders()
            .iter()
            .find(|o| o.scope_id == scope)
            .map(|o| o.entities.as_slice())
    }
}
pub(crate) trait IfcdrBlockInstancesAccess {
    fn len(&self) -> usize;
    fn get(&self, row: usize) -> Option<IfcdrBlockInstanceRow>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl IfcdrBlockInstancesAccess for &[IfcdrBlockInstanceRow] {
    fn len(&self) -> usize {
        <[IfcdrBlockInstanceRow]>::len(self)
    }
    fn get(&self, row: usize) -> Option<IfcdrBlockInstanceRow> {
        <[IfcdrBlockInstanceRow]>::get(self, row).copied()
    }
}
pub(crate) trait IfcdrLinesAccess {
    fn len(&self) -> usize;
    fn get(&self, row: usize) -> Option<IfcdrLineRow>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub(crate) trait IfcdrPolylinesAccess {
    type Polyline<'a>: IfcdrPolylineAccess
    where
        Self: 'a;
    fn len(&self) -> usize;
    fn get(&self, row: usize) -> Option<Self::Polyline<'_>>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub(crate) trait IfcdrPolylineAccess {
    fn entity(&self) -> IfcdrEntityRow;
    fn closed(&self) -> bool;
    fn placement(&self) -> crate::ifcdr::geometry::PlanePlacementComponents;
    fn vertex_count(&self) -> usize;
    fn vertex(&self, index: usize) -> Option<Point2>;
}
