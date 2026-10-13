//! CAD gradient authorship and qualification, independent of native colors/IDs.
use super::{unsupported, CadHatchPreparationError, HatchSourceLoss};
use ocdraw::geometry_kernel::{hatch::*, CoordinateFrame3};
use opencadcodec::{entities::hatch::*, types::Matrix3, Color};

pub struct PreparedNativeGradient {
    pub gradient: Option<HatchGradient<Color>>,
    pub losses: Vec<HatchSourceLoss>,
}

fn kind(name: &str) -> Option<HatchGradientType> {
    use HatchGradientType::*;
    Some(match name {
        "LINEAR" => Linear,
        "CYLINDER" => Cylinder,
        "INVCYLINDER" => InverseCylinder,
        "SPHERICAL" => Spherical,
        "INVSPHERICAL" => InverseSpherical,
        "HEMISPHERICAL" => Hemispherical,
        "INVHEMISPHERICAL" => InverseHemispherical,
        "CURVED" => Curved,
        "INVCURVED" => InverseCurved,
        _ => return None,
    })
}
fn name(kind: HatchGradientType) -> &'static str {
    use HatchGradientType::*;
    match kind {
        Linear => "LINEAR",
        Cylinder => "CYLINDER",
        InverseCylinder => "INVCYLINDER",
        Spherical => "SPHERICAL",
        InverseSpherical => "INVSPHERICAL",
        Hemispherical => "HEMISPHERICAL",
        InverseHemispherical => "INVHEMISPHERICAL",
        Curved => "CURVED",
        InverseCurved => "INVCURVED",
    }
}
fn concrete(c: &Color) -> bool {
    matches!(c, Color::Rgb { .. } | Color::Index(1..=6 | 8..=255))
}
fn unqualified(
    h: &Hatch,
    field: &'static str,
    detail: &str,
) -> Result<PreparedNativeGradient, CadHatchPreparationError> {
    if h.gradient_color.enabled {
        Err(unsupported(field, detail))
    } else {
        Ok(PreparedNativeGradient {
            gradient: None,
            losses: vec![HatchSourceLoss {
                field,
                detail: detail.into(),
            }],
        })
    }
}

pub fn prepare_gradient_from_cad(
    h: &Hatch,
) -> Result<PreparedNativeGradient, CadHatchPreparationError> {
    let p = &h.gradient_color;
    let mut g = HatchGradient {
        gradient_type: None,
        colors: if p.colors.len() == 2 {
            Some([p.colors[0].color, p.colors[1].color])
        } else {
            None
        },
        rotation: p.angle,
        shift: p.shift,
        tint: p.color_tint,
        color_mode: if p.is_single_color {
            HatchGradientColorMode::OneColor
        } else {
            HatchGradientColorMode::TwoColor
        },
    };
    // First validate known scalar rules; completeness and exposed color methods
    // are classified below. EmptyDormantState alone may be the codec's default.
    if let Err(error) = validate_hatch_gradient(false, Some(&g), |_| true) {
        if error.reason != HatchGradientFailureReason::EmptyDormantState {
            return Err(error.into());
        }
    }
    for entry in &p.colors {
        if !entry.value.is_finite() {
            return Err(CadHatchPreparationError::InvalidGradientSource {
                field: "gradient.colors.value",
                detail: "endpoint value must be finite".into(),
            });
        }
    }
    if p.colors.len() == 1 {
        return Err(CadHatchPreparationError::InvalidGradientSource {
            field: "gradient.colors",
            detail: "incomplete endpoint pair".into(),
        });
    }
    if p.reserved != 0 {
        return unqualified(
            h,
            "gradient.reserved",
            "reserved gradient meaning is unqualified",
        );
    }
    if !p.name.is_empty() {
        g.gradient_type = kind(&p.name);
        if g.gradient_type.is_none() {
            return unqualified(h, "gradient.type", "unknown gradient definition");
        }
    }
    if p.colors.len() > 2 {
        return unqualified(
            h,
            "gradient.colors",
            "more than two endpoints are unqualified",
        );
    }
    if p.colors.len() == 2 && (p.colors[0].value != 0. || p.colors[1].value != 1.) {
        return unqualified(
            h,
            "gradient.colors.value",
            "endpoint positions must be 0 and 1",
        );
    }
    if p.colors.iter().any(|e| !concrete(&e.color)) {
        return unqualified(
            h,
            "gradient.colors",
            "gradient requires concrete RGB or fixed ACI colors; symbolic ACI 7 is unqualified",
        );
    }
    if !p.enabled
        && g.gradient_type.is_none()
        && g.colors.is_none()
        && g.rotation == 0.
        && g.shift == 0.
        && g.tint == 0.
        && g.color_mode == HatchGradientColorMode::TwoColor
    {
        return Ok(PreparedNativeGradient {
            gradient: None,
            losses: vec![],
        });
    }
    validate_hatch_gradient(p.enabled, Some(&g), concrete)?;
    Ok(PreparedNativeGradient {
        gradient: Some(g),
        losses: vec![],
    })
}

pub fn apply_gradient_to_cad(
    h: &mut Hatch,
    placement: CoordinateFrame3,
    fill: &HatchFill,
    gradient: Option<&HatchGradient<Color>>,
) -> Result<Vec<HatchSourceLoss>, CadHatchPreparationError> {
    let active = matches!(fill, HatchFill::Gradient);
    validate_hatch_gradient(active, gradient, concrete)?;
    let Some(g) = gradient else {
        h.gradient_color = HatchGradientPattern::default();
        return Ok(vec![]);
    };
    let basis = Matrix3::arbitrary_axis(h.normal).m;
    let x = placement.x_axis();
    let y = placement.y_axis();
    let nx = [x.x(), x.y(), x.z()];
    let ny = [y.x(), y.y(), y.z()];
    if (0..3).any(|i| nx[i] != basis[i][0] || ny[i] != basis[i][1]) {
        if active {
            return Err(unsupported(
                "gradient.frame",
                "HATCH_GRADIENT_FRAME_UNSUPPORTED: local XY basis differs from target OCS",
            ));
        }
        h.gradient_color = HatchGradientPattern::default();
        return Ok(vec![HatchSourceLoss{field:"gradient.frame",detail:"HATCH_GRADIENT_FRAME_UNSUPPORTED: dormant settings omitted because local XY basis differs from target OCS".into()}]);
    }
    h.gradient_color = HatchGradientPattern {
        enabled: active,
        reserved: 0,
        angle: g.rotation,
        shift: g.shift,
        is_single_color: g.color_mode == HatchGradientColorMode::OneColor,
        color_tint: g.tint,
        name: g.gradient_type.map(name).unwrap_or("").into(),
        colors: g
            .colors
            .map(|cs| {
                vec![
                    GradientColorEntry {
                        value: 0.,
                        color: cs[0],
                    },
                    GradientColorEntry {
                        value: 1.,
                        color: cs[1],
                    },
                ]
            })
            .unwrap_or_default(),
    };
    if active {
        h.is_solid = true;
        h.pattern = Hatch::solid().pattern;
        h.pattern_type = HatchPatternType::Predefined;
        h.pattern_angle = 0.;
        h.pattern_scale = 1.;
        h.is_double = false;
    }
    Ok(vec![])
}
