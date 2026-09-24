use crate::ifcdr::{Bounds2d, Point2};

/// Conservative local XY enclosure of the circular segment defined by a CAD bulge.
pub(crate) fn bulge_segment_bounds(start: Point2, end: Point2, bulge: f64) -> Option<Bounds2d> {
    let [x0, y0, x1, y1] = [start.x(), start.y(), end.x(), end.y()];
    if ![x0, y0, x1, y1, bulge].into_iter().all(f64::is_finite) {
        return None;
    }
    if bulge != 0.0 && start == end {
        return None;
    }
    let (min_x, max_x, min_y, max_y) = if bulge == 0.0 {
        (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1))
    } else if bulge.abs() <= 1.0 {
        // A minor circular arc stays within the endpoint chord plus its sagitta.
        let ex = (bulge * (y1 - y0)).abs() * 0.5;
        let ey = (bulge * (x1 - x0)).abs() * 0.5;
        (
            x0.min(x1) - ex,
            x0.max(x1) + ex,
            y0.min(y1) - ey,
            y0.max(y1) + ey,
        )
    } else {
        // Major arcs may pass around the remote side of the circle.
        let dx = x1 - x0;
        let dy = y1 - y0;
        let factor = (1.0 / bulge - bulge) * 0.25;
        let cx = (x0 + x1) * 0.5 - dy * factor;
        let cy = (y0 + y1) * 0.5 + dx * factor;
        let radius = (x0 - cx).hypot(y0 - cy);
        (cx - radius, cx + radius, cy - radius, cy + radius)
    };
    if ![min_x, max_x, min_y, max_y].into_iter().all(f64::is_finite) {
        return None;
    }
    let expand = bulge != 0.0;
    let outward = |low: f64, high: f64| {
        if expand {
            (low.next_down().next_down(), high.next_up().next_up())
        } else {
            (low, high)
        }
    };
    let (min_x, max_x) = outward(min_x, max_x);
    let (min_y, max_y) = outward(min_y, max_y);
    if ![min_x, max_x, min_y, max_y].into_iter().all(f64::is_finite) {
        return None;
    }
    Some(Bounds2d {
        min: Point2::new(min_x, min_y),
        max: Point2::new(max_x, max_y),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bulge_between_horizontal_endpoints_extends_vertical_bounds() {
        let start = Point2::new(0.0, 0.0);
        let end = Point2::new(2.0, 0.0);
        let bounds = bulge_segment_bounds(start, end, 1.0).unwrap();
        assert!(bounds.min().y() < 0.0 && bounds.max().y() > 0.0);
        assert!(bulge_segment_bounds(start, start, 1.0).is_none());
        assert!(bulge_segment_bounds(start, start, 0.0).is_some());
    }
}
