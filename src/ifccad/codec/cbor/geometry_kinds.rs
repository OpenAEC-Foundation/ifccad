use super::geometry::{decode_line_point, encode_line_point};
use super::*;
impl CborValue for IfccadEntityKind {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "LineSegment"),
            (1, "Point"),
            (2, "Circle"),
            (3, "Arc"),
            (4, "Ellipse"),
            (5, "EllipseArc"),
            (6, "PlanarPolyline"),
            (7, "SpatialPolyline"),
            (8, "BlockInstance"),
            (9, "Viewport"),
            (10, "Text"),
            (11, "MText"),
            (12, "Hatch"),
            (13, "Dimension"),
            (14, "Solid"),
            (15, "Trace"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2)?;
        w.u64(0)?;
        w.u64(self.code())?;
        w.u64(1)?;
        self.encode_body(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code = f.take(0, None)?;
        let mut body = f.span(1)?;
        f.finish()?;
        Self::decode_body(code, &mut body)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::LineSegment {
                    start,
                    end,
                    thickness,
                    extrusion_direction,
                    ..
                },
                Self::LineSegment {
                    start: start_other,
                    end: end_other,
                    thickness: thickness_other,
                    extrusion_direction: extrusion_direction_other,
                    ..
                },
            ) => {
                start.exact(start_other)
                    && end.exact(end_other)
                    && thickness.exact(thickness_other)
                    && extrusion_direction.exact(extrusion_direction_other)
            }
            (
                Self::Point {
                    placement,
                    thickness,
                    ..
                },
                Self::Point {
                    placement: placement_other,
                    thickness: thickness_other,
                    ..
                },
            ) => placement.exact(placement_other) && thickness.exact(thickness_other),
            (
                Self::Circle {
                    placement,
                    radius,
                    thickness,
                    ..
                },
                Self::Circle {
                    placement: placement_other,
                    radius: radius_other,
                    thickness: thickness_other,
                    ..
                },
            ) => {
                placement.exact(placement_other)
                    && radius.exact(radius_other)
                    && thickness.exact(thickness_other)
            }
            (
                Self::Arc {
                    placement,
                    radius,
                    start_parameter,
                    sweep_parameter,
                    thickness,
                    ..
                },
                Self::Arc {
                    placement: placement_other,
                    radius: radius_other,
                    start_parameter: start_parameter_other,
                    sweep_parameter: sweep_parameter_other,
                    thickness: thickness_other,
                    ..
                },
            ) => {
                placement.exact(placement_other)
                    && radius.exact(radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep_parameter.exact(sweep_parameter_other)
                    && thickness.exact(thickness_other)
            }
            (
                Self::Ellipse {
                    placement,
                    semi_major_radius,
                    semi_minor_radius,
                },
                Self::Ellipse {
                    placement: placement_other,
                    semi_major_radius: semi_major_radius_other,
                    semi_minor_radius: semi_minor_radius_other,
                },
            ) => {
                placement.exact(placement_other)
                    && semi_major_radius.exact(semi_major_radius_other)
                    && semi_minor_radius.exact(semi_minor_radius_other)
            }
            (
                Self::EllipseArc {
                    placement,
                    semi_major_radius,
                    semi_minor_radius,
                    start_parameter,
                    sweep_parameter,
                },
                Self::EllipseArc {
                    placement: placement_other,
                    semi_major_radius: semi_major_radius_other,
                    semi_minor_radius: semi_minor_radius_other,
                    start_parameter: start_parameter_other,
                    sweep_parameter: sweep_parameter_other,
                },
            ) => {
                placement.exact(placement_other)
                    && semi_major_radius.exact(semi_major_radius_other)
                    && semi_minor_radius.exact(semi_minor_radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep_parameter.exact(sweep_parameter_other)
            }
            (
                Self::PlanarPolyline {
                    placement,
                    vertices,
                    bulges,
                    closed,
                    line_pattern_generation,
                    widths,
                    thickness,
                    ..
                },
                Self::PlanarPolyline {
                    placement: placement_other,
                    vertices: vertices_other,
                    bulges: bulges_other,
                    closed: closed_other,
                    line_pattern_generation: line_pattern_generation_other,
                    widths: widths_other,
                    thickness: thickness_other,
                    ..
                },
            ) => {
                placement.exact(placement_other)
                    && vertices.exact(vertices_other)
                    && bulges.exact(bulges_other)
                    && closed.exact(closed_other)
                    && line_pattern_generation.exact(line_pattern_generation_other)
                    && widths.exact(widths_other)
                    && thickness.exact(thickness_other)
            }
            (
                Self::SpatialPolyline {
                    vertices,
                    closed,
                    line_pattern_generation,
                },
                Self::SpatialPolyline {
                    vertices: vertices_other,
                    closed: closed_other,
                    line_pattern_generation: line_pattern_generation_other,
                },
            ) => {
                vertices.exact(vertices_other)
                    && closed.exact(closed_other)
                    && line_pattern_generation.exact(line_pattern_generation_other)
            }
            (
                Self::BlockInstance {
                    definition_id,
                    transform,
                },
                Self::BlockInstance {
                    definition_id: definition_id_other,
                    transform: transform_other,
                },
            ) => definition_id.exact(definition_id_other) && transform.exact(transform_other),
            (Self::Viewport(value_0), Self::Viewport(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Text(value_0), Self::Text(value_0_other)) => value_0.exact(value_0_other),
            (Self::MText(value_0), Self::MText(value_0_other)) => value_0.exact(value_0_other),
            (Self::Hatch(value_0), Self::Hatch(value_0_other)) => value_0.exact(value_0_other),
            (Self::Dimension(value_0), Self::Dimension(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Solid(value_0), Self::Solid(value_0_other)) => value_0.exact(value_0_other),
            (Self::Trace(value_0), Self::Trace(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
impl IfccadEntityKind {
    pub(super) fn code(&self) -> u64 {
        match self {
            Self::LineSegment { .. } => 0,
            Self::Point { .. } => 1,
            Self::Circle { .. } => 2,
            Self::Arc { .. } => 3,
            Self::Ellipse { .. } => 4,
            Self::EllipseArc { .. } => 5,
            Self::PlanarPolyline { .. } => 6,
            Self::SpatialPolyline { .. } => 7,
            Self::BlockInstance { .. } => 8,
            Self::Viewport(..) => 9,
            Self::Text(..) => 10,
            Self::MText(..) => 11,
            Self::Hatch(..) => 12,
            Self::Dimension(..) => 13,
            Self::Solid(..) => 14,
            Self::Trace(..) => 15,
        }
    }
    pub(super) fn encode_body(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::LineSegment {
                start,
                end,
                thickness,
                extrusion_direction,
                ..
            } => {
                w.map(
                    u64::from(true)
                        + u64::from(true)
                        + u64::from(!thickness.exact(&(0.0)))
                        + u64::from(!extrusion_direction.exact(&([0., 0., 1.]))),
                )?;
                if true {
                    w.u64(0)?;
                    encode_line_point(start, w)?;
                }
                if true {
                    w.u64(1)?;
                    encode_line_point(end, w)?;
                }
                if !thickness.exact(&(0.0)) {
                    w.u64(2)?;
                    thickness.encode(w)?;
                }
                if !extrusion_direction.exact(&([0., 0., 1.])) {
                    w.u64(3)?;
                    extrusion_direction.encode(w)?;
                }
                Ok(())
            }
            Self::Point {
                placement,
                thickness,
                ..
            } => {
                w.map(u64::from(true) + u64::from(!thickness.exact(&(0.0))))?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if !thickness.exact(&(0.0)) {
                    w.u64(1)?;
                    thickness.encode(w)?;
                }
                Ok(())
            }
            Self::Circle {
                placement,
                radius,
                thickness,
                ..
            } => {
                w.map(u64::from(true) + u64::from(true) + u64::from(!thickness.exact(&(0.0))))?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                if !thickness.exact(&(0.0)) {
                    w.u64(2)?;
                    thickness.encode(w)?;
                }
                Ok(())
            }
            Self::Arc {
                placement,
                radius,
                start_parameter,
                sweep_parameter,
                thickness,
                ..
            } => {
                w.map(
                    u64::from(true)
                        + u64::from(true)
                        + u64::from(true)
                        + u64::from(true)
                        + u64::from(!thickness.exact(&(0.0))),
                )?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                if true {
                    w.u64(2)?;
                    start_parameter.encode(w)?;
                }
                if true {
                    w.u64(3)?;
                    sweep_parameter.encode(w)?;
                }
                if !thickness.exact(&(0.0)) {
                    w.u64(4)?;
                    thickness.encode(w)?;
                }
                Ok(())
            }
            Self::Ellipse {
                placement,
                semi_major_radius,
                semi_minor_radius,
            } => {
                w.map(u64::from(true) + u64::from(true) + u64::from(true))?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    semi_major_radius.encode(w)?;
                }
                if true {
                    w.u64(2)?;
                    semi_minor_radius.encode(w)?;
                }
                Ok(())
            }
            Self::EllipseArc {
                placement,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => {
                w.map(
                    u64::from(true)
                        + u64::from(true)
                        + u64::from(true)
                        + u64::from(true)
                        + u64::from(true),
                )?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    semi_major_radius.encode(w)?;
                }
                if true {
                    w.u64(2)?;
                    semi_minor_radius.encode(w)?;
                }
                if true {
                    w.u64(3)?;
                    start_parameter.encode(w)?;
                }
                if true {
                    w.u64(4)?;
                    sweep_parameter.encode(w)?;
                }
                Ok(())
            }
            Self::PlanarPolyline {
                placement,
                vertices,
                bulges,
                closed,
                line_pattern_generation,
                widths,
                thickness,
                ..
            } => {
                w.map(
                    u64::from(true)
                        + u64::from(true)
                        + u64::from(true)
                        + u64::from(!closed.exact(&(false)))
                        + u64::from(
                            !line_pattern_generation
                                .exact(&(IfccadLinePatternGeneration::PerSegment)),
                        )
                        + u64::from(!widths.exact(&(Default::default())))
                        + u64::from(!thickness.exact(&(0.0))),
                )?;
                if true {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    vertices.encode(w)?;
                }
                if true {
                    w.u64(2)?;
                    bulges.encode(w)?;
                }
                if !closed.exact(&(false)) {
                    w.u64(3)?;
                    closed.encode(w)?;
                }
                if !line_pattern_generation.exact(&(IfccadLinePatternGeneration::PerSegment)) {
                    w.u64(4)?;
                    line_pattern_generation.encode(w)?;
                }
                if !widths.exact(&(Default::default())) {
                    w.u64(5)?;
                    widths.encode(w)?;
                }
                if !thickness.exact(&(0.0)) {
                    w.u64(6)?;
                    thickness.encode(w)?;
                }
                Ok(())
            }
            Self::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            } => {
                w.map(
                    u64::from(true)
                        + u64::from(!closed.exact(&(false)))
                        + u64::from(
                            !line_pattern_generation
                                .exact(&(IfccadLinePatternGeneration::PerSegment)),
                        ),
                )?;
                if true {
                    w.u64(0)?;
                    vertices.encode(w)?;
                }
                if !closed.exact(&(false)) {
                    w.u64(1)?;
                    closed.encode(w)?;
                }
                if !line_pattern_generation.exact(&(IfccadLinePatternGeneration::PerSegment)) {
                    w.u64(2)?;
                    line_pattern_generation.encode(w)?;
                }
                Ok(())
            }
            Self::BlockInstance {
                definition_id,
                transform,
            } => {
                w.map(u64::from(true) + u64::from(true))?;
                if true {
                    w.u64(0)?;
                    definition_id.encode(w)?;
                }
                if true {
                    w.u64(1)?;
                    transform.encode(w)?;
                }
                Ok(())
            }
            Self::Viewport(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Text(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::MText(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Hatch(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Dimension(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Solid(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Trace(value_0) => {
                w.map(u64::from(true))?;
                if true {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    pub(super) fn decode_body(
        code: u64,
        r: &mut CborReader<'_>,
    ) -> Result<Self, IfccadCborReadError> {
        match code {
            0 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "start"),
                        (1, "end"),
                        (2, "thickness"),
                        (3, "extrusion_direction"),
                    ],
                )?;
                let start: [f64; 3] = decode_line_point(&mut b.span(0)?)?;
                let end: [f64; 3] = decode_line_point(&mut b.span(1)?)?;
                let thickness: f64 = b.take(2, Some(0.0))?;
                let extrusion_direction: [f64; 3] = b.take(3, Some([0., 0., 1.]))?;
                b.finish()?;
                Ok(Self::LineSegment {
                    start,
                    end,
                    thickness,
                    extrusion_direction,
                })
            }
            1 => {
                let mut b = RecordFields::new(r, &[(0, "placement"), (1, "thickness")])?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let thickness: f64 = b.take(1, Some(0.0))?;
                b.finish()?;
                Ok(Self::Point {
                    placement,
                    thickness,
                })
            }
            2 => {
                let mut b =
                    RecordFields::new(r, &[(0, "placement"), (1, "radius"), (2, "thickness")])?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let thickness: f64 = b.take(2, Some(0.0))?;
                b.finish()?;
                Ok(Self::Circle {
                    placement,
                    radius,
                    thickness,
                })
            }
            3 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "placement"),
                        (1, "radius"),
                        (2, "start_parameter"),
                        (3, "sweep_parameter"),
                        (4, "thickness"),
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let start_parameter: f64 = b.take(2, None)?;
                let sweep_parameter: f64 = b.take(3, None)?;
                let thickness: f64 = b.take(4, Some(0.0))?;
                b.finish()?;
                Ok(Self::Arc {
                    placement,
                    radius,
                    start_parameter,
                    sweep_parameter,
                    thickness,
                })
            }
            4 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "placement"),
                        (1, "semi_major_radius"),
                        (2, "semi_minor_radius"),
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let semi_major_radius: f64 = b.take(1, None)?;
                let semi_minor_radius: f64 = b.take(2, None)?;
                b.finish()?;
                Ok(Self::Ellipse {
                    placement,
                    semi_major_radius,
                    semi_minor_radius,
                })
            }
            5 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "placement"),
                        (1, "semi_major_radius"),
                        (2, "semi_minor_radius"),
                        (3, "start_parameter"),
                        (4, "sweep_parameter"),
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let semi_major_radius: f64 = b.take(1, None)?;
                let semi_minor_radius: f64 = b.take(2, None)?;
                let start_parameter: f64 = b.take(3, None)?;
                let sweep_parameter: f64 = b.take(4, None)?;
                b.finish()?;
                Ok(Self::EllipseArc {
                    placement,
                    semi_major_radius,
                    semi_minor_radius,
                    start_parameter,
                    sweep_parameter,
                })
            }
            6 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "placement"),
                        (1, "vertices"),
                        (2, "bulges"),
                        (3, "closed"),
                        (4, "line_pattern_generation"),
                        (5, "widths"),
                        (6, "thickness"),
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let vertices: Vec<[f64; 2]> = b.take(1, None)?;
                let bulges: Vec<f64> = b.take(2, None)?;
                let closed: bool = b.take(3, Some(false))?;
                let line_pattern_generation: IfccadLinePatternGeneration =
                    b.take(4, Some(IfccadLinePatternGeneration::PerSegment))?;
                let widths: IfccadPlanarPolylineWidths = b.take(5, Some(Default::default()))?;
                let thickness: f64 = b.take(6, Some(0.0))?;
                b.finish()?;
                Ok(Self::PlanarPolyline {
                    placement,
                    vertices,
                    bulges,
                    closed,
                    line_pattern_generation,
                    widths,
                    thickness,
                })
            }
            7 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "vertices"),
                        (1, "closed"),
                        (2, "line_pattern_generation"),
                    ],
                )?;
                let vertices: Vec<[f64; 3]> = b.take(0, None)?;
                let closed: bool = b.take(1, Some(false))?;
                let line_pattern_generation: IfccadLinePatternGeneration =
                    b.take(2, Some(IfccadLinePatternGeneration::PerSegment))?;
                b.finish()?;
                Ok(Self::SpatialPolyline {
                    vertices,
                    closed,
                    line_pattern_generation,
                })
            }
            8 => {
                let mut b = RecordFields::new(r, &[(0, "definition_id"), (1, "transform")])?;
                let definition_id: u64 = b.take(0, None)?;
                let transform: IfccadBlockTransform = b.take(1, None)?;
                b.finish()?;
                Ok(Self::BlockInstance {
                    definition_id,
                    transform,
                })
            }
            9 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: IfccadViewport = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Viewport(value_0))
            }
            10 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: IfccadText = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Text(value_0))
            }
            11 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: Box<IfccadMText> = b.take(0, None)?;
                b.finish()?;
                Ok(Self::MText(value_0))
            }
            12 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: IfccadHatch = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Hatch(value_0))
            }
            13 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: Box<IfccadDimension> = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Dimension(value_0))
            }
            14 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: IfccadFilledQuadrilateral = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Solid(value_0))
            }
            15 => {
                let mut b = RecordFields::new(r, &[(0, "value_0")])?;
                let value_0: IfccadFilledQuadrilateral = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Trace(value_0))
            }
            _ => Err(r.error("unknown entity kind")),
        }
    }
}
