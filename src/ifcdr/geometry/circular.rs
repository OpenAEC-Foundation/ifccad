use super::PlanePlacementComponents;
use crate::ifcdr::{Bounds3d, Point3};

/// Conservative scope-axis bounds of a circle or directed circular arc.
pub(crate) fn circular_bounds(
    placement: PlanePlacementComponents,
    radius: f64,
    arc: Option<(f64, f64)>,
) -> Option<Bounds3d> {
    elliptic_bounds(placement, radius, radius, arc)
}

pub(crate) fn elliptic_bounds(
    placement: PlanePlacementComponents,
    major_radius: f64,
    minor_radius: f64,
    arc: Option<(f64, f64)>,
) -> Option<Bounds3d> {
    placement.validate().ok()?;
    if !major_radius.is_finite()
        || !minor_radius.is_finite()
        || minor_radius <= 0.0
        || major_radius < minor_radius
    {
        return None;
    }
    let (start, sweep) = match arc {
        Some((start, sweep))
            if start.is_finite()
                && sweep.is_finite()
                && sweep != 0.0
                && sweep.abs() < std::f64::consts::TAU =>
        {
            (start.rem_euclid(std::f64::consts::TAU), Some(sweep))
        }
        Some(_) => return None,
        None => (0.0, None),
    };
    let origin = placement.origin.components();
    let x = placement.x.components();
    let y = placement.y.components();
    let mut lo = [0.0; 3];
    let mut hi = [0.0; 3];
    for i in 0..3 {
        let ax = major_radius * x[i];
        let by = minor_radius * y[i];
        let amplitude = ax.hypot(by);
        if !amplitude.is_finite() {
            return None;
        }
        if x[i] == 0.0 && y[i] == 0.0 {
            lo[i] = origin[i];
            hi[i] = origin[i];
            continue;
        }
        let (minimum, maximum) = if let Some(sweep) = sweep {
            let end = start + sweep;
            let mut values = vec![
                coordinate(origin[i], ax, by, start)?,
                coordinate(origin[i], ax, by, end)?,
            ];
            let critical = by.atan2(ax);
            for angle in [critical, critical + std::f64::consts::PI] {
                for turn in -2..=2 {
                    let candidate = angle + (turn as f64) * std::f64::consts::TAU;
                    let inside = if sweep > 0.0 {
                        candidate >= start - 1e-14 && candidate <= end + 1e-14
                    } else {
                        candidate <= start + 1e-14 && candidate >= end - 1e-14
                    };
                    if inside {
                        values.push(coordinate(origin[i], ax, by, candidate)?);
                    }
                }
            }
            (
                values.iter().copied().fold(f64::INFINITY, f64::min),
                values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        } else {
            (origin[i] - amplitude, origin[i] + amplitude)
        };
        if !minimum.is_finite() || !maximum.is_finite() {
            return None;
        }
        lo[i] = minimum.next_down().next_down().next_down().next_down();
        hi[i] = maximum.next_up().next_up().next_up().next_up();
        if !lo[i].is_finite() || !hi[i].is_finite() {
            return None;
        }
    }
    Some(Bounds3d {
        min: Point3::new(lo[0], lo[1], lo[2]),
        max: Point3::new(hi[0], hi[1], hi[2]),
    })
}

fn coordinate(origin: f64, ax: f64, by: f64, angle: f64) -> Option<f64> {
    let result = origin + ax * angle.cos() + by * angle.sin();
    result.is_finite().then_some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifcdr::{PlanePlacement, Vector3};

    #[test]
    fn arc_bounds_include_interior_extremum_and_respect_direction() {
        let frame = PlanePlacement::default().components();
        let positive = circular_bounds(frame, 2.0, Some((0.1, 3.0))).unwrap();
        assert!(positive.max.y() >= 2.0);
        let negative = circular_bounds(frame, 2.0, Some((0.1, -3.0))).unwrap();
        assert!(negative.min.y() <= -2.0);
        assert!(circular_bounds(frame, 1.0, Some((0.0, 0.0))).is_none());
        assert!(circular_bounds(frame, 1.0, Some((0.0, std::f64::consts::TAU))).is_none());
        assert!(circular_bounds(frame, 1.0, Some((0.0, f64::NAN))).is_none());
        assert!(circular_bounds(frame, 1.0, Some((f64::INFINITY, 1.0))).is_none());
        assert!(
            circular_bounds(frame, 1.0, Some((0.0, std::f64::consts::TAU.next_down()))).is_some()
        );
    }

    #[test]
    fn circle_bounds_include_oblique_axis_projection() {
        let frame = PlanePlacement::try_new(
            Point3::new(5.0, 6.0, 7.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap()
        .components();
        let bounds = circular_bounds(frame, 3.0, None).unwrap();
        assert!(bounds.min.x() <= 2.0 && bounds.max.x() >= 8.0);
        assert!(bounds.min.z() <= 4.0 && bounds.max.z() >= 10.0);
    }

    #[test]
    fn ellipse_keeps_equal_axes_and_rejects_inverted_axes() {
        let frame = PlanePlacement::default().components();
        assert!(elliptic_bounds(frame, 2.0, 2.0, None).is_some());
        assert!(elliptic_bounds(frame, 1.0, 2.0, None).is_none());
        assert!(elliptic_bounds(frame, 2.0, 0.0, None).is_none());
        assert!(elliptic_bounds(
            frame,
            2.0,
            1.0,
            Some((0.0, std::f64::consts::TAU.next_down()))
        )
        .is_some());
    }
}
