mod common;
use common::*;
use ifccad_convert::*;

#[test]
fn newly_exposed_header_settings_are_diagnosed_and_rejectable() {
    let edits: [fn(&mut opencadcodec::document::HeaderVariables); 4] = [
        |h| h.dwf_frame = 1,
        |h| h.dgn_frame = 2,
        |h| h.universal_create_date_julian = 2_460_000.,
        |h| h.universal_update_date_julian = 2_460_001.,
    ];
    for edit in edits {
        let mut doc = cad();
        edit(&mut doc.header);
        let out = cad_document_to_encoded_ifccad(&doc, metadata(), Default::default()).unwrap();
        assert!(out
            .diagnostics()
            .iter()
            .any(|d| d.code == "source-field" && d.location.starts_with("header.")));
        assert!(matches!(
            from_cad(&doc, metadata()),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
}

#[test]
fn off_screen_state_prevents_default_viewport_scaffold_elision() {
    let mut doc = opencadcodec::CadDocument::new();
    let layer = doc.layers.get_mut("0").unwrap();
    layer.color = opencadcodec::Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    layer.line_weight = opencadcodec::LineWeight::Value(25);
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 1;
    let handle = doc
        .add_entity_to_layout(opencadcodec::EntityType::Viewport(v), "Layout1")
        .unwrap();
    let layout = doc
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Layout1" => Some(l),
            _ => None,
        })
        .unwrap();
    layout.viewport = handle;
    layout.viewports = vec![handle];
    from_cad(&doc, metadata()).unwrap();
    let opencadcodec::EntityType::Viewport(v) = doc.get_entity_mut(handle).unwrap() else {
        panic!()
    };
    v.off_screen = true;
    let out = cad_document_to_encoded_ifccad(&doc, metadata(), Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "source-field" && d.location == format!("entity/{handle}.off_screen")));
    assert!(matches!(
        from_cad(&doc, metadata()),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

#[test]
fn model_role_bit_is_scaffold_but_authored_plot_flags_remain_loss() {
    let mut doc = cad();
    let model = doc
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Model" => Some(l),
            _ => None,
        })
        .unwrap();
    model.plot_flags.model_type = true;
    let reject = CadToIfccadOptions {
        loss_policy: IfccadLossPolicy::Reject,
        ..Default::default()
    };
    cad_document_to_encoded_ifccad(&doc, metadata(), reject).unwrap();
    let model = doc
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Model" => Some(l),
            _ => None,
        })
        .unwrap();
    model.plot_flags.plot_hidden = true;
    assert!(matches!(
        cad_document_to_encoded_ifccad(&doc, metadata(), reject),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

fn object_xdata_documents() -> [opencadcodec::CadDocument; 2] {
    use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
    use std::io::Cursor;
    let source = CadDocument::new();
    let root = format!("{:X}", source.header.named_objects_dict_handle.value());
    let bytes = DxfWriter::new(&source).write_to_vec().unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let lines: Vec<_> = text.lines().collect();
    let mut augmented = String::new();
    let mut dictionary = false;
    let mut target = false;
    for pair in lines.as_chunks::<2>().0 {
        if pair[0].trim() == "0" {
            if target {
                augmented.push_str("1001\nACAD\n1000\nobject payload\n");
            }
            dictionary = pair[1].trim() == "DICTIONARY";
            target = false;
        }
        if dictionary && pair[0].trim() == "5" && pair[1].trim() == root {
            target = true;
        }
        augmented.push_str(pair[0]);
        augmented.push('\n');
        augmented.push_str(pair[1]);
        augmented.push('\n');
    }
    assert!(augmented.contains("object payload"));
    let dxf = DxfReader::from_reader(Cursor::new(augmented.into_bytes()))
        .unwrap()
        .read()
        .unwrap();
    let dwg = DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&dxf).unwrap()))
        .read()
        .unwrap();
    [dxf, dwg]
}

#[test]
fn object_xdata_is_visible_after_both_codecs_and_never_silently_dropped() {
    for doc in object_xdata_documents() {
        let out = cad_document_to_encoded_ifccad(&doc, metadata(), Default::default()).unwrap();
        assert_eq!(
            out.diagnostics()
                .iter()
                .filter(|d| d.code == "xdata")
                .count(),
            1
        );
        ocdraw::ifccad::load_ifccad_bytes(out.encoded().bytes(), Default::default()).unwrap();
        assert!(matches!(
            cad_document_to_encoded_ifccad(
                &doc,
                metadata(),
                CadToIfccadOptions {
                    loss_policy: IfccadLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
}

#[test]
fn unresolved_entity_layer_handle_is_not_the_codec_layer_zero_fallback() {
    let mut source = opencadcodec::CadDocument::new();
    let mut line = opencadcodec::Line::new();
    line.end = opencadcodec::Vector3::new(1., 0., 0.);
    line.common.layer_handle = Some(opencadcodec::Handle::new(0xdead));
    source
        .add_entity(opencadcodec::EntityType::Line(line))
        .unwrap();
    assert!(
        cad_document_to_ifccad_document(&source, metadata(), CadToIfccadOptions::default())
            .is_err()
    );
}

#[test]
fn noncanonical_insert_count_storage_remains_a_located_loss() {
    for bytes in [vec![1], vec![2]] {
        let mut source = opencadcodec::CadDocument::new();
        source
            .block_records
            .get_mut("*Model_Space")
            .unwrap()
            .insert_count_bytes = bytes;
        let out =
            cad_document_to_ifccad_document(&source, metadata(), CadToIfccadOptions::default())
                .unwrap();
        assert!(out
            .diagnostics()
            .iter()
            .any(|d| d.location.ends_with("insert_count_bytes")));
    }
}
