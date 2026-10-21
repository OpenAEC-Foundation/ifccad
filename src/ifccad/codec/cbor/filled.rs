use super::*;
record!(IfccadFilledQuadrilateral;
    0 => placement: IfccadPlacement = None,
    1 => corners: [[f64;2];4] = None,
    2 => thickness: f64 = Some(0.),
);
