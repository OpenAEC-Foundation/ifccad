//! Characterization of the unmodified pinned codec at the converter boundary.
use cadcodec::entities::Insert;
use cadcodec::{
    BlockRecord, CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Line, Vector3,
};
use std::io::Cursor;

fn fixture(scale: (f64, f64, f64), unit: i16, empty: bool) -> CadDocument {
    let mut document = CadDocument::new();
    let mut record = BlockRecord::new("Door");
    record.handle = document.allocate_handle();
    record.block_entity_handle = document.allocate_handle();
    record.block_end_handle = document.allocate_handle();
    record.base_point = Vector3::new(2., 0., 0.);
    record.units = unit;
    record.description = "Door description".into();
    record.flags.anonymous = true;
    record.explodable = false;
    record.scale_uniformly = scale.0 == scale.1 && scale.1 == scale.2;
    let owner = record.handle;
    document.block_records.add(record).unwrap();
    if !empty {
        let mut line = Line::from_coords(2., 0., 0., 3., 0., 0.);
        line.common.owner_handle = owner;
        document.add_entity(EntityType::Line(line)).unwrap();
    }
    let mut insert = Insert::new("Door", Vector3::new(3., 4., 5.));
    insert.normal = Vector3::new(1., 2., 3.);
    insert.rotation = 0.7;
    insert.set_x_scale(scale.0);
    insert.set_y_scale(scale.1);
    insert.set_z_scale(scale.2);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    document
}

fn roundtrip(document: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(document).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(document).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    }
}

fn check_roundtrip(dwg: bool) {
    for unit in 0..=24 {
        for scale in [(2., 3., -4.), (-2., -2., -2.)] {
            let result = roundtrip(&fixture(scale, unit, false), dwg);
            let block = result
                .block_records
                .get("Door")
                .expect("definition survives");
            assert_eq!(block.base_point, Vector3::new(2., 0., 0.));
            assert_eq!(block.units, unit);
            assert!(block.flags.anonymous);
            assert!(!block.explodable);
            assert_eq!(
                block.scale_uniformly,
                scale.0 == scale.1 && scale.1 == scale.2
            );
            if dwg {
                let marker = result
                    .get_entity(block.block_entity_handle)
                    .expect("block marker");
                let EntityType::Block(marker) = marker else {
                    panic!("wrong block marker kind")
                };
                assert_eq!(marker.name, "Door");
                // Known limitation: DWG constructs a public marker with zero
                // base point, inconsistent with the correctly decoded record.
                assert_eq!(marker.base_point, Vector3::ZERO);
                assert_ne!(marker.base_point, block.base_point);
                assert_eq!(marker.common.owner_handle, block.handle);
            } else {
                // DXF deliberately keeps structural markers out of entities.
                assert!(result.get_entity(block.block_entity_handle).is_none());
            }
            // The DXF BLOCKS writer omits group 4; DWG preserves description.
            assert_eq!(block.description, if dwg { "Door description" } else { "" });
            let insert = result
                .entities()
                .find_map(|entity| match entity {
                    EntityType::Insert(value) => Some(value),
                    _ => None,
                })
                .unwrap();
            assert_eq!(insert.insert_point, Vector3::new(3., 4., 5.));
            assert_eq!(insert.normal, Vector3::new(1., 2., 3.));
            assert!((insert.rotation - 0.7).abs() < 1e-14);
            assert_eq!(
                (insert.x_scale(), insert.y_scale(), insert.z_scale()),
                scale
            );
            let lines: Vec<_> = result
                .entities_in_block("Door")
                .filter_map(|entity| match entity {
                    EntityType::Line(value) => Some(value),
                    _ => None,
                })
                .collect();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].start, Vector3::new(2., 0., 0.));
            assert_eq!(lines[0].end, Vector3::new(3., 0., 0.));
            // Independent AAA derivation for n=(1,2,3):
            // u=(-2,1,0)/sqrt(5), v=(-3,-6,5)/sqrt(70), w=n/sqrt(14).
            // The literals bracket sin/cos(0.7) to the last displayed digit;
            // this sampled comparison is not a whole-domain accuracy proof.
            let u = [-2. / 5_f64.sqrt(), 1. / 5_f64.sqrt(), 0.];
            let v = [-3. / 70_f64.sqrt(), -6. / 70_f64.sqrt(), 5. / 70_f64.sqrt()];
            let w = [1. / 14_f64.sqrt(), 2. / 14_f64.sqrt(), 3. / 14_f64.sqrt()];
            for local_x in [0., 1.] {
                let point = insert.get_transform().apply(Vector3::new(local_x, 0., 0.));
                for (axis, actual) in [point.x, point.y, point.z].into_iter().enumerate() {
                    let expected = (3. + local_x * scale.0 * 0.764_842_187_284_488_5) * u[axis]
                        + (4. + local_x * scale.0 * 0.644_217_687_237_691) * v[axis]
                        + 5. * w[axis];
                    assert!(
                        (actual - expected).abs() < 1e-13,
                        "axis {axis}: {actual} != {expected}"
                    );
                }
            }
        }
    }
}

#[test]
fn dxf_block_boundary() {
    check_roundtrip(false);
}

#[test]
fn dwg_block_boundary() {
    check_roundtrip(true);
}

#[test]
fn tiny_scale_is_clamped_before_geometry_or_serialization() {
    for scale in [1e-13, -1e-13] {
        let document = fixture((scale, 1., 1.), 0, true);
        assert_eq!(document.entities_in_block("Door").count(), 0);
        let insert = document
            .entities()
            .find_map(|entity| match entity {
                EntityType::Insert(value) => Some(value),
                _ => None,
            })
            .unwrap();
        assert_eq!(insert.x_scale(), 1e-12);
        assert_ne!(insert.x_scale(), scale);
        for dwg in [false, true] {
            let result = roundtrip(&document, dwg);
            assert_eq!(
                result
                    .entities_in_block("Door")
                    .filter(|entity| matches!(entity, EntityType::Line(_)))
                    .count(),
                0
            );
            let insert = result
                .entities()
                .find_map(|entity| match entity {
                    EntityType::Insert(value) => Some(value),
                    _ => None,
                })
                .unwrap();
            assert_eq!(insert.x_scale(), 1e-12);
        }
    }
}
