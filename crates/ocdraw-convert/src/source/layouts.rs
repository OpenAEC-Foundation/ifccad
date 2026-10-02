use opencadcodec::objects::Layout;
use opencadcodec::{CadDocument, Handle};

pub(crate) fn overall_viewport_handle(document: &CadDocument, layout: &Layout) -> Option<Handle> {
    if matches!(document.get_entity(layout.viewport), Some(opencadcodec::EntityType::Viewport(viewport)) if viewport.id == 1 && viewport.common.owner_handle == layout.block_record)
    {
        return Some(layout.viewport);
    }
    let mut candidates = document.entities().filter_map(|entity| match entity {
        opencadcodec::EntityType::Viewport(viewport)
            if viewport.id == 1 && viewport.common.owner_handle == layout.block_record =>
        {
            Some(viewport.common.handle)
        }
        _ => None,
    });
    let candidate = candidates.next()?;
    candidates.next().is_none().then_some(candidate)
}

// Compare typed plot state; raw codes are only checked for unclassified fields.
pub(crate) fn is_untouched_scaffold(layout: &Layout, document: &CadDocument) -> bool {
    if layout.name != "Layout1" || layout.block_record != document.header.paper_space_block_handle {
        return false;
    }
    if document
        .entities()
        .any(|entity| entity.common().owner_handle == layout.block_record)
    {
        return false;
    }
    let mut candidate = layout.clone();
    let mut baseline = Layout::new("Layout1");
    candidate.handle = Handle::NULL;
    candidate.owner = Handle::NULL;
    candidate.block_record = Handle::NULL;
    candidate.tab_order = 0;
    if candidate.raw_plot_settings_codes.as_ref().is_some_and(|codes| {
        codes.iter().any(|(code, _)| !matches!(code, 1 | 2 | 4 | 6 | 7 | 40..=49 | 140..=143 | 147..=149 | 70 | 72..=78))
    }) {
        return false;
    }
    candidate.raw_plot_settings_codes = None;
    baseline.tab_order = 0;
    candidate == baseline
}

/// The codec needs its reserved primary paper block even in a model-only file.
/// Only a completely empty default record without a layout or insert reference
/// is bookkeeping. Authored metadata and content must stay represented.
pub(crate) fn is_empty_reserved_paper_block(
    record: &opencadcodec::BlockRecord,
    document: &CadDocument,
) -> bool {
    if record.handle != document.header.paper_space_block_handle
        || !record.name.eq_ignore_ascii_case("*Paper_Space")
        || !record.layout.is_null()
        || document.objects.values().any(|object| matches!(object,
            opencadcodec::objects::ObjectType::Layout(layout) if layout.block_record == record.handle))
        || document.entities().any(|entity| entity.common().owner_handle == record.handle
            || matches!(entity, opencadcodec::EntityType::Insert(insert) if insert.block_name.eq_ignore_ascii_case(&record.name)))
    {
        return false;
    }
    let mut candidate = record.clone();
    candidate.handle = Handle::NULL;
    candidate.block_entity_handle = Handle::NULL;
    candidate.block_end_handle = Handle::NULL;
    candidate.entity_handles.clear(); // Validated structural BLOCK/ENDBLK handles only.
    candidate.name = "*Paper_Space".into();
    candidate == opencadcodec::BlockRecord::paper_space()
}
