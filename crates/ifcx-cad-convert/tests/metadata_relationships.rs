mod common;
use cadcodec::{
    objects::{ClassObject, ClassObjectData, ObjectType, Sun},
    Handle,
};
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::read_native_cad_ifcx;

fn sample() -> cadcodec::CadDocument {
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
    let logical = cad_document_to_ifcx_cad_document(&source, metadata()).unwrap();
    let encoded = cad_document_to_ifcx_cad(&source, metadata()).unwrap();
    assert_eq!(logical.document(), encoded.validated_ifcx().document());
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
    read_native_cad_ifcx(encoded.ifcx_bytes()).unwrap();
    for result in [
        cad_document_to_ifcx_cad_document_with_options(
            &source,
            metadata(),
            IfcxCadConversionOptions {
                loss_policy: IfcxCadLossPolicy::Reject,
            },
        )
        .map(|_| ()),
        cad_document_to_ifcx_cad_with_options(
            &source,
            metadata(),
            IfcxCadConversionOptions {
                loss_policy: IfcxCadLossPolicy::Reject,
            },
        )
        .map(|_| ()),
    ] {
        assert!(
            matches!(result, Err(IfcxCadConversionError::Unsupported(d)) if d.iter().any(|d| d.code == "unresolved-relationship" && d.location == location))
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
        let allow = cad_document_to_ifcx_cad(&source, metadata()).unwrap();
        assert_eq!(
            allow.validated_ifcx().document().model.entities.len(),
            primitives().model.entities.len()
        );
        assert!(allow
            .diagnostics()
            .iter()
            .any(|d| d.code == "unresolved-relationship"
                && d.location.starts_with(&format!("entity/{handle}."))));
        read_native_cad_ifcx(allow.ifcx_bytes()).unwrap();
        assert!(matches!(
            cad_document_to_ifcx_cad_with_options(
                &source,
                metadata(),
                IfcxCadConversionOptions {
                    loss_policy: IfcxCadLossPolicy::Reject
                }
            ),
            Err(IfcxCadConversionError::Unsupported(_))
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
        for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
            let options = IfcxCadConversionOptions {
                loss_policy: policy,
            };
            assert!(matches!(
                cad_document_to_ifcx_cad_document_with_options(&source, metadata(), options),
                Err(IfcxCadConversionError::InvalidStructure(_))
            ));
            assert!(matches!(
                cad_document_to_ifcx_cad_with_options(&source, metadata(), options),
                Err(IfcxCadConversionError::InvalidStructure(_))
            ));
        }
    }
}
