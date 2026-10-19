use super::*;
use crate::geometry_kernel::hatch::{
    HatchAreaRule, HatchBoundary2, HatchDash, HatchEdge2, HatchFill, HatchGradient,
    HatchGradientColorMode, HatchGradientType, HatchLineFamily, HatchLinePattern,
    HatchLoopProvenance,
};
record!(IfccadHatch;
0 => gradient: Option<IfccadHatchGradient> = Some(None),
1 => placement: IfccadPlacement = None,
2 => loops: Vec<IfccadHatchLoop> = None,
3 => area_rule: HatchAreaRule = None,
4 => join_tolerance: f64 = None,
5 => fill: HatchFill = None,
6 => association_enabled: bool = Some(false),
);
unit_enum!(HatchAreaRule;
0 => Normal,
1 => Outer,
2 => Ignore,
);
impl CborValue for HatchFill {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Solid"), (1, "LinePattern"), (2, "Gradient")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Solid => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::LinePattern(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<HatchLinePattern>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<HatchLinePattern>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Gradient => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Solid,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: HatchLinePattern = b.take(0, None)?;
                b.finish()?;
                Self::LinePattern(value_0)
            }
            2 => Self::Gradient,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Solid, Self::Solid) => true,
            (Self::LinePattern(value_0), Self::LinePattern(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Gradient, Self::Gradient) => true,
            _ => false,
        }
    }
}
record!(HatchGradient<IfccadColor>;
0 => gradient_type: Option<HatchGradientType> = Some(None),
1 => colors: Option<[IfccadColor;2]> = Some(None),
2 => rotation: f64 = None,
3 => shift: f64 = None,
4 => color_mode: HatchGradientColorMode = None,
5 => tint: f64 = None,
);
record!(IfccadHatchLoop;
0 => boundary: HatchBoundary2 = None,
2 => source_entity_ids: Vec<u64> = Some(Vec::new()),
3 => provenance: Option<HatchLoopProvenance> = Some(None),
4 => text_island: bool = Some(false),
);
record!(HatchLinePattern;
0 => name: Option<String> = Some(None),
1 => description: Option<String> = Some(None),
2 => origin: [f64;2] = Some([0.0, 0.0]),
3 => rotation: f64 = Some(0.0),
4 => scale: f64 = Some(1.0),
5 => families: Vec<HatchLineFamily> = None,
);
unit_enum!(HatchGradientColorMode;
0 => OneColor,
1 => TwoColor,
);
impl CborValue for HatchBoundary2 {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Polyline"), (1, "Circle"), (2, "Ellipse"), (3, "Edges")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Polyline { vertices, bulges } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if vertices.present()
                    && (None as Option<Vec<[f64; 2]>>)
                        .as_ref()
                        .is_none_or(|d| !vertices.exact(d))
                {
                    __cbor_fields += 1;
                }
                if bulges.present()
                    && (None as Option<Vec<f64>>)
                        .as_ref()
                        .is_none_or(|d| !bulges.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if vertices.present()
                    && (None as Option<Vec<[f64; 2]>>)
                        .as_ref()
                        .is_none_or(|d| !vertices.exact(d))
                {
                    w.u64(0)?;
                    vertices.encode(w)?;
                }
                if bulges.present()
                    && (None as Option<Vec<f64>>)
                        .as_ref()
                        .is_none_or(|d| !bulges.exact(d))
                {
                    w.u64(1)?;
                    bulges.encode(w)?;
                }
                Ok(())
            }
            Self::Circle { center, radius } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    __cbor_fields += 1;
                }
                if radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !radius.exact(d))
                {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                Ok(())
            }
            Self::Ellipse {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    __cbor_fields += 1;
                }
                if x_axis.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !x_axis.exact(d))
                {
                    __cbor_fields += 1;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if x_axis.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !x_axis.exact(d))
                {
                    w.u64(1)?;
                    x_axis.encode(w)?;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    w.u64(2)?;
                    semi_major_radius.encode(w)?;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
                    w.u64(3)?;
                    semi_minor_radius.encode(w)?;
                }
                Ok(())
            }
            Self::Edges(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<Vec<HatchEdge2>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<Vec<HatchEdge2>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "vertices"), (1, "bulges")])?;
                let vertices: Vec<[f64; 2]> = b.take(0, None)?;
                let bulges: Vec<f64> = b.take(1, None)?;
                b.finish()?;
                Self::Polyline { vertices, bulges }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "center"), (1, "radius")])?;
                let center: [f64; 2] = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                b.finish()?;
                Self::Circle { center, radius }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "x_axis"),
                        (2, "semi_major_radius"),
                        (3, "semi_minor_radius"),
                    ],
                )?;
                let center: [f64; 2] = b.take(0, None)?;
                let x_axis: [f64; 2] = b.take(1, None)?;
                let semi_major_radius: f64 = b.take(2, None)?;
                let semi_minor_radius: f64 = b.take(3, None)?;
                b.finish()?;
                Self::Ellipse {
                    center,
                    x_axis,
                    semi_major_radius,
                    semi_minor_radius,
                }
            }
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Vec<HatchEdge2> = b.take(0, None)?;
                b.finish()?;
                Self::Edges(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Polyline { vertices, bulges },
                Self::Polyline {
                    vertices: vertices_other,
                    bulges: bulges_other,
                },
            ) => vertices.exact(vertices_other) && bulges.exact(bulges_other),
            (
                Self::Circle { center, radius },
                Self::Circle {
                    center: center_other,
                    radius: radius_other,
                },
            ) => center.exact(center_other) && radius.exact(radius_other),
            (
                Self::Ellipse {
                    center,
                    x_axis,
                    semi_major_radius,
                    semi_minor_radius,
                },
                Self::Ellipse {
                    center: center_other,
                    x_axis: x_axis_other,
                    semi_major_radius: semi_major_radius_other,
                    semi_minor_radius: semi_minor_radius_other,
                },
            ) => {
                center.exact(center_other)
                    && x_axis.exact(x_axis_other)
                    && semi_major_radius.exact(semi_major_radius_other)
                    && semi_minor_radius.exact(semi_minor_radius_other)
            }
            (Self::Edges(value_0), Self::Edges(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
unit_enum!(HatchGradientType;
0 => Linear,
1 => Cylinder,
2 => InverseCylinder,
3 => Spherical,
4 => InverseSpherical,
5 => Hemispherical,
6 => InverseHemispherical,
7 => Curved,
8 => InverseCurved,
);
record!(HatchLineFamily;
0 => angle: f64 = None,
1 => base_point: [f64;2] = None,
2 => offset: [f64;2] = None,
3 => dashes: Vec<HatchDash> = None,
);
impl CborValue for HatchEdge2 {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Line"), (1, "CircularArc"), (2, "EllipticArc")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Line { start, end } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if start.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !start.exact(d))
                {
                    __cbor_fields += 1;
                }
                if end.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !end.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if start.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !start.exact(d))
                {
                    w.u64(0)?;
                    start.encode(w)?;
                }
                if end.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !end.exact(d))
                {
                    w.u64(1)?;
                    end.encode(w)?;
                }
                Ok(())
            }
            Self::CircularArc {
                center,
                radius,
                start_parameter,
                sweep_parameter,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    __cbor_fields += 1;
                }
                if radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                if start_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !start_parameter.exact(d))
                {
                    __cbor_fields += 1;
                }
                if sweep_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !sweep_parameter.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !radius.exact(d))
                {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                if start_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !start_parameter.exact(d))
                {
                    w.u64(2)?;
                    start_parameter.encode(w)?;
                }
                if sweep_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !sweep_parameter.exact(d))
                {
                    w.u64(3)?;
                    sweep_parameter.encode(w)?;
                }
                Ok(())
            }
            Self::EllipticArc {
                center,
                x_axis,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    __cbor_fields += 1;
                }
                if x_axis.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !x_axis.exact(d))
                {
                    __cbor_fields += 1;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
                    __cbor_fields += 1;
                }
                if start_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !start_parameter.exact(d))
                {
                    __cbor_fields += 1;
                }
                if sweep_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !sweep_parameter.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if center.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !center.exact(d))
                {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if x_axis.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !x_axis.exact(d))
                {
                    w.u64(1)?;
                    x_axis.encode(w)?;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    w.u64(2)?;
                    semi_major_radius.encode(w)?;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
                    w.u64(3)?;
                    semi_minor_radius.encode(w)?;
                }
                if start_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !start_parameter.exact(d))
                {
                    w.u64(4)?;
                    start_parameter.encode(w)?;
                }
                if sweep_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !sweep_parameter.exact(d))
                {
                    w.u64(5)?;
                    sweep_parameter.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "start"), (1, "end")])?;
                let start: [f64; 2] = b.take(0, None)?;
                let end: [f64; 2] = b.take(1, None)?;
                b.finish()?;
                Self::Line { start, end }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "radius"),
                        (2, "start_parameter"),
                        (3, "sweep_parameter"),
                    ],
                )?;
                let center: [f64; 2] = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let start_parameter: f64 = b.take(2, None)?;
                let sweep_parameter: f64 = b.take(3, None)?;
                b.finish()?;
                Self::CircularArc {
                    center,
                    radius,
                    start_parameter,
                    sweep_parameter,
                }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "x_axis"),
                        (2, "semi_major_radius"),
                        (3, "semi_minor_radius"),
                        (4, "start_parameter"),
                        (5, "sweep_parameter"),
                    ],
                )?;
                let center: [f64; 2] = b.take(0, None)?;
                let x_axis: [f64; 2] = b.take(1, None)?;
                let semi_major_radius: f64 = b.take(2, None)?;
                let semi_minor_radius: f64 = b.take(3, None)?;
                let start_parameter: f64 = b.take(4, None)?;
                let sweep_parameter: f64 = b.take(5, None)?;
                b.finish()?;
                Self::EllipticArc {
                    center,
                    x_axis,
                    semi_major_radius,
                    semi_minor_radius,
                    start_parameter,
                    sweep_parameter,
                }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Line { start, end },
                Self::Line {
                    start: start_other,
                    end: end_other,
                },
            ) => start.exact(start_other) && end.exact(end_other),
            (
                Self::CircularArc {
                    center,
                    radius,
                    start_parameter,
                    sweep_parameter,
                },
                Self::CircularArc {
                    center: center_other,
                    radius: radius_other,
                    start_parameter: start_parameter_other,
                    sweep_parameter: sweep_parameter_other,
                },
            ) => {
                center.exact(center_other)
                    && radius.exact(radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep_parameter.exact(sweep_parameter_other)
            }
            (
                Self::EllipticArc {
                    center,
                    x_axis,
                    semi_major_radius,
                    semi_minor_radius,
                    start_parameter,
                    sweep_parameter,
                },
                Self::EllipticArc {
                    center: center_other,
                    x_axis: x_axis_other,
                    semi_major_radius: semi_major_radius_other,
                    semi_minor_radius: semi_minor_radius_other,
                    start_parameter: start_parameter_other,
                    sweep_parameter: sweep_parameter_other,
                },
            ) => {
                center.exact(center_other)
                    && x_axis.exact(x_axis_other)
                    && semi_major_radius.exact(semi_major_radius_other)
                    && semi_minor_radius.exact(semi_minor_radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep_parameter.exact(sweep_parameter_other)
            }
            _ => false,
        }
    }
}
impl CborValue for HatchDash {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Dash"), (1, "Gap"), (2, "Dot")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Dash { length } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    w.u64(0)?;
                    length.encode(w)?;
                }
                Ok(())
            }
            Self::Gap { length } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    w.u64(0)?;
                    length.encode(w)?;
                }
                Ok(())
            }
            Self::Dot => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "length")])?;
                let length: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Dash { length }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "length")])?;
                let length: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Gap { length }
            }
            2 => Self::Dot,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Dash { length },
                Self::Dash {
                    length: length_other,
                },
            ) => length.exact(length_other),
            (
                Self::Gap { length },
                Self::Gap {
                    length: length_other,
                },
            ) => length.exact(length_other),
            (Self::Dot, Self::Dot) => true,
            _ => false,
        }
    }
}

record!(HatchLoopProvenance;
0 => from_source_objects: bool = Some(false),
1 => derived_from_pick_point: bool = Some(false),
2 => outermost_hint: bool = Some(false),
3 => text_box: bool = Some(false),
);

#[cfg(test)]
mod current_main_tests {
    use super::*;

    #[test]
    fn provenance_wire_codes_default_false_and_keep_independent_flags() {
        for (code, expected) in [
            (
                0,
                HatchLoopProvenance {
                    from_source_objects: true,
                    ..Default::default()
                },
            ),
            (
                1,
                HatchLoopProvenance {
                    derived_from_pick_point: true,
                    ..Default::default()
                },
            ),
            (
                2,
                HatchLoopProvenance {
                    outermost_hint: true,
                    ..Default::default()
                },
            ),
            (
                3,
                HatchLoopProvenance {
                    text_box: true,
                    ..Default::default()
                },
            ),
        ] {
            let hand_authored = [0xa1, code, 0xf5];
            let decoded = HatchLoopProvenance::decode(&mut CborReader::new(
                &hand_authored,
                0,
                "/provenance".into(),
            ))
            .unwrap();
            assert_eq!(decoded, expected);
            let mut w = CborWriter::new();
            decoded.encode(&mut w).unwrap();
            assert_eq!(w.finish().unwrap().bytes, hand_authored);
        }
    }

    #[test]
    fn retired_single_source_code_rejects_instead_of_becoming_default() {
        // Field 1 carried the old scalar source reference in revision 1.
        let hand_authored = [0xa1, 1, 0];
        let err = IfccadHatchLoop::decode(&mut CborReader::new(&hand_authored, 0, "/loop".into()))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }
}
