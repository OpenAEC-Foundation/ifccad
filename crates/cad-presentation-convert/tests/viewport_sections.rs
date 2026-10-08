use cad_presentation_convert::*;
use opencadcodec::objects::{KnownXRecordKind as K, XRecordEntry, XRecordValue as V};
use opencadcodec::{CadDocument, EntityType, Handle};

fn document() -> (CadDocument, Handle, Handle) {
    let mut d = CadDocument::new();
    let layer = d.layers.get("0").unwrap().handle;
    let viewport = d
        .add_entity_to_layout(
            EntityType::Viewport(opencadcodec::entities::Viewport::new()),
            "Layout1",
        )
        .unwrap();
    d.set_layer_viewport_override(
        layer,
        K::LayerViewportColorOverride,
        viewport,
        V::Int32(-1023410175),
    );
    (d, layer, viewport)
}

#[test]
fn contradictory_record_owner_is_not_qualified() {
    let (mut d, layer, _) = document();
    d.xrecord_mut(layer, "ADSK_XREC_LAYER_COLOR_OVR")
        .unwrap()
        .owner = Handle::NULL;
    assert!(cad_override_sections(&d).is_empty());
}

#[test]
fn nested_unknown_section_and_extra_payload_stay_residual() {
    let (mut d, layer, _) = document();
    let record = d.xrecord_mut(layer, "ADSK_XREC_LAYER_COLOR_OVR").unwrap();
    let canonical = record.entries.clone();
    record.entries.push(XRecordEntry::string(102, "{FUTURE"));
    record.entries.extend(canonical);
    record.entries.push(XRecordEntry::string(102, "}"));
    record.synchronize_object_references();
    let sections = cad_override_sections(&d);
    assert_eq!(sections.len(), 1);
    assert!(consumed_override_objects(&d, &sections).is_empty());
}

#[test]
fn an_unrelated_dictionary_alias_to_a_known_record_is_not_consumed() {
    let (mut d, layer, _) = document();
    let record = d
        .xrecord(layer, "ADSK_XREC_LAYER_COLOR_OVR")
        .unwrap()
        .handle;
    let dictionary = d.extension_dictionary_handle(layer).unwrap();
    let opencadcodec::objects::ObjectType::Dictionary(object) =
        d.objects.get_mut(&dictionary).unwrap()
    else {
        panic!()
    };
    object.add_entry("FUTURE_APPLICATION_ALIAS", record);
    let sections = cad_override_sections(&d);
    let consumed = consumed_override_objects(&d, &sections);
    assert!(consumed.contains(&record));
    assert!(!consumed.contains(&dictionary));
}
