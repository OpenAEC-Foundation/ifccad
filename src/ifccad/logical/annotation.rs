//! Saved scale membership and independent presentation snapshots.

/// Drawing-local scale identity; equal numeric ratios do not merge identities.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct IfccadAnnotationScaleId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAnnotationScale {
    pub id: IfccadAnnotationScaleId,
    pub name: String,
    pub paper_units: f64,
    pub drawing_units: f64,
    pub unit_scale: bool,
}

impl IfccadAnnotationScale {
    /// Requested size factor q(selected)/q(reference), without changing stored units.
    /// Exact intermediate arithmetic avoids overflow of individually extreme q's.
    pub fn size_ratio_to(&self, selected: &Self) -> Result<f64, super::IfccadReport> {
        use num_rational::BigRational;
        use num_traits::ToPrimitive;
        let values = [
            self.paper_units,
            self.drawing_units,
            selected.paper_units,
            selected.drawing_units,
        ];
        let error = |detail: &str| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-003",
                "/annotationScale/sizeRatio",
                detail,
            )
        };
        if values
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(error("annotation units must be positive and finite"));
        }
        let [reference_paper, reference_drawing, selected_paper, selected_drawing] =
            values.map(|value| BigRational::from_float(value).expect("checked finite units"));
        let ratio = selected_drawing * reference_paper / (selected_paper * reference_drawing);
        ratio
            .to_f64()
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| error("requested annotation size ratio is not representable"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadAnnotationMode {
    Disabled,
    Enabled,
    ByDefinition,
}

/// Scale membership key, independent of Hatch's separate view-context identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadAnnotationContextKey {
    pub scale_id: IfccadAnnotationScaleId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAnnotation<P> {
    /// Explicit object orientation; absence is unspecified, independent of style intent.
    pub orientation: Option<IfccadAnnotationOrientation>,
    pub mode: IfccadAnnotationMode,
    /// Source default in this scale collection, without overwriting base geometry.
    pub reference: Option<IfccadAnnotationContextKey>,
    pub contexts: Vec<IfccadAnnotationContext<P>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAnnotationContext<P> {
    pub key: IfccadAnnotationContextKey,
    /// None retains membership without fabricating a generated presentation.
    pub presentation: Option<P>,
}

/// Validate drawing-local scale names/identities, saved units and allocation state.
/// Individual drawing/paper quotients need not fit in binary64.
pub fn validate_ifccad_annotation_scale_registry(
    scales: &[IfccadAnnotationScale],
    next_scale_id: u64,
) -> Result<(), super::IfccadReport> {
    use std::collections::BTreeSet;
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for scale in scales {
        let path = format!("/annotationScale/{}", scale.id.0);
        if !ids.insert(scale.id)
            || scale.name.trim().is_empty()
            || !names.insert(crate::names::name_key(&scale.name))
            || [scale.paper_units, scale.drawing_units]
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-001",
                &path,
                "invalid or duplicate annotation scale",
            ));
        }
    }
    if next_scale_id == 0 || scales.iter().any(|scale| scale.id.0 >= next_scale_id) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-ANNOTATION-002",
            "/allocation/nextAnnotationScaleId",
            "annotation scale watermark must exceed all saved IDs",
        ));
    }
    Ok(())
}

impl<P> IfccadAnnotation<P> {
    /// Shared membership checks; presentation and owning-family checks are separate.
    /// Only Dimension and Block owners permit definition-inherited annotation mode.
    pub fn validate_memberships(
        &self,
        scales: &[IfccadAnnotationScale],
        allow_by_definition: bool,
    ) -> Result<(), super::IfccadReport> {
        use std::collections::BTreeSet;
        let invalid = |field: &str, detail: &str| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-004",
                &format!("/ifccad::annotation/{field}"),
                detail,
            )
        };
        if self.mode == IfccadAnnotationMode::ByDefinition && !allow_by_definition {
            return Err(invalid(
                "mode",
                "definition inheritance is invalid for this family",
            ));
        }
        let known: BTreeSet<_> = scales.iter().map(|scale| scale.id).collect();
        let mut keys = BTreeSet::new();
        for (index, context) in self.contexts.iter().enumerate() {
            if !known.contains(&context.key.scale_id) || !keys.insert(context.key) {
                return Err(invalid(
                    &format!("contexts/{index}/key"),
                    "duplicate or unresolved scale membership",
                ));
            }
        }
        if self
            .reference
            .is_some_and(|reference| !keys.contains(&reference))
        {
            return Err(invalid(
                "reference",
                "default must resolve within this scale collection",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadAnnotationSelection<'a, P> {
    Disabled,
    SelectionUnspecified,
    NoStoredContexts,
    ScaleNotAssociated,
    MembershipOnly(&'a IfccadAnnotationContextKey),
    Available(&'a IfccadAnnotationContext<P>),
}

/// Select an exact stored scale snapshot without editing or generating anything.
///
/// `enabled` is the caller's resolved entity/definition state. Selection never
/// chooses the reference as a fallback, substitutes an equal ratio, or evaluates
/// application layout. Context-key uniqueness is part of shared validation.
pub fn select_ifccad_annotation_context<'a, P>(
    annotation: Option<&'a IfccadAnnotation<P>>,
    enabled: bool,
    requested_key: Option<&IfccadAnnotationContextKey>,
) -> IfccadAnnotationSelection<'a, P> {
    if !enabled {
        return IfccadAnnotationSelection::Disabled;
    }
    let Some(key) = requested_key else {
        return IfccadAnnotationSelection::SelectionUnspecified;
    };
    let Some(annotation) = annotation.filter(|annotation| !annotation.contexts.is_empty()) else {
        return IfccadAnnotationSelection::NoStoredContexts;
    };
    match annotation
        .contexts
        .iter()
        .find(|context| context.key == *key)
    {
        None => IfccadAnnotationSelection::ScaleNotAssociated,
        Some(context) if context.presentation.is_none() => {
            IfccadAnnotationSelection::MembershipOnly(&context.key)
        }
        Some(context) => IfccadAnnotationSelection::Available(context),
    }
}

/// Definition intent is independent of existing entity memberships.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadAnnotationDefinition {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub orientation: IfccadAnnotationOrientation,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadAnnotationOrientation {
    #[default]
    Fixed,
    MatchLayout,
}

/// A complete saved occurrence transform, replacing rather than multiplying the base.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadBlockAnnotationContext {
    pub transform: super::IfccadBlockTransform,
}
