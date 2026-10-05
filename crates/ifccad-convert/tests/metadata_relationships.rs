mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::load_ifccad_bytes;
use opencadcodec::{
    objects::{ClassObject, ClassObjectData, ObjectType, Sun},
    Handle,
};

fn sample() -> opencadcodec::CadDocument {
    to_cad(&validated(&primitives())).unwrap().into_document()
}

#[test]
fn unsupported_sun_owner_is_located_loss_not_structural_failure() {
    let mut source = sample();
    let handle = source.allocate_handle();
    let missing = Handle::new(99999);
    let mut sun = ClassObject::new(ClassObjectData::Sun(Sun::default()));
    sun.handle = handle;
    sun.owner = missing;
    source.objects.insert(handle, ObjectType::ClassObject(sun));
    let location = format!("object/{handle}.owner");
    let logical = cad_document_to_ifccad_document(&source, metadata(), Default::default()).unwrap();
    let encoded = cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
    assert_eq!(logical.document(), encoded.validated_source().document());
    assert_eq!(logical.diagnostics(), encoded.diagnostics());
    assert!(logical
        .diagnostics()
        .iter()
        .any(|d| d.code == "unresolved-relationship"
            && d.location == location
            && d.message.contains("SUN")
            && d.message.contains(&missing.to_string())));
    assert!(logical
        .diagnostics()
        .iter()
        .any(|d| d.code == "object" && d.location == format!("object/{handle}")));
    assert_eq!(
        logical.document().model.entities.len(),
        primitives().model.entities.len()
    );
    load_ifccad_bytes(encoded.encoded().bytes(), Default::default()).unwrap();
    for result in [
        cad_document_to_ifccad_document(
            &source,
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Reject,
            },
        )
        .map(|_| ()),
        cad_document_to_encoded_ifccad(
            &source,
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Reject,
            },
        )
        .map(|_| ()),
    ] {
        assert!(
            matches!(result, Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "unresolved-relationship" && d.location == location))
        );
    }
    assert!(
        matches!(source.objects.get(&handle), Some(ObjectType::ClassObject(s)) if s.owner == missing)
    );
}

#[test]
fn unresolved_optional_entity_relationships_keep_supported_geometry() {
    for dictionary in [false, true] {
        let mut source = sample();
        let handle = source
            .block_records
            .get("*Model_Space")
            .unwrap()
            .entity_handles[0];
        let common = source.get_entity_mut(handle).unwrap().common_mut();
        if dictionary {
            common.xdictionary_handle = Some(Handle::new(99999));
        } else {
            common.reactors.push(Handle::new(99999));
        }
        let allow =
            cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
        assert_eq!(
            allow.validated_source().document().model.entities.len(),
            primitives().model.entities.len()
        );
        assert!(allow
            .diagnostics()
            .iter()
            .any(|d| d.code == "unresolved-relationship"
                && d.location.starts_with(&format!("entity/{handle}."))));
        load_ifccad_bytes(allow.encoded().bytes(), Default::default()).unwrap();
        assert!(matches!(
            cad_document_to_encoded_ifccad(
                &source,
                metadata(),
                CadToIfccadOptions {
                    loss_policy: IfccadLossPolicy::Reject
                }
            ),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
}

#[test]
fn essential_entity_and_layout_ownership_remain_fatal_under_both_policies() {
    for layout in [false, true] {
        let mut source = sample();
        if layout {
            let ObjectType::Layout(l) = source
                .objects
                .values_mut()
                .find(|o| matches!(o, ObjectType::Layout(l) if l.name == "Model"))
                .unwrap()
            else {
                unreachable!()
            };
            l.owner = Handle::new(99999);
        } else {
            let handle = source
                .block_records
                .get("*Model_Space")
                .unwrap()
                .entity_handles[0];
            source
                .get_entity_mut(handle)
                .unwrap()
                .common_mut()
                .owner_handle = Handle::new(99999);
        }
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let options = CadToIfccadOptions {
                loss_policy: policy,
            };
            assert!(matches!(
                cad_document_to_ifccad_document(&source, metadata(), options),
                Err(IfccadConversionError::InvalidStructure(_))
            ));
            assert!(matches!(
                cad_document_to_encoded_ifccad(&source, metadata(), options),
                Err(IfccadConversionError::InvalidStructure(_))
            ));
        }
    }
}
