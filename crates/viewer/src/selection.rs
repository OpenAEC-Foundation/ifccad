//! Transient selection links qualified against the CAD file readback.
use ifccad_convert::IfccadMappings;
use ocdraw::ifccad::IfccadDocument;
use ocdraw_convert::opencadcodec::{objects::ObjectType, CadDocument, EntityType, Handle};
use serde_json::{json, Value};

pub(crate) struct IfccadSelectionCandidate {
    path: String,
    handle: Handle,
    owner: Handle,
    kind: std::mem::Discriminant<EntityType>,
    layout: String,
}

pub(crate) fn ifccad_candidates(
    drawing: &IfccadDocument,
    mappings: &IfccadMappings,
    cad: &CadDocument,
) -> Vec<IfccadSelectionCandidate> {
    let mut candidates = Vec::new();
    for (layout_id, entities) in std::iter::once((drawing.model.id, &drawing.model.entities))
        .chain(drawing.paper_layouts.iter().map(|p| (p.id, &p.entities)))
    {
        let layout = mappings.layouts.cad_handle(layout_id).and_then(|handle| {
            cad.objects.values().find_map(|object| match object {
                ObjectType::Layout(layout) if layout.handle == handle => Some(layout.name.clone()),
                _ => None,
            })
        });
        let Some(layout) = layout else { continue };
        for entity in entities {
            let Some(handle) = mappings.entities.cad_handle(entity.id) else {
                continue;
            };
            let Some(target) = cad.get_entity(handle) else {
                continue;
            };
            candidates.push(IfccadSelectionCandidate {
                path: format!("/cad/d{}/e{}", drawing.drawing_id, entity.id),
                handle,
                owner: target.common().owner_handle,
                kind: std::mem::discriminant(target),
                layout: layout.clone(),
            });
        }
    }
    candidates
}

pub(crate) fn qualified_ifccad_selection(
    candidates: &[IfccadSelectionCandidate],
    readback: &CadDocument,
) -> Value {
    let entities: Vec<_> = candidates.iter().filter_map(|candidate| {
        let entity = readback.get_entity(candidate.handle)?;
        (entity.common().owner_handle == candidate.owner
            && std::mem::discriminant(entity) == candidate.kind)
            .then(|| json!({"path":candidate.path,"handle":format!("{:X}",candidate.handle.value()),"layout":candidate.layout}))
    }).collect();
    json!({"format":"ifccad","entities":entities})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_or_changed_readback_identity_is_not_linked() {
        let bytes = include_bytes!("../../../examples/ifccad/hello-line-patterns.ifcx");
        let source = ocdraw::ifccad::load_ifccad_bytes(bytes, Default::default()).unwrap();
        let converted =
            ifccad_convert::ifccad_source_to_cad_document(&source, Default::default()).unwrap();
        let candidates = ifccad_candidates(
            source.document(),
            converted.mappings(),
            converted.document(),
        );
        assert!(!candidates.is_empty());
        let mut readback = converted.document().clone();
        readback.remove_entity(candidates[0].handle);
        assert_eq!(
            qualified_ifccad_selection(&candidates, &readback)["entities"]
                .as_array()
                .unwrap()
                .len(),
            candidates.len() - 1
        );
        let mut readback = converted.document().clone();
        readback
            .get_entity_mut(candidates[0].handle)
            .unwrap()
            .common_mut()
            .owner_handle = Handle::NULL;
        assert_eq!(
            qualified_ifccad_selection(&candidates, &readback)["entities"]
                .as_array()
                .unwrap()
                .len(),
            candidates.len() - 1
        );
    }
}
