//! Feature facts only; no new geometry preparation or version rules.
use ifccad::{geometry_kernel::hatch::*, *};
use ifccad_convert::opencadcodec::entities::hatch::{Hatch, HatchGradientPattern};
pub(crate) struct HatchExportFacts {
    pub location: String,
    pub unbound: bool,
    pub active_gradient: bool,
    pub inactive_gradient: bool,
    pub description: bool,
    pub solid_classic: bool,
    pub pixel_without_derived: bool,
}
impl HatchExportFacts {
    pub fn cad(h: &Hatch) -> Self {
        Self {
            location: format!("entity/{}", h.common.handle),
            unbound: h.is_associative && h.paths.iter().all(|p| p.boundary_handles.is_empty()),
            active_gradient: h.gradient_color.enabled,
            inactive_gradient: !h.gradient_color.enabled
                && h.gradient_color != HatchGradientPattern::default(),
            description: !h.pattern.description.is_empty(),
            solid_classic: h.is_solid
                && (!h.pattern.lines.is_empty()
                    || h.pattern_angle != 0.
                    || h.pattern_scale != 1.
                    || h.is_double),
            pixel_without_derived: h.pixel_size != 0.
                && !h.paths.iter().any(|p| p.flags.is_derived()),
        }
    }
    fn native(h: &IfccadHatch, location: String) -> Self {
        let solid = match h.fill {
            HatchFill::Solid => true,
            HatchFill::LinePattern(_) => false,
            HatchFill::Gradient => {
                h.classic_settings.as_ref().and_then(|c| c.preferred_fill)
                    != Some(HatchClassicFill::LinePattern)
            }
        };
        Self {
            location,
            unbound: h.association_enabled
                && h.loops.iter().all(|p| p.source_entity_ids.is_empty()),
            active_gradient: matches!(h.fill, HatchFill::Gradient),
            inactive_gradient: !matches!(h.fill, HatchFill::Gradient) && h.gradient.is_some(),
            description: h
                .classic_settings
                .as_ref()
                .is_some_and(|c| c.description.as_ref().is_some_and(|d| !d.is_empty()))
                || matches!(&h.fill,HatchFill::LinePattern(p) if p.description.as_ref().is_some_and(|d|!d.is_empty())),
            solid_classic: solid
                && h.classic_settings.as_ref().is_some_and(|c| {
                    c.rotation != 0.
                        || c.scale != 1.
                        || c.double
                        || c.definition_lines.as_ref().is_some_and(|f| !f.is_empty())
                }),
            pixel_without_derived: h
                .boundary_construction
                .as_ref()
                .is_some_and(|c| c.pixel_size != 0.)
                && !h.loops.iter().any(|p| {
                    p.provenance
                        .as_ref()
                        .is_some_and(|p| p.derived_from_pick_point)
                }),
        }
    }
}
pub(crate) fn native_facts(d: &IfccadDocument) -> Vec<HatchExportFacts> {
    let mut facts = Vec::new();
    for e in d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
        .filter_map(IfccadEntity::as_native)
    {
        let path = format!("/cad/d{}/e{}", d.drawing_id, e.id);
        match &e.kind {
            IfccadEntityKind::Hatch(h) => facts.push(HatchExportFacts::native(h, path)),
            IfccadEntityKind::Dimension(dim) => {
                if let Some(g) = &dim.graphics {
                    for (i, p) in g.parts.iter().enumerate() {
                        if let IfccadDimensionGraphicsKind::Hatch(h) = &p.kind {
                            facts
                                .push(HatchExportFacts::native(h, format!("{path}/graphics/p{i}")));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    facts
}
