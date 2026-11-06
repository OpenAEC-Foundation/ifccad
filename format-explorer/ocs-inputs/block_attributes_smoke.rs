//! Bounded local IFCCAD open/edit/history and existing ATTSYNC qualification.
use super::{Message, OpenCADStudio};
use codec::{EntityType, Handle};
use serde_json::json;
fn opened() -> OpenCADStudio {
    let mut a = OpenCADStudio::new_for_test();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ifccad-block-attributes.ifcx");
    let r = a.automation_op(&json!({"op":"open","path":path}).to_string());
    assert_ne!(r["ok"], false, "{r}");
    a
}
fn insert<'a>(a: &'a OpenCADStudio, h: Handle) -> &'a codec::entities::Insert {
    let EntityType::Insert(i) = a.tabs[a.active_tab].scene.document.get_entity(h).unwrap() else {
        panic!("INSERT")
    };
    i
}
fn wires(a: &OpenCADStudio) -> String {
    let w = a.tabs[a.active_tab].scene.entity_wires();
    assert!(!w.is_empty(), "dependent graphics must exist");
    format!(
        "{:?}",
        w.iter()
            .map(|w| (&w.name, w.aabb, &w.points, &w.text_verts))
            .collect::<Vec<_>>()
    )
}
fn history(a: &mut OpenCADStudio, op: &str) {
    let _ = a.update(Message::CloseModal);
    assert!(a.active_modal.is_none());
    let r = a.automation_op(&json!({"op":op}).to_string());
    assert_ne!(r["ok"], false, "{r}");
}
fn attribute_snapshot(doc: &codec::Document) -> Vec<String> {
    let owner_name = |h| {
        doc.block_records
            .iter()
            .find(|b| b.handle == h)
            .map(|b| b.name.clone())
    };
    let mut rows=doc.entities().filter_map(|e|{let EntityType::Insert(i)=e else{return None};Some(json!({"owner":owner_name(i.common.owner_handle),"block":i.block_name,"position":i.insert_point,"attributes":i.attributes.iter().map(|a|json!({"tag":a.tag,"value":a.value,"position":a.insertion_point,"alignment":(a.horizontal_alignment != codec::entities::attribute_definition::HorizontalAlignment::Left || a.vertical_alignment != codec::entities::attribute_definition::VerticalAlignment::Baseline).then_some(a.alignment_point),"height":a.height,"flags":a.flags.to_bits(),"locked":a.lock_position,"thickness":a.thickness,"mtext":a.embedded_mtext.as_ref().map(|m|json!({"value":m.value,"position":m.insertion_point,"width":m.rectangle_width,"height":m.height}))})).collect::<Vec<_>>()})).map(|v|v.to_string())}).collect::<Vec<_>>();
    rows.sort();
    rows
}
fn save(a: &mut OpenCADStudio, name: &str) {
    a.prepare_native_save(a.active_tab);
    let d = a.tabs[a.active_tab].scene.document_for_save();
    let bytes = crate::io::save_to_bytes(&d, "ifcx", d.version).unwrap();
    let reopened = crate::io::load_bytes_finalized(
        std::path::Path::new("ifccad-block-attributes.ifcx"),
        bytes.clone(),
    )
    .unwrap();
    assert_eq!(
        attribute_snapshot(&d),
        attribute_snapshot(&reopened),
        "{name}: saved attributes must reopen with the same values, flags and frames"
    );
    let n = codec::Ifcx.export_ifccad(&reopened).unwrap();
    codec::ifccad::validate_ifccad_document(&n.drawing).unwrap();
    if let Ok(root) = std::env::var("IFCCAD_HOST_EVIDENCE") {
        let root = std::path::Path::new(&root);
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join(format!("{name}.ifcx")), bytes).unwrap();
        std::fs::write(root.join(format!("{name}-graphics.txt")), wires(a)).unwrap();
    }
}
#[test]
fn block_attributes_smoke_ifcx_editor_grip_and_history() {
    let mut a = opened();
    let tab = a.active_tab;
    let h = a.tabs[tab]
        .scene
        .document
        .entities()
        .find_map(|e| {
            if let EntityType::Insert(i) = e {
                (i.common.owner_handle
                    == a.tabs[tab]
                        .scene
                        .document
                        .block_records
                        .get("*Model_Space")
                        .unwrap()
                        .handle
                    && i.block_name == "ATTR_REVIEW_DOOR"
                    && i.attributes.iter().any(|a| a.value == "D-101"))
                .then_some(i.common.handle)
            } else {
                None
            }
        })
        .unwrap();
    assert!(insert(&a, h)
        .attributes
        .iter()
        .any(|a| a.tag == "HIDDEN" && a.flags.invisible));
    save(&mut a, "edit-before");
    let original = wires(&a);
    let _ = a.update(Message::AttrEditorOpen(h));
    let code = a
        .attr_editor_rows
        .iter()
        .position(|r| r.tag == "CODE")
        .unwrap();
    let _ = a.update(Message::AttrEditorInput {
        idx: code,
        value: "D-EDIT".into(),
    });
    let _ = a.update(Message::AttrEditorApply);
    assert_eq!(
        insert(&a, h)
            .attributes
            .iter()
            .find(|a| a.tag == "CODE")
            .unwrap()
            .value,
        "D-EDIT"
    );
    let text_edit = wires(&a);
    assert!(
        text_edit != original,
        "Text edit must refresh visible graphics"
    );
    history(&mut a, "undo");
    assert_eq!(insert(&a, h).attributes[code].value, "D-101");
    assert_eq!(wires(&a), original);
    history(&mut a, "redo");
    assert_eq!(insert(&a, h).attributes[code].value, "D-EDIT");
    assert_eq!(wires(&a), text_edit);
    let _ = a.update(Message::AttrEditorOpen(h));
    let room_index = a
        .attr_editor_rows
        .iter()
        .position(|r| r.tag == "ROOM")
        .unwrap();
    assert_eq!(
        a.attr_editor_rows[room_index].value,
        insert(&a, h).attributes[room_index]
            .embedded_mtext
            .as_ref()
            .unwrap()
            .value
    );
    let _ = a.update(Message::AttrEditorInput {
        idx: room_index,
        value: "Changed room\\PChanged detail".into(),
    });
    let _ = a.update(Message::AttrEditorApply);
    let room = insert(&a, h)
        .attributes
        .iter()
        .find(|a| a.tag == "ROOM")
        .unwrap();
    assert_eq!(room.value, "Changed room\\PChanged detail");
    assert_eq!(
        room.embedded_mtext.as_ref().unwrap().value,
        room.value,
        "active MText must follow the editor"
    );
    let m_edit = wires(&a);
    assert!(
        m_edit != text_edit,
        "MText edit must refresh visible graphics"
    );
    history(&mut a, "undo");
    assert_ne!(
        insert(&a, h).attributes[room_index]
            .embedded_mtext
            .as_ref()
            .unwrap()
            .value,
        "Changed room\\PChanged detail"
    );
    assert_eq!(wires(&a), text_edit);
    history(&mut a, "redo");
    assert_eq!(
        insert(&a, h).attributes[room_index]
            .embedded_mtext
            .as_ref()
            .unwrap()
            .value,
        "Changed room\\PChanged detail"
    );
    assert_eq!(wires(&a), m_edit);
    let code_index = insert(&a, h)
        .attributes
        .iter()
        .position(|a| a.tag == "CODE")
        .unwrap();
    let before = insert(&a, h).attributes[code_index].insertion_point;
    a.push_undo_snapshot(tab, "Attribute grip smoke");
    a.tabs[tab].scene.apply_grip(
        h,
        1 + code_index,
        crate::scene::model::object::GripApply::Translate(glam::DVec3::new(5., 3., 0.)),
    );
    assert_eq!(
        insert(&a, h).attributes[code_index].insertion_point,
        before + codec::Vector3::new(5., 3., 0.)
    );
    // The viewport grip commit refreshes the resident representation after apply_grip.
    a.tabs[tab]
        .scene
        .bump_entities_after_parametric_solve(&[(h, crate::scene::ChangeKind::Modified)]);
    let grip = wires(&a);
    assert_ne!(grip, m_edit);
    history(&mut a, "undo");
    assert_eq!(wires(&a), m_edit);
    history(&mut a, "redo");
    assert_eq!(wires(&a), grip);
    let locked = insert(&a, h)
        .attributes
        .iter()
        .position(|a| a.tag == "VERIFY")
        .unwrap();
    let before = insert(&a, h).attributes[locked].insertion_point;
    a.tabs[tab].scene.apply_grip(
        h,
        1 + locked,
        crate::scene::model::object::GripApply::Translate(glam::DVec3::new(5., 3., 0.)),
    );
    assert_eq!(insert(&a, h).attributes[locked].insertion_point, before);
    save(&mut a, "edit-after");
}
#[test]
fn block_attributes_smoke_existing_attsync_multiple_values_and_history() {
    use codec::entities::{AttributeDefinition, MText};
    let mut a = opened();
    let tab = a.active_tab;
    let text = AttributeDefinition::new("CODE".into(), "Code".into(), "default".into());
    let mut room = AttributeDefinition::new("ROOM".into(), "Room".into(), "default room".into());
    room.is_multiline = true;
    room.mtext_flag = codec::entities::attribute_definition::MTextFlag::ConstantMultiLine;
    room.embedded_mtext = Some(Box::new(MText::with_value(
        "default room",
        codec::Vector3::new(0., -18., 0.),
    )));
    room.insertion_point = codec::Vector3::new(0., -18., 0.);
    a.tabs[tab].scene.define_block_raw(
        "ATTSYNC_REVIEW",
        codec::Vector3::ZERO,
        vec![
            EntityType::AttributeDefinition(text),
            EntityType::AttributeDefinition(room),
        ],
    );
    let state = a.automation_op(r#"{"protocol":1,"op":"state"}"#);
    let r=a.automation_op(&json!({"protocol":1,"op":"entities_create","request_id":"sync-fixture","document_id":state["document_id"],"entities":[{"type":"Insert","block":"ATTSYNC_REVIEW","position":[400,0],"attributes":{"CODE":"SYNC-A","ROOM":"Room A\\PDetail"}},{"type":"Insert","block":"ATTSYNC_REVIEW","position":[500,0],"attributes":{"CODE":"SYNC-B","ROOM":"Room B\\PDetail"}}]}).to_string());
    assert_eq!(r["ok"], true, "{r}");
    let handles = a.tabs[tab]
        .scene
        .document
        .entities()
        .filter_map(|e| {
            if let EntityType::Insert(i) = e {
                (i.block_name == "ATTSYNC_REVIEW").then_some(i.common.handle)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(handles.len(), 2);
    let values = handles
        .iter()
        .map(|h| {
            insert(&a, *h)
                .attributes
                .iter()
                .map(|a| (a.tag.clone(), a.value.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    save(&mut a, "attsync-before");
    let owner = a.tabs[tab]
        .scene
        .document
        .block_records
        .get("ATTSYNC_REVIEW")
        .unwrap()
        .handle;
    let def = a.tabs[tab]
        .scene
        .document
        .entities()
        .find_map(|e| {
            if let EntityType::AttributeDefinition(d) = e {
                (d.common.owner_handle == owner && d.tag == "CODE").then_some(d.common.handle)
            } else {
                None
            }
        })
        .unwrap();
    a.push_undo_snapshot(tab, "Edit definitions for ATTSYNC");
    a.tabs[tab].scene.apply_grip(
        def,
        0,
        crate::scene::model::object::GripApply::Translate(glam::DVec3::new(4., 0., 0.)),
    );
    let mut new =
        AttributeDefinition::new("NEW_NOTE".into(), "New note".into(), "New\\Pdefault".into());
    new.common.owner_handle = owner;
    new.is_multiline = true;
    new.mtext_flag = codec::entities::attribute_definition::MTextFlag::ConstantMultiLine;
    new.insertion_point = codec::Vector3::new(0., -36., 0.);
    new.embedded_mtext = Some(Box::new(MText::with_value(
        "New\\Pdefault",
        new.insertion_point,
    )));
    a.tabs[tab]
        .scene
        .document
        .add_entity(EntityType::AttributeDefinition(new))
        .unwrap();
    a.tabs[tab].scene.rebuild_derived_caches();
    save(&mut a, "attsync-unsynced");
    let unsynced = wires(&a);
    let r = a.automation_op(r#"{"op":"run","cmd":"ATTSYNC Name ATTSYNC_REVIEW"}"#);
    assert_eq!(r["ok"], true, "{r}");
    for (j, h) in handles.iter().enumerate() {
        let i = insert(&a, *h);
        assert_eq!(i.attributes.len(), 3);
        for (tag, value) in &values[j] {
            assert_eq!(
                &i.attributes.iter().find(|a| &a.tag == tag).unwrap().value,
                value
            );
        }
        let code = i.attributes.iter().find(|a| a.tag == "CODE").unwrap();
        assert_eq!(code.insertion_point.x, i.insert_point.x + 4.);
        let new = i.attributes.iter().find(|a| a.tag == "NEW_NOTE").unwrap();
        assert_eq!(new.embedded_mtext.as_ref().unwrap().value, "New\\Pdefault");
    }
    let synced = wires(&a);
    assert_ne!(synced, unsynced);
    history(&mut a, "undo");
    assert_eq!(wires(&a), unsynced);
    assert_eq!(insert(&a, handles[0]).attributes.len(), 2);
    history(&mut a, "redo");
    assert_eq!(wires(&a), synced);
    assert_eq!(insert(&a, handles[0]).attributes.len(), 3);
    save(&mut a, "attsync-after");
}
