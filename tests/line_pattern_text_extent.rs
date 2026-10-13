use ocdraw::line_pattern::*;

fn input(text: &str) -> ResolvedLinePatternTextExtentInput<'_> {
    ResolvedLinePatternTextExtentInput {
        text,
        fixed_style_height: None,
        marker_scale: 2.,
        pattern_scale: 1.,
        offset: [0.; 2],
    }
}

#[test]
fn empty_marker_has_no_radius() {
    assert_eq!(
        estimate_line_pattern_text_extent(&input(""))
            .unwrap()
            .radius,
        None
    );
}

#[test]
fn fixed_and_variable_height_use_cad_scale_meaning() {
    let variable = estimate_line_pattern_text_extent(&input("GAS"))
        .unwrap()
        .radius
        .unwrap();
    let fixed = estimate_line_pattern_text_extent(&ResolvedLinePatternTextExtentInput {
        fixed_style_height: Some(3.),
        ..input("GAS")
    })
    .unwrap()
    .radius
    .unwrap();
    assert!((fixed / variable - 3.).abs() < 1e-12);
}

#[test]
fn offset_is_not_multiplied_by_marker_scale() {
    for marker_scale in [0.1, 2., 10.] {
        let base = ResolvedLinePatternTextExtentInput {
            marker_scale,
            pattern_scale: 3.,
            ..input("GAS")
        };
        let a = estimate_line_pattern_text_extent(&base)
            .unwrap()
            .radius
            .unwrap();
        let b = estimate_line_pattern_text_extent(&ResolvedLinePatternTextExtentInput {
            offset: [4., 0.],
            ..base
        })
        .unwrap()
        .radius
        .unwrap();
        assert!((b - a - 12.).abs() < 1e-10);
    }
}

#[test]
fn overflow_is_hard_error() {
    let input = ResolvedLinePatternTextExtentInput {
        marker_scale: f64::MAX,
        ..input("GAS")
    };
    assert!(estimate_line_pattern_text_extent(&input).is_err());
    let input = ResolvedLinePatternTextExtentInput {
        pattern_scale: f64::MAX,
        ..self::input("GAS")
    };
    assert!(estimate_line_pattern_text_extent(&input).is_err());
}

#[test]
fn radius_covers_rotated_estimate() {
    use ocdraw::{geometry_kernel::*, text::*};
    let runs = [TextRun {
        text: "GAS α".into(),
        ..Default::default()
    }];
    let estimate = estimate_text_extent(&ResolvedTextExtentInput {
        content: &runs,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.,
            width_factor: 1.,
        },
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        oblique_angle: 0.,
        thickness: 0.,
        vertical: false,
    })
    .unwrap();
    let radius = estimate_line_pattern_text_extent(&input("GAS α"))
        .unwrap()
        .radius
        .unwrap();
    let b = estimate.local_bounds.unwrap();
    for angle in [0_f64, 0.25, 1., 2., -3.] {
        for x in [b.min().components()[0], b.max().components()[0]] {
            for y in [b.min().components()[1], b.max().components()[1]] {
                let (s, c) = angle.sin_cos();
                assert!((x * c - y * s).abs() <= radius);
                assert!((x * s + y * c).abs() <= radius);
            }
        }
    }
}
