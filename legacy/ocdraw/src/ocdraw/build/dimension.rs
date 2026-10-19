use crate::{dimension::*, ocdraw::*};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct DimensionStyleDefinition {
    pub name: String,
    pub settings: OcdrawDimensionSettings,
    pub type_overrides: BTreeMap<DimensionKind, OcdrawDimensionOverrides>,
}
impl DimensionStyleDefinition {
    pub fn new(name: impl Into<String>, text_style: OcdrawTextStyleId) -> Self {
        Self {
            name: name.into(),
            settings: DimensionStyleSettings::new(text_style),
            type_overrides: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct DimensionEntityDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub placement: CoordinateFrame3,
    pub style_id: OcdrawDimensionStyleId,
    pub definition: DimensionDefinition,
    pub text_placement: DimensionTextPlacement,
    pub label: DimensionLabel,
    pub overrides: OcdrawDimensionOverrides,
    pub associations: Vec<OcdrawDimensionAssociation>,
}
impl DimensionEntityDefinition {
    pub fn new(
        layer_id: u32,
        style_id: OcdrawDimensionStyleId,
        definition: DimensionDefinition,
    ) -> Self {
        Self {
            scope_id: 0,
            layer_id,
            style_id,
            definition,
            visible: true,
            appearance: Default::default(),
            placement: Default::default(),
            text_placement: Default::default(),
            label: Default::default(),
            overrides: Default::default(),
            associations: Vec::new(),
        }
    }
    pub fn in_scope(mut self, scope_id: u32) -> Self {
        self.scope_id = scope_id;
        self
    }
}
impl OcdrawBuilder {
    fn check_dimension_settings(
        &self,
        v: &OcdrawDimensionSettings,
    ) -> Result<(), OcdrawBuildError> {
        let block_first = 1 + self.paper_layouts.len();
        let block_end = block_first + self.block_definitions.len();
        let errors = super::super::logical::dimension_validation::settings_in_context(
            "/dimension/settings",
            v,
            |id| self.text_styles.iter().any(|s| s.id == id),
            |id| (block_first..block_end).contains(&(id as usize)),
            |id| (id.0 as usize) < self.line_patterns.len(),
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(OcdrawBuildError::Invalid(format!("{errors:?}")))
        }
    }
    pub fn add_dimension_style(
        &mut self,
        d: DimensionStyleDefinition,
    ) -> Result<OcdrawDimensionStyleId, OcdrawBuildError> {
        let id = OcdrawDimensionStyleId(self.next_dimension_style_id);
        let next = id.0.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        if d.name.is_empty()
            || d.name.contains('\0')
            || self.dimension_styles.iter().any(|s| {
                crate::ocdraw::names::name_key(&s.name) == crate::ocdraw::names::name_key(&d.name)
            })
        {
            return Err(OcdrawBuildError::Invalid(
                "invalid or duplicate DimensionStyle name".into(),
            ));
        }
        self.check_dimension_settings(&d.settings)?;
        for overrides in d.type_overrides.values() {
            let mut effective = d.settings.clone();
            overrides.apply(&mut effective);
            self.check_dimension_settings(&effective)?;
        }
        self.dimension_styles.push(DrawingDimensionStyle {
            id,
            name: d.name,
            settings: d.settings,
            type_overrides: d.type_overrides,
        });
        self.next_dimension_style_id = next;
        Ok(id)
    }
    pub fn add_dimension(&mut self, d: DimensionEntityDefinition) -> Result<u64, OcdrawBuildError> {
        if d.scope_id as usize >= 1 + self.paper_layouts.len() + self.block_definitions.len()
            || d.layer_id as usize >= self.layers.len()
        {
            return Err(OcdrawBuildError::Invalid(
                "Dimension owner/layer must exist".into(),
            ));
        }
        let id = self.next_entity_id;
        let next = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        let style = self
            .dimension_styles
            .iter()
            .find(|s| s.id == d.style_id)
            .ok_or_else(|| OcdrawBuildError::Invalid("missing DimensionStyle".into()))?;
        let effective = resolve_dimension_settings(
            &style.settings,
            d.definition.kind(),
            &style.type_overrides,
            &d.overrides,
        );
        self.check_dimension_settings(&effective)?;
        let scope = d.scope_id;
        let e = DrawingDimensionEntity {
            id,
            layer_id: d.layer_id,
            visible: d.visible,
            appearance: d.appearance,
            placement: d.placement,
            style_id: d.style_id,
            definition: d.definition,
            text_placement: d.text_placement,
            label: d.label,
            overrides: d.overrides,
            associations: d.associations,
        };
        let errors = super::super::logical::dimension_validation::entity_values(&e);
        if !errors.is_empty() {
            return Err(OcdrawBuildError::Invalid(format!("{errors:?}")));
        }
        if !e.appearance.line_pattern_scale.is_finite()
            || e.appearance.line_pattern_scale <= 0.0
            || matches!(e.appearance.line_pattern,AppearanceSelection::Explicit(p) if p.0 as usize>=self.line_patterns.len())
        {
            return Err(OcdrawBuildError::Invalid(
                "invalid Dimension pattern/scale".into(),
            ));
        }
        // Source binding is final-document validation, allowing forward references.
        self.dimension_entities.push(e);
        self.scope_entities.entry(scope).or_default().push(id);
        self.next_entity_id = next;
        Ok(id)
    }
}
