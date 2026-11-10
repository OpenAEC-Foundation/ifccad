//! Exact homogeneous Bézier spans for spline boundary certificates.
use super::{
    numeric::exact, Bounds3d, GeometryValidationError as Error, SplineControl, SplineKnots,
};
use num_rational::BigRational as Q;
use num_traits::{One, Zero};

#[derive(Clone)]
pub(super) struct Homogeneous<const D: usize> {
    coords: [Q; D],
    weight: Q,
}

impl<const D: usize> Homogeneous<D> {
    fn blend(&self, other: &Self, alpha: &Q) -> Self {
        let beta = Q::one() - alpha;
        Self {
            coords: std::array::from_fn(|i| &beta * &self.coords[i] + alpha * &other.coords[i]),
            weight: &beta * &self.weight + alpha * &other.weight,
        }
    }

    pub(super) fn point(&self) -> [Q; D] {
        self.coords.each_ref().map(|v| v / &self.weight)
    }
}

pub(super) fn spans<const D: usize>(
    control: &SplineControl<D>,
) -> Result<Vec<Vec<Homogeneous<D>>>, Error> {
    control.prepare().map_err(|reason| match reason {
        super::SplineError::EvaluationUnavailable => Error::BoundaryProofIncomplete,
        _ => Error::InvalidGeometry,
    })?;
    let degree = control.degree;
    let count = control.points.len();
    let order = degree + 1;
    if count.saturating_mul(order.saturating_pow(3)) > 1_000_000 {
        return Err(Error::BoundaryProofIncomplete);
    }
    let (knots, end) = match &control.knots {
        SplineKnots::Full(knots) => (knots.iter().copied().map(exact).collect::<Vec<_>>(), count),
        SplineKnots::PeriodicCycle(knots) => (periodic_knots(knots, degree), count + degree),
    };
    let controls: Vec<_> = control
        .points
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let weight = control
                .weights
                .as_ref()
                .map_or_else(Q::one, |w| exact(w[i]));
            Homogeneous {
                coords: point.map(|v| exact(v) * &weight),
                weight,
            }
        })
        .collect();
    let mut result = Vec::new();
    for span in degree..end {
        let (lo, hi) = (&knots[span], &knots[span + 1]);
        if lo == hi {
            continue;
        }
        let first = span - degree;
        let mut bezier = Vec::with_capacity(order);
        for i in 0..order {
            // A Bézier pole is the spline blossom at p-i copies of lo and i
            // copies of hi, evaluated on the same nonempty knot span.
            let mut work: Vec<_> = (first..=span)
                .map(|j| controls[j % count].clone())
                .collect();
            for level in 1..=degree {
                let parameter = if level <= degree - i { lo } else { hi };
                for j in (level..=degree).rev() {
                    let k = first + j;
                    let left = &knots[k];
                    let right = &knots[k + degree - level + 1];
                    let alpha = (parameter - left) / (right - left);
                    work[j] = work[j - 1].blend(&work[j], &alpha);
                }
            }
            bezier.push(work.pop().unwrap());
        }
        result.push(bezier);
    }
    Ok(result)
}

pub(super) fn periodic_knots(knots: &[f64], degree: usize) -> Vec<Q> {
    let base: Vec<_> = knots.iter().copied().map(exact).collect();
    let count = base.len() - 1;
    let period = &base[count] - &base[0];
    let mut expanded = Vec::with_capacity(count + 2 * degree + 1);
    expanded.extend(base[count - degree..count].iter().map(|k| k - &period));
    expanded.extend(base.iter().cloned());
    expanded.extend(base[1..=degree].iter().map(|k| k + &period));
    expanded
}

pub(super) fn continuous_endpoints<const D: usize>(
    control: &SplineControl<D>,
) -> Result<[[Q; D]; 2], Error> {
    let spans = spans(control)?;
    if spans
        .windows(2)
        .any(|pair| pair[0].last().unwrap().point() != pair[1][0].point())
    {
        return Err(Error::UnsupportedBoundary);
    }
    Ok([
        spans[0][0].point(),
        spans.last().unwrap().last().unwrap().point(),
    ])
}

fn split_half<const D: usize>(
    mut poles: Vec<Homogeneous<D>>,
) -> (Vec<Homogeneous<D>>, Vec<Homogeneous<D>>) {
    let half = exact(0.5);
    let mut left = Vec::with_capacity(poles.len());
    let mut right = Vec::with_capacity(poles.len());
    while poles.len() > 1 {
        left.push(poles[0].clone());
        right.push(poles.last().unwrap().clone());
        for i in 0..poles.len() - 1 {
            poles[i] = poles[i].blend(&poles[i + 1], &half);
        }
        poles.pop();
    }
    left.push(poles[0].clone());
    right.push(poles.pop().unwrap());
    right.reverse();
    (left, right)
}

pub(super) fn validate_clip(control: &SplineControl<3>, bounds: Bounds3d) -> Result<(), Error> {
    let spans = spans(control)?;
    let start = spans[0][0].point();
    let end = |span: &[Homogeneous<3>]| span.last().unwrap().point();
    if end(spans.last().unwrap()) != start
        || spans
            .windows(2)
            .any(|pair| end(&pair[0]) != pair[1][0].point())
        || spans.iter().flatten().all(|p| p.point() == start)
    {
        return Err(Error::UnsupportedBoundary);
    }
    if spans.iter().flatten().any(|p| !p.coords[2].is_zero()) {
        return Err(Error::BoundaryOutsideFrame);
    }
    let lo = bounds.min().components().map(exact);
    let hi = bounds.max().components().map(exact);
    let fits = |p: &Homogeneous<3>| {
        (0..2).all(|axis| {
            p.coords[axis] >= &lo[axis] * &p.weight && p.coords[axis] <= &hi[axis] * &p.weight
        })
    };
    let mut splits_left = 4096;
    let mut incomplete = false;
    for span in spans {
        let mut pending = vec![(span, 0)];
        while let Some((poles, depth)) = pending.pop() {
            if !fits(&poles[0]) || !fits(poles.last().unwrap()) {
                return Err(Error::BoundaryOutsideFrame);
            }
            if poles.iter().all(&fits) {
                continue;
            }
            if depth == 32 || splits_left == 0 {
                incomplete = true;
                continue;
            }
            splits_left -= 1;
            let (a, b) = split_half(poles);
            pending.push((b, depth + 1));
            pending.push((a, depth + 1));
        }
    }
    if incomplete {
        Err(Error::BoundaryProofIncomplete)
    } else {
        Ok(())
    }
}
