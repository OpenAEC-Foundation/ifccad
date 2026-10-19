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
        let code: u64 = f.take(0, None)?;
        let mut body = f.span(1)?;
        f.finish()?;
        Self::decode_body(code, &mut body)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::LineSegment { start, end },
                Self::LineSegment {
                    start: start_other,
                    end: end_other,
                },
            ) => start.exact(start_other) && end.exact(end_other),
            (
                Self::Point { placement },
                Self::Point {
                    placement: placement_other,
                },
            ) => placement.exact(placement_other),
            (
                Self::Circle { placement, radius },
                Self::Circle {
                    placement: placement_other,
                    radius: radius_other,
                },
            ) => placement.exact(placement_other) && radius.exact(radius_other),
            (
                Self::Arc {
                    placement,
                    radius,
                    start_parameter,
                    sweep_parameter,
                },
                Self::Arc {
                    placement: placement_other,
                    radius: radius_other,
                    start_parameter: start_parameter_other,
                    sweep_parameter: sweep_parameter_other,
                },
            ) => {
                placement.exact(placement_other)
                    && radius.exact(radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep_parameter.exact(sweep_parameter_other)
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
                },
                Self::PlanarPolyline {
                    placement: placement_other,
                    vertices: vertices_other,
                    bulges: bulges_other,
                    closed: closed_other,
                    line_pattern_generation: line_pattern_generation_other,
                },
            ) => {
                placement.exact(placement_other)
                    && vertices.exact(vertices_other)
                    && bulges.exact(bulges_other)
                    && closed.exact(closed_other)
                    && line_pattern_generation.exact(line_pattern_generation_other)
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
        }
    }
    pub(super) fn encode_body(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::LineSegment { start, end } => {
                let mut __cbor_fields = 0_u64;
                if start.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !start.exact(d))
                {
                    __cbor_fields += 1;
                }
                if end.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !end.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if start.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !start.exact(d))
                {
                    w.u64(0)?;
                    encode_line_point(start, w)?;
                }
                if end.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !end.exact(d))
                {
                    w.u64(1)?;
                    encode_line_point(end, w)?;
                }
                Ok(())
            }
            Self::Point { placement } => {
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                Ok(())
            }
            Self::Circle { placement, radius } => {
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
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
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
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
            Self::Arc {
                placement,
                radius,
                start_parameter,
                sweep_parameter,
            } => {
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
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
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
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
            Self::Ellipse {
                placement,
                semi_major_radius,
                semi_minor_radius,
            } => {
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
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
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    w.u64(1)?;
                    semi_major_radius.encode(w)?;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
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
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
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
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if semi_major_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_major_radius.exact(d))
                {
                    w.u64(1)?;
                    semi_major_radius.encode(w)?;
                }
                if semi_minor_radius.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !semi_minor_radius.exact(d))
                {
                    w.u64(2)?;
                    semi_minor_radius.encode(w)?;
                }
                if start_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !start_parameter.exact(d))
                {
                    w.u64(3)?;
                    start_parameter.encode(w)?;
                }
                if sweep_parameter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !sweep_parameter.exact(d))
                {
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
            } => {
                let mut __cbor_fields = 0_u64;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    __cbor_fields += 1;
                }
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
                if closed.present()
                    && (Some(false) as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !closed.exact(d))
                {
                    __cbor_fields += 1;
                }
                if line_pattern_generation.present()
                    && (Some(IfccadLinePatternGeneration::PerSegment)
                        as Option<IfccadLinePatternGeneration>)
                        .as_ref()
                        .is_none_or(|d| !line_pattern_generation.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if placement.present()
                    && (None as Option<IfccadPlacement>)
                        .as_ref()
                        .is_none_or(|d| !placement.exact(d))
                {
                    w.u64(0)?;
                    placement.encode(w)?;
                }
                if vertices.present()
                    && (None as Option<Vec<[f64; 2]>>)
                        .as_ref()
                        .is_none_or(|d| !vertices.exact(d))
                {
                    w.u64(1)?;
                    vertices.encode(w)?;
                }
                if bulges.present()
                    && (None as Option<Vec<f64>>)
                        .as_ref()
                        .is_none_or(|d| !bulges.exact(d))
                {
                    w.u64(2)?;
                    bulges.encode(w)?;
                }
                if closed.present()
                    && (Some(false) as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !closed.exact(d))
                {
                    w.u64(3)?;
                    closed.encode(w)?;
                }
                if line_pattern_generation.present()
                    && (Some(IfccadLinePatternGeneration::PerSegment)
                        as Option<IfccadLinePatternGeneration>)
                        .as_ref()
                        .is_none_or(|d| !line_pattern_generation.exact(d))
                {
                    w.u64(4)?;
                    line_pattern_generation.encode(w)?;
                }
                Ok(())
            }
            Self::SpatialPolyline {
                vertices,
                closed,
                line_pattern_generation,
            } => {
                let mut __cbor_fields = 0_u64;
                if vertices.present()
                    && (None as Option<Vec<[f64; 3]>>)
                        .as_ref()
                        .is_none_or(|d| !vertices.exact(d))
                {
                    __cbor_fields += 1;
                }
                if closed.present()
                    && (Some(false) as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !closed.exact(d))
                {
                    __cbor_fields += 1;
                }
                if line_pattern_generation.present()
                    && (Some(IfccadLinePatternGeneration::PerSegment)
                        as Option<IfccadLinePatternGeneration>)
                        .as_ref()
                        .is_none_or(|d| !line_pattern_generation.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if vertices.present()
                    && (None as Option<Vec<[f64; 3]>>)
                        .as_ref()
                        .is_none_or(|d| !vertices.exact(d))
                {
                    w.u64(0)?;
                    vertices.encode(w)?;
                }
                if closed.present()
                    && (Some(false) as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !closed.exact(d))
                {
                    w.u64(1)?;
                    closed.encode(w)?;
                }
                if line_pattern_generation.present()
                    && (Some(IfccadLinePatternGeneration::PerSegment)
                        as Option<IfccadLinePatternGeneration>)
                        .as_ref()
                        .is_none_or(|d| !line_pattern_generation.exact(d))
                {
                    w.u64(2)?;
                    line_pattern_generation.encode(w)?;
                }
                Ok(())
            }
            Self::BlockInstance {
                definition_id,
                transform,
            } => {
                let mut __cbor_fields = 0_u64;
                if definition_id.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !definition_id.exact(d))
                {
                    __cbor_fields += 1;
                }
                if transform.present()
                    && (None as Option<IfccadBlockTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if definition_id.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !definition_id.exact(d))
                {
                    w.u64(0)?;
                    definition_id.encode(w)?;
                }
                if transform.present()
                    && (None as Option<IfccadBlockTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    w.u64(1)?;
                    transform.encode(w)?;
                }
                Ok(())
            }
            Self::Viewport(value_0) => {
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadViewport>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadViewport>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Text(value_0) => {
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadText>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadText>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::MText(value_0) => {
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<Box<IfccadMText>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<Box<IfccadMText>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Hatch(value_0) => {
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadHatch>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadHatch>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Dimension(value_0) => {
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<Box<IfccadDimension>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<Box<IfccadDimension>>)
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
    pub(super) fn decode_body(
        code: u64,
        r: &mut CborReader<'_>,
    ) -> Result<Self, IfccadCborReadError> {
        match code {
            0 => {
                let mut b = RecordFields::new(r, &[(0, "start"), (1, "end")])?;
                let start: [f64; 3] = decode_line_point(&mut b.span(0)?)?;
                let end: [f64; 3] = decode_line_point(&mut b.span(1)?)?;
                b.finish()?;
                Ok(Self::LineSegment { start, end })
            }
            1 => {
                let mut b = RecordFields::new(r, &[(0, "placement")])?;
                let placement: IfccadPlacement = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Point { placement })
            }
            2 => {
                let mut b = RecordFields::new(r, &[(0, "placement"), (1, "radius")])?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                b.finish()?;
                Ok(Self::Circle { placement, radius })
            }
            3 => {
                let mut b = RecordFields::new(
                    r,
                    &[
                        (0, "placement"),
                        (1, "radius"),
                        (2, "start_parameter"),
                        (3, "sweep_parameter"),
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let start_parameter: f64 = b.take(2, None)?;
                let sweep_parameter: f64 = b.take(3, None)?;
                b.finish()?;
                Ok(Self::Arc {
                    placement,
                    radius,
                    start_parameter,
                    sweep_parameter,
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
                    ],
                )?;
                let placement: IfccadPlacement = b.take(0, None)?;
                let vertices: Vec<[f64; 2]> = b.take(1, None)?;
                let bulges: Vec<f64> = b.take(2, None)?;
                let closed: bool = b.take(3, Some(false))?;
                let line_pattern_generation: IfccadLinePatternGeneration =
                    b.take(4, Some(IfccadLinePatternGeneration::PerSegment))?;
                b.finish()?;
                Ok(Self::PlanarPolyline {
                    placement,
                    vertices,
                    bulges,
                    closed,
                    line_pattern_generation,
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
            _ => Err(r.error("unknown native kind")),
        }
    }
}
