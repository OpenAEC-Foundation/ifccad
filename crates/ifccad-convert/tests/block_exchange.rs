//! Real codecs are part of this boundary test; no patched dependency or markers.
use cadcodec::{
    BlockRecord, CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Line, Vector3,
};
use ifccad::package::{load_directory_package, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, drawing_to_cad_document, ExportError, ExportOptions,
};
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};

fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("exchange").unwrap(),
        data_version: "1".into(),
        author: "Tests".into(),
        timestamp: "2026-09-22T10:00:00Z".into(),
    }
}

fn exchange(dwg: bool, base: f64) {
    let mut source = CadDocument::new();
    source.header.insertion_units = 6;
    let mut record = BlockRecord::new("Door");
    record.handle = source.allocate_handle();
    record.base_point = Vector3::new(base, 0., 0.);
    record.description = "Entrance".into();
    record.units = 1;
    let owner = record.handle;
    source.block_records.add(record).unwrap();
    let mut line = Line::from_coords(base, 0., 0., base + 1., 0., 0.);
    line.common.owner_handle = owner;
    source.add_entity(EntityType::Line(line)).unwrap();
    for x in [10., 20.] {
        let mut insert = cadcodec::entities::Insert::new("Door", Vector3::new(x, 0., 0.));
        insert.set_x_scale(-2.);
        source.add_entity(EntityType::Insert(insert)).unwrap();
    }
    let encoded = cad_document_to_package(&source, options(), ExportOptions::default()).unwrap();
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "ifccad-block-exchange-{}-{tick}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    encoded.package().write_directory(&path).unwrap();
    let loaded = load_directory_package(&path).unwrap();
    let target = drawing_to_cad_document(
        loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap(),
    )
    .unwrap();
    let decoded = if dwg {
        DwgReader::from_stream(Cursor::new(
            DwgWriter::write_to_vec(target.document()).unwrap(),
        ))
        .read()
        .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(target.document()).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    };
    let record = decoded.block_records.get("Door").unwrap();
    assert_eq!(record.base_point, Vector3::new(base, 0., 0.));
    assert_eq!(record.units, 1);
    assert_eq!(decoded.header.insertion_units, 6);
    assert_eq!(record.description, if dwg { "Entrance" } else { "" }); // upstream #49
    let lines: Vec<_> = decoded
        .entities_in_block("Door")
        .filter_map(|e| {
            if let EntityType::Line(l) = e {
                Some(l)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].start, Vector3::new(base, 0., 0.));
    let inserts: Vec<_> = decoded
        .entities()
        .filter_map(|e| {
            if let EntityType::Insert(i) = e {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(inserts.len(), 2);
    for (insert, x) in inserts.iter().zip([10., 20.]) {
        assert_eq!(insert.block_name, "Door");
        assert_eq!(insert.x_scale(), -2.);
        // Independently expected transformed endpoint, including base subtraction.
        assert_eq!(
            insert
                .get_transform()
                .apply(Vector3::new(lines[0].end.x - base, 0., 0.)),
            Vector3::new(x - 2., 0., 0.)
        );
    }
    let returned = cad_document_to_package(&decoded, options(), ExportOptions::default());
    if dwg && base != 0. {
        assert!(
            matches!(returned, Err(ExportError::InvalidSourceStructure { .. })),
            "upstream #52 must not be hidden"
        );
    } else {
        let returned = returned.unwrap();
        let back_path = path.join("returned");
        returned.package().write_directory(&back_path).unwrap();
        assert!(load_directory_package(&back_path)
            .unwrap()
            .validated_package()
            .is_some());
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn dxf_local_blocks_exchange_with_nonzero_base() {
    exchange(false, 2.);
}
#[test]
fn dwg_local_blocks_exchange_with_zero_base() {
    exchange(true, 0.);
}
#[test]
fn dwg_nonzero_base_remains_explicitly_blocked_on_return() {
    exchange(true, 2.);
}
