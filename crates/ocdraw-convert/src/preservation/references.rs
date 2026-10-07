use super::*;
use ocdraw::ocdraw::*;
use opencadcodec::entities::Spline;
use opencadcodec::xdata::XDataValue;
use opencadcodec::{CadDocument, EntityType, Handle};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReferenceBaseline {
    slot: String,
    source_key: String,
    target_role: String,
}

pub(crate) fn target_role(target: OcdrawPreservationTarget) -> &'static str {
    use OcdrawPreservationTarget::*;
    match target {
        Drawing => "drawing",
        Entity(_) => "entity",
        Layer(_) => "layer",
        LinePattern(_) => "linePattern",
        Layout(_) => "layout",
        Scope(_) => "scope",
        BlockDefinition(_) => "blockDefinition",
        Record(_) => "record",
    }
}
pub(crate) fn target_exists(doc: &OcdrawDocument, target: OcdrawPreservationTarget) -> bool {
    use OcdrawPreservationTarget::*;
    match target {
        Drawing => true,
        Entity(id) => {
            doc.geometric_entities.iter().any(|e| e.id == id)
                || doc.viewports.iter().any(|e| e.id == id)
                || doc.opaque_entities.iter().any(|e| e.id == id)
        }
        Layer(id) => doc.layers.iter().any(|e| e.id == id),
        LinePattern(id) => doc.line_patterns.iter().any(|e| e.id == id),
        Layout(id) => doc.layouts.iter().any(|e| e.id == id),
        Scope(id) => doc.scopes.iter().any(|e| e.id == id),
        BlockDefinition(id) => doc.block_definitions.iter().any(|e| e.scope_id == id),
        Record(id) => doc
            .preservation
            .as_ref()
            .is_some_and(|p| p.records.iter().any(|r| r.id == id)),
    }
}

#[derive(Clone)]
enum Reference {
    Handle(Handle),
    Layer(String),
    Pattern(String),
}
fn references(spline: &Spline) -> Vec<(String, Reference)> {
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
fn active(slot: &str, entity: &DrawingOpaqueEntity) -> bool {
    match slot {
        "common.layer" | "common.layerHandle" => entity.layer_id.is_none(),
        "common.linetype" | "common.linetypeHandle" => entity.appearance.is_none(),
        _ => true,
    }
}
fn lookup(
    source: &CadDocument,
    doc: &OcdrawDocument,
    mapping: &BTreeMap<Handle, u64>,
    reference: &Reference,
) -> Option<(String, OcdrawPreservationTarget)> {
    let layer = |name: &str| {
        let source_layer = source.layers.get(name)?;
        let native = doc
            .layers
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(name))?;
        Some((
            format!("{:x}", source_layer.handle),
            OcdrawPreservationTarget::Layer(native.id),
        ))
    };
    let pattern = |name: &str| {
        let source_pattern = source.line_types.get(name)?;
        let native = doc
            .line_patterns
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(name))?;
        Some((
            format!("{:x}", source_pattern.handle),
            OcdrawPreservationTarget::LinePattern(native.id),
        ))
    };
    match reference {
        Reference::Layer(name) => layer(name),
        Reference::Pattern(name) => pattern(name),
        Reference::Handle(handle) => {
            if let Some(id) = mapping.get(handle) {
                return Some((format!("{handle:x}"), OcdrawPreservationTarget::Entity(*id)));
            }
            if let Some(l) = source.layers.iter().find(|l| l.handle == *handle) {
                return layer(&l.name);
            }
            if let Some(p) = source.line_types.iter().find(|p| p.handle == *handle) {
                return pattern(&p.name);
            }
            if let Some(b) = source.block_records.iter().find(|b| b.handle == *handle) {
                if let Some(native) = doc
                    .block_definitions
                    .iter()
                    .find(|n| n.name.eq_ignore_ascii_case(&b.name))
                {
                    return Some((
                        format!("{handle:x}"),
                        OcdrawPreservationTarget::BlockDefinition(native.scope_id),
                    ));
                }
                if let Some(layout) = source.objects.values().find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l) if l.block_record == *handle => {
                        Some(l)
                    }
                    _ => None,
                }) {
                    if let Some(native) = doc.layouts.iter().find(|l| l.name == layout.name) {
                        return Some((
                            format!("{handle:x}"),
                            OcdrawPreservationTarget::Scope(native.scope_id),
                        ));
                    }
                }
            }
            None
        }
    }
}

pub(crate) fn bind_source_references(
    source: &CadDocument,
    doc: &mut OcdrawDocument,
    mapping: &BTreeMap<Handle, u64>,
    report: &mut OcdrawPreservationReport,
) {
    let mut changes = Vec::new();
    for entity in &doc.opaque_entities {
        let p = doc.preservation.as_ref().expect("captured collection");
        let record = p
            .records
            .iter()
            .find(|r| r.id == entity.preservation_record_id)
            .expect("captured record");
        let handle = u64::from_str_radix(&record.source_key, 16).expect("captured source handle");
        let Some(EntityType::Spline(spline)) = source.get_entity(Handle::new(handle)) else {
            continue;
        };
        let mut bindings = Vec::new();
        let mut conditions = Vec::new();
        let mut missing = false;
        for (slot, reference) in references(spline) {
            if let Some((source_key, target)) = lookup(source, doc, mapping, &reference) {
                conditions.push(OcdrawPreservationCondition {
                    target,
                    predicate: "openaec.ocdraw.sourceReferenceBinding".into(),
                    version: 1,
                    baseline: serde_json::to_vec(&ReferenceBaseline {
                        slot: slot.clone(),
                        source_key: source_key.clone(),
                        target_role: target_role(target).into(),
                    })
                    .expect("reference baseline"),
                });
                bindings.push(OcdrawPreservationBinding {
                    slot,
                    source_key,
                    target,
                });
            } else if active(&slot, entity) {
                missing = true;
            }
        }
        if missing {
            let mut entry = spline_entry(
                record.id,
                record.source_key.clone(),
                OcdrawPreservationResult::RestorationUnavailable,
                format!("/preservation/records/{}/bindings", record.id.0),
                "required typed source reference has no native target",
            );
            entry.reason = Some(OcdrawPreservationReason::UnresolvedReference);
            report.entries.push(entry);
        }
        let coverage = if unsupported_common_context(spline) {
            OcdrawPreservationDependencyCoverage::Unknown
        } else if missing {
            OcdrawPreservationDependencyCoverage::Conservative
        } else {
            OcdrawPreservationDependencyCoverage::Qualified
        };
        changes.push((record.id, bindings, conditions, coverage));
    }
    if let Some(p) = &mut doc.preservation {
        for (id, bindings, conditions, coverage) in changes {
            let r = p
                .records
                .iter_mut()
                .find(|r| r.id == id)
                .expect("captured record");
            r.bindings = bindings;
            r.dependency_coverage = coverage;
            r.conditions.extend(conditions);
        }
    }
}

pub(crate) fn validate_reference_condition(
    record: &OcdrawPreservationRecord,
    condition: &OcdrawPreservationCondition,
) -> Result<(), OcdrawPreservationReason> {
    use OcdrawPreservationReason::*;
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
    record: &OcdrawPreservationRecord,
    entity: &DrawingOpaqueEntity,
    doc: &OcdrawDocument,
) -> Result<(), OcdrawPreservationReason> {
    use OcdrawPreservationReason::*;
    if record
        .bindings
        .iter()
        .any(|b| matches!(b.target, OcdrawPreservationTarget::Record(_)))
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
            c.predicate == "openaec.ocdraw.sourceReferenceBinding"
                && serde_json::from_slice::<ReferenceBaseline>(&c.baseline)
                    .is_ok_and(|b| b.slot == slot)
        });
        if conditions.next().is_none() || conditions.next().is_some() {
            return Err(UnsupportedContext);
        }
        match reference {
            Reference::Handle(_)
                if slot == "common.layerHandle"
                    && !matches!(binding.target, OcdrawPreservationTarget::Layer(_)) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Handle(_)
                if slot == "common.linetypeHandle"
                    && !matches!(binding.target, OcdrawPreservationTarget::LinePattern(_)) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Handle(h) if binding.source_key != format!("{h:x}") => {
                return Err(UnsupportedContext)
            }
            Reference::Layer(_)
                if !matches!(binding.target, OcdrawPreservationTarget::Layer(_)) =>
            {
                return Err(UnsupportedContext)
            }
            Reference::Pattern(_)
                if !matches!(binding.target, OcdrawPreservationTarget::LinePattern(_)) =>
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
    target: OcdrawPreservationTarget,
    doc: &OcdrawDocument,
    cad: &CadDocument,
    mapping: &BTreeMap<u64, Handle>,
) -> Option<Handle> {
    use OcdrawPreservationTarget::*;
    let handle = match target {
        Entity(id) => *mapping.get(&id)?,
        Layer(id) => {
            cad.layers
                .get(&doc.layers.iter().find(|l| l.id == id)?.name)?
                .handle
        }
        LinePattern(id) => {
            cad.line_types
                .get(&doc.line_patterns.iter().find(|l| l.id == id)?.name)?
                .handle
        }
        BlockDefinition(id) | Scope(id) => {
            if matches!(target, Scope(_))
                && doc
                    .scopes
                    .iter()
                    .any(|s| s.id == id && s.kind == DrawingScopeKind::Model)
            {
                // The CAD model layout name is runtime-defined; its owning
                // block remains the same semantic target after native renames.
                cad.header.model_space_block_handle
            } else if let Some(d) = doc.block_definitions.iter().find(|d| d.scope_id == id) {
                cad.block_records.get(&d.name)?.handle
            } else {
                let layout = doc.layouts.iter().find(|l| l.scope_id == id)?;
                cad.objects.values().find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l) if l.name == layout.name => {
                        Some(l.block_record)
                    }
                    _ => None,
                })?
            }
        }
        _ => return None,
    };
    (!handle.is_null()).then_some(handle)
}

pub(crate) fn rebind_spline_references(
    spline: &mut Spline,
    record: &OcdrawPreservationRecord,
    entity: &DrawingOpaqueEntity,
    doc: &OcdrawDocument,
    cad: &CadDocument,
    mapping: &BTreeMap<u64, Handle>,
) -> Result<(), OcdrawPreservationReason> {
    use OcdrawPreservationReason::*;
    for (slot, reference) in references(spline) {
        if !active(&slot, entity) {
            continue;
        }
        let binding = record
            .bindings
            .iter()
            .find(|b| b.slot == slot)
            .ok_or(UnresolvedReference)?;
        let handle =
            constructed_handle(binding.target, doc, cad, mapping).ok_or(MissingDependency)?;
        match reference {
            Reference::Layer(_) | Reference::Pattern(_) => {
                let name = match binding.target {
                    OcdrawPreservationTarget::Layer(id) => doc
                        .layers
                        .iter()
                        .find(|l| l.id == id)
                        .map(|l| l.name.clone()),
                    OcdrawPreservationTarget::LinePattern(id) => doc
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
) -> Result<(), OcdrawPreservationReason> {
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
        return Err(OcdrawPreservationReason::UnresolvedReference);
    }
    data.raw_dwg_eed = spline.common.extended_data.raw_dwg_eed.clone();
    spline.common.extended_data = data;
    Ok(())
}
