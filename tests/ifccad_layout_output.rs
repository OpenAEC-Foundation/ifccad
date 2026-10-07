use ocdraw::{geometry_kernel::CoordinateLengthUnit as Unit, ifccad::*, plot_kernel::*};
use serde_json::Value;
fn sample() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
#[test]
fn model_and_paper_medium_only_have_strict_roundtrip() {
    let mut d = sample();
    d.model.settings.media = Some(LayoutMedia {
        unit: MediaUnit::Physical(Unit::Millimetre),
        width: 420.,
        height: 297.,
    });
    d.paper_layouts[0].settings.plot_settings = None;
    d.paper_layouts[0].settings.paper_space_linetype_scaling = false;
    let bytes = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(read.document(), &d);
    assert!(read.document().paper_layouts[0]
        .settings
        .plot_settings
        .is_none());
    assert_eq!(read.document().id_counters, d.id_counters);
}
#[test]
fn optional_plot_values_are_not_silently_discarded() {
    let d = sample();
    let encoded = encode_ifccad_document(&d).unwrap();
    let baseline: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for field in ["unknown", "noncustomDpi", "nullHint", "nullStyleMode"] {
        let mut v = baseline.clone();
        for node in v["data"].as_array_mut().unwrap() {
            if node["attributes"]["ifccad::layout"]["plotSettings"].is_object() {
                let plot = &mut node["attributes"]["ifccad::layout"]["plotSettings"];
                match field {
                    "unknown" => plot["page"]["foreign"] = serde_json::json!(1),
                    "noncustomDpi" => {
                        plot["output"]["shadedPlot"]["quality"]["dpi"] = serde_json::json!(300)
                    }
                    "nullHint" => plot["page"]["deviceName"] = Value::Null,
                    _ => (),
                }
            }
            if field == "nullStyleMode" && node["attributes"]["ifccad::drawing"].is_object() {
                node["attributes"]["ifccad::drawing"]["plotStyleMode"] = Value::Null;
            }
        }
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "{field}"
        );
    }
}
