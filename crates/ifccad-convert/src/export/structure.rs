use super::SourceStructureProblem;
use cadcodec::objects::ObjectType;
use cadcodec::{CadDocument, Handle};

pub(crate) struct ModelSpaceInfo<'a> {
    pub(crate) block_handle: Handle,
    pub(crate) layout_name: &'a str,
}

pub(crate) fn inspect_model_space(
    document: &CadDocument,
) -> Result<ModelSpaceInfo<'_>, Vec<SourceStructureProblem>> {
    let model_space_block = document.header.model_space_block_handle;
    if model_space_block == Handle::NULL {
        return Err(vec![SourceStructureProblem::ModelSpaceBlockMissing]);
    }

    let mut problems = Vec::new();
    if !document
        .block_records
        .iter()
        .any(|record| record.handle == model_space_block)
    {
        problems.push(SourceStructureProblem::ModelSpaceBlockRecordMissing { model_space_block });
    }

    let layouts = document
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout) if layout.block_record == model_space_block => Some(layout),
            _ => None,
        })
        .collect::<Vec<_>>();
    match layouts.len() {
        0 => problems.push(SourceStructureProblem::ModelLayoutMissing { model_space_block }),
        1 => {}
        count => problems.push(SourceStructureProblem::MultipleModelLayouts {
            model_space_block,
            count,
        }),
    }

    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(ModelSpaceInfo {
        block_handle: model_space_block,
        layout_name: &layouts[0].name,
    })
}
