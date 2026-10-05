//! Resolve CAD layout roles without trusting derived header caches.
use crate::diagnostics::diagnostic;
use crate::{IfccadConversionError as Error, IfccadDiagnostic};
use opencadcodec::objects::{Layout, ObjectType};
use opencadcodec::{CadDocument, EntityType, Handle};

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
    if dictionary.handle != *handle || dictionary.owner != root_handle {
        return Err(invalid(
            "named ACAD_LAYOUT dictionary identity or ownership disagrees",
        ));
    }
    if doc.header.acad_layout_dict_handle != *handle {
        if matches!(
            doc.objects.get(&doc.header.acad_layout_dict_handle),
            Some(ObjectType::Dictionary(_))
        ) {
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
        if !valid {
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
                issues.push(diagnostic(
                    "model-viewport-selection",
                    format!("layout/{}.{field}", layout.name),
                    "Model VPORT selection is outside the supported drawing projection; VPORT settings are assessed separately",
                ));
            }
        }
    }
    Ok(())
}
