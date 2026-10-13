use cad_dimension::*;
use ocdraw::dimension::*;
use opencadcodec::{
    entities::{Dimension, DimensionAligned},
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Vector3,
};
use std::io::Cursor;

#[test]
fn literal_dxf_group_11_is_ocs_independently_of_codec_comments() {
    let doc = DxfReader::from_reader(Cursor::new(include_bytes!(
        "../../../tests/fixtures/dimension/tilted-aligned.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let d = doc
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(d.base().text_middle_point, Vector3::new(2.0, 7.0, 3.0));
    let p = prepare_dimension_from_cad(d, &doc).unwrap();
    assert_eq!(p.definition.points(), ([0.0, 0.0], [4.0, 5.0]));
    assert!(matches!(
        p.text_placement,
        DimensionTextPlacement::Explicit {
            position: [2.0, 7.0],
            ..
        }
    ));
    // AA basis for N=(1,0,0): U=(0,1,0), V=(0,0,1).
    // Raw text (2,7,3) therefore means WCS (3,2,7), not WCS (2,7,3).
    assert_eq!(p.placement.origin().components(), [3.0, 0.0, 0.0]);
}

#[test]
fn literal_clone_group_12_and_angular_group_16_remain_outside_slice_one() {
    let literal = include_str!("../../../tests/fixtures/dimension/tilted-aligned.dxf");
    let clone = literal.replace("AcDbDimension\n", "AcDbDimension\n12\n19\n22\n23\n32\n29\n");
    let doc = DxfReader::from_reader(Cursor::new(clone.into_bytes()))
        .unwrap()
        .read()
        .unwrap();
    let d = doc
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    // Pinned codec defect: read_dimension collects group 12 but never assigns it.
    // Record the defect explicitly, without qualifying clones as supported.
    assert_eq!(d.base().insertion_point, Vector3::ZERO);
    let mut typed = d.clone();
    typed.base_mut().insertion_point = Vector3::new(19.0, 23.0, 29.0);
    assert!(
        matches!(prepare_dimension_from_cad(&typed,&doc),Err(CadDimensionPreparationError::Unsupported{field,..}) if field=="insertion")
    );
    let angular = literal
        .replace("161\n", "162\n")
        .replace("AcDbAlignedDimension", "AcDb2LineAngularDimension")
        .replace(
            "0\nENDSEC\n0\nEOF",
            "15\n3\n25\n0\n35\n0\n16\n2\n26\n7\n36\n3\n0\nENDSEC\n0\nEOF",
        );
    let doc = DxfReader::from_reader(Cursor::new(angular.into_bytes()))
        .unwrap()
        .read()
        .unwrap();
    let d = doc
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    let Dimension::Angular2Ln(d) = d else {
        panic!("angular variant")
    };
    assert_eq!(d.dimension_arc, Vector3::new(2.0, 7.0, 3.0));
    assert!(
        matches!(prepare_dimension_from_cad(&Dimension::Angular2Ln(d.clone()),&doc),Err(CadDimensionPreparationError::Unsupported{field,..}) if field=="kind")
    );
}

#[test]
fn actual_dxf_and_ac1032_dwg_keep_wcs_definition_and_ocs_text() {
    let mut source =
        DimensionAligned::new(Vector3::new(3.0, 0.0, 0.0), Vector3::new(3.0, 4.0, 5.0));
    source.definition_point = Vector3::new(3.0, 8.0, 0.0);
    source.base.definition_point = source.definition_point;
    source.base.normal = Vector3::new(1.0, 0.0, 0.0);
    source.base.text_user_positioned = true;
    source.base.text_middle_point = Vector3::new(2.0, 7.0, 3.0);
    source.base.actual_measurement = 999.0;
    source.base.set_text_override(Some("distance <> mm".into()));
    let mut doc = CadDocument::new();
    doc.add_entity(EntityType::Dimension(Dimension::Aligned(source)))
        .unwrap();
    for dwg in [false, true] {
        let loaded = if dwg {
            let bytes = DwgWriter::write_to_vec(&doc).unwrap();
            assert_eq!(&bytes[..6], b"AC1032");
            DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(DxfWriter::new(&doc).write_to_vec().unwrap()))
                .unwrap()
                .read()
                .unwrap()
        };
        let d = loaded
            .entities()
            .find_map(|e| {
                if let EntityType::Dimension(d) = e {
                    Some(d)
                } else {
                    None
                }
            })
            .unwrap();
        let p = prepare_dimension_from_cad(d, &loaded).unwrap();
        assert_eq!(
            p.placement.origin().components(),
            [3.0, 0.0, 0.0],
            "dwg={dwg}"
        );
        assert_eq!(p.definition.points(), ([0.0, 0.0], [4.0, 5.0]), "dwg={dwg}");
        assert!(
            matches!(
                p.text_placement,
                DimensionTextPlacement::Explicit {
                    position: [2.0, 7.0],
                    ..
                }
            ),
            "dwg={dwg}"
        );
        assert!(matches!(p.label, DimensionLabel::Template(_)));
        assert_ne!(
            evaluate_dimension(&p.definition).unwrap(),
            DimensionMeasurement::Distance(999.0)
        );
    }
}

#[test]
fn dimassoc_start_reference_survives_both_codecs() {
    use opencadcodec::{
        objects::{
            AssocDimensionAssociation, AssocDimensionReference, AssociativeData, AssociativeObject,
            ObjectType,
        },
        Line,
    };
    let mut doc = CadDocument::new();
    let source = doc
        .add_entity(EntityType::Line(Line::from_coords(
            4500.0, 0.0, 0.0, 9000.0, 0.0, 0.0,
        )))
        .unwrap();
    let dimension = doc
        .add_entity(EntityType::Dimension(Dimension::Aligned(
            DimensionAligned::new(
                Vector3::new(4000.0, 0.0, 0.0),
                Vector3::new(8000.0, 0.0, 0.0),
            ),
        )))
        .unwrap();
    let mut a = AssocDimensionAssociation {
        associativity: 1,
        dimension,
        ..Default::default()
    };
    a.references[0].push(AssocDimensionReference {
        class_name: "AcDbOsnapPointRef".into(),
        osnap_type: 13,
        xrefs: vec![source],
        osnap_point: Vector3::new(4000.0, 0.0, 0.0),
        ..Default::default()
    });
    let mut object = AssociativeObject::new("DIMASSOC", "AcDbDimAssoc");
    object.handle = doc.allocate_handle();
    object.owner = dimension;
    object.data = AssociativeData::DimensionAssociation(a);
    doc.objects
        .insert(object.handle, ObjectType::Associative(object));
    for dwg in [false, true] {
        let loaded = if dwg {
            DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
                .read()
                .unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(DxfWriter::new(&doc).write_to_vec().unwrap()))
                .unwrap()
                .read()
                .unwrap()
        };
        let d = loaded
            .entities()
            .find_map(|e| {
                if let EntityType::Dimension(d) = e {
                    Some(d)
                } else {
                    None
                }
            })
            .unwrap();
        let p = prepare_dimension_from_cad(d, &loaded).unwrap();
        assert_eq!(p.pending_associations.len(), 1, "dwg={dwg}");
        assert_eq!(
            p.pending_associations[0].selector,
            DimensionPointSelector::Start
        );
        assert_eq!(p.consumed_association_objects.len(), 1);
        assert_eq!(
            evaluate_dimension(&p.definition).unwrap(),
            DimensionMeasurement::Distance(4000.0)
        );
    }
}
