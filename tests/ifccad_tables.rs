use ifccad::{
    encode_ifccad_cbor_document, encode_ifccad_document, load_ifccad_bytes, load_ifccad_cbor_bytes,
};
use serde_json::{json, Value};

fn sample_table(merged_ranges: Value) -> Value {
    let mut file: Value =
        serde_json::from_str(include_str!("../conformance/next/ifccad/valid-circle.ifcx")).unwrap();
    let attributes = file["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap()["attributes"]
        .as_object_mut()
        .unwrap();
    attributes.retain(|name, _| name == "ifccad::entity" || name == "ifccad::geom::placement");
    attributes.insert(
        "ifccad::geom::placement".into(),
        json!({"origin":[0.0,0.0,0.0],"xAxis":[1.0,0.0,0.0],"yAxis":[0.0,1.0,0.0]}),
    );

    let empty_cell = json!({"contents":[]});
    let mut rows = vec![json!({"height":5.0,"cells":vec![empty_cell;3]}); 7];
    rows[0]["cells"][0]["contents"] = json!([{
        "kind":"value","value":{"kind":"text","value":"Table sample"}
    }]);
    rows[4]["cells"][1]["contents"] = json!([
        {"kind":"value","value":{"kind":"text","value":""}},
        {"kind":"value","value":{"kind":"real","value":0.0},"display":"0"}
    ]);
    attributes.insert(
        "ifccad::table".into(),
        json!({
            "columns":[{"width":10.0},{"width":20.0},{"width":30.0}],
            "rows":rows,
            "mergedRanges":merged_ranges
        }),
    );
    file
}

#[test]
fn native_roundtrip_preserves_merges_and_hidden_cell_contents() {
    let source = sample_table(json!([
        {"topRow":0,"leftColumn":0,"bottomRow":0,"rightColumn":2},
        {"topRow":4,"leftColumn":0,"bottomRow":5,"rightColumn":2}
    ]));
    let expected_table = source["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let result_table = readback["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    assert_eq!(result_table["mergedRanges"], expected_table["mergedRanges"]);
    assert_eq!(
        result_table["rows"][4]["cells"][1]["contents"],
        expected_table["rows"][4]["cells"][1]["contents"]
    );
}

#[test]
fn overlapping_merges_are_rejected_at_the_table() {
    let source = sample_table(json!([
        {"topRow":0,"leftColumn":0,"bottomRow":1,"rightColumn":1},
        {"topRow":1,"leftColumn":1,"bottomRow":2,"rightColumn":2}
    ]));
    let error =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap_err();
    assert!(error.to_string().contains("overlap"), "{error}");
}

#[test]
fn explicit_cell_and_facing_edge_overrides_survive_native_storage() {
    let mut source = sample_table(json!([]));
    let grid = &mut source["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap()["attributes"]["ifccad::table"];
    grid["format"] = json!({"textHeight":4.0});
    grid["rows"][1]["format"] = json!({"textHeight":5.0});
    grid["columns"][0]["format"] = json!({"textHeight":6.0});
    grid["rows"][1]["cells"][0]["format"] =
        json!({"textHeight":4.0,"rightBorder":{"visible":false}});
    grid["rows"][1]["cells"][1]["format"] =
        json!({"leftBorder":{"visible":true,"doubleSpacing":0.75}});
    let original = grid.clone();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let grid = readback["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    assert_eq!(grid["format"], original["format"]);
    assert_eq!(grid["columns"], original["columns"]);
    assert_eq!(grid["rows"], original["rows"]);
}

#[test]
fn typed_cell_values_and_item_format_survive_native_storage() {
    let mut source = sample_table(json!([]));
    let grid = &mut source["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap()["attributes"]["ifccad::table"];
    let contents = json!([
        {"kind":"value","value":{"kind":"unset","value":"real"}},
        {"kind":"value","value":{"kind":"date","value":[232,7,11,12,0,0,0,0]}},
        {"kind":"value","value":{"kind":"point3d","value":[0.0,1.0,2.0]}},
        {"kind":"value","value":{"kind":"binary","value":[0,128,255]}},
        {"kind":"value","value":{"kind":"real","value":12.5},
         "unit":"distance","valueFormat":"%lu2%pr2","display":"12.50",
         "format":{"rotation":0.5,"textHeight":2.0}}
    ]);
    grid["rows"][1]["cells"][0]["contents"] = contents.clone();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let grid = readback["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    assert_eq!(grid["rows"][1]["cells"][0]["contents"], contents);
}

#[test]
fn shared_table_style_and_explicit_cell_assignment_survive_native_storage() {
    let mut source = sample_table(json!([]));
    let nodes = source["data"].as_array_mut().unwrap();
    let drawing = nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1")
        .unwrap();
    let style = json!({
        "name":"Schedule",
        "flowDirection":"topToBottom",
        "cellStyles":[{"id":4,"name":"Note","format":{"textHeight":2.5}}]
    });
    drawing["children"]["tableStyle3"] = json!("/cad/d1/tableStyle/3");
    drawing["attributes"]["ifccad::drawing"]["nextTableStyleId"] = json!(4);
    nodes.push(json!({
        "path":"/cad/d1/tableStyle/3","attributes":{"ifccad::tableStyle":style}
    }));
    let table = &mut nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap()["attributes"]["ifccad::table"];
    table["styleId"] = json!(3);
    table["rows"][1]["cells"][0]["styleId"] = json!(4);
    table["rows"][1]["cells"][0]["format"] = json!({"textHeight":2.5});
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let nodes = readback["data"].as_array().unwrap();
    let stored_style = nodes
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::tableStyle"))
        .unwrap();
    let stored_grid = nodes
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    assert_eq!(stored_style, &style);
    assert_eq!(stored_grid["styleId"], json!(3));
    assert_eq!(stored_grid["rows"][1]["cells"][0]["styleId"], json!(4));
    assert_eq!(
        stored_grid["rows"][1]["cells"][0]["format"],
        json!({"textHeight":2.5})
    );
}

#[test]
fn native_table_roundtrip_keeps_ordered_block_items_and_checks_their_targets() {
    let mut source = sample_table(json!([]));
    let nodes = source["data"].as_array_mut().unwrap();
    let drawing = nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1")
        .unwrap();
    drawing["children"]["block1"] = json!("/cad/d1/block/1");
    drawing["attributes"]["ifccad::drawing"]["nextBlockId"] = json!(2);
    nodes.push(json!({
        "path":"/cad/d1/block/1",
        "children":{},
        "attributes":{"ifccad::blockDefinition":{
            "name":"Symbol","basePoint":[0.0,0.0,0.0],"insertionUnit":"cm"
        }}
    }));
    let contents = json!([
        {"kind":"block","block":{"definitionId":1},"format":{"blockScale":2.0}},
        {"kind":"block","block":{"definitionId":1}},
        {"kind":"value","value":{"kind":"text","value":"Two symbols"}}
    ]);
    let table_index = nodes
        .iter()
        .position(|node| node["path"] == "/cad/d1/e1000")
        .unwrap();
    nodes[table_index]["attributes"]["ifccad::table"]["rows"][1]["cells"][0]["contents"] =
        contents.clone();
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let grid = readback["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| node["attributes"].get("ifccad::table"))
        .unwrap();
    assert_eq!(grid["rows"][1]["cells"][0]["contents"], contents);

    source["data"][table_index]["attributes"]["ifccad::table"]["rows"][1]["cells"][0]["contents"]
        [0]["block"]["definitionId"] = json!(1000);
    let error =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap_err();
    assert!(
        error.to_string().contains("rows/1/cells/0/contents/0"),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("unresolved table cell block definition"),
        "{error}"
    );
}

#[test]
fn table_graphics_are_optional_and_keep_owned_ordered_parts() {
    for count in [0, 2] {
        let mut source = sample_table(json!([]));
        let nodes = source["data"].as_array_mut().unwrap();
        let owner = nodes
            .iter_mut()
            .find(|node| node["path"] == "/cad/d1/e1000")
            .unwrap();
        let graphics = "/cad/d1/e1000/graphics";
        owner["children"] = json!({"graphics":graphics});
        let parts: serde_json::Map<_, _> = (0..count)
            .map(|index| (index.to_string(), json!(format!("{graphics}/p{index}"))))
            .collect();
        nodes.push(json!({
            "path": graphics,
            "children": parts,
            "attributes":{"ifccad::tableGraphics":{"origin":"sourceSnapshot"}}
        }));
        for index in 0..count {
            let offset = f64::from(index);
            nodes.push(json!({
                "path":format!("{graphics}/p{index}"),
                "attributes":{
                    "ifccad::tableGraphicsPart":{},
                    "ifccad::geom::lineSegment":{"start":[0.0,offset],"end":[1.0,offset]}
                }
            }));
        }
        let loaded =
            load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
        let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
        let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
        let ifcx = encode_ifccad_document(cbor.document()).unwrap();
        let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
        let nodes = readback["data"].as_array().unwrap();
        let table = nodes
            .iter()
            .find(|node| node["path"] == "/cad/d1/e1000")
            .unwrap();
        let group = nodes.iter().find(|node| node["path"] == graphics).unwrap();
        assert_eq!(table["children"], json!({"graphics":graphics}));
        assert_eq!(group["children"], json!(parts));
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node["attributes"]
                    .get("ifccad::tableGraphicsPart")
                    .is_some())
                .count(),
            count as usize
        );
    }
}

#[test]
fn stored_formula_preserves_forward_table_reference_and_cached_value() {
    let mut source = sample_table(json!([]));
    let nodes = source["data"].as_array_mut().unwrap();
    let table_index = nodes
        .iter()
        .position(|node| node["path"] == "/cad/d1/e1000")
        .unwrap();
    let mut other = nodes[table_index].clone();
    other["path"] = json!("/cad/d1/e1001");
    *nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1001")
        .unwrap() = other;
    let formula = json!({
        "kind":"field",
        "field":{
            "root":1,"nextId":2,"records":[{
                "id":1,"evaluator":"AcExpr","children":[],"objectTargets":[],
                "code":[{"kind":"literal","value":"Sum("},{
                    "kind":"cells","value":{"tableId":1001,"range":{
                        "topRow":0,"leftColumn":0,"bottomRow":0,"rightColumn":1
                    }}
                },{"kind":"literal","value":")"}],
                "format":"%pr2","evaluationOptions":31,"filingOptions":0,"state":0,
                "evaluationStatus":0,"errorCode":0,"errorMessage":"",
                "cache":{"value":{"kind":"real","value":12.5},"display":"12.50",
                         "valueFormat":"%pr2","formattedValue":"12.50","flags":0},
                "auxiliaryValues":[]
            }]
        }
    });
    let slot =
        &mut nodes[table_index]["attributes"]["ifccad::table"]["rows"][1]["cells"][0]["contents"];
    *slot = json!([formula]);
    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let table = readback["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap();
    assert_eq!(
        table["attributes"]["ifccad::table"]["rows"][1]["cells"][0]["contents"][0],
        formula
    );
}

#[test]
fn linked_range_and_standalone_data_table_keep_separate_resources() {
    let mut source = sample_table(json!([]));
    let nodes = source["data"].as_array_mut().unwrap();
    let drawing = nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1")
        .unwrap();
    drawing["children"]["tableDataLink3"] = json!("/cad/d1/tableDataLink/3");
    drawing["children"]["dataTable3"] = json!("/cad/d1/dataTable/3");
    drawing["attributes"]["ifccad::drawing"]["nextTableDataLinkId"] = json!(4);
    drawing["attributes"]["ifccad::drawing"]["nextDataTableId"] = json!(4);
    let link = json!({
        "name":"Stock","adapter":"AcExcel","source":"stock.xlsx","sourceSelection":"Items!A1:B2",
        "options":1,"updateOptions":2,"pathOption":1,"statusFlags":0,"flags":0,
        "customData":[],"customTargets":[]
    });
    let data = json!({
        "name":"Stock records","flags":0,"rowCount":1,
        "columns":[
            {"name":"Stock","valueType":"integer","values":[{"kind":"integer","value":2}]},
            {"name":"Marker","valueType":"object","values":[{"kind":"object","value":1001}]}
        ]
    });
    nodes.push(
        json!({"path":"/cad/d1/tableDataLink/3","attributes":{"ifccad::tableDataLink":link}}),
    );
    nodes.push(json!({"path":"/cad/d1/dataTable/3","attributes":{"ifccad::dataTable":data}}));
    let ranges =
        json!([{"linkId":3,"range":{"topRow":1,"leftColumn":0,"bottomRow":2,"rightColumn":1}}]);
    let table = nodes
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1/e1000")
        .unwrap();
    table["attributes"]["ifccad::table"]["dataLinks"] = ranges.clone();

    let loaded =
        load_ifccad_bytes(&serde_json::to_vec(&source).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_cbor_document(loaded.document()).unwrap();
    let cbor = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    let ifcx = encode_ifccad_document(cbor.document()).unwrap();
    let readback: Value = serde_json::from_slice(ifcx.bytes()).unwrap();
    let nodes = readback["data"].as_array().unwrap();
    let value = |role: &str| {
        nodes
            .iter()
            .find_map(|node| node["attributes"].get(role))
            .unwrap()
    };
    assert_eq!(value("ifccad::tableDataLink"), &link);
    assert_eq!(value("ifccad::dataTable"), &data);
    assert_eq!(value("ifccad::table")["dataLinks"], ranges);
}
