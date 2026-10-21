use ifccad::geometry_kernel::*;
fn filled(corners: &[[f64; 2]; 4], thickness: f64) -> GeometryRef<'_> {
    GeometryRef::FilledQuadrilateral {
        placement: CoordinateFrame3::default(),
        corners,
        thickness,
    }
}
fn cases() -> Vec<[[f64; 2]; 4]> {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/geometry/fixtures/solid-trace-profile/cases.json"
    ))
    .unwrap();
    v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::from_value(v["corners"].clone()).unwrap())
        .collect()
}
#[test]
fn filled_all_finite_topologies_are_valid() {
    for c in cases() {
        for h in [0., 3., -3.] {
            validate_geometry(filled(&c, h)).unwrap();
        }
    }
}
#[test]
fn filled_bounds_cover_signed_sweep() {
    let c = [[0., 0.], [4., 0.], [0., 3.], [0., 3.]];
    let b = geometry_bounds(filled(&c, -2.)).unwrap();
    assert_eq!(b.min().components(), [0., 0., -2.]);
    assert_eq!(b.max().components(), [4., 3., 0.]);
    let c = [[4., 4.]; 4];
    let b = geometry_bounds(filled(&c, 3.)).unwrap();
    assert_eq!(b.min().components(), [4., 4., 0.]);
    assert_eq!(b.max().components(), [4., 4., 3.]);
}
#[test]
fn filled_surface_cover_is_optional() {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/geometry/fixtures/solid-trace-profile/cases.json"
    ))
    .unwrap();
    for (c, record) in cases().iter().zip(v["cases"].as_array().unwrap()) {
        assert_eq!(
            filled_quadrilateral_surface_cover(c).is_some(),
            record["simpleSurfaceCoverCandidate"].as_bool().unwrap(),
            "{record}"
        );
    }
}
#[test]
fn filled_rejects_only_invalid_values_frame_or_enclosure() {
    let mut c = [[0., 0.]; 4];
    c[0][0] = f64::NAN;
    assert!(validate_geometry(filled(&c, 0.)).is_err());
    assert!(validate_geometry(filled(&[[0., 0.]; 4], f64::INFINITY)).is_err());
    let p = CoordinateFrame3::try_new(
        Point3::new(0., 0., f64::MAX),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    assert!(validate_geometry(GeometryRef::FilledQuadrilateral {
        placement: p,
        corners: &[[0., 0.]; 4],
        thickness: f64::MAX
    })
    .is_err());
}
#[test]
fn filled_is_not_a_clip_boundary() {
    assert_eq!(
        validate_paper_boundary(
            PaperFrame {
                center: [0., 0.],
                width: 20.,
                height: 20.
            },
            filled(&[[0., 0.]; 4], 0.)
        ),
        Err(GeometryValidationError::UnsupportedBoundary)
    );
}

#[test]
fn filled_winding_contacts_and_spurs_do_not_gate_storage() {
    for c in cases() {
        let c = [c[0], c[2], c[1], c[3]];
        for h in [0., 3., -3.] {
            validate_geometry(filled(&c, h)).unwrap();
        }
    }
    for c in [
        [[0., 0.], [2., 0.], [0., 2.], [0., 1.]],
        [[0., 0.], [2., 0.], [1., 0.], [3., 0.]],
        [
            [0., 0.],
            [f64::from_bits(1), 0.],
            [0., f64::from_bits(1)],
            [f64::from_bits(1), f64::from_bits(1)],
        ],
    ] {
        validate_geometry(filled(&c, 0.)).unwrap();
    }
}
#[test]
fn filled_concave_covers_keep_the_interior_diagonal() {
    let c = cases();
    assert_eq!(
        filled_quadrilateral_surface_cover(&c[2]),
        Some(FilledQuadrilateralSurfaceCover::Quadrilateral {
            admissible_diagonals: [true, false]
        })
    );
    assert_eq!(
        filled_quadrilateral_surface_cover(&c[3]),
        Some(FilledQuadrilateralSurfaceCover::Quadrilateral {
            admissible_diagonals: [false, true]
        })
    );
}
