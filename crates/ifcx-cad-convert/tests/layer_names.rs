mod common;
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;

#[test]
fn target_layer_name_collisions_fail_under_both_policies_and_routes() {
    for (first, second) in [
        ("0", "0"),
        ("Notes", "Notes"),
        ("Notes", "nOtEs"),
        ("Straße", "STRASSE"),
    ] {
        let mut source = primitives();
        source.layers[0].name = first.into();
        source.layers[1].name = second.into();
        source.layers[1].appearance.color = "#FF0000".into();
        let before = source.clone();
        validate_ifcx_cad_document(&source).unwrap();
        let loaded = validated(&source);
        let original = loaded.graph().source_bytes().to_vec();
        for policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
            let options = CadToIfcxCadOptions {
                loss_policy: policy,
            };
            for result in [
                ifcx_cad_document_to_cad_document(
                    &source,
                    IfcxCadToCadOptions {
                        loss_policy: (options).loss_policy,
                    },
                ),
                ifcx_cad_source_to_cad_document(
                    &loaded,
                    IfcxCadToCadOptions {
                        loss_policy: (options).loss_policy,
                    },
                ),
            ] {
                match result {
                    Err(IfcxCadConversionError::InvalidStructure(message)) => {
                        assert!(message.contains("layer/0"), "{message}");
                        assert!(message.contains("layer/4"), "{message}");
                        assert!(message.contains("lookup collision"), "{message}");
                    }
                    Err(error) => panic!("wrong collision error for {first:?}/{second:?}: {error}"),
                    Ok(_) => panic!("silently accepted colliding layers {first:?}/{second:?}"),
                }
            }
        }
        assert_eq!(source, before);
        assert_eq!(loaded.document(), &before);
        assert_eq!(loaded.graph().source_bytes(), original);
    }
}
