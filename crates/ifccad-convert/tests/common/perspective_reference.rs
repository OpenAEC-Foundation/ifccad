use opencadcodec::{entities::Viewport, CadDocument, EntityType};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerspectiveReferenceCase {
    pub name: String,
    pub center: [f64; 2],
    pub width: f64,
    pub height: f64,
    pub target: [f64; 3],
    pub direction: [f64; 3],
    pub view_center: [f64; 2],
    pub view_height: f64,
    pub twist: f64,
    pub lens: f64,
    pub front: Option<f64>,
    pub back: Option<f64>,
    pub landmarks: Vec<Landmark>,
}
#[derive(Deserialize)]
pub struct Landmark {
    pub model: [f64; 3],
    pub paper: [f64; 2],
    pub visible: bool,
}
pub fn load_perspective_reference_cases() -> Vec<PerspectiveReferenceCase> {
    serde_json::from_str(include_str!(
        "../fixtures/viewports/reference-perspective.json"
    ))
    .unwrap()
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn components(v: opencadcodec::Vector3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

// Test-only calculation derived from the documented Autodesk WCS/DCS matrix.
// Expected landmarks are independently specified constants, never codec output.
pub fn assert_reference_camera(case: &PerspectiveReferenceCase, v: &Viewport) {
    let tolerance = 1e-10; // external calculation comparison, not conversion tolerance
    assert_eq!(components(v.view_target), case.target);
    assert_eq!(components(v.view_direction), case.direction);
    assert_eq!([v.view_center.x, v.view_center.y], case.view_center);
    assert_eq!([v.center.x, v.center.y], case.center);
    assert_eq!(
        [v.width, v.height, v.view_height, v.lens_length],
        [case.width, case.height, case.view_height, case.lens]
    );
    assert!((v.twist_angle - case.twist).abs() < tolerance);
    assert!(v.status.perspective);
    assert_eq!(v.status.front_clipping, case.front.is_some());
    assert_eq!(v.status.back_clipping, case.back.is_some());
    if let Some(front) = case.front {
        assert_eq!(v.front_clip_z, front);
        assert!(v.status.front_clip_not_at_eye);
    }
    if let Some(back) = case.back {
        assert_eq!(v.back_clip_z, back);
    }
    let direction = components(v.view_direction);
    let distance = dot(direction, direction).sqrt();
    let z = direction.map(|x| x / distance);
    let planar = direction[0].hypot(direction[1]);
    let (x, y) = if planar == 0. {
        ([z[2].signum(), 0., 0.], [0., 1., 0.])
    } else {
        let x = [-direction[1] / planar, direction[0] / planar, 0.];
        (x, [-z[2] * x[1], z[2] * x[0], z[0] * x[1] - z[1] * x[0]])
    };
    let (s, c) = v.twist_angle.sin_cos();
    let x: [f64; 3] = std::array::from_fn(|i| c * x[i] - s * y[i]);
    // Derive rotated y directly from z cross rotated x.
    let y = [
        z[1] * x[2] - z[2] * x[1],
        z[2] * x[0] - z[0] * x[2],
        z[0] * x[1] - z[1] * x[0],
    ];
    let focal = v.height * v.lens_length * (1. + (v.width / v.height).powi(2)).sqrt() / 42.;
    for landmark in &case.landmarks {
        let delta = std::array::from_fn(|i| landmark.model[i] - case.target[i]);
        let depth = dot(delta, z);
        let paper = [
            v.center.x + focal * (dot(delta, x) - v.view_center.x) / (distance - depth),
            v.center.y + focal * (dot(delta, y) - v.view_center.y) / (distance - depth),
        ];
        for (actual, expected) in paper.into_iter().zip(landmark.paper) {
            assert!(
                (actual - expected).abs() < tolerance,
                "{}: {actual} != {expected}",
                case.name
            );
        }
        let visible = depth < distance
            && case.front.is_none_or(|d| depth <= d)
            && case.back.is_none_or(|d| depth >= d);
        assert_eq!(visible, landmark.visible);
    }
}
pub fn assert_reference_view(case: &PerspectiveReferenceCase, document: &CadDocument) {
    let viewport = document
        .entities()
        .find_map(|e| match e {
            EntityType::Viewport(v) if v.width == case.width => Some(v),
            _ => None,
        })
        .unwrap();
    assert_reference_camera(case, viewport);
}
