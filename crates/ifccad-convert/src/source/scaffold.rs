//! Bounded recovery for the pinned DXF reader's incomplete Model-only scaffold.
use crate::{diagnostics::diagnostic, IfccadDiagnostic, IfccadDiagnosticAction};
use opencadcodec::{
    objects::{Dictionary, Layout, ObjectType},
    CadDocument,
};
use std::borrow::Cow;

/// Do not guess a root in a general authored object graph. The only accepted
/// shape is one empty default root plus a default Model dictionary/layout pair,
/// with the dictionary owner still pointing at the same absent header root.
pub(crate) fn prepare(doc: &CadDocument) -> (Cow<'_, CadDocument>, Vec<IfccadDiagnostic>) {
    let unchanged = || (Cow::Borrowed(doc), Vec::new());
    let stale = doc.header.named_objects_dict_handle;
    if stale.is_null()
        || doc.semantic_inventory_v1().resolve(stale).is_some()
        || doc.objects.len() != 3
    {
        return unchanged();
    }
    let Some(ObjectType::Dictionary(dictionary)) =
        doc.objects.get(&doc.header.acad_layout_dict_handle)
    else {
        return unchanged();
    };
    if dictionary.handle != doc.header.acad_layout_dict_handle
        || dictionary.owner != stale
        || dictionary.entries.len() != 1
        || dictionary.entries[0].0 != "Model"
    {
        return unchanged();
    }
    let mut expected_dictionary = Dictionary::new();
    expected_dictionary.handle = dictionary.handle;
    expected_dictionary.owner = stale;
    expected_dictionary.entries = dictionary.entries.clone();
    if *dictionary != expected_dictionary {
        return unchanged();
    }
    let layout_handle = dictionary.entries[0].1;
    let Some(ObjectType::Layout(layout)) = doc.objects.get(&layout_handle) else {
        return unchanged();
    };
    let models = doc
        .block_records
        .iter()
        .filter(|b| b.is_model_space())
        .collect::<Vec<_>>();
    if models.len() != 1
        || models[0].handle != doc.header.model_space_block_handle
        || models[0].layout != layout_handle
    {
        return unchanged();
    }
    let mut expected_layout = Layout::new("Model");
    expected_layout.handle = layout_handle;
    expected_layout.owner = dictionary.handle;
    expected_layout.block_record = models[0].handle;
    expected_layout.flags = 1;
    // Derived extents do not establish authored layout or workspace state.
    expected_layout.min_extents = layout.min_extents;
    expected_layout.max_extents = layout.max_extents;
    if *layout != expected_layout {
        return unchanged();
    }
    let roots = doc
        .objects
        .iter()
        .filter_map(|(h, o)| match o {
            ObjectType::Dictionary(d) if *h != dictionary.handle => Some((*h, d)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return unchanged();
    }
    let (root_handle, root) = roots[0];
    let mut expected_root = Dictionary::new();
    expected_root.handle = root_handle;
    if *root != expected_root {
        return unchanged();
    }

    let mut prepared = doc.clone();
    prepared.header.named_objects_dict_handle = root_handle;
    let Some(ObjectType::Dictionary(root)) = prepared.objects.get_mut(&root_handle) else {
        unreachable!()
    };
    root.entries.push(("ACAD_LAYOUT".into(), dictionary.handle));
    let Some(ObjectType::Dictionary(dictionary)) = prepared.objects.get_mut(&dictionary.handle)
    else {
        unreachable!()
    };
    dictionary.owner = root_handle;
    let mut recovery = diagnostic("model-scaffold-recovered", "header.named_objects_dict_handle",
        format!("unique empty root {root_handle} and default Model-only dictionary/layout chain complete the absent root {stale}; source unchanged"));
    recovery.action = IfccadDiagnosticAction::Recovery;
    (Cow::Owned(prepared), vec![recovery])
}
