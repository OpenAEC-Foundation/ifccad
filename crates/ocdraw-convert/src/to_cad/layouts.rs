//! Adapt the fresh CAD document's paper scaffold to authored OCDraw layouts.
use super::OcdrawToCadError;
use ocdraw::ocdraw::{DrawingLayoutKind, OcdrawDocument};
use opencadcodec::{objects::ObjectType, CadDocument, EntityType, Handle};

pub(super) fn prepare_primary_paper_layout(
    drawing: &OcdrawDocument,
    document: &mut CadDocument,
) -> Result<(), OcdrawToCadError> {
    let primary = document.header.paper_space_block_handle;
    let handle = document
        .objects
        .values()
        .find_map(|object| match object {
            ObjectType::Layout(layout) if layout.block_record == primary => Some(layout.handle),
            _ => None,
        })
        .ok_or_else(|| OcdrawToCadError::Cad("fresh paper layout is missing".into()))?;
    let first = drawing
        .layouts
        .iter()
        .filter(|layout| layout.kind == DrawingLayoutKind::Paper)
        .min_by_key(|layout| layout.tab_index);
    let Some(ObjectType::Dictionary(dictionary)) = document
        .objects
        .get_mut(&document.header.acad_layout_dict_handle)
    else {
        return Err(OcdrawToCadError::Cad(
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
        .ok_or_else(|| OcdrawToCadError::Cad("fresh paper block is missing".into()))?;
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
    drawing: &OcdrawDocument,
    document: &mut CadDocument,
) -> Result<(), OcdrawToCadError> {
    for source in drawing
        .layouts
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
            .paper_canvases
            .iter()
            .any(|canvas| canvas.scope_id == source.scope_id);
        if has_canvas && overall.is_null() {
            let mut viewport = opencadcodec::entities::Viewport::new();
            viewport.id = 1;
            document
                .add_entity_to_layout(EntityType::Viewport(viewport), &source.name)
                .map_err(|error| {
                    OcdrawToCadError::Cad(format!("paper canvas {}: {error}", source.name))
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

/// Change the complete reserved Paper role, keeping layout/entity handles and owners.
pub(super) fn activate_paper_layout(
    document: &mut CadDocument,
    block: Handle,
) -> Result<(), OcdrawToCadError> {
    let active = document
        .block_records
        .get("*Paper_Space")
        .ok_or_else(|| OcdrawToCadError::Cad("reserved Paper block is missing".into()))?;
    let active_handle = active.handle;
    let selected_name = document
        .block_records
        .iter()
        .find(|b| b.handle == block)
        .ok_or_else(|| OcdrawToCadError::Cad("selected Paper block is missing".into()))?
        .name
        .clone();
    if active_handle != block {
        let mut temporary = "__active_paper_swap".to_string();
        while document.block_records.contains(&temporary) {
            temporary.push('_');
        }
        document
            .block_records
            .rename("*Paper_Space", &temporary)
            .map_err(OcdrawToCadError::Cad)?;
        document
            .block_records
            .rename(&selected_name, "*Paper_Space")
            .map_err(OcdrawToCadError::Cad)?;
        document
            .block_records
            .rename(&temporary, selected_name)
            .map_err(OcdrawToCadError::Cad)?;
    }
    // DWG writes an existing BLOCK begin marker verbatim. Its name must agree
    // with the renamed record, otherwise readback reconstructs the former role.
    let markers: Vec<_> = document
        .block_records
        .iter()
        .filter(|record| record.is_paper_space())
        .map(|record| (record.block_entity_handle, record.name.clone()))
        .collect();
    for (handle, name) in markers {
        if let Some(EntityType::Block(marker)) = document.get_entity_mut(handle) {
            marker.name = name;
        }
    }
    document.header.paper_space_block_handle = block;
    document.header.show_model_space = false;
    Ok(())
}
