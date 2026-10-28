use ifccad::dimension::{evaluate_dimension, DimensionDefinition, DimensionMeasurement};
use serde_json::{json, Value};

fn distance(value: Value, expected: f64) {
    let definition: DimensionDefinition = serde_json::from_value(value).unwrap();
    let DimensionMeasurement::Distance(actual) = evaluate_dimension(&definition).unwrap() else {
        panic!("expected a distance")
    };
    assert_eq!(actual, expected);
}

#[test]
fn radius_uses_projected_distance_but_retains_xyz() {
    let source =
        json!({"kind":"radius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"leaderLength":0.});
    distance(source.clone(), 5.);
    let definition: DimensionDefinition = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(serde_json::to_value(definition).unwrap(), source);
}

#[test]
fn diameter_uses_opposite_projected_points() {
    distance(
        json!({"kind":"diameter","firstDiameterPoint":[-3.,-4.,7.],"secondDiameterPoint":[3.,4.,9.],"leaderLength":0.}),
        10.,
    );
}

#[test]
fn ordinate_uses_stored_axis_and_mode() {
    let mut source = json!({"kind":"ordinate","origin":[10.,20.,3.],"featurePoint":[7.,24.,8.],"axisAngle":0.,"leaderEnd":[20.,40.],"valueMode":"signed"});
    distance(source.clone(), -3.);
    source["valueMode"] = json!("absolute");
    distance(source, 3.);
}

#[test]
fn jogged_radius_uses_true_center() {
    distance(
        json!({"kind":"joggedRadius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"displayCenter":[100.,200.],"jogPoint":[50.,50.],"jogAngle":0.5}),
        5.,
    );
}

#[test]
fn angular_three_points_preserves_selected_major_sector() {
    let source = json!({"kind":"angularThreePoints","vertex":[0.,0.,7.],"firstPoint":[1.,0.,8.],"secondPoint":[0.,1.,9.],"traversal":"clockwise","dimensionArcPoint":[5.,5.]});
    let definition: DimensionDefinition = serde_json::from_value(source).unwrap();
    let DimensionMeasurement::Angle(actual) = evaluate_dimension(&definition).unwrap() else {
        panic!("expected an angle")
    };
    assert!((actual - 1.5 * std::f64::consts::PI).abs() <= 2. * f64::EPSILON * actual.abs());
}

#[test]
fn angular_two_lines_preserves_selected_rays() {
    let source = json!({"kind":"angularTwoLines","line1Start":[0.,0.,7.],"line1End":[1.,0.,8.],"line2Start":[0.,0.,9.],"line2End":[0.,1.,10.],"ray1Sign":"positive","ray2Sign":"positive","traversal":"counterclockwise","dimensionArcPoint":[5.,5.]});
    let definition: DimensionDefinition = serde_json::from_value(source).unwrap();
    let DimensionMeasurement::Angle(actual) = evaluate_dimension(&definition).unwrap() else {
        panic!("expected an angle")
    };
    assert!((actual - 0.5 * std::f64::consts::PI).abs() <= 2. * f64::EPSILON * actual.abs());
}

#[test]
fn arc_length_preserves_explicit_major_and_full_spans() {
    for (span, expected) in [
        (
            json!({"kind":"partial","radians":1.5 * std::f64::consts::PI}),
            150. * std::f64::consts::PI,
        ),
        (
            json!({"kind":"full","traversal":"clockwise"}),
            200. * std::f64::consts::PI,
        ),
    ] {
        let source = json!({"kind":"arcLength","center":[0.,0.,7.],"radius":100.,"startParameter":0.,"sweep":span,"dimensionArcPoint":[120.,0.]});
        let definition: DimensionDefinition = serde_json::from_value(source).unwrap();
        let DimensionMeasurement::Distance(actual) = evaluate_dimension(&definition).unwrap()
        else {
            panic!("expected distance")
        };
        assert!((actual - expected).abs() <= 2. * f64::EPSILON * expected.abs());
    }
}

#[test]
fn two_line_arc_position_cannot_be_the_intersection() {
    let source = json!({"kind":"angularTwoLines","line1Start":[0.,0.,0.],"line1End":[1.,0.,0.],"line2Start":[0.,0.,0.],"line2End":[0.,1.,0.],"ray1Sign":"positive","ray2Sign":"positive","traversal":"counterclockwise","dimensionArcPoint":[0.,0.]});
    let definition: DimensionDefinition = serde_json::from_value(source).unwrap();
    assert!(evaluate_dimension(&definition).is_err());
}

#[test]
fn positive_arc_measure_that_underflows_is_unavailable_not_zero() {
    let definition:DimensionDefinition=serde_json::from_value(json!({"kind":"arcLength","center":[0.,0.,0.],"radius":f64::from_bits(1),"startParameter":0.,"sweep":{"kind":"partial","radians":0.25},"dimensionArcPoint":[1.,1.]})).unwrap();
    assert!(evaluate_dimension(&definition).is_err());
}
