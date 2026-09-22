use crate::ResourceId;

pub(crate) const IFCCAD_IFCDR_ENTITY_ID_INVALID: &str = "IFCCAD_IFCDR_ENTITY_ID_INVALID";
pub(crate) const IFCCAD_IFCDR_ENTITY_ID_DUPLICATE: &str = "IFCCAD_IFCDR_ENTITY_ID_DUPLICATE";
pub(crate) const IFCCAD_IFCDR_REFERENCE_MISSING: &str = "IFCCAD_IFCDR_REFERENCE_MISSING";
pub(crate) const IFCCAD_IFCDR_ENTITY_ORDER_INVALID: &str = "IFCCAD_IFCDR_ENTITY_ORDER_INVALID";
pub(crate) const IFCCAD_IFCDR_BOUNDS_INVALID: &str = "IFCCAD_IFCDR_BOUNDS_INVALID";
pub(crate) const IFCCAD_IFCDR_GEOMETRY_INVALID: &str = "IFCCAD_IFCDR_GEOMETRY_INVALID";
pub(crate) const IFCCAD_IFCDR_POLYLINE_INVALID: &str = "IFCCAD_IFCDR_POLYLINE_INVALID";
pub(crate) const IFCCAD_IFCDR_APPEARANCE_INVALID: &str = "IFCCAD_IFCDR_APPEARANCE_INVALID";
pub(crate) const IFCCAD_IFCDR_STRUCTURE_INVALID: &str = "IFCCAD_IFCDR_STRUCTURE_INVALID";
pub(crate) const IFCCAD_IFCDR_SCOPE_INVALID: &str = "IFCCAD_IFCDR_SCOPE_INVALID";
pub(crate) const IFCCAD_IFCDR_BLOCK_INVALID: &str = "IFCCAD_IFCDR_BLOCK_INVALID";
pub(crate) const IFCCAD_IFCDR_BLOCK_CYCLE: &str = "IFCCAD_IFCDR_BLOCK_CYCLE";
pub(crate) const IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE: &str =
    "IFCCAD_IFCDR_NUMERICAL_PROOF_INCOMPLETE";

#[derive(Clone, Debug)]
pub(crate) struct IfcdrDiagnostic {
    pub category: crate::diagnostic::PackageDiagnosticCategory,
    pub code: &'static str,
    pub resource_id: ResourceId,
    pub collection: &'static str,
    pub row: Option<usize>,
    pub property: &'static str,
    pub message: String,
}
