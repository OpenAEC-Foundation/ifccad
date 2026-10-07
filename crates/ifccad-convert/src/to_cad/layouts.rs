//! CAD Paper ownership and layout allocation; plot mapping is separate.
use crate::{IfccadConversionError, IfccadDiagnostic, IfccadMappings};
use ocdraw::ifccad::IfccadPaperLayout;
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, EntityType, Handle};
pub(crate) fn allocate(
    document: &mut CadDocument,
    papers: &[IfccadPaperLayout],
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Vec<(u64, Handle)>, IfccadConversionError> {
    // Check before constructing any target so native u32 tabs never truncate to i16.
    let mut names = std::collections::BTreeSet::from(["MODEL".to_string()]);
    for paper in papers {
        if !names.insert(paper.name.to_uppercase()) {
            return Err(IfccadConversionError::CadConstruction(
                "ambiguous CAD layout name".into(),
            ));
        }
        i16::try_from(paper.tab_index).map_err(|_| {
            IfccadConversionError::CadConstruction("layout tab index exceeds CAD i16 range".into())
        })?;
    }
    let mut ordered: Vec<_> = papers.iter().collect();
    ordered.sort_by_key(|p| p.tab_index);
    let mut owners = Vec::new();
    for paper in ordered {
        let handle = if owners.is_empty() {
            let Some((handle, old_name)) = document.objects.iter().find_map(|(h, o)| match o {
                ObjectType::Layout(l)
                    if l.block_record == document.header.paper_space_block_handle =>
                {
                    Some((*h, l.name.clone()))
                }
                _ => None,
            }) else {
                return Err(IfccadConversionError::CadConstruction(
                    "missing runtime Paper scaffold".into(),
                ));
            };
            if let Some(ObjectType::Dictionary(dictionary)) = document
                .objects
                .get_mut(&document.header.acad_layout_dict_handle)
            {
                for (name, target) in &mut dictionary.entries {
                    if *target == handle && *name == old_name {
                        *name = paper.name.clone();
                    }
                }
            }
            let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
                unreachable!()
            };
            layout.name = paper.name.clone();
            let mut viewport = opencadcodec::entities::Viewport::new();
            viewport.id = 1;
            document
                .add_entity_to_layout(EntityType::Viewport(viewport), &paper.name)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
            handle
        } else {
            document
                .add_layout(&paper.name)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?
        };
        let Some(ObjectType::Layout(layout)) = document.objects.get_mut(&handle) else {
            unreachable!()
        };
        layout.tab_order = i16::try_from(paper.tab_index).expect("checked tab");
        crate::mapping::layout::apply_settings(layout, &paper.settings, issues)?;
        let owner = layout.block_record;
        // DWG implicit Paper mode omits a marker's owner. Unlike drawable
        // membership, the reader cannot recover that owner from the record's
        // ordered contents. Explicit owned markers retain every sheet boundary.
        let record = document
            .block_records
            .iter()
            .find(|b| b.handle == owner)
            .expect("allocated Paper record");
        let mut begin = opencadcodec::entities::Block::new(&record.name, record.base_point);
        begin.common.handle = record.block_entity_handle;
        begin.common.owner_handle = owner;
        begin.common.entity_mode = Some(0);
        let mut end = opencadcodec::entities::BlockEnd::new();
        end.common.handle = record.block_end_handle;
        end.common.owner_handle = owner;
        end.common.entity_mode = Some(0);
        for marker in [EntityType::Block(begin), EntityType::BlockEnd(end)] {
            document
                .add_entity(marker)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
        }
        owners.push((paper.id, owner));
        mappings.layouts.insert(paper.id, handle);
    }
    Ok(owners)
}
