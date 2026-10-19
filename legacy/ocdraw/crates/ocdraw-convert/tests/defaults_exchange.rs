use ocdraw::ocdraw::*;
use ocdraw_convert::*;

#[test]
fn compact_and_expanded_sources_have_equal_cad_semantics() {
    let mut outputs = Vec::new();
    for bytes in [
        include_bytes!("../../../conformance/next/ocdraw/valid/defaults-expanded.ocdraw.json")
            .as_slice(),
        include_bytes!("../../../conformance/next/ocdraw/valid/defaults-compact.ocdraw.json")
            .as_slice(),
    ] {
        let source = load_ocdraw_bytes(bytes).unwrap();
        let cad = ocdraw_source_to_cad_document(
            &source,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Allow,
                ..Default::default()
            },
        )
        .unwrap();
        let restored =
            cad_document_to_ocdraw_document(cad.document(), CadToOcdrawOptions::default()).unwrap();
        outputs.push(restored.into_document());
    }
    assert_eq!(outputs[0].geometric_entities, outputs[1].geometric_entities);
    assert_eq!(outputs[0].layers, outputs[1].layers);
    assert_eq!(outputs[0].block_definitions, outputs[1].block_definitions);
    assert_eq!(outputs[0].scopes, outputs[1].scopes);
    assert_eq!(outputs[0].line_patterns, outputs[1].line_patterns);
}
