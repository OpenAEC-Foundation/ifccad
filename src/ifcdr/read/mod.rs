mod codes;
pub(crate) mod decoded;
mod entity;
mod resource;
mod streams;
mod validation;

pub use entity::{
    ArcRef, BlockInstanceRef, CircleRef, EllipseArcRef, EllipseRef, EntityIterator, IfcdrEntityRef,
    PointRef, SpatialPolylineRef, ViewportRef,
};
pub use resource::{
    AppearanceOverrideRef, BlockDefinitionRef, IfcdrResourceRef, ModelSpaceRef, PaperSpaceRef,
    ScopeRef,
};
pub(crate) use resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
pub use streams::{Line, LocalPointIterator, PlanarPolylineRef, PointIterator, ScopePointIterator};
pub(crate) use validation::validate_ifcdr;
