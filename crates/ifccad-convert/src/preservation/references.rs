use super::IfccadPreservationReason;
use crate::*;
use ocdraw::ifccad::*;
use opencadcodec::{entities::Spline, xdata::XDataValue, CadDocument, Handle};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReferenceBaseline {
    pub slot: String,
    pub source_key: String,
    pub target_role: String,
}
#[derive(Clone)]
pub(crate) enum Reference {
    Handle(Handle),
    Layer(String),
    Pattern(String),
}
pub(crate) fn references(spline: &Spline) -> Vec<(String, Reference)> {
    let mut refs = vec![(
        "common.layer".into(),
        Reference::Layer(spline.common.layer.clone()),
    )];
    if let Some(handle) = spline.common.layer_handle.filter(|h| !h.is_null()) {
        refs.push(("common.layerHandle".into(), Reference::Handle(handle)));
    }
    if let Some(handle) = spline.common.linetype_handle.filter(|h| !h.is_null()) {
        refs.push(("common.linetypeHandle".into(), Reference::Handle(handle)));
    }
    if !spline.common.linetype.is_empty()
        && !spline.common.linetype.eq_ignore_ascii_case("ByLayer")
        && !spline.common.linetype.eq_ignore_ascii_case("ByBlock")
    {
        refs.push((
            "common.linetype".into(),
            Reference::Pattern(spline.common.linetype.clone()),
        ));
    }
    for (i, r) in spline.common.extended_data.records().iter().enumerate() {
        for (j, v) in r.values.iter().enumerate() {
            let value = match v {
                XDataValue::Handle(h) => Some(Reference::Handle(*h)),
                XDataValue::LayerName(n) => Some(Reference::Layer(n.clone())),
                _ => None,
            };
            if let Some(value) = value {
                refs.push((
                    format!("common.extendedData.records[{i}].values[{j}]"),
                    value,
                ));
            }
        }
    }
    refs
}

pub(crate) fn active(slot: &str, e: &IfccadOpaqueEntity) -> bool {
    match slot {
        "common.layer" | "common.layerHandle" => e.layer_id.is_none(),
        "common.linetype" | "common.linetypeHandle" => e.appearance.is_none(),
        _ => true,
    }
}
pub(crate) fn target_role(t: IfccadPreservationTarget) -> &'static str {
    use IfccadPreservationTarget::*;
    match t {
        Drawing => "drawing",
        Entity(_) => "entity",
        Layer(_) => "layer",
        LinePattern(_) => "linePattern",
        Layout(_) => "layout",
        BlockDefinition(_) => "blockDefinition",
        Record(_) => "record",
    }
}
pub(crate) fn entities(d: &IfccadDocument) -> impl Iterator<Item = &IfccadEntity> {
    d.model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities))
}
pub(crate) fn target_exists(d: &IfccadDocument, t: IfccadPreservationTarget) -> bool {
    use IfccadPreservationTarget::*;
    match t {
        Drawing => true,
        Entity(id) => entities(d).any(|e| e.id() == id),
        Layer(id) => d.layers.iter().any(|l| l.id == id),
        LinePattern(id) => d.line_patterns.iter().any(|p| p.id == id),
        Layout(id) => d.model.id == id || d.paper_layouts.iter().any(|p| p.id == id),
        BlockDefinition(id) => d.blocks.iter().any(|b| b.id == id),
        Record(id) => d
            .preservation
            .as_ref()
            .is_some_and(|p| p.records.iter().any(|r| r.id == id)),
    }
}
fn lookup(
    reference: &Reference,
    source: &CadDocument,
    m: &IfccadMappings,
) -> Option<(String, IfccadPreservationTarget)> {
    let layer = |name: &str| {
        let l = source.layers.get(name)?;
        Some((
            format!("{:x}", l.handle),
            IfccadPreservationTarget::Layer(m.layers.ifccad_id(l.handle)?),
        ))
    };
    let pattern = |name: &str| {
        let p = source.line_types.get(name)?;
        Some((
            format!("{:x}", p.handle),
            IfccadPreservationTarget::LinePattern(IfccadLinePatternId(
                m.line_patterns.ifccad_id(p.handle)?,
            )),
        ))
    };
    match reference {
        Reference::Layer(name) => layer(name),
        Reference::Pattern(name) => pattern(name),
        Reference::Handle(h) => {
            let key = format!("{h:x}");
            if let Some(p) = source.line_types.iter().find(|p| {
                p.handle == *h
                    && (p.name.eq_ignore_ascii_case("ByLayer")
                        || p.name.eq_ignore_ascii_case("ByBlock"))
            }) {
                let role = if p.name.eq_ignore_ascii_case("ByLayer") {
                    "byLayer"
                } else {
                    "byBlock"
                };
                return Some((format!("{role}:{h:x}"), IfccadPreservationTarget::Drawing));
            }
            if let Some(id) = m.entities.ifccad_id(*h) {
                return Some((key, IfccadPreservationTarget::Entity(id)));
            }
            if let Some(id) = m.layers.ifccad_id(*h) {
                return Some((key, IfccadPreservationTarget::Layer(id)));
            }
            if let Some(id) = m.line_patterns.ifccad_id(*h) {
                return Some((
                    key,
                    IfccadPreservationTarget::LinePattern(IfccadLinePatternId(id)),
                ));
            }
            if let Some(id) = m.blocks.ifccad_id(*h) {
                return Some((key, IfccadPreservationTarget::BlockDefinition(id)));
            }
            let layout = source.objects.values().find_map(|o| match o {
                opencadcodec::objects::ObjectType::Layout(l) if l.block_record == *h => {
                    m.layouts.ifccad_id(l.handle)
                }
                _ => None,
            })?;
            Some((key, IfccadPreservationTarget::Layout(layout)))
        }
    }
}
pub(crate) fn bind(
    s: &Spline,
    r: &mut IfccadPreservationRecord,
    d: &IfccadDocument,
    source: &CadDocument,
    m: &IfccadMappings,
) -> Result<(), IfccadConversionError> {
    let Some(IfccadPreservationTarget::Entity(id)) = r.subject else {
        return Ok(());
    };
    let e = entities(d)
        .find(|e| e.id() == id)
        .and_then(IfccadEntity::as_opaque)
        .expect("captured opaque entity");
    let mut missing = false;
    for (slot, reference) in references(s) {
        if let Some((source_key, target)) = lookup(&reference, source, m) {
            r.conditions.push(IfccadPreservationCondition {
                target,
                predicate: super::conditions::REFERENCE_PREDICATE.into(),
                version: 1,
                baseline: serde_json::to_vec(&ReferenceBaseline {
                    slot: slot.clone(),
                    source_key: source_key.clone(),
                    target_role: target_role(target).into(),
                })
                .unwrap(),
            });
            r.bindings.push(IfccadPreservationBinding {
                slot,
                source_key,
                target,
            });
        } else if active(&slot, e) {
            missing = true;
        }
    }
    r.dependency_coverage = if missing {
        IfccadPreservationDependencyCoverage::Conservative
    } else {
        IfccadPreservationDependencyCoverage::Qualified
    };
    Ok(())
}
pub(crate) fn validate_reference_condition(
    record: &IfccadPreservationRecord,
    condition: &IfccadPreservationCondition,
) -> Result<(), IfccadPreservationReason> {
    use IfccadPreservationReason::*;
    let b: ReferenceBaseline =
        serde_json::from_slice(&condition.baseline).map_err(|_| MalformedPayload)?;
    let binding = record
        .bindings
        .iter()
        .find(|r| r.slot == b.slot)
        .ok_or(UnsupportedContext)?;
    if binding.target != condition.target
        || binding.source_key != b.source_key
        || target_role(binding.target) != b.target_role
    {
        return Err(UnsupportedContext);
    }
    Ok(())
}

pub(crate) fn qualify_references(
    spline: &Spline,
    record: &IfccadPreservationRecord,
    entity: &IfccadOpaqueEntity,
    doc: &IfccadDocument,
) -> Result<(), IfccadPreservationReason> {
    use IfccadPreservationReason::*;
    if record
        .bindings
        .iter()
        .any(|b| matches!(b.target, IfccadPreservationTarget::Record(_)))
    {
        return Err(UnsupportedContext);
    }
    for (slot, reference) in references(spline) {
        if !active(&slot, entity) {
            continue;
        }
        let binding = record
            .bindings
            .iter()
            .find(|b| b.slot == slot)
            .ok_or(UnresolvedReference)?;
        let mut conditions = record.conditions.iter().filter(|c| {
            c.predicate == "openaec.ifccad.sourceReferenceBinding"
                && serde_json::from_slice::<ReferenceBaseline>(&c.baseline)
                    .is_ok_and(|b| b.slot == slot)
        });
        if conditions.next().is_none() || conditions.next().is_some() {
            return Err(UnsupportedContext);
        }
        if binding.target == IfccadPreservationTarget::Drawing {
            let role = if spline.common.linetype.is_empty()
                || spline.common.linetype.eq_ignore_ascii_case("ByLayer")
            {
                "byLayer"
            } else if spline.common.linetype.eq_ignore_ascii_case("ByBlock") {
                "byBlock"
            } else {
                return Err(UnsupportedContext);
            };
            let Reference::Handle(h) = reference else {
                return Err(UnsupportedContext);
            };
            if slot != "common.linetypeHandle" || binding.source_key != format!("{role}:{h:x}") {
                return Err(UnsupportedContext);
            }
            if !target_exists(doc, binding.target) {
                return Err(MissingDependency);
            }
            continue;
        }
        match reference {
            Reference::Handle(_)
                if slot == "common.layerHandle"
                    && !matches!(binding.target, IfccadPreservationTarget::Layer(_)) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Handle(_)
                if slot == "common.linetypeHandle"
                    && !matches!(
                        binding.target,
                        IfccadPreservationTarget::LinePattern(_)
                            | IfccadPreservationTarget::Drawing
                    ) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Handle(h) if binding.source_key != format!("{h:x}") => {
                return Err(UnsupportedContext)
            }
            Reference::Layer(_)
                if !matches!(binding.target, IfccadPreservationTarget::Layer(_)) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Pattern(_)
                if !matches!(binding.target, IfccadPreservationTarget::LinePattern(_)) =>
            {
                return Err(UnsupportedContext)
            }
            _ => {}
        }
        if !target_exists(doc, binding.target) {
            return Err(MissingDependency);
        }
    }
    Ok(())
}

fn constructed_handle(
    binding: &IfccadPreservationBinding,
    cad: &CadDocument,
    m: &IfccadMappings,
) -> Option<Handle> {
    use IfccadPreservationTarget::*;
    match binding.target {
        Drawing
            if binding.slot == "common.linetypeHandle"
                && binding.source_key.starts_with("byLayer:") =>
        {
            Some(cad.header.bylayer_linetype_handle)
        }
        Drawing
            if binding.slot == "common.linetypeHandle"
                && binding.source_key.starts_with("byBlock:") =>
        {
            Some(cad.header.byblock_linetype_handle)
        }
        Entity(id) => m.entities.cad_handle(id),
        Layer(id) => m.layers.cad_handle(id),
        LinePattern(id) => m.line_patterns.cad_handle(id.0),
        BlockDefinition(id) => m.blocks.cad_handle(id),
        Layout(id) => {
            let h = m.layouts.cad_handle(id)?;
            match cad.objects.get(&h)? {
                opencadcodec::objects::ObjectType::Layout(l) => Some(l.block_record),
                _ => None,
            }
        }
        _ => None,
    }
    .filter(|h| !h.is_null())
}
pub(crate) fn rebind_spline_references(
    spline: &mut Spline,
    record: &IfccadPreservationRecord,
    entity: &IfccadOpaqueEntity,
    doc: &IfccadDocument,
    cad: &CadDocument,
    mapping: &IfccadMappings,
) -> Result<(), IfccadPreservationReason> {
    use IfccadPreservationReason::*;
    for (slot, reference) in references(spline) {
        if !active(&slot, entity) {
            continue;
        }
        let binding = record
            .bindings
            .iter()
            .find(|b| b.slot == slot)
            .ok_or(UnresolvedReference)?;
        let handle = constructed_handle(binding, cad, mapping).ok_or(MissingDependency)?;
        match reference {
            Reference::Layer(_) | Reference::Pattern(_) => {
                let name = match binding.target {
                    IfccadPreservationTarget::Layer(id) => doc
                        .layers
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.name.clone()),
                    IfccadPreservationTarget::LinePattern(id) => doc
                        .line_patterns
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.name.clone()),
                    _ => None,
                }
                .ok_or(UnresolvedReference)?;
                if slot == "common.layer" {
                    spline.common.layer = name;
                } else if slot == "common.linetype" {
                    spline.common.linetype = name;
                    spline.common.linetype_handle = Some(handle);
                } else {
                    replace_xdata_value(spline, &slot, XDataValue::LayerName(name))?;
                }
            }
            Reference::Handle(_) if slot == "common.linetypeHandle" => {
                spline.common.linetype_handle = Some(handle)
            }
            Reference::Handle(_) if slot == "common.layerHandle" => {
                spline.common.layer_handle = Some(handle)
            }
            Reference::Handle(_) => replace_xdata_value(spline, &slot, XDataValue::Handle(handle))?,
        }
    }
    Ok(())
}
fn replace_xdata_value(
    spline: &mut Spline,
    slot: &str,
    value: XDataValue,
) -> Result<(), IfccadPreservationReason> {
    let mut data = opencadcodec::xdata::ExtendedData::new();
    let mut found = false;
    for (i, r) in spline.common.extended_data.records().iter().enumerate() {
        let mut r = r.clone();
        for (j, v) in r.values.iter_mut().enumerate() {
            if slot == format!("common.extendedData.records[{i}].values[{j}]") {
                *v = value.clone();
                found = true;
            }
        }
        data.add_record(r);
    }
    if !found {
        return Err(IfccadPreservationReason::UnresolvedReference);
    }
    data.raw_dwg_eed = spline.common.extended_data.raw_dwg_eed.clone();
    spline.common.extended_data = data;
    Ok(())
}
