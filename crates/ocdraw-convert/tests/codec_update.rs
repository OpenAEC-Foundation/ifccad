use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, CadToOcdrawError, CadToOcdrawLossReason, CadToOcdrawOptions,
    OcdrawLossPolicy,
};
use opencadcodec::{CadDocument, EntityType};

#[test]
fn autocad_anonymous_names_reach_ocdraw_without_name_repair() {
    let doc = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
        include_bytes!("fixtures/anonymous-names.dwg").as_slice(),
    ))
    .read()
    .unwrap();
    for name in ["NamedBefore", "*U2", "NamedBetween", "*U4", "*U5"] {
        let record = doc.block_records.get(name).unwrap();
        let EntityType::Block(marker) = doc.get_entity(record.block_entity_handle).unwrap() else {
            panic!()
        };
        assert_eq!(marker.name, name);
        assert_eq!(marker.common.owner_handle, record.handle);
        assert!(doc
            .entities()
            .any(|e| matches!(e, EntityType::Insert(i) if i.block_name == name)));
    }
    let out = cad_document_to_encoded_ocdraw(&doc, CadToOcdrawOptions::default()).unwrap();
    let loaded = ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    for name in ["NamedBefore", "*U2", "NamedBetween", "*U4", "*U5"] {
        assert!(drawing.block_definitions().iter().any(|b| b.name == name));
    }
}

#[test]
fn newly_exposed_header_settings_are_diagnosed_and_rejectable() {
    let edits: [fn(&mut opencadcodec::document::HeaderVariables); 4] = [
        |h| h.dwf_frame = 1,
        |h| h.dgn_frame = 2,
        |h| h.universal_create_date_julian = 2_460_000.,
        |h| h.universal_update_date_julian = 2_460_001.,
    ];
    for edit in edits {
        let mut doc = CadDocument::new();
        edit(&mut doc.header);
        let out = cad_document_to_encoded_ocdraw(&doc, CadToOcdrawOptions::default()).unwrap();
        assert!(out.diagnostics().iter().flat_map(|d| d.reasons()).any(|r|
            matches!(r, CadToOcdrawLossReason::UnsupportedHeaderField { name } if name == "other_header_semantics")));
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &doc,
                CadToOcdrawOptions {
                    loss_policy: OcdrawLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::LossRejected { .. })
        ));
    }
}

#[test]
fn off_screen_viewports_are_diagnosed_in_canvas_and_authored_viewports() {
    for overall in [false, true] {
        let mut doc = CadDocument::new();
        let layout = doc.add_layout("Sheet").unwrap();
        let opencadcodec::objects::ObjectType::Layout(layout) = &doc.objects[&layout] else {
            panic!()
        };
        let handle = if overall {
            layout.viewport
        } else {
            let mut v = opencadcodec::entities::Viewport::new();
            v.id = 2;
            doc.add_entity_to_layout(EntityType::Viewport(v), "Sheet")
                .unwrap()
        };
        let EntityType::Viewport(v) = doc.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        v.off_screen = true;
        let bytes = opencadcodec::DxfWriter::new(&doc).write_to_vec().unwrap();
        let decoded = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap();
        assert!(decoded
            .entities()
            .any(|e| matches!(e, EntityType::Viewport(v) if v.off_screen)));
        let out = cad_document_to_encoded_ocdraw(&decoded, CadToOcdrawOptions::default()).unwrap();
        assert!(out.diagnostics().iter().flat_map(|d| d.reasons()).any(|r|
            matches!(r, CadToOcdrawLossReason::UnsupportedSemantic { name } if name.contains("off-screen"))
        ), "{overall}: {:?}", out.diagnostics());
        assert!(ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes())
            .as_ref()
            .ok()
            .is_some());
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &decoded,
                CadToOcdrawOptions {
                    loss_policy: OcdrawLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::LossRejected { .. })
        ));
    }
}

#[test]
fn typed_plot_edits_after_dxf_read_are_not_overwritten_by_retained_raw_codes() {
    let mut source = CadDocument::new();
    let handle = source.add_layout("Sheet").unwrap();
    let opencadcodec::objects::ObjectType::Layout(layout) =
        source.objects.get_mut(&handle).unwrap()
    else {
        panic!()
    };
    layout.paper_width = 210.;
    layout.paper_height = 297.;
    layout.plot_type = 1; // Extents plotting has a supported mapping.
    let bytes = opencadcodec::DxfWriter::new(&source)
        .write_to_vec()
        .unwrap();
    let mut doc = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let layout = doc
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Sheet" => Some(l),
            _ => None,
        })
        .unwrap();
    assert!(layout.raw_plot_settings_codes.is_some());
    layout.plot_rotation = 1;
    let out = cad_document_to_encoded_ocdraw(&doc, CadToOcdrawOptions::default()).unwrap();
    let loaded = ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes());
    let layout = loaded
        .as_ref()
        .ok()
        .unwrap()
        .typed_layouts()
        .iter()
        .find(|l| l.name == "Sheet")
        .unwrap();
    assert_eq!(
        layout
            .settings
            .plot_settings
            .as_ref()
            .unwrap_or_else(|| panic!("{:?}", out.diagnostics()))
            .page
            .rotation,
        ocdraw::ocdraw::PlotRotation::CounterClockwise90
    );
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
        let mut records = 0;
        doc.semantic_inventory_v1().visit(|part| {
            if let opencadcodec::SemanticPartV1::NonEntityExtendedData {
                values: Some(values),
                ..
            } = part
            {
                if values
                    == [opencadcodec::xdata::XDataValue::String(
                        "object payload".into(),
                    )]
                {
                    records += 1;
                }
            }
        });
        assert_eq!(records, 1);
        let out = cad_document_to_encoded_ocdraw(&doc, CadToOcdrawOptions::default()).unwrap();
        assert!(out.diagnostics().iter().flat_map(|d| d.reasons()).any(|r|
            matches!(r, CadToOcdrawLossReason::UnsupportedCollection { kind, count } if kind == "inventory.non_entity_extended_data" && *count == 1)));
        assert!(ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes())
            .as_ref()
            .ok()
            .is_some());
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &doc,
                CadToOcdrawOptions {
                    loss_policy: OcdrawLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::LossRejected { .. })
        ));
    }
}
