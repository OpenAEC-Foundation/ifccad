mod encoder;

use crate::ifcdr::{AppearanceId, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId};
use crate::ResourceId;

pub(crate) use encoder::{encode, EncodedIfcdrResource, IfcdrEncodeError};

pub(crate) struct IfcdrEncodeInput<'a> {
    pub resource_id: &'a ResourceId,
    pub unit: IfcdrLengthUnit,
    pub scope: IfcdrScopeInput<'a>,
    pub layers: &'a [IfcdrLayerBindingInput<'a>],
    pub appearances: &'a [IfcdrAppearanceBindingInput<'a>],
    pub entities: &'a [IfcdrEntityInput<'a>],
}

pub(crate) struct IfcdrScopeInput<'a> {
    pub id: ScopeId,
    pub kind: u32,
    pub name: &'a str,
    pub base: Point2,
    pub flags: u32,
}

pub(crate) struct IfcdrLayerBindingInput<'a> {
    pub id: LayerId,
    pub ifcx_path: &'a str,
}

pub(crate) struct IfcdrAppearanceBindingInput<'a> {
    pub id: AppearanceId,
    pub ifcx_path: &'a str,
}

pub(crate) enum IfcdrEntityInput<'a> {
    Line {
        entity_id: EntityId,
        start: Point2,
        end: Point2,
        layer_id: LayerId,
        appearance_id: AppearanceId,
        visible: bool,
    },
    Polyline {
        entity_id: EntityId,
        points: &'a [Point2],
        closed: bool,
        layer_id: LayerId,
        appearance_id: AppearanceId,
        visible: bool,
    },
}
