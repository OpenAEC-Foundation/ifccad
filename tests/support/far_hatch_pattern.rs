//! Far-coordinate periodic family reproducing the test.dxf pattern proof gap.
pub fn hatch() -> opencadcodec::entities::hatch::Hatch {
    use opencadcodec::{entities::hatch::*, Vector2};
    let mut h = Hatch::with_pattern(HatchPattern::new("Explicit far family"));
    h.pattern_type = HatchPatternType::Custom;
    h.pattern_angle = std::f64::consts::FRAC_PI_2;
    h.pattern_scale = 0.5;
    h.pattern.lines.push(HatchPatternLine {
        angle: 1.0035643198967394,
        base_point: Vector2::new(-29999.99999999953, -28.32099999999272),
        offset: Vector2::new(2.864371996336726, 67.7952975834357),
        dash_lengths: vec![
            0.,
            -31.74999999999999,
            0.,
            -99.05999999999997,
            0.,
            -131.4449999999999,
        ],
    });
    let mut path = BoundaryPath::new();
    path.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::new(-596000., -52000.),
        radius: 4000.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(path);
    h
}
