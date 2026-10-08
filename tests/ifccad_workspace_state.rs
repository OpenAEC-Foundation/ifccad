use ocdraw::ifccad::*;
use serde_json::{json, Value};
const BIG: u64 = 9_007_199_254_740_993;

fn source() -> Value {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-viewports.ifcx")).unwrap();
    let nodes = value["data"].as_array_mut().unwrap();
    let viewport = nodes
        .iter()
        .find(|n| n["attributes"].get("ifccad::viewport").is_some())
        .unwrap();
    let viewport_path = viewport["path"].as_str().unwrap().to_owned();
    let mut view = viewport["attributes"]["ifccad::viewport"]["view"].clone();
    view["projection"] = json!("Orthographic");
    let aids = json!({"grid":{"enabled":false,"spacing":[0,3],"style":"Dots","majorLineFrequency":5,"beyondLimits":true,"adaptive":false,"subdivision":true,"followsWorkplane":false},"snap":{"enabled":false,"base":[4,5],"spacing":[0,0],"angle":0.0,"style":"Isometric","isometricPlane":"Left"},"storedUcs":{"kind":"Named","ucs":format!("/cad/d1/ucs/{BIG}")},"useStoredUcs":false});
    let mut window = aids.clone();
    window["rectangle"] = json!([0, 0, 1, 1]);
    window["view"] = view.clone();
    window["aspectRatio"] = json!(1.5);
    window["renderMode"] = json!("TwoDimensional");
    let drawing = nodes.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap();
    drawing["children"]["ucs"] = json!(format!("/cad/d1/ucs/{BIG}"));
    drawing["children"]["window"] = json!(format!("/cad/d1/modelWindow/{BIG}"));
    drawing["attributes"]["ifccad::drawing"]["nextUcsId"] = json!(BIG + 1);
    drawing["attributes"]["ifccad::drawing"]["nextModelWindowId"] = json!(BIG + 1);
    drawing["attributes"]["ifccad::drawing"]["modelWindows"] =
        json!([format!("/cad/d1/modelWindow/{BIG}")]);
    drawing["attributes"]["ifccad::modelViewState"] = json!({"currentModelUcs":{"kind":"World"},"activeModelWindow":format!("/cad/d1/modelWindow/{BIG}")});
    nodes.push(json!({"path":format!("/cad/d1/ucs/{BIG}"),"attributes":{"ifccad::ucsDefinition":{"name":"Survey","frame":{"origin":[3,4,5],"xAxis":[1,0,0],"yAxis":[0,1,0]},"elevation":6}}}));
    nodes.push(json!({"path":format!("/cad/d1/modelWindow/{BIG}"),"attributes":{"ifccad::modelWindow":window}}));
    let paper = nodes
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::layout"]["kind"] == "Paper")
        .unwrap();
    let mut canvas = aids.clone();
    canvas["view"] = view;
    canvas["frame"] = json!({"center":[100,-50,3],"width":10,"height":8});
    paper["attributes"]["ifccad::paperCanvas"] = canvas;
    let viewport = nodes
        .iter_mut()
        .find(|n| n["path"] == viewport_path)
        .unwrap();
    viewport["attributes"]["ifccad::viewportWorkspace"] = aids;
    value
}
fn read(value: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}
fn roundtrip(value: &Value) -> Value {
    let loaded = read(value).unwrap();
    validate_ifccad_document(loaded.document()).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    serde_json::from_slice(encoded.bytes()).unwrap()
}
fn node<'a>(v: &'a Value, path: &str) -> &'a Value {
    v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == path)
        .unwrap()
}

#[test]
fn workspace_roundtrips_with_full_width_ids_and_composed_fragments() {
    let mut v = source();
    let path = format!("/cad/d1/ucs/{BIG}");
    let nodes = v["data"].as_array_mut().unwrap();
    let ucs = nodes.iter_mut().find(|n| n["path"] == path).unwrap();
    let mut definition = ucs["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::ucsDefinition")
        .unwrap();
    definition["name"] = json!("Renamed survey");
    nodes.push(json!({"path":path,"attributes":{"ifccad::ucsDefinition":definition}}));
    nodes.reverse();
    let back = roundtrip(&v);
    let drawing = node(&back, "/cad/d1");
    assert_eq!(
        drawing["attributes"]["ifccad::drawing"]["nextUcsId"].as_u64(),
        Some(BIG + 1)
    );
    assert_eq!(
        node(&back, &path)["attributes"]["ifccad::ucsDefinition"]["name"],
        "Renamed survey"
    );
    assert_eq!(
        drawing["attributes"]["ifccad::drawing"]["modelWindows"][0],
        format!("/cad/d1/modelWindow/{BIG}")
    );
}
#[test]
fn unknown_context_and_disabled_zero_snap_survive() {
    let v = source();
    let back = roundtrip(&v);
    let paper = back["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"]["ifccad::layout"]["kind"] == "Paper")
        .unwrap();
    let canvas = &paper["attributes"]["ifccad::paperCanvas"];
    assert!(canvas.get("activeContext").is_none());
    assert!(canvas.get("currentUcs").is_none());
    assert_eq!(canvas["useStoredUcs"], false);
    assert_eq!(canvas["snap"]["spacing"][0].as_f64(), Some(0.));
    assert_eq!(canvas["frame"]["center"][2].as_f64(), Some(3.));
}
#[test]
fn workspace_wrong_owner_and_invalid_variants_are_rejected() {
    for which in 0..5 {
        let mut v = source();
        let nodes = v["data"].as_array_mut().unwrap();
        let drawing = nodes.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap();
        match which {
            0 => {
                drawing["attributes"]["ifccad::modelViewState"]["activeModelWindow"] =
                    json!("/cad/d2/modelWindow/1")
            }
            1 => {
                drawing["attributes"]["ifccad::drawing"]["modelWindows"] = json!([
                    format!("/cad/d1/modelWindow/{BIG}"),
                    format!("/cad/d1/modelWindow/{BIG}")
                ])
            }
            2 => drawing["attributes"]["ifccad::drawing"]["nextUcsId"] = json!(BIG),
            3 => {
                drawing["attributes"]["ifccad::modelViewState"]["currentModelUcs"] =
                    json!({"kind":"World","ucs":"/cad/d1/ucs/1"})
            }
            _ => drawing["attributes"]["ifccad::modelViewState"]["activeModelWindow"] = Value::Null,
        }
        assert!(read(&v).is_err(), "mutation {which}");
    }
}

#[test]
fn unused_ucs_and_advanced_watermarks_survive() {
    let mut value = source();
    let nodes = value["data"].as_array_mut().unwrap();
    let drawing = nodes.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap();
    drawing["children"]["unused"] = json!("/cad/d1/ucs/7");
    nodes.push(json!({"path":"/cad/d1/ucs/7","attributes":{"ifccad::ucsDefinition":{"name":"Unused","frame":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"elevation":0}}}));
    let mut document = read(&value).unwrap().into_document();
    document.ucs_definitions.retain(|v| v.id != IfccadUcsId(7));
    let encoded = encode_ifccad_document(&document).unwrap();
    let mut document = load_ifccad_bytes(encoded.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(
        document.id_counters.allocate_ucs_id().unwrap(),
        IfccadUcsId(BIG + 1)
    );
    assert_eq!(
        document.id_counters.allocate_model_window_id().unwrap(),
        IfccadModelWindowId(BIG + 1)
    );
    let mut exhausted = IfccadIdCounters {
        next_ucs_id: u64::MAX,
        ..Default::default()
    };
    assert_eq!(
        exhausted.allocate_ucs_id().unwrap_err().domain,
        IfccadIdDomain::Ucs
    );
    assert_eq!(exhausted.next_ucs_id, u64::MAX);
}

#[test]
fn model_window_reference_order_is_authoritative() {
    let mut value = source();
    let nodes = value["data"].as_array_mut().unwrap();
    let window = nodes
        .iter()
        .find(|n| n["path"] == format!("/cad/d1/modelWindow/{BIG}"))
        .unwrap()["attributes"]
        .clone();
    nodes.push(json!({"path":"/cad/d1/modelWindow/2","attributes":window}));
    let drawing = nodes.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap();
    drawing["children"]["second"] = json!("/cad/d1/modelWindow/2");
    drawing["attributes"]["ifccad::drawing"]["modelWindows"] = json!([
        format!("/cad/d1/modelWindow/{BIG}"),
        "/cad/d1/modelWindow/2"
    ]);
    nodes.reverse();
    let loaded = read(&value).unwrap();
    assert_eq!(
        loaded
            .document()
            .model_windows
            .iter()
            .map(|v| v.id.0)
            .collect::<Vec<_>>(),
        [BIG, 2]
    );
}
