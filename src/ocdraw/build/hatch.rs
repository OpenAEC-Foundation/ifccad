use crate::geometry_kernel::hatch::{
    hatch_bounds, validate_hatch_boundaries, HatchAreaRule, HatchFill,
};
use crate::ocdraw::*;

#[derive(Clone, Debug)]
pub struct HatchEntityDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub placement: CoordinateFrame3,
    pub loops: Vec<OcdrawHatchLoop>,
    pub area_rule: HatchAreaRule,
    pub join_tolerance: f64,
    pub fill: HatchFill,
}
impl OcdrawBuilder {
    /// Resolves a creation limit in this builder's existing owner domain.
    pub fn resolve_hatch_join_tolerance(
        &self,
        scope_id: u32,
        request: crate::geometry_kernel::hatch::HatchJoinToleranceRequest,
    ) -> Result<f64, OcdrawBuildError> {
        if scope_id as usize >= 1 + self.paper_layouts.len() + self.block_definitions.len() {
            return Err(OcdrawBuildError::Invalid(
                "Hatch owner scope does not exist".into(),
            ));
        }
        let paper = scope_id > 0 && scope_id as usize <= self.paper_layouts.len();
        let plot = if paper {
            self.layout_settings[scope_id as usize]
                .plot_settings
                .as_ref()
        } else {
            None
        };
        crate::ocdraw::logical::hatch::resolve_context(
            &self.options.unit,
            scope_id,
            paper,
            plot,
            request,
        )
        .map_err(|e| OcdrawBuildError::Invalid(e.to_string()))
    }
    pub fn add_hatch(&mut self, d: HatchEntityDefinition) -> Result<u64, OcdrawBuildError> {
        if d.scope_id as usize >= 1 + self.paper_layouts.len() + self.block_definitions.len()
            || d.layer_id as usize >= self.layers.len()
        {
            return Err(OcdrawBuildError::Invalid(
                "Hatch scope and layer must exist".into(),
            ));
        }
        if !d.appearance.line_pattern_scale.is_finite()
            || d.appearance.line_pattern_scale <= 0.0
            || matches!(d.appearance.line_pattern,AppearanceSelection::Explicit(id)if id.0 as usize>=self.line_patterns.len())
        {
            return Err(OcdrawBuildError::Invalid(
                "invalid Hatch appearance reference or scale".into(),
            ));
        }
        crate::geometry_kernel::hatch::validate_hatch_fill(&d.fill)
            .map_err(|e| OcdrawBuildError::Invalid(e.to_string()))?;
        validate_hatch_boundaries(d.loops.iter().map(|l| &l.boundary), d.join_tolerance)
            .map_err(|e| OcdrawBuildError::Invalid(e.to_string()))?;
        hatch_bounds(d.placement, d.loops.iter().map(|l| &l.boundary))
            .map_err(|e| OcdrawBuildError::Invalid(e.to_string()))?;
        let id = self.next_entity_id;
        let next = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        self.scope_entities.entry(d.scope_id).or_default().push(id);
        self.hatch_entities.push(DrawingHatchEntity {
            id,
            layer_id: d.layer_id,
            visible: d.visible,
            appearance: d.appearance,
            placement: d.placement,
            loops: d.loops,
            area_rule: d.area_rule,
            join_tolerance: d.join_tolerance,
            fill: d.fill,
        });
        self.next_entity_id = next;
        Ok(id)
    }
}
