use super::CadSourceStructureProblem;
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, Handle};
use std::borrow::Cow;

pub(crate) struct ModelSpaceInfo<'a> {
    pub(crate) block_handle: Handle,
    pub(crate) layout_name: &'a str,
}

/// Recover a stale header cache left by the pinned DXF reader when its
/// post-read default-handle repair changes the header but not the authored
/// model block record. The block and exactly one layout must agree first.
pub(crate) fn with_recovered_model_space_handle(document: &CadDocument) -> Cow<'_, CadDocument> {
    let header_handle = document.header.model_space_block_handle;
    if header_handle == Handle::NULL
        || document
            .block_records
            .iter()
            .any(|record| record.handle == header_handle)
    {
        return Cow::Borrowed(document);
    }
    let Some(record) = document.block_records.get("*Model_Space") else {
        return Cow::Borrowed(document);
    };
    let layouts = document
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout) if layout.block_record == record.handle => Some(layout),
            _ => None,
        })
        .collect::<Vec<_>>();
    if record.handle == Handle::NULL
        || layouts.len() != 1
        || (record.layout != Handle::NULL && record.layout != layouts[0].handle)
    {
        return Cow::Borrowed(document);
    }
    let mut recovered = document.clone();
    recovered.header.model_space_block_handle = record.handle;
    Cow::Owned(recovered)
}

pub(crate) fn inspect_model_space(
    document: &CadDocument,
) -> Result<ModelSpaceInfo<'_>, Vec<CadSourceStructureProblem>> {
    let model_space_block = document.header.model_space_block_handle;
    if model_space_block == Handle::NULL {
        return Err(vec![CadSourceStructureProblem::ModelSpaceBlockMissing]);
    }

    let mut problems = Vec::new();
    if !document
        .block_records
        .iter()
        .any(|record| record.handle == model_space_block)
    {
        problems
            .push(CadSourceStructureProblem::ModelSpaceBlockRecordMissing { model_space_block });
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
        0 => problems.push(CadSourceStructureProblem::ModelLayoutMissing { model_space_block }),
        1 => {}
        count => problems.push(CadSourceStructureProblem::MultipleModelLayouts {
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
