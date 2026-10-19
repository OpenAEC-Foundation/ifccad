use ifccad::geometry_kernel::hatch::HatchLoopProvenance;

#[test]
fn empty_provenance_is_exactly_all_false() {
    for bits in 0..16 {
        let value = HatchLoopProvenance {
            from_source_objects: bits & 1 != 0,
            derived_from_pick_point: bits & 2 != 0,
            outermost_hint: bits & 4 != 0,
            text_box: bits & 8 != 0,
        };
        assert_eq!(value.is_empty(), bits == 0);
    }
    assert!(HatchLoopProvenance::default().is_empty());
}
