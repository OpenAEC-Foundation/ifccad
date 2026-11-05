use ifccad::{
    encode_ifccad_cbor_document, encode_ifccad_document, load_ifccad_bytes, load_ifccad_cbor_bytes,
};
use serde_json::{json, Value};

fn fixture() -> Value {
    let mut source: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/text.ifcx")).unwrap();
    let mut changed = false;
    for node in source["data"].as_array_mut().unwrap() {
        let Some(attrs) = node["attributes"].as_object_mut() else {
            continue;
        };
        let Some(text) = attrs.remove("ifccad::text") else {
            continue;
        };
        attrs.insert(
            "ifccad::attributeDefinition".into(),
            json!({
                "tag": "DOOR_CODE", "prompt": "Door code", "text": text,
                "flags": { "constant": true, "invisible": true, "verify": true, "preset": true },
                "lockPosition": true, "fieldLength": 12
            }),
        );
        changed = true;
        break;
    }
    assert!(changed, "fixture must contain Text");
    source
}

#[test]
fn attribute_definition_keeps_default_and_flags_through_native_storage() {
    let source = fixture();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    let written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let definition = written["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|n| n["attributes"].get("ifccad::attributeDefinition"))
        .unwrap();
    assert_eq!(definition["tag"], "DOOR_CODE");
    assert_eq!(definition["prompt"], "Door code");
    assert_eq!(definition["flags"]["constant"], true);
    assert_eq!(definition["flags"]["invisible"], true);
    assert_eq!(definition["lockPosition"], true);
    let binary = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let returned = load_ifccad_cbor_bytes(binary.bytes(), Default::default()).unwrap();
    assert_eq!(returned.document(), loaded.document());
}

fn with_insert() -> Value {
    let mut source = fixture();
    let def = source["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::attributeDefinition").is_some())
        .unwrap()
        .clone();
    let def_path = def["path"].as_str().unwrap().to_owned();
    let payload = &def["attributes"]["ifccad::attributeDefinition"];
    let insert = "/cad/d1/e9007199254740992";
    let child = "/cad/d1/e9007199254740993";
    for n in source["data"].as_array_mut().unwrap() {
        if let Some(d) = n["attributes"].get_mut("ifccad::drawing") {
            d["nextEntityId"] = json!(9007199254740994u64);
            d["nextBlockId"] = json!(3);
            n["children"]["block2"] = json!("/cad/d1/block/2");
        }
        if n["attributes"].get("ifccad::layout").is_some() {
            if let Some(c) = n["children"].as_object_mut() {
                for path in c.values_mut() {
                    if *path == json!(def_path) {
                        *path = json!(insert);
                    }
                }
            }
        }
    }
    let common = def["attributes"]["ifccad::entity"].clone();
    source["data"].as_array_mut().unwrap().extend([
        json!({"path":"/cad/d1/block/2","children":{"0":def_path},"attributes":{"ifccad::blockDefinition":{"name":"Door","basePoint":[7,9,0],"insertionUnit":"mm"}}}),
        json!({"path":insert,"children":{"0":child},"attributes":{"ifccad::entity":common,"ifccad::blockInstance":{"definition":"/cad/d1/block/2","transform":{"placement":{"origin":[100,200,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"rotation":0.3,"scale":[-2,3,1]}}}}),
        json!({"path":child,"attributes":{"ifccad::entity":common,"ifccad::geom::placement":{"origin":[101,202,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"ifccad::blockAttribute":{"tag":"RENAMED","definition":def_path,"text":payload["text"],"secondaryAttributes":[def_path]}}}),
    ]);
    // Generated owner bounds in the source example no longer describe the changed drawing.
    for n in source["data"].as_array_mut().unwrap() {
        for role in ["ifccad::layout", "ifccad::blockDefinition"] {
            if let Some(v) = n["attributes"].get_mut(role).and_then(Value::as_object_mut) {
                v.remove("bounds");
                v.remove("boundsQuality");
            }
        }
    }
    source
}

#[test]
fn renamed_child_keeps_explicit_definition_and_owner_coordinates() {
    let source = with_insert();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let insert = loaded
        .document()
        .model
        .entities
        .iter()
        .filter_map(|e| e.as_native())
        .find_map(|e| {
            if let ifccad::IfccadEntityKind::BlockInstance { attributes, .. } = &e.kind {
                Some(attributes)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(insert.len(), 1);
    assert_eq!(insert[0].id, 9007199254740993);
    assert_eq!(insert[0].value.tag, "RENAMED");
    assert_eq!(insert[0].value.text.placement.origin, [101., 202., 0.]);
    let output = encode_ifccad_document(loaded.document()).unwrap();
    let returned = load_ifccad_bytes(output.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), returned.document());
    let binary = encode_ifccad_cbor_document(loaded.document()).unwrap();
    assert_eq!(
        loaded.document(),
        load_ifccad_cbor_bytes(binary.bytes(), Default::default())
            .unwrap()
            .document()
    );
}

#[test]
fn wrong_definition_secondary_target_and_shared_ownership_are_rejected() {
    for field in ["definition", "secondaryAttributes"] {
        let mut v = with_insert();
        let child = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["path"] == "/cad/d1/e9007199254740993")
            .unwrap();
        child["attributes"]["ifccad::blockAttribute"][field] = if field == "definition" {
            json!("/cad/d1/e9007199254740992")
        } else {
            json!(["/cad/d1/e9007199254740992"])
        };
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
    let mut v = with_insert();
    let block = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/block/2")
        .unwrap();
    block["children"]["1"] = json!("/cad/d1/e9007199254740993");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
}

#[test]
fn attribute_field_cache_and_target_reopen_without_evaluation() {
    let mut v = with_insert();
    let child = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e9007199254740993")
        .unwrap();
    child["attributes"]["ifccad::blockAttribute"]["fields"] = json!({"root":0,"nextId":2,"records":[
        {"id":0,"evaluator":"TextTemplate","code":[{"kind":"Literal","value":"Door "},{"kind":"Child","value":1}],"format":"","cache":{"value":{"kind":"Text","value":"old"},"display":"stale"},"evaluationOptions":0,"filingOptions":0,"state":0,"evaluationStatus":0,"errorCode":0,"errorMessage":""},
        {"id":1,"evaluator":"ObjectProperty","code":[{"kind":"Literal","value":"\\AcObjProp Object("},{"kind":"Object","value":"/cad/d1/e9007199254740992"},{"kind":"Literal","value":").Length"}],"format":"","cache":{"value":{"kind":"Real","value":12.5},"display":"12.50"},"evaluationOptions":0,"filingOptions":0,"state":0,"evaluationStatus":0,"errorCode":0,"errorMessage":""}
    ]});
    let d = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    let bytes = encode_ifccad_document(d.document()).unwrap();
    let reopened = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(d.document(), reopened.document());
    let binary = encode_ifccad_cbor_document(d.document()).unwrap();
    assert_eq!(
        d.document(),
        load_ifccad_cbor_bytes(binary.bytes(), Default::default())
            .unwrap()
            .document()
    );
}

#[test]
fn empty_mtext_and_independent_embedded_scale_survive_native_storage() {
    use ifccad::*;
    let mut v = fixture();
    let m = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::mText").is_some())
        .unwrap()
        .clone();
    let def = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::attributeDefinition").is_some())
        .unwrap();
    let mut payload = m["attributes"]["ifccad::mText"].clone();
    payload["content"] = json!([{"inlines":[]}]);
    def["attributes"]["ifccad::attributeDefinition"]["mText"] = json!({"layer":def["attributes"]["ifccad::entity"]["layer"],"text":payload,"placement":m["attributes"]["ifccad::geom::placement"]});
    for n in v["data"].as_array_mut().unwrap() {
        for role in ["ifccad::layout", "ifccad::blockDefinition"] {
            if let Some(v) = n["attributes"].get_mut(role).and_then(Value::as_object_mut) {
                v.remove("bounds");
                v.remove("boundsQuality");
            }
        }
    }
    let mut d = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default())
        .unwrap()
        .into_document();
    d.annotation_scales = vec![
        IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(1),
            name: "1:50".into(),
            paper_units: 1.,
            drawing_units: 50.,
            unit_scale: false,
        },
        IfccadAnnotationScale {
            id: IfccadAnnotationScaleId(2),
            name: "1:100".into(),
            paper_units: 1.,
            drawing_units: 100.,
            unit_scale: false,
        },
    ];
    d.id_counters.next_annotation_scale_id = 3;
    let a = d
        .model
        .entities
        .iter_mut()
        .filter_map(|e| e.as_native_mut())
        .find_map(|e| {
            if let IfccadEntityKind::AttributeDefinition(a) = &mut e.kind {
                Some(a)
            } else {
                None
            }
        })
        .unwrap();
    a.annotation = Some(Box::new(IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: Some(IfccadAnnotationContextKey {
            scale_id: IfccadAnnotationScaleId(1),
        }),
        contexts: vec![IfccadAnnotationContext {
            key: IfccadAnnotationContextKey {
                scale_id: IfccadAnnotationScaleId(1),
            },
            presentation: Some(IfccadAttributeAnnotationContext::MText {
                text: IfccadTextAnnotationContext {
                    insertion: [1., 2.],
                    alignment: [3., 4.],
                    rotation: 0.1,
                    horizontal: IfccadTextAnnotationHorizontal::Left,
                },
                m_text_enabled: false,
                embedded: Some(IfccadAttributeEmbeddedContext {
                    scale_id: IfccadAnnotationScaleId(2),
                    is_default: true,
                    presentation: IfccadMTextAnnotationContext {
                        insertion: [5., 6., 0.],
                        x_direction: [1., 0., 0.],
                        attachment: ifccad::text::MTextAttachment::TopLeft,
                        rectangle_width: 25.,
                        rectangle_height: 7.,
                        columns: None,
                    },
                }),
            }),
        }],
    }));
    let bytes = encode_ifccad_document(&d).unwrap();
    let returned = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(returned.document(), &d);
    let bytes = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
}
