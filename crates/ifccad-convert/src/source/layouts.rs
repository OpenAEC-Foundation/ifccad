//! Resolve CAD layout roles without trusting derived header caches.
use crate::diagnostics::diagnostic;
use crate::{IfccadConversionError as Error, IfccadDiagnostic};
use opencadcodec::objects::{Layout, ObjectType};
use opencadcodec::{CadDocument, EntityType, Handle};

/// Missing owners are recoverable only from one exact dictionary membership.
/// A non-null conflict, duplicate entry or second parent is never repaired.
pub(crate) fn owner(
    doc: &CadDocument,
    handle: Handle,
    actual: Handle,
    expected: Handle,
    name: &str,
    recoveries: &mut Vec<IfccadDiagnostic>,
) -> bool {
    let memberships = doc
        .objects
        .iter()
        .flat_map(|(parent, object)| {
            let entries = match object {
                ObjectType::Dictionary(d) => Some(&d.entries),
                ObjectType::DictionaryWithDefault(d) => Some(&d.entries),
                _ => None,
            };
            entries
                .into_iter()
                .flatten()
                .filter(move |(_, h)| *h == handle)
                .map(move |(key, _)| (*parent, key.as_str()))
        })
        .collect::<Vec<_>>();
    if memberships != [(expected, name)] || (!actual.is_null() && actual != expected) {
        return false;
    }
    if actual.is_null() {
        recoveries.push(diagnostic(
            "layout-owner-recovered",
            format!("object/{handle}.owner"),
            format!("unique dictionary {expected} entry {name:?} establishes missing owner"),
        ));
    }
    true
}

/// A model-only CAD document still needs the codec's reserved primary Paper
/// record. It is bookkeeping only when no layout, content or authored state
/// depends on it. Structural markers remain checked by the ordinary inspection.
pub(crate) fn is_empty_reserved_paper_block(
    doc: &CadDocument,
    record: &opencadcodec::BlockRecord,
) -> bool {
    if record.handle != doc.header.paper_space_block_handle
        || !record.name.eq_ignore_ascii_case("*Paper_Space")
        || !record.layout.is_null()
        || !record.entity_handles.is_empty()
        || doc.objects.values().any(|o| matches!(o, ObjectType::Layout(l) if l.block_record == record.handle))
        || doc.entities().any(|e| {
            (e.common().owner_handle == record.handle
                && !matches!(e, EntityType::Block(_) | EntityType::BlockEnd(_)))
                || matches!(e, EntityType::Insert(i) if i.block_name.eq_ignore_ascii_case(&record.name))
        })
    {
        return false;
    }
    let mut candidate = record.clone();
    candidate.handle = Handle::NULL;
    candidate.block_entity_handle = Handle::NULL;
    candidate.block_end_handle = Handle::NULL;
    candidate.name = "*Paper_Space".into();
    candidate == opencadcodec::BlockRecord::paper_space()
}

pub(crate) fn dictionary(
    doc: &CadDocument,
    recoveries: &mut Vec<IfccadDiagnostic>,
) -> Result<Handle, Error> {
    let invalid = |s: &str| Error::InvalidStructure(s.into());
    let root_handle = doc.header.named_objects_dict_handle;
    let Some(ObjectType::Dictionary(root)) = doc.objects.get(&root_handle) else {
        return Err(invalid(
            "named objects root dictionary missing or wrong type",
        ));
    };
    if root.handle != root_handle || !root.owner.is_null() {
        return Err(invalid(
            "named objects root dictionary identity or ownership disagrees",
        ));
    }
    let mut targets = root
        .entries
        .iter()
        .filter(|(name, _)| name == "ACAD_LAYOUT");
    let Some((_, handle)) = targets.next() else {
        return Err(invalid("named ACAD_LAYOUT dictionary missing"));
    };
    if handle.is_null() || targets.next().is_some() {
        return Err(invalid("named ACAD_LAYOUT dictionary is null or ambiguous"));
    }
    let Some(ObjectType::Dictionary(dictionary)) = doc.objects.get(handle) else {
        return Err(invalid("named ACAD_LAYOUT target missing or wrong type"));
    };
    if dictionary.handle != *handle
        || !owner(
            doc,
            *handle,
            dictionary.owner,
            root_handle,
            "ACAD_LAYOUT",
            recoveries,
        )
    {
        return Err(invalid(
            "named ACAD_LAYOUT dictionary identity or ownership disagrees",
        ));
    }
    if doc.header.acad_layout_dict_handle != *handle {
        if doc.header.acad_layout_dict_handle == root_handle
            || matches!(
                doc.objects.get(&doc.header.acad_layout_dict_handle),
                Some(ObjectType::Dictionary(d)) if d.entries.iter().any(|(name,h)| name == "ACAD_LAYOUT" || matches!(doc.objects.get(h), Some(ObjectType::Layout(_))))
            )
        {
            return Err(invalid(
                "layout dictionary cache points to a conflicting dictionary",
            ));
        }
        recoveries.push(diagnostic(
            "layout-dictionary-cache-recovered",
            "header.acad_layout_dict_handle",
            "unique named ACAD_LAYOUT relationship establishes the layout dictionary despite a stale cache",
        ));
    }
    Ok(*handle)
}

pub(crate) fn viewports(
    doc: &CadDocument,
    layout: &Layout,
    model: bool,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), Error> {
    for handle in layout
        .viewports
        .iter()
        .chain(std::iter::once(&layout.viewport))
        .filter(|h| !h.is_null())
    {
        let valid = if model {
            doc.vports.iter().any(|v| v.handle == *handle)
        } else {
            matches!(doc.get_entity(*handle), Some(EntityType::Viewport(v)) if v.common.owner_handle == layout.block_record)
        };
        if !valid && !(model && doc.semantic_inventory_v1().resolve(*handle).is_none()) {
            return Err(Error::InvalidStructure(if model {
                "Model layout references missing or wrong-kind VPORT record".into()
            } else {
                "Paper layout references missing or foreign VIEWPORT entity".into()
            }));
        }
    }
    if model {
        for (field, present) in [
            ("viewport", !layout.viewport.is_null()),
            ("viewports", layout.viewports.iter().any(|h| !h.is_null())),
        ] {
            if present {
                let handles = if field == "viewport" {
                    std::slice::from_ref(&layout.viewport)
                } else {
                    &layout.viewports
                };
                let missing = handles
                    .iter()
                    .filter(|h| !h.is_null() && doc.semantic_inventory_v1().resolve(**h).is_none())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                issues.push(diagnostic(
                    "model-viewport-selection",
                    format!("layout/{}.{field}", layout.name),
                    format!("Model VPORT selection is outside the supported drawing projection; VPORT settings are assessed separately; absent selection targets: {}", missing.join(", ")),
                ));
            }
        }
    }
    Ok(())
}
