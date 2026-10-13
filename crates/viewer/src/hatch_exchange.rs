//! Physical verification against the already mapped target model.
use ocdraw_convert::opencadcodec::{
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
    if !expected
        .entities()
        .any(|e| matches!(e,EntityType::Hatch(h) if meaningful(h)))
        && !actual
            .entities()
            .any(|e| matches!(e,EntityType::Hatch(h) if meaningful(h)))
    {
        return Ok(vec![]);
    }
    let a = groups(expected)?;
    let b = groups(actual)?;
    let mut issues = Vec::new();
    for (scope, hatches) in &a {
        let empty = Vec::new();
        let back = b.get(scope).unwrap_or(&empty);
        if !hatches.iter().chain(back.iter()).any(|h| meaningful(h)) {
            continue;
        }
        if hatches.len() != back.len() {
            return Err(format!(
                "Gradient Hatch count changed in {scope}; correspondence is unconfirmed"
            ));
        }
        for (index, (wanted, read)) in hatches.iter().zip(back).enumerate() {
            // These are emitted target identities within this one file exchange,
            // not original native IDs or a promise of stable conversion handles.
            if wanted.common.handle != read.common.handle {
                return Err(format!("Gradient Hatch identity/order changed in {scope}; correspondence is unconfirmed"));
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
        if !a.contains_key(&scope) && hatches.iter().any(|h| meaningful(h)) {
            return Err(format!(
                "Unexpected gradient Hatch scope {scope}; correspondence is unconfirmed"
            ));
        }
    }
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> CadDocument {
        let mut d = CadDocument::new();
        for angle in [0.25, 0.75] {
            let mut h = Hatch::solid();
            h.gradient_color.angle = angle;
            d.add_entity(EntityType::Hatch(h)).unwrap();
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
}
