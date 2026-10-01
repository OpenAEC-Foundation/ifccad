//! Adapt the fresh CAD document's paper scaffold to authored OCDraw layouts.
use super::DirectImportError;
use cadcodec::{objects::ObjectType, CadDocument, EntityType, Handle};
use ocdraw::ocdraw::{DrawingLayoutKind, ValidatedDrawing};

pub(super) fn prepare_primary_paper_layout(
    drawing: &ValidatedDrawing,
    document: &mut CadDocument,
) -> Result<(), DirectImportError> {
    let primary = document.header.paper_space_block_handle;
    let handle = document
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.block_record == primary => Some(layout.handle),
            _ => None,
        })
        .ok_or_else(|| DirectImportError::Cad("fresh paper layout is missing".into()))?;
    let first = drawing
        .typed_layouts()
        .iter()
        .filter(|layout| layout.kind == DrawingLayoutKind::Paper)
        .min_by_key(|layout| layout.tab_index);
    let Some(ObjectType::Dictionary(dictionary)) = document
        .objects
        .get_mut(&document.header.acad_layout_dict_handle)
    else {
        return Err(DirectImportError::Cad(
            "fresh layout dictionary is missing".into(),
        ));
    };
    // Update the dictionary by handle, so an authored later Layout1 stays distinct.
    dictionary.entries.retain(|(_, target)| *target != handle);
    dictionary
        .hard_owner_entries
        .retain(|name| name != "Layout1");
    if let Some(source) = first {
        dictionary.add_entry(&source.name, handle);
        let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
            unreachable!()
        };
        layout.name.clone_from(&source.name);
    } else {
        document.objects.remove(&handle);
    }
    let record = document
        .block_records
        .iter_mut()
        .find(|record| record.handle == primary)
        .ok_or_else(|| DirectImportError::Cad("fresh paper block is missing".into()))?;
    record.layout = if first.is_some() {
        handle
    } else {
        Handle::NULL
    };
    // Keep the codec's reserved paper block/header scaffolding even without a
    // visible paper layout. A block record does not itself define a layout tab.
    Ok(())
}

pub(super) fn prepare_paper_canvases(
    drawing: &ValidatedDrawing,
    document: &mut CadDocument,
) -> Result<(), DirectImportError> {
    for source in drawing
        .typed_layouts()
        .iter()
        .filter(|layout| layout.kind == DrawingLayoutKind::Paper)
    {
        let (handle, overall) = document
            .objects
            .values()
            .find_map(|object| match object {
                ObjectType::Layout(layout) if layout.name == source.name => {
                    Some((layout.handle, layout.viewport))
                }
                _ => None,
            })
            .expect("allocated paper layout");
        let has_canvas = drawing
            .paper_canvases()
            .iter()
            .any(|canvas| canvas.scope_id == source.scope_id);
        if has_canvas && overall.is_null() {
            let mut viewport = cadcodec::entities::Viewport::new();
            viewport.id = 1;
            document
                .add_entity_to_layout(EntityType::Viewport(viewport), &source.name)
                .map_err(|error| {
                    DirectImportError::Cad(format!("paper canvas {}: {error}", source.name))
                })?;
        } else if !has_canvas && !overall.is_null() {
            document.remove_entity(overall);
            for record in document.block_records.iter_mut() {
                record.entity_handles.retain(|handle| *handle != overall);
            }
            let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
                unreachable!()
            };
            layout.viewport = Handle::NULL;
            layout.viewports.retain(|target| *target != overall);
        }
    }
    Ok(())
}
