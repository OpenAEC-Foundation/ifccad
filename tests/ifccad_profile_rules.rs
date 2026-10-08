use ocdraw::ifccad::*;
use serde_json::{json, Value};

#[path = "support/ifccad_preservation.rs"]
mod preservation;

fn graph() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ifccad/hello-text.ifcx")).unwrap()
}
fn attr<'a>(graph: &'a mut Value, key: &str) -> &'a mut Value {
    graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|n| n["attributes"].get_mut(key))
        .unwrap()
}
fn read(graph: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(graph).unwrap(), Default::default())
}

#[test]
fn layer_extras_are_rejected_instead_of_silently_dropped() {
    for nested in [false, true] {
        let mut g = graph();
        let layer = attr(&mut g, "ifccad::layer");
        if nested {
            layer["appearance"]["unknown"] = json!(1);
        } else {
            layer["unknown"] = json!(1);
        }
        let report = read(&g).unwrap_err();
        assert!(report.to_string().contains("IFCCAD-APPEARANCE-001"));
        assert!(report.to_string().contains("/cad/d1/layer/"));
        assert!(!supplement("layerExtras").is_valid(attr(&mut g, "ifccad::layer")));
    }
}

#[test]
fn text_unknown_and_null_members_remain_invalid() {
    for value in [json!(1), Value::Null] {
        let mut g = graph();
        attr(&mut g, "ifccad::textStyle")["unknown"] = value;
        assert!(read(&g).is_err());
    }
    let mut g = graph();
    attr(&mut g, "ifccad::textStyle")["font"] = Value::Null;
    assert!(read(&g).is_err());
}

#[test]
fn plot_unknown_members_remain_invalid() {
    let mut g: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ifccad/valid/layout-plot-inch.ifcx"
    ))
    .unwrap();
    let layout = g["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|n| {
            n["attributes"]
                .get_mut("ifccad::layout")
                .filter(|v| v.get("plotSettings").is_some())
        })
        .unwrap();
    layout["plotSettings"]["unknown"] = json!(true);
    assert!(read(&g).is_err());
}

#[test]
fn foreign_objects_and_unknown_provider_bytes_remain_transportable() {
    let mut g = graph();
    g["data"].as_array_mut().unwrap().push(json!({
        "path":"foreign", "attributes":{"foreign::note":{"arbitrary":null}},
        "children":{"cad":"/cad/d1/e1"}
    }));
    let loaded = read(&g).unwrap();
    assert!(loaded.graph().composed_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["path"] == "foreign"));
    let d = preservation::with_opaque();
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
    assert_eq!(
        loaded.document().preservation.as_ref().unwrap().records[0]
            .payload
            .bytes,
        [0xff, 0, 3]
    );
}

#[test]
fn replaced_invalid_fragment_is_not_validated_before_composition() {
    let mut g = graph();
    let node = g["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::textStyle").is_some())
        .unwrap();
    let replacement = node.clone();
    node["attributes"]["ifccad::textStyle"]["font"] = json!(42);
    g["data"].as_array_mut().unwrap().push(replacement);
    let bytes = serde_json::to_vec(&g).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts,
        }
    )
    .is_err());
}

fn supplement(entry: &str) -> jsonschema::Validator {
    let mut s: Value = serde_json::from_str(include_str!(
        "../schemas/ifccad/supplemental-values-0.1.0.schema.json"
    ))
    .unwrap();
    assert!(
        s["$defs"].get(entry).is_some(),
        "missing formal entry {entry}"
    );
    s["$ref"] = json!(format!("#/$defs/{entry}"));
    jsonschema::draft202012::new(&s).unwrap()
}

#[test]
fn formal_non_text_ranges_and_variants_have_real_boundaries() {
    let layer = supplement("layerExtras");
    for opacity in [0., 1.] {
        assert!(layer.is_valid(&json!({"appearance":{"opacity":opacity}})));
    }
    for opacity in [-0.1, 1.1] {
        assert!(!layer.is_valid(&json!({"appearance":{"opacity":opacity}})));
    }
    assert!(!layer.is_valid(&json!({"appearance":{"color":"#xyz123"}})));
    assert!(!layer.is_valid(&json!({"appearance":{"lineWeight":-1}})));
    let entity = supplement("entityExtras");
    let mut value = json!({"appearance":{
        "color":{"mode":"ByLayer"},"opacity":{"mode":"Explicit","value":0},
        "lineWeight":{"mode":"ByBlock"},"linePattern":{"mode":"ByLayer"}}});
    assert!(entity.is_valid(&value));
    value["appearance"]["opacity"]
        .as_object_mut()
        .unwrap()
        .remove("value");
    assert!(!entity.is_valid(&value));
    value["appearance"]["opacity"] = json!({"mode":"ByLayer","value":0});
    assert!(!entity.is_valid(&value));
    let drawing = supplement("drawingExtras");
    assert!(drawing.is_valid(&json!({"nextEntityId":9_007_199_254_740_993u64})));
    assert!(drawing.is_valid(&json!({"nextEntityId":u64::MAX})));
    assert!(!drawing.is_valid(&json!({"nextEntityId":0})));
    assert!(!drawing.is_valid(&json!({"nextEntityId":-1})));
    let circle = supplement("geometryCircleExtras");
    assert!(circle.is_valid(&json!({"radius":1})));
    assert!(!circle.is_valid(&json!({"radius":0})));
    assert!(!circle.is_valid(&json!({"radius":1,"unknown":0})));
}

#[test]
fn text_root_fields_and_types_match_ifcx_declarations() {
    fn resolved<'a>(value: &'a Value, defs: &'a Value) -> &'a Value {
        if let Some(r) = value["$ref"].as_str() {
            &defs[r.strip_prefix("#/$defs/").unwrap()]
        } else {
            value
        }
    }
    fn compare(ifcx: &Value, schema: &Value, defs: &Value) {
        let schema = resolved(schema, defs);
        if schema.get("oneOf").is_some() {
            assert_eq!(ifcx["dataType"], "Object"); // Explicit IFCX variant limitation.
            return;
        }
        let expected = if schema.get("enum").is_some() {
            "Enum"
        } else {
            match schema["type"].as_str().unwrap() {
                "object" => "Object",
                "array" => "Array",
                "number" => "Real",
                "integer" => "Integer",
                "boolean" => "Boolean",
                "string" => {
                    if schema.get("enum").is_some() {
                        "Enum"
                    } else {
                        "String"
                    }
                }
                other => panic!("unclassified local type {other}"),
            }
        };
        assert_eq!(ifcx["dataType"], expected);
        if expected == "Enum" {
            assert_eq!(ifcx["enumRestrictions"]["options"], schema["enum"]);
        }
        if expected == "Object" {
            let fields = ifcx["objectRestrictions"]["values"].as_object().unwrap();
            let local = schema["properties"].as_object().unwrap();
            assert_eq!(
                fields.keys().collect::<Vec<_>>(),
                local.keys().collect::<Vec<_>>()
            );
            for (name, field) in fields {
                compare(field, &local[name], defs);
                let required = schema["required"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|n| n == name));
                assert_eq!(
                    field["optional"].as_bool().unwrap_or(false),
                    !required,
                    "{name}"
                );
            }
        }
        if expected == "Array" {
            compare(&ifcx["arrayRestrictions"]["value"], &schema["items"], defs);
        }
    }
    let profile: Value =
        serde_json::from_str(include_str!("../schemas/ifccad/ifccad-profile-0.1.0.ifcx")).unwrap();
    let supplemental: Value = serde_json::from_str(include_str!(
        "../schemas/ifccad/supplemental-values-0.1.0.schema.json"
    ))
    .unwrap();
    for name in ["textStyle", "text", "mText"] {
        compare(
            &profile["schemas"][format!("ifccad::{name}")]["value"],
            &supplemental["$defs"][name],
            &supplemental["$defs"],
        );
    }
}

#[test]
fn workspace_view_plot_and_preservation_supplements_preserve_boundaries() {
    let mut g: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/workspace-state.ifcx")).unwrap();
    let window = attr(&mut g, "ifccad::modelWindow").clone();
    let check = supplement("modelWindowExtras");
    assert!(check.is_valid(&window));
    let mut bad = window.clone();
    bad["snap"]["enabled"] = json!(true);
    bad["snap"]["spacing"] = json!([0, 0]);
    assert!(!check.is_valid(&bad));
    bad["snap"]["enabled"] = json!(false);
    assert!(check.is_valid(&bad));
    bad["storedUcs"] = json!({"kind":"World","ucs":"/cad/d1/ucs/1"});
    assert!(!check.is_valid(&bad));
    let mut viewport: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-viewports.ifcx")).unwrap();
    let view = attr(&mut viewport, "ifccad::viewport");
    let check = supplement("viewportExtras");
    assert!(check.is_valid(view));
    view["view"]["projection"] = json!("Perspective");
    view["view"].as_object_mut().unwrap().remove("lensLengthMm");
    assert!(!check.is_valid(view));
    view["view"]["lensLengthMm"] = json!(50);
    assert!(check.is_valid(view));
    view["view"]["backClip"]["mode"] = json!("AtCamera");
    assert!(!check.is_valid(view));
    let d = preservation::with_opaque();
    let mut p: Value = serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let record = attr(&mut p, "ifccad::preservationRecord");
    let check = supplement("preservationRecordExtras");
    assert!(check.is_valid(record));
    record["payload"]["schema"] = json!("");
    assert!(!check.is_valid(record));
}

#[test]
fn optional_nulls_are_rejected_and_exact_watermarks_keep_their_native_meaning() {
    let mut g = graph();
    attr(&mut g, "ifccad::drawing")["nextEntityId"] = json!(u64::MAX);
    let loaded = read(&g).unwrap();
    assert_eq!(loaded.document().id_counters.next_entity_id, u64::MAX);
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        loaded.document()
    );
    for field in ["media", "limits", "plotSettings"] {
        let mut null = g.clone();
        attr(&mut null, "ifccad::layout")[field] = Value::Null;
        assert!(read(&null).is_err());
        assert!(!supplement("layoutExtras").is_valid(attr(&mut null, "ifccad::layout")));
    }
    let mut null = g.clone();
    attr(&mut null, "ifccad::drawing")["nextPreservationRecordId"] = Value::Null;
    assert!(read(&null).is_err());
    assert!(!supplement("drawingExtras").is_valid(attr(&mut null, "ifccad::drawing")));
    for bad in [json!(0), json!(-1), json!(1.5)] {
        attr(&mut g, "ifccad::drawing")["nextEntityId"] = bad;
        assert!(read(&g).is_err());
    }
}

#[test]
fn viewport_nulls_are_rejected_even_when_dormant() {
    let g: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-viewports.ifcx")).unwrap();
    for pointer in [
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/view/backClip/distance",
        "/paperClip/boundary",
    ] {
        let mut v = g.clone();
        let viewport = attr(&mut v, "ifccad::viewport");
        viewport["view"]["projection"] = json!("Orthographic");
        viewport["view"]["frontClip"] = json!({"mode":"Disabled","distance":0});
        viewport["view"]["backClip"] = json!({"mode":"Disabled","distance":0});
        viewport["paperClip"] = json!({"enabled":false,"boundary":"/cad/d1/e2"});
        *viewport.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(
            !supplement("viewportExtras").is_valid(viewport),
            "{pointer}"
        );
        assert!(read(&v).is_err());
    }
}

#[test]
fn workspace_view_nulls_are_rejected_like_authored_viewport_nulls() {
    for pointer in [
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/view/backClip/distance",
    ] {
        let mut g: Value =
            serde_json::from_slice(include_bytes!("../examples/ifccad/workspace-state.ifcx"))
                .unwrap();
        let window = attr(&mut g, "ifccad::modelWindow");
        window["view"]["projection"] = json!("Orthographic");
        window["view"]["lensLengthMm"] = json!(50);
        window["view"]["frontClip"] = json!({"mode":"Disabled","distance":0});
        window["view"]["backClip"] = json!({"mode":"Disabled","distance":0});
        *window.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(
            !supplement("modelWindowExtras").is_valid(window),
            "{pointer}"
        );
        assert!(read(&g).is_err(), "{pointer}");
    }
}

#[test]
fn closed_non_text_overlay_fields_agree_with_ordinary_ifcx_shapes() {
    let profile: Value =
        serde_json::from_str(include_str!("../schemas/ifccad/ifccad-profile-0.1.0.ifcx")).unwrap();
    let supplement: Value = serde_json::from_str(include_str!(
        "../schemas/ifccad/supplemental-values-0.1.0.schema.json"
    ))
    .unwrap();
    for (attribute, entry) in [
        ("layout", "layoutExtras"),
        ("geom::placement", "placementExtras"),
        ("geom::point", "geometryPointExtras"),
        ("geom::lineSegment", "geometryLineExtras"),
        ("geom::circle", "geometryCircleExtras"),
        ("geom::arc", "geometryArcExtras"),
        ("geom::ellipse", "geometryEllipseExtras"),
        ("geom::ellipseArc", "geometryEllipseArcExtras"),
        ("geom::planarPolyline", "geometryPlanarExtras"),
        ("geom::spatialPolyline", "geometrySpatialExtras"),
        ("viewport", "viewportExtras"),
        ("preservation", "preservationExtras"),
        ("preservationRecord", "preservationRecordExtras"),
        ("opaqueEntity", "opaqueEntityExtras"),
        ("drawingWorkspace", "drawingWorkspaceExtras"),
        ("modelViewState", "modelViewStateExtras"),
        ("ucsDefinition", "ucsDefinitionExtras"),
        ("modelWindow", "modelWindowExtras"),
        ("paperCanvas", "paperCanvasExtras"),
        ("viewportWorkspace", "viewportWorkspaceExtras"),
    ] {
        let fields = profile["schemas"][format!("ifccad::{attribute}")]["value"]
            ["objectRestrictions"]["values"]
            .as_object()
            .unwrap();
        let local = supplement["$defs"][entry]["properties"]
            .as_object()
            .unwrap();
        assert_eq!(
            fields.keys().collect::<Vec<_>>(),
            local.keys().collect::<Vec<_>>(),
            "{attribute}"
        );
    }
}

#[test]
fn older_native_wrappers_and_transforms_are_closed() {
    for (key, nested) in [
        ("ifccad::entity", false),
        ("ifccad::blockDefinition", false),
        ("ifccad::blockInstance", false),
        ("ifccad::blockInstance", true),
        ("ifccad::linePattern", false),
    ] {
        let mut g = graph();
        let value = attr(&mut g, key);
        if nested {
            value["transform"]["unknown"] = json!(1);
        } else {
            value["unknown"] = json!(1);
        }
        assert!(read(&g).is_err(), "{key} nested={nested}");
    }
}

#[test]
fn domain_graph_and_parse_errors_have_rule_and_location() {
    let mut d = read(&graph()).unwrap().into_document();
    d.length_unit = "invalid".into();
    let report = validate_ifccad_document(&d).unwrap_err();
    assert!(report
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-") && m.contains("/cad/d1")));
    let mut g = graph();
    attr(&mut g, "ifccad::entity")["layer"] = json!("/cad/d1/layer/999");
    let report = read(&g).unwrap_err();
    assert!(report
        .report()
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-") && m.contains("/cad/d1/e")));
    let report = load_ifccad_bytes(b"{", Default::default()).unwrap_err();
    assert!(report
        .report()
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-WIRE-") && m.contains(" /")));
}

#[test]
fn standalone_scalar_validators_keep_semantic_rule_and_component_location() {
    let mut d = read(&graph()).unwrap().into_document();
    d.line_patterns[0].name.clear();
    let report = validate_ifccad_line_patterns(&d.line_patterns).unwrap_err();
    assert!(report
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-PATTERN-") && m.contains("/linePattern/")));
    let frame = IfccadPlacement {
        origin: [f64::NAN, 0., 0.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    let report = frame.coordinate_frame().unwrap_err();
    assert!(report
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-GEOMETRY-") && m.contains("/ifccad::geom::placement")));
    let viewport_d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap();
    let mut v = viewport_d
        .document()
        .paper_layouts
        .iter()
        .flat_map(|p| &p.entities)
        .find_map(|e| {
            e.as_native().and_then(|e| {
                if let IfccadEntityKind::Viewport(v) = &e.kind {
                    Some(v.clone())
                } else {
                    None
                }
            })
        })
        .unwrap();
    v.view.height = 0.;
    let report = validate_ifccad_viewport_parameters(&v).unwrap_err();
    assert!(report
        .errors
        .iter()
        .all(|m| m.starts_with("IFCCAD-VIEWPORT-") && m.contains("/view/height")));
}

#[test]
fn document_wrappers_preserve_specific_rule_and_owner_field() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut entity_id = 0;
    for e in d.paper_layouts.iter_mut().flat_map(|p| &mut p.entities) {
        if let Some(e) = e.as_native_mut() {
            if let IfccadEntityKind::Viewport(v) = &mut e.kind {
                v.view.height = 0.;
                entity_id = e.id;
                break;
            }
        }
    }
    let report = validate_ifccad_document(&d).unwrap_err();
    assert!(report.errors.iter().any(|m| m.starts_with(&format!(
        "IFCCAD-VIEWPORT-002 /cad/d{}/e{entity_id}/ifccad::viewport/view/height:",
        d.drawing_id
    ))));
    assert!(report
        .errors
        .iter()
        .all(|m| !m.contains("IFCCAD validation failed:")));
}
