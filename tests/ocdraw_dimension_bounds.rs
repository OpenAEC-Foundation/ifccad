#[path = "support/dimension.rs"]
mod support;
use ocdraw::ocdraw::*;

#[test]
fn measurement_points_are_not_an_enclosing_display_box() {
    let d = support::drawing();
    validate_ocdraw_document(&d).unwrap();
    let a = assess_ocdraw_document_bounds(&d).unwrap();
    assert_eq!(a[0].quality, Some(OcdrawBoundsQuality::Partial));
    assert!(!a[0].enclosure_verified);
    assert!(d.scopes[0].bounds.is_none());
}
