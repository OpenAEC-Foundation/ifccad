mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;

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
        source.layers[1].appearance.color = IfccadColor::rgb(255, 0, 0);
        let before = source.clone();
        validate_ifccad_document(&source).unwrap();
        let loaded = validated(&source);
        let original = loaded.graph().source_bytes().to_vec();
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let options = CadToIfccadOptions {
                loss_policy: policy,
                ..Default::default()
            };
            for result in [
                ifccad_document_to_cad_document(
                    &source,
                    IfccadToCadOptions {
                        loss_policy: (options).loss_policy,
                        ..Default::default()
                    },
                ),
                ifccad_source_to_cad_document(
                    &loaded,
                    IfccadToCadOptions {
                        loss_policy: (options).loss_policy,
                        ..Default::default()
                    },
                ),
            ] {
                match result {
                    Err(IfccadConversionError::InvalidStructure(message)) => {
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
