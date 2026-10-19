use ifccad_convert::opencadcodec::{
    objects::{KnownXRecordKind as K, XRecordValue},
    CadDocument, Color, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Line, MText,
    Vector3,
};
use std::io::Cursor;

pub fn source() -> CadDocument {
    let mut d = CadDocument::new();
    d.header.insertion_units = 4;
    d.layers.get_mut("0").unwrap().color = Color::Index(7);
    for (i, color) in [
        Color::Index(7),
        Color::from_rgb(255, 255, 255),
        Color::from_rgb(0, 0, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let mut line = Line::new();
        line.start = Vector3::new(0., i as f64, 0.);
        line.end = Vector3::new(1., i as f64, 0.);
        line.common.color = color;
        d.add_entity(EntityType::Line(line)).unwrap();
    }
    let mut text = MText::new();
    text.value = "Fixed\\C7;Foreground".into();
    text.height = 2.;
    text.common.color = Color::Index(7);
    text.background_fill_flags = 1;
    text.background_color = Color::Index(7);
    d.add_entity(EntityType::MText(Box::new(text))).unwrap();
    d.add_layout("Sheet").unwrap();
    let layer = d.layers.get("0").unwrap().handle;
    let mut view = opencadcodec::entities::Viewport::new();
    view.id = 2;
    view.width = 100.;
    view.height = 80.;
    view.view_height = 120.;
    let view = d
        .add_entity_to_layout(EntityType::Viewport(Box::new(view)), "Sheet")
        .unwrap();
    assert!(d.set_layer_viewport_override(
        layer,
        K::LayerViewportColorOverride,
        view,
        XRecordValue::Int32(0xc3000007u32 as i32)
    ));
    d
}

pub fn readback(d: &CadDocument, dwg: bool) -> CadDocument {
    let bytes = if dwg {
        DwgWriter::write_to_vec(d).unwrap()
    } else {
        DxfWriter::new(d).write_to_vec().unwrap()
    };
    if dwg {
        DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap()
    }
}

pub fn assert_cad_colors(d: &CadDocument) {
    assert_eq!(d.layers.get("0").unwrap().color, Color::Index(7));
    let colors: Vec<_> = d
        .model_space_entities()
        .filter(|e| matches!(e, EntityType::Line(_)))
        .map(|e| e.common().color)
        .collect();
    assert_eq!(
        colors,
        [
            Color::Index(7),
            Color::from_rgb(255, 255, 255),
            Color::from_rgb(0, 0, 0)
        ]
    );
    let text = d
        .model_space_entities()
        .find_map(|e| {
            if let EntityType::MText(t) = e {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(text.background_color, Color::Index(7));
    assert!(text.value.contains("\\C7;"), "{}", text.value);
    let layer = d.layers.get("0").unwrap().handle;
    assert_eq!(
        d.layer_viewport_overrides(layer, K::LayerViewportColorOverride)[0].1,
        XRecordValue::Int32(0xc3000007u32 as i32)
    );
}
