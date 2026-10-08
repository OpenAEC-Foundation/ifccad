use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::ocdraw::load_ocdraw_bytes;
use ocdraw_convert::opencadcodec::{objects::ObjectType, DwgReader, DxfReader, Handle};
use std::{collections::BTreeSet, io::Cursor};

#[test]
fn ocdraw_selection_links_written_layout_entities_and_excludes_block_contents() {
    for bytes in [
        include_bytes!("../../../examples/ocdraw/overview.ocdraw.json").as_slice(),
        include_bytes!("../../../examples/ocdraw/layouts-viewports.ocdraw.json").as_slice(),
    ] {
        let drawing = load_ocdraw_bytes(bytes).unwrap();
        let expected: BTreeSet<_> = drawing
            .document()
            .layouts
            .iter()
            .flat_map(|layout| {
                drawing
                    .scopes()
                    .iter()
                    .find(|scope| scope.id == layout.scope_id)
                    .unwrap()
                    .entities
                    .iter()
                    .map(|id| format!("entity:{id}"))
            })
            .collect();
        for format in ["dxf", "dwg"] {
            let output =
                viewer::export_drawing_bytes("overview.ocdraw.json", bytes, format, "AC1032");
            assert!(output["failure"].is_null(), "{output}");
            let mapping = &output["export"]["viewerSelection"];
            assert_eq!(mapping["format"], "ocdraw");
            let links = mapping["entities"]
                .as_array()
                .expect("OCDraw selection links");
            assert_eq!(
                links
                    .iter()
                    .map(|link| link["path"].as_str().unwrap().to_owned())
                    .collect::<BTreeSet<_>>(),
                expected
            );
            let cad_bytes = STANDARD
                .decode(output["export"]["download"]["base64"].as_str().unwrap())
                .unwrap();
            let cad = if format == "dxf" {
                DxfReader::from_reader(Cursor::new(cad_bytes))
                    .unwrap()
                    .read()
                    .unwrap()
            } else {
                DwgReader::from_stream(Cursor::new(cad_bytes))
                    .read()
                    .unwrap()
            };
            for link in links {
                let handle =
                    Handle::new(u64::from_str_radix(link["handle"].as_str().unwrap(), 16).unwrap());
                let entity = cad.get_entity(handle).expect("written CAD entity");
                assert!(cad.objects.values().any(|object| matches!(object, ObjectType::Layout(layout) if layout.block_record == entity.common().owner_handle && layout.name == link["layout"].as_str().unwrap())));
            }
        }
    }
}
