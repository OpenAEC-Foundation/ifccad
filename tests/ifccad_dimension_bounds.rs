#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::*;
#[test]
fn definition_points_do_not_certify_display_enclosure() {
    let d = support::drawing();
    let a = assess_ifccad_document_bounds(&d).unwrap();
    let e = &a.scopes[&IfccadScopeId::Layout(d.model.id)];
    assert_eq!(e.quality, Some(IfccadBoundsQuality::Partial));
    assert!(!e.enclosure_verified);
    assert!(d.model.bounds.is_none());
}
