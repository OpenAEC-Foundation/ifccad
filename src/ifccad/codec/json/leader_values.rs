//! Typed Leader-family value projection. Semantic validation remains shared.
use super::*;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
#[path = "leader_value_records.rs"]
mod records;

pub(super) trait LeaderJsonValue: Sized {
    fn write_value(&self, prefix: &str) -> Value;
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport>;
    fn saved(&self) -> bool {
        true
    }
    fn absent() -> Option<Self> {
        None
    }
}
fn invalid(path: &str, detail: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-LEADER-JSON-001", path, detail.to_string())
}
struct Fields<'a> {
    object: &'a Map<String, Value>,
    path: String,
}
impl<'a> Fields<'a> {
    fn new(value: &'a Value, allowed: &[&str], path: &str) -> Result<Self, IfccadReport> {
        let object = value
            .as_object()
            .ok_or_else(|| invalid(path, "object required"))?;
        if let Some(name) = object.keys().find(|name| !allowed.contains(&name.as_str())) {
            return Err(invalid(&format!("{path}/{name}"), "unknown field"));
        }
        Ok(Self {
            object,
            path: path.into(),
        })
    }
    fn value(&self, name: &str) -> Option<&'a Value> {
        self.object.get(name)
    }
    fn child(&self, name: &str) -> String {
        format!("{}/{name}", self.path)
    }
    fn required(&self, name: &str) -> Result<&'a Value, IfccadReport> {
        self.value(name)
            .ok_or_else(|| invalid(&self.child(name), "missing required field"))
    }
    fn take<T: LeaderJsonValue>(&self, name: &str, prefix: &str) -> Result<T, IfccadReport> {
        match self.value(name) {
            Some(value) => T::read_value(value, prefix, &self.child(name)),
            None => T::absent().ok_or_else(|| invalid(&self.child(name), "missing required field")),
        }
    }
}
fn write_reference(id: u64, prefix: &str, domain: &str) -> Value {
    json!(format!("{prefix}/{domain}{id}"))
}
fn read_reference(
    value: &Value,
    prefix: &str,
    domain: &str,
    path: &str,
) -> Result<u64, IfccadReport> {
    super::dimension_values::reference(
        prefix,
        domain,
        value
            .as_str()
            .ok_or_else(|| invalid(path, "canonical reference required"))?,
        path,
    )
}
macro_rules! serde_value {
    ($($ty:ty),+ $(,)?)=>{$(impl LeaderJsonValue for $ty {
        fn write_value(&self,_prefix:&str)->Value {serde_json::to_value(self).expect("validated native value")}
        fn read_value(value:&Value,_prefix:&str,path:&str)->Result<Self,IfccadReport> {
            serde_json::from_value(value.clone()).map_err(|error|invalid(path,error))
        }
    })+};
}
serde_value!(
    bool,
    u32,
    u64,
    i32,
    f64,
    String,
    IfccadColor,
    IfccadPlacement,
    IfccadBlockTransform
);
impl<T: LeaderJsonValue> LeaderJsonValue for Option<T> {
    fn write_value(&self, prefix: &str) -> Value {
        self.as_ref()
            .expect("present optional value")
            .write_value(prefix)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        if value.is_null() {
            return Err(invalid(path, "explicit null is forbidden"));
        }
        Ok(Some(T::read_value(value, prefix, path)?))
    }
    fn saved(&self) -> bool {
        self.is_some()
    }
    fn absent() -> Option<Self> {
        Some(None)
    }
}
impl<T: LeaderJsonValue> LeaderJsonValue for Box<T> {
    fn write_value(&self, prefix: &str) -> Value {
        (**self).write_value(prefix)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        Ok(Box::new(T::read_value(value, prefix, path)?))
    }
}
impl<T: LeaderJsonValue> LeaderJsonValue for Vec<T> {
    fn write_value(&self, prefix: &str) -> Value {
        Value::Array(self.iter().map(|value| value.write_value(prefix)).collect())
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        value
            .as_array()
            .ok_or_else(|| invalid(path, "array required"))?
            .iter()
            .enumerate()
            .map(|(index, value)| T::read_value(value, prefix, &format!("{path}/{index}")))
            .collect()
    }
}
impl<T: LeaderJsonValue, const N: usize> LeaderJsonValue for [T; N] {
    fn write_value(&self, prefix: &str) -> Value {
        Value::Array(self.iter().map(|value| value.write_value(prefix)).collect())
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        Vec::<T>::read_value(value, prefix, path)?
            .try_into()
            .map_err(|_| invalid(path, "incorrect fixed array length"))
    }
}
impl<T: LeaderJsonValue + Ord> LeaderJsonValue for BTreeSet<T> {
    fn write_value(&self, prefix: &str) -> Value {
        Value::Array(self.iter().map(|value| value.write_value(prefix)).collect())
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let values = Vec::<T>::read_value(value, prefix, path)?;
        let count = values.len();
        let result = values.into_iter().collect::<Self>();
        if result.len() != count {
            return Err(invalid(path, "duplicate set member"));
        }
        Ok(result)
    }
}
macro_rules! reference_value {
    ($($ty:ident=>$domain:literal),+ $(,)?)=>{$(impl LeaderJsonValue for $ty {
        fn write_value(&self,prefix:&str)->Value {write_reference(self.0,prefix,$domain)}
        fn read_value(value:&Value,prefix:&str,path:&str)->Result<Self,IfccadReport> {Ok(Self(read_reference(value,prefix,$domain,path)?))}
    })+};
}
reference_value!(IfccadTextStyleId=>"textStyle/",IfccadDimensionStyleId=>"dimensionStyle/",IfccadMultiLeaderStyleId=>"multiLeaderStyle/",IfccadLinePatternId=>"linePattern/");
impl<T: LeaderJsonValue> LeaderJsonValue for IfccadMode<T> {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::ByLayer => json!({"mode":"ByLayer"}),
            Self::ByBlock => json!({"mode":"ByBlock"}),
            Self::Explicit(value) => json!({"mode":"Explicit","value":value.write_value(prefix)}),
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["mode", "value"], path)?;
        let mode: String = f.take("mode", prefix)?;
        match mode.as_str() {
            "Explicit" => Ok(Self::Explicit(f.take("value", prefix)?)),
            "ByLayer" | "ByBlock" => {
                if f.value("value").is_some() {
                    return Err(invalid(path, "inherited mode forbids value"));
                }
                Ok(if mode == "ByLayer" {
                    Self::ByLayer
                } else {
                    Self::ByBlock
                })
            }
            _ => Err(invalid(path, "unknown inheritance mode")),
        }
    }
}
impl LeaderJsonValue for IfccadDimensionOverrides {
    fn write_value(&self, prefix: &str) -> Value {
        serde_json::to_value(super::dimension_values::encode_overrides(self, prefix))
            .expect("typed overrides")
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let wire = serde_json::from_value(value.clone()).map_err(|error| invalid(path, error))?;
        super::dimension_values::decode_overrides(&wire, prefix, path)
    }
}
impl<P: LeaderJsonValue> LeaderJsonValue for IfccadAnnotation<P> {
    fn write_value(&self, prefix: &str) -> Value {
        let wire = IfccadAnnotation {
            orientation: self.orientation,
            mode: self.mode,
            reference: self.reference,
            contexts: self
                .contexts
                .iter()
                .map(|context| IfccadAnnotationContext {
                    key: context.key,
                    presentation: context.presentation.as_ref().map(|p| p.write_value(prefix)),
                })
                .collect(),
        };
        super::annotation::encode(&wire, prefix)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let wire = super::annotation::decode::<Value>(value, prefix, path)?;
        Ok(Self {
            orientation: wire.orientation,
            mode: wire.mode,
            reference: wire.reference,
            contexts: wire
                .contexts
                .into_iter()
                .enumerate()
                .map(|(index, context)| {
                    Ok(IfccadAnnotationContext {
                        key: context.key,
                        presentation: context
                            .presentation
                            .map(|value| {
                                P::read_value(
                                    &value,
                                    prefix,
                                    &format!("{path}/contexts/{index}/presentation"),
                                )
                            })
                            .transpose()?,
                    })
                })
                .collect::<Result<_, IfccadReport>>()?,
        })
    }
}
impl LeaderJsonValue for IfccadMText {
    fn write_value(&self, prefix: &str) -> Value {
        let mut attrs = Map::new();
        super::text::encode_entity(
            &IfccadEntityKind::MText(Box::new(self.clone())),
            prefix,
            &mut attrs,
        );
        json!({"placement":attrs["ifccad::geom::placement"],"value":attrs["ifccad::mText"]})
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["placement", "value"], path)?;
        let payload = f.required("value")?;
        let style = payload
            .get("style")
            .ok_or_else(|| invalid(path, "missing text style"))?;
        let id = IfccadTextStyleId(read_reference(style, prefix, "textStyle/", path)?);
        let styles = BTreeMap::from([(style.as_str().expect("checked reference").to_owned(), id)]);
        match super::text::decode_entity(
            payload,
            f.required("placement")?,
            "ifccad::mText",
            path,
            &styles,
        )? {
            IfccadEntityKind::MText(text) => Ok(*text),
            _ => unreachable!("MText decoder"),
        }
    }
}
macro_rules! text_value {
    ($ty:ty,$encode:ident,$decode:ident) => {
        impl LeaderJsonValue for $ty {
            fn write_value(&self, _prefix: &str) -> Value {
                super::text_values::$encode(self)
            }
            fn read_value(value: &Value, _prefix: &str, path: &str) -> Result<Self, IfccadReport> {
                super::text_values::$decode(value)
                    .ok_or_else(|| invalid(path, "invalid structured text value"))
            }
        }
    };
}
text_value!(
    crate::text::CharacterFormat<IfccadColor>,
    character,
    read_character
);
text_value!(crate::text::ParagraphFormat, paragraph, read_paragraph);
impl LeaderJsonValue for crate::text::MTextParagraph<IfccadColor> {
    fn write_value(&self, _prefix: &str) -> Value {
        super::text_values::content(std::slice::from_ref(self))[0].clone()
    }
    fn read_value(value: &Value, _prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        super::text_values::read_content(&Value::Array(vec![value.clone()]))
            .and_then(|mut values| values.pop())
            .ok_or_else(|| invalid(path, "invalid structured paragraph"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const PREFIX: &str = "/cad/d1";
    #[test]
    fn typed_leader_values_rebind_resources_and_keep_presence() {
        let mut leader = IfccadMultiLeader::new();
        leader.style_id = Some(IfccadMultiLeaderStyleId(41));
        leader.settings.common.text.style_id = Some(IfccadTextStyleId(23));
        leader.settings.common.block.definition_id = Some(600);
        leader.settings.common.line.line_pattern = IfccadMode::Explicit(IfccadLinePatternId(77));
        leader
            .overridden_properties
            .insert(IfccadMultiLeaderProperty::ArrowSize);
        let wire = leader.write_value(PREFIX);
        assert_eq!(wire["style"], json!("/cad/d1/multiLeaderStyle/41"));
        assert_eq!(
            wire["settings"]["common"]["text"]["style"],
            json!("/cad/d1/textStyle/23")
        );
        assert_eq!(
            wire["settings"]["common"]["block"]["definition"],
            json!("/cad/d1/block/600")
        );
        let back = IfccadMultiLeader::read_value(&wire, PREFIX, "/test").unwrap();
        assert_eq!(leader, back);
        let command = IfccadLeaderGraphicCommand::SetLayer(None);
        assert_eq!(
            command,
            IfccadLeaderGraphicCommand::read_value(
                &command.write_value(PREFIX),
                PREFIX,
                "/command"
            )
            .unwrap()
        );
    }
    #[test]
    fn typed_leader_values_reject_unknown_null_cross_drawing_and_duplicate_overrides() {
        let leader = IfccadMultiLeader::new();
        let mut wire = leader.write_value(PREFIX);
        wire["unexpected"] = json!(1);
        assert!(IfccadMultiLeader::read_value(&wire, PREFIX, "/test").is_err());
        let mut wire = leader.write_value(PREFIX);
        wire["style"] = json!(null);
        assert!(IfccadMultiLeader::read_value(&wire, PREFIX, "/test").is_err());
        wire["style"] = json!("/cad/d2/multiLeaderStyle/41");
        assert!(IfccadMultiLeader::read_value(&wire, PREFIX, "/test").is_err());
        let mut wire = leader.write_value(PREFIX);
        wire["overriddenProperties"] = json!(["arrowSize", "arrowSize"]);
        assert!(IfccadMultiLeader::read_value(&wire, PREFIX, "/test").is_err());
    }
}
