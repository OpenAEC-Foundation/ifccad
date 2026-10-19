//! Physical verification against the already mapped target model.
use ifccad_convert::opencadcodec::{
    entities::hatch::{Hatch, HatchGradientPattern},
    CadDocument, Color, DxfVersion, EntityType,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(crate) struct HatchExchangeIssue {
    pub code: &'static str,
    pub location: String,
    pub expected: Value,
    pub actual: Value,
    pub message: String,
}
impl HatchExchangeIssue {
    pub(crate) fn value(self) -> Value {
        json!({"code":self.code,"location":self.location,"expected":self.expected,
            "actual":self.actual,"message":self.message,"phase":"cadExchange","action":"Modified"})
    }
}
fn meaningful(h: &Hatch) -> bool {
    h.gradient_color != HatchGradientPattern::default()
}
pub(crate) fn check_gradient_target_version(
    d: &CadDocument,
    version: DxfVersion,
) -> Result<(), String> {
    if version != DxfVersion::AC1032
        && d.entities()
            .any(|e| matches!(e,EntityType::Hatch(h) if meaningful(h)))
    {
        Err("Gradient export is qualified only for AC1032; choose that version to retain gradient settings".into())
    } else {
        Ok(())
    }
}
fn groups(d: &CadDocument) -> Result<BTreeMap<String, Vec<&Hatch>>, String> {
    let mut scopes = BTreeMap::<String, Vec<&Hatch>>::new();
    for e in d.entities() {
        if let EntityType::Hatch(h) = e {
            let owner = d
                .block_records
                .iter()
                .find(|b| b.handle == h.common.owner_handle)
                .ok_or_else(|| "Hatch owner could not be confirmed".to_string())?;
            let key = if owner.is_model_space() {
                "Model".to_string()
            } else {
                format!("block/{}", owner.name)
            };
            scopes.entry(key).or_default().push(h);
        }
    }
    Ok(scopes)
}
fn fields(g: &HatchGradientPattern) -> Value {
    let colors: Vec<_> = g
        .colors
        .iter()
        .map(|e| {
            let color = match e.color {
                Color::Rgb { r, g, b } => json!({"rgb":[r,g,b]}),
                Color::Index(i) => json!({"indexedColor":{"system":"ACI","index":i}}),
                other => json!({"method":format!("{other:?}")}),
            };
            json!({"value":e.value,"color":color})
        })
        .collect();
    json!({"enabled":g.enabled,"type":g.name,"colors":colors,"rotation":g.angle,
        "shift":g.shift,"colorMode":g.is_single_color,"tint":g.color_tint,"reserved":g.reserved})
}
pub(crate) fn check_hatch_exchange(
    expected: &CadDocument,
    actual: &CadDocument,
) -> Result<Vec<HatchExchangeIssue>, String> {
    let a = groups(expected)?;
    let b = groups(actual)?;
    let mut issues = Vec::new();
    for (scope, hatches) in &a {
        let empty = Vec::new();
        let back = b.get(scope).unwrap_or(&empty);
        if hatches.len() != back.len() {
            return Err(format!(
                "Hatch count changed in {scope}; correspondence is unconfirmed"
            ));
        }
        for (index, (wanted, read)) in hatches.iter().zip(back).enumerate() {
            // These are emitted target identities within this one file exchange,
            // not original native IDs or a promise of stable conversion handles.
            if wanted.common.handle != read.common.handle {
                return Err(format!(
                    "Hatch identity/order changed in {scope}; correspondence is unconfirmed"
                ));
            }
            let base = format!("{scope}/hatches/{index}");
            if wanted.paths.len() != read.paths.len() {
                return Err(format!(
                    "Hatch loop count changed at {base}; correspondence is unconfirmed"
                ));
            }
            if wanted.is_associative != read.is_associative {
                issues.push(HatchExchangeIssue {
                    code: "TARGET_CODEC_HATCH_ASSOCIATION_STATE_LOSS",
                    location: format!("{base}/associationEnabled"),
                    expected: json!(wanted.is_associative),
                    actual: json!(read.is_associative),
                    message: "The CAD file changed the Hatch association switch".into(),
                });
            }
            for (loop_index, (w, r)) in wanted.paths.iter().zip(&read.paths).enumerate() {
                let path = format!("{base}/loops/{loop_index}");
                if w.edges.len() != r.edges.len()
                    || (w.edges != r.edges
                        && wanted
                            .paths
                            .iter()
                            .enumerate()
                            .any(|(j, p)| j != loop_index && p.edges == r.edges))
                {
                    return Err(format!(
                        "Hatch loop shape/order changed at {path}; correspondence is unconfirmed"
                    ));
                }
                for handle in w.boundary_handles.iter().chain(&r.boundary_handles) {
                    let original = expected.get_entity(*handle).ok_or_else(|| {
                        format!("Expected Hatch source could not be confirmed at {path}")
                    })?;
                    let back = actual.get_entity(*handle).ok_or_else(|| {
                        format!("Readback Hatch source could not be confirmed at {path}")
                    })?;
                    if original.common().owner_handle != wanted.common.owner_handle
                        || back.common().owner_handle != read.common.owner_handle
                        || original.as_entity().entity_type() != back.as_entity().entity_type()
                    {
                        return Err(format!(
                            "Hatch source owner/kind correspondence changed at {path}"
                        ));
                    }
                }
                if w.boundary_handles != r.boundary_handles {
                    if !r.boundary_handles.is_empty() {
                        return Err(format!("The CAD file changed a nonempty source group at {path}; partial or substituted group delivery is unqualified"));
                    }
                    issues.push(HatchExchangeIssue {
                        code: "TARGET_CODEC_HATCH_SOURCE_GROUP_LOSS",
                        location: format!("{path}/sources"),
                        expected: json!(w
                            .boundary_handles
                            .iter()
                            .map(|h| h.to_string())
                            .collect::<Vec<_>>()),
                        actual: json!(r
                            .boundary_handles
                            .iter()
                            .map(|h| h.to_string())
                            .collect::<Vec<_>>()),
                        message: "The CAD file changed a complete Hatch source group".into(),
                    });
                }
                let wb = w.flags.bits();
                let rb = r.flags.bits();
                if rb & !0x9f != 0 {
                    return Err(format!("Unqualified readback Hatch flags at {path}"));
                }
                if (wb & 128) != (rb & 128) {
                    issues.push(HatchExchangeIssue {
                        code: "TARGET_CODEC_HATCH_TEXT_ISLAND_LOSS",
                        location: format!("{path}/textIsland"),
                        expected: json!(wb & 128 != 0),
                        actual: json!(rb & 128 != 0),
                        message: "The CAD file changed active text-island fill semantics".into(),
                    });
                }
                for (bit, field) in [
                    (1, "fromSourceObjects"),
                    (4, "derivedFromPickPoint"),
                    (8, "textBox"),
                    (16, "outermostHint"),
                ] {
                    if wb & bit != rb & bit {
                        issues.push(HatchExchangeIssue {
                            code: "TARGET_CODEC_HATCH_LOOP_PROVENANCE_LOSS",
                            location: format!("{path}/provenance/{field}"),
                            expected: json!(wb & bit != 0),
                            actual: json!(rb & bit != 0),
                            message: format!("The CAD file changed loop {field}"),
                        });
                    }
                }
            }
            let path = format!("{scope}/hatches/{index}/gradient");
            if !wanted.gradient_color.enabled && meaningful(wanted) && !meaningful(read) {
                issues.push(HatchExchangeIssue {
                    code: "TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS",
                    location: path,
                    expected: fields(&wanted.gradient_color),
                    actual: Value::Null,
                    message: "Inactive gradient settings were not retained by the target CAD file"
                        .into(),
                });
                continue;
            }
            let x = fields(&wanted.gradient_color);
            let y = fields(&read.gradient_color);
            for (field, value) in x.as_object().expect("gradient fields") {
                if value != &y[field] {
                    issues.push(HatchExchangeIssue {
                        code: "TARGET_CODEC_HATCH_GRADIENT_PARAMETER_LOSS",
                        location: format!("{path}/{field}"),
                        expected: value.clone(),
                        actual: y[field].clone(),
                        message: format!("The CAD file changed gradient {field}"),
                    });
                }
            }
        }
    }
    for (scope, hatches) in b {
        if !a.contains_key(&scope) && !hatches.is_empty() {
            return Err(format!(
                "Unexpected Hatch scope {scope}; correspondence is unconfirmed"
            ));
        }
    }
    Ok(issues)
}

pub(crate) fn check_association_target_version(
    d: &CadDocument,
    version: DxfVersion,
) -> Result<(), String> {
    if version!=DxfVersion::AC1032 && d.entities().any(|e|matches!(e,EntityType::Hatch(h) if h.is_associative || h.paths.iter().any(|p|!p.boundary_handles.is_empty() || p.flags.bits()&0x9d!=0))) {
        return Err("Hatch source groups and text-loop semantics are qualified only for AC1032".into());
    }
    Ok(())
}

pub(crate) fn gradient_status(fill: &Value, gradient: &Value) -> &'static str {
    if gradient.is_null() {
        "absent"
    } else if fill["kind"] == "gradient" {
        "active"
    } else if gradient.get("type").is_some() && gradient.get("colors").is_some() {
        "inactive"
    } else {
        "partial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> CadDocument {
        let mut d = CadDocument::new();
        for angle in [0.25, 0.75] {
            let mut h = Hatch::solid();
            h.gradient_color.angle = angle;
            d.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
        }
        d
    }
    #[test]
    fn removed_hatch_never_shifts_the_next_hatch_into_a_successful_match() {
        let expected = document();
        let handle = expected.entities().next().unwrap().common().handle;
        let mut actual = expected.clone();
        actual.remove_entity(handle);
        assert!(check_hatch_exchange(&expected, &actual).is_err());
    }
    #[test]
    fn changed_parameter_is_located_without_inventing_native_color_metadata() {
        let expected = document();
        let mut actual = expected.clone();
        let handle = actual.entities().next().unwrap().common().handle;
        let EntityType::Hatch(h) = actual.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        h.gradient_color.angle = 0.5;
        let issues = check_hatch_exchange(&expected, &actual).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].location.ends_with("/rotation"));
        assert_eq!(issues[0].expected, json!(0.25));
        assert_eq!(issues[0].actual, json!(0.5));
    }

    fn solid_document() -> CadDocument {
        use ifccad_convert::opencadcodec::{entities::hatch::*, Circle, Vector2};
        let mut d = CadDocument::new();
        let source = d.add_entity(EntityType::Circle(Circle::new())).unwrap();
        let mut h = Hatch::solid();
        h.is_associative = true;
        for radius in [1., 3.] {
            let mut p = BoundaryPath::new();
            p.flags = BoundaryPathFlags::TEXT_ISLAND;
            p.boundary_handles = vec![source];
            p.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
                center: Vector2::ZERO,
                radius,
                start_angle: 0.,
                end_angle: std::f64::consts::TAU,
                counter_clockwise: true,
            }));
            h.paths.push(p);
        }
        d.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
        d
    }
    #[test]
    fn solid_flag_loss_is_active_located_semantic_loss() {
        let expected = solid_document();
        let mut actual = expected.clone();
        let handle = actual
            .entities()
            .find(|e| matches!(e, EntityType::Hatch(_)))
            .unwrap()
            .common()
            .handle;
        let EntityType::Hatch(h) = actual.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        h.paths[0].flags =
            ifccad_convert::opencadcodec::entities::hatch::BoundaryPathFlags::DEFAULT;
        let issues = check_hatch_exchange(&expected, &actual).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "TARGET_CODEC_HATCH_TEXT_ISLAND_LOSS");
        assert!(issues[0].location.ends_with("/loops/0/textIsland"));
    }

    #[test]
    fn physical_codec_cannot_deliver_a_partial_nonempty_source_group() {
        let mut expected = solid_document();
        let handle = expected
            .entities()
            .find(|e| matches!(e, EntityType::Hatch(_)))
            .unwrap()
            .common()
            .handle;
        let EntityType::Hatch(h) = expected.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        let source = h.paths[0].boundary_handles[0];
        h.paths[0].boundary_handles.push(source);
        let mut actual = expected.clone();
        let EntityType::Hatch(h) = actual.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        h.paths[0].boundary_handles.pop();
        assert!(check_hatch_exchange(&expected, &actual).is_err());
        let EntityType::Hatch(h) = actual.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        h.paths[0].boundary_handles.clear();
        let issues = check_hatch_exchange(&expected, &actual).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "TARGET_CODEC_HATCH_SOURCE_GROUP_LOSS");
    }
    #[test]
    fn solid_correspondence_failure_never_becomes_success() {
        let expected = solid_document();
        let handle = expected
            .entities()
            .find(|e| matches!(e, EntityType::Hatch(_)))
            .unwrap()
            .common()
            .handle;
        for mutation in 0..4 {
            let mut actual = expected.clone();
            if mutation == 0 {
                actual.remove_entity(handle);
            } else if mutation == 3 {
                let source = actual
                    .entities()
                    .find(|e| matches!(e, EntityType::Circle(_)))
                    .unwrap()
                    .common()
                    .handle;
                actual.remove_entity(source);
            } else {
                let EntityType::Hatch(h) = actual.get_entity_mut(handle).unwrap() else {
                    panic!()
                };
                if mutation == 1 {
                    h.paths.pop();
                } else {
                    h.paths.swap(0, 1);
                }
            }
            assert!(
                check_hatch_exchange(&expected, &actual).is_err(),
                "mutation {mutation}"
            );
        }
    }
}
