//! Bounded host lifecycle qualification; no external tolerance-payload claim.
use super::{Message, OpenCADStudio};
use codec::{EntityType, Format, Handle};
use serde_json::json;

fn source() -> codec::Document {
    use codec::entities::{Leader, LeaderCreationType, MultiLeader, Tolerance};
    let mut d = codec::Document::new();
    let target = d
        .add_entity(EntityType::Tolerance(Tolerance::with_text(
            codec::Vector3::new(30., 60., 0.),
            "{\\Fgdt;j}%%v0.1%%vA",
        )))
        .unwrap();
    for y in [40., 80.] {
        d.add_entity(EntityType::Leader(
            Leader::two_point(
                codec::Vector3::new(0., y, 0.),
                codec::Vector3::new(30., 60., 0.),
            )
            .with_creation_type(LeaderCreationType::WithTolerance)
            .with_annotation(target),
        ))
        .unwrap();
    }
    let mut m = MultiLeader::with_text(
        "Source leader",
        codec::Vector3::new(30., 20., 0.),
        vec![codec::Vector3::ZERO, codec::Vector3::new(20., 20., 0.)],
    );
    m.text_height = 2.5;
    m.context.text_height = 2.5;
    m.common.graphic_data = Some(
        codec::ProxyGraphics {
            records: vec![codec::ProxyGraphicRecord::Command(
                codec::ProxyGraphicCommand::Polyline(vec![
                    codec::Vector3::ZERO,
                    codec::Vector3::new(20., 20., 0.),
                ]),
            )],
        }
        .encode_checked()
        .unwrap(),
    );
    d.add_entity(EntityType::MultiLeader(Box::new(m))).unwrap();
    d
}
fn mleader(a: &OpenCADStudio, h: Handle) -> &codec::entities::MultiLeader {
    let EntityType::MultiLeader(v) = a.tabs[a.active_tab].scene.document.get_entity(h).unwrap()
    else {
        panic!("MLeader owner")
    };
    v
}
fn wires(a: &OpenCADStudio) -> String {
    let wires = a.tabs[a.active_tab].scene.entity_wires();
    assert!(!wires.is_empty());
    format!(
        "{:?}",
        wires
            .iter()
            .map(|w| (&w.name, w.aabb, &w.points, &w.text_verts))
            .collect::<Vec<_>>()
    )
}
fn history(a: &mut OpenCADStudio, op: &str) {
    let _ = a.update(Message::CloseModal);
    let r = a.automation_op(&json!({"op":op}).to_string());
    assert_ne!(r["ok"], false, "{r}");
}
fn shared_target(d: &codec::Document) {
    let targets = d
        .entities()
        .filter_map(|e| match e {
            EntityType::Leader(v) => Some(v.annotation_handle),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0], targets[1]);
    assert!(matches!(
        d.get_entity(targets[0]),
        Some(EntityType::Tolerance(_))
    ));
}
#[test]
fn leader_slice_smoke_picture_noop_and_grip_invalidation() {
    let drawing = source();
    let mut entity = drawing
        .entities()
        .find(|e| matches!(e, EntityType::MultiLeader(_)))
        .unwrap()
        .clone();
    let original = entity.clone();
    crate::scene::view::dispatch::apply_geom_prop(&mut entity, "text_string", "Source leader");
    assert_eq!(entity, original, "a no-op must retain the saved picture");
    crate::scene::view::dispatch::apply_grip(
        &mut entity,
        0,
        crate::scene::model::object::GripApply::Translate(glam::DVec3::new(5., 3., 0.)),
    );
    assert_ne!(entity, original);
    assert!(
        entity.common().graphic_data.is_none(),
        "a real grip invalidates the old picture"
    );
}

#[test]
fn leader_slice_smoke_edit_grip_history_and_native_save() {
    let evidence = std::env::var("IFCCAD_HOST_EVIDENCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::temp_dir().join(format!("ifccad-leader-smoke-{}", std::process::id()))
        });
    std::fs::create_dir_all(&evidence).unwrap();
    for format in ["ifcx", "dwg"] {
        let d = source();
        let input = if format == "ifcx" {
            codec::Ifcx.write(&d).unwrap().0
        } else {
            codec::DwgWriter::write_to_vec(&d).unwrap()
        };
        let path = evidence.join(format!("input.{format}"));
        std::fs::write(&path, input).unwrap();
        let mut a = OpenCADStudio::new_for_test();
        let r = a.automation_op(&json!({"op":"open","path":path}).to_string());
        assert_ne!(r["ok"], false, "{format}: {r}");
        let tab = a.active_tab;
        shared_target(&a.tabs[tab].scene.document);
        let h = a.tabs[tab]
            .scene
            .document
            .entities()
            .find_map(|e| match e {
                EntityType::MultiLeader(v) => Some(v.common.handle),
                _ => None,
            })
            .unwrap();
        let original = mleader(&a, h).clone();
        assert!(original.common.graphic_data.is_some());
        let initial = wires(&a);
        a.tabs[tab]
            .scene
            .replace_selection([h].into_iter().collect());
        a.refresh_properties();
        let _ = a.update(Message::PropGeomInput {
            field: "text_string",
            value: "Edited leader".into(),
        });
        let _ = a.update(Message::PropGeomCommit("text_string"));
        assert_eq!(
            mleader(&a, h).context.text_string,
            "Edited leader",
            "{format}"
        );
        assert!(
            mleader(&a, h).common.graphic_data.is_none(),
            "{format}: stale saved picture must be removed after definition edit"
        );
        let edited = wires(&a);
        assert_ne!(edited, initial);
        history(&mut a, "undo");
        assert_eq!(mleader(&a, h).context.text_string, "Source leader");
        assert_eq!(
            mleader(&a, h).common.graphic_data,
            original.common.graphic_data
        );
        assert_eq!(wires(&a), initial);
        history(&mut a, "redo");
        assert_eq!(mleader(&a, h).context.text_string, "Edited leader");
        assert_eq!(wires(&a), edited);
        let point = mleader(&a, h).context.leader_roots[0].lines[0].points[0];
        a.push_undo_snapshot(tab, "Leader grip smoke");
        a.tabs[tab].scene.apply_grip(
            h,
            0,
            crate::scene::model::object::GripApply::Translate(glam::DVec3::new(5., 3., 0.)),
        );
        a.tabs[tab]
            .scene
            .bump_entities_after_parametric_solve(&[(h, crate::scene::ChangeKind::Modified)]);
        assert_eq!(
            mleader(&a, h).context.leader_roots[0].lines[0].points[0],
            point + codec::Vector3::new(5., 3., 0.)
        );
        let moved = wires(&a);
        assert_ne!(moved, edited);
        history(&mut a, "undo");
        assert_eq!(wires(&a), edited);
        history(&mut a, "redo");
        assert_eq!(wires(&a), moved);
        shared_target(&a.tabs[tab].scene.document);
        a.prepare_native_save(tab);
        let saved = a.tabs[tab].scene.document_for_save();
        let bytes = crate::io::save_to_bytes(&saved, "ifcx", saved.version).unwrap();
        let strict = codec::ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
        codec::ifccad::validate_ifccad_document(strict.document()).unwrap();
        let reopened =
            crate::io::load_bytes_finalized(std::path::Path::new("leaders.ifcx"), bytes.clone())
                .unwrap();
        shared_target(&reopened);
        assert!(reopened.entities().any(|e|matches!(e,EntityType::MultiLeader(v)if v.context.text_string=="Edited leader"&&v.common.graphic_data.is_none())));
        std::fs::write(evidence.join(format!("after-{format}.ifcx")), bytes).unwrap();
        std::fs::write(evidence.join(format!("after-{format}-graphics.txt")), moved).unwrap();
    }
}
