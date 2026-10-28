use super::*;
use crate::dimension::{
    DimensionArcLeader, DimensionArcSpan, DimensionOrdinateMode, DimensionRaySign,
    DimensionTraversal,
};
use crate::dimension::{
    DimensionDefinition, DimensionKind, DimensionLabel, DimensionLabelPart, DimensionPointSelector,
    DimensionRole, DimensionStyleOverrides, DimensionTextPlacement,
};
use crate::text::{MTextAttachment, TextRun};
unit_enum!(DimensionKind;
0 => Linear,
1 => Aligned,
2 => AngularTwoLines,
3 => AngularThreePoints,
4 => Radius,
5 => Diameter,
6 => Ordinate,
7 => ArcLength,
8 => JoggedRadius,
);
record!(IfccadDimension;
0 => placement: IfccadPlacement = None,
1 => style_id: IfccadDimensionStyleId = None,
2 => definition: DimensionDefinition = None,
3 => text_placement: DimensionTextPlacement = None,
4 => label: DimensionLabel = None,
5 => overrides: DimensionStyleOverrides<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId> = None,
6 => associations: Vec<IfccadDimensionAssociation> = None,
7 => graphics: Option<IfccadDimensionGraphics> = Some(None),
);
unit_enum!(DimensionTraversal;0 => Counterclockwise,1 => Clockwise,);
unit_enum!(DimensionRaySign;0 => Positive,1 => Negative,);
unit_enum!(DimensionOrdinateMode;0 => Absolute,1 => Signed,);
record!(DimensionArcLeader;0 => start: [f64;2] = None,1 => end: [f64;2] = None,);
impl CborValue for DimensionArcSpan {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Partial"), (1, "Full")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Partial { radians } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                radians.encode(w)?;
                Ok(())
            }
            Self::Full { traversal } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                traversal.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        let v = match code {
            0 => {
                let mut b = RecordFields::new(&body, &[(0, "radians")])?;
                let radians: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Partial { radians }
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "traversal")])?;
                let traversal: DimensionTraversal = b.take(0, None)?;
                b.finish()?;
                Self::Full { traversal }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(v)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Partial { radians },
                Self::Partial {
                    radians: radians_other,
                },
            ) => radians.exact(radians_other),
            (
                Self::Full { traversal },
                Self::Full {
                    traversal: traversal_other,
                },
            ) => traversal.exact(traversal_other),
            _ => false,
        }
    }
}
impl CborValue for DimensionDefinition {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Linear"),
            (1, "Aligned"),
            (2, "AngularTwoLines"),
            (3, "AngularThreePoints"),
            (4, "Radius"),
            (5, "Diameter"),
            (6, "Ordinate"),
            (7, "ArcLength"),
            (8, "JoggedRadius"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Linear {
                first_point,
                second_point,
                direction_angle,
                dimension_line_point,
                extension_direction_angle,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(4 + u64::from(extension_direction_angle.is_some()))?;
                w.u64(0)?;
                first_point.encode(w)?;
                w.u64(1)?;
                second_point.encode(w)?;
                w.u64(2)?;
                direction_angle.encode(w)?;
                w.u64(3)?;
                dimension_line_point.encode(w)?;
                if extension_direction_angle.is_some() {
                    w.u64(4)?;
                    extension_direction_angle.encode(w)?;
                }
                Ok(())
            }
            Self::Aligned {
                first_point,
                second_point,
                dimension_line_point,
                extension_direction_angle,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(3 + u64::from(extension_direction_angle.is_some()))?;
                w.u64(0)?;
                first_point.encode(w)?;
                w.u64(1)?;
                second_point.encode(w)?;
                w.u64(2)?;
                dimension_line_point.encode(w)?;
                if extension_direction_angle.is_some() {
                    w.u64(3)?;
                    extension_direction_angle.encode(w)?;
                }
                Ok(())
            }
            Self::AngularTwoLines {
                line1_start,
                line1_end,
                line2_start,
                line2_end,
                ray1_sign,
                ray2_sign,
                traversal,
                dimension_arc_point,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(8)?;
                w.u64(0)?;
                line1_start.encode(w)?;
                w.u64(1)?;
                line1_end.encode(w)?;
                w.u64(2)?;
                line2_start.encode(w)?;
                w.u64(3)?;
                line2_end.encode(w)?;
                w.u64(4)?;
                ray1_sign.encode(w)?;
                w.u64(5)?;
                ray2_sign.encode(w)?;
                w.u64(6)?;
                traversal.encode(w)?;
                w.u64(7)?;
                dimension_arc_point.encode(w)?;
                Ok(())
            }
            Self::AngularThreePoints {
                vertex,
                first_point,
                second_point,
                traversal,
                dimension_arc_point,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(5)?;
                w.u64(0)?;
                vertex.encode(w)?;
                w.u64(1)?;
                first_point.encode(w)?;
                w.u64(2)?;
                second_point.encode(w)?;
                w.u64(3)?;
                traversal.encode(w)?;
                w.u64(4)?;
                dimension_arc_point.encode(w)?;
                Ok(())
            }
            Self::Radius {
                center,
                curve_point,
                leader_length,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(3)?;
                w.u64(0)?;
                center.encode(w)?;
                w.u64(1)?;
                curve_point.encode(w)?;
                w.u64(2)?;
                leader_length.encode(w)?;
                Ok(())
            }
            Self::Diameter {
                first_diameter_point,
                second_diameter_point,
                leader_length,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                w.map(3)?;
                w.u64(0)?;
                first_diameter_point.encode(w)?;
                w.u64(1)?;
                second_diameter_point.encode(w)?;
                w.u64(2)?;
                leader_length.encode(w)?;
                Ok(())
            }
            Self::Ordinate {
                origin,
                feature_point,
                axis_angle,
                leader_end,
                value_mode,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                w.map(5)?;
                w.u64(0)?;
                origin.encode(w)?;
                w.u64(1)?;
                feature_point.encode(w)?;
                w.u64(2)?;
                axis_angle.encode(w)?;
                w.u64(3)?;
                leader_end.encode(w)?;
                w.u64(4)?;
                value_mode.encode(w)?;
                Ok(())
            }
            Self::ArcLength {
                center,
                radius,
                start_parameter,
                sweep,
                dimension_arc_point,
                leader,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(7)?;
                w.u64(1)?;
                w.map(5 + u64::from(leader.is_some()))?;
                w.u64(0)?;
                center.encode(w)?;
                w.u64(1)?;
                radius.encode(w)?;
                w.u64(2)?;
                start_parameter.encode(w)?;
                w.u64(3)?;
                sweep.encode(w)?;
                w.u64(4)?;
                dimension_arc_point.encode(w)?;
                if leader.is_some() {
                    w.u64(5)?;
                    leader.encode(w)?;
                }
                Ok(())
            }
            Self::JoggedRadius {
                center,
                curve_point,
                display_center,
                jog_point,
                jog_angle,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(8)?;
                w.u64(1)?;
                w.map(5)?;
                w.u64(0)?;
                center.encode(w)?;
                w.u64(1)?;
                curve_point.encode(w)?;
                w.u64(2)?;
                display_center.encode(w)?;
                w.u64(3)?;
                jog_point.encode(w)?;
                w.u64(4)?;
                jog_angle.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        let v = match code {
            0 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "first_point"),
                        (1, "second_point"),
                        (2, "direction_angle"),
                        (3, "dimension_line_point"),
                        (4, "extension_direction_angle"),
                    ],
                )?;
                let first_point: [f64; 3] = b.take(0, None)?;
                let second_point: [f64; 3] = b.take(1, None)?;
                let direction_angle: f64 = b.take(2, None)?;
                let dimension_line_point: [f64; 2] = b.take(3, None)?;
                let extension_direction_angle: Option<f64> = b.take(4, Some(None))?;
                b.finish()?;
                Self::Linear {
                    first_point,
                    second_point,
                    direction_angle,
                    dimension_line_point,
                    extension_direction_angle,
                }
            }
            1 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "first_point"),
                        (1, "second_point"),
                        (2, "dimension_line_point"),
                        (3, "extension_direction_angle"),
                    ],
                )?;
                let first_point: [f64; 3] = b.take(0, None)?;
                let second_point: [f64; 3] = b.take(1, None)?;
                let dimension_line_point: [f64; 2] = b.take(2, None)?;
                let extension_direction_angle: Option<f64> = b.take(3, Some(None))?;
                b.finish()?;
                Self::Aligned {
                    first_point,
                    second_point,
                    dimension_line_point,
                    extension_direction_angle,
                }
            }
            2 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "line1_start"),
                        (1, "line1_end"),
                        (2, "line2_start"),
                        (3, "line2_end"),
                        (4, "ray1_sign"),
                        (5, "ray2_sign"),
                        (6, "traversal"),
                        (7, "dimension_arc_point"),
                    ],
                )?;
                let line1_start: [f64; 3] = b.take(0, None)?;
                let line1_end: [f64; 3] = b.take(1, None)?;
                let line2_start: [f64; 3] = b.take(2, None)?;
                let line2_end: [f64; 3] = b.take(3, None)?;
                let ray1_sign: DimensionRaySign = b.take(4, None)?;
                let ray2_sign: DimensionRaySign = b.take(5, None)?;
                let traversal: DimensionTraversal = b.take(6, None)?;
                let dimension_arc_point: [f64; 2] = b.take(7, None)?;
                b.finish()?;
                Self::AngularTwoLines {
                    line1_start,
                    line1_end,
                    line2_start,
                    line2_end,
                    ray1_sign,
                    ray2_sign,
                    traversal,
                    dimension_arc_point,
                }
            }
            3 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "vertex"),
                        (1, "first_point"),
                        (2, "second_point"),
                        (3, "traversal"),
                        (4, "dimension_arc_point"),
                    ],
                )?;
                let vertex: [f64; 3] = b.take(0, None)?;
                let first_point: [f64; 3] = b.take(1, None)?;
                let second_point: [f64; 3] = b.take(2, None)?;
                let traversal: DimensionTraversal = b.take(3, None)?;
                let dimension_arc_point: [f64; 2] = b.take(4, None)?;
                b.finish()?;
                Self::AngularThreePoints {
                    vertex,
                    first_point,
                    second_point,
                    traversal,
                    dimension_arc_point,
                }
            }
            4 => {
                let mut b = RecordFields::new(
                    &body,
                    &[(0, "center"), (1, "curve_point"), (2, "leader_length")],
                )?;
                let center: [f64; 3] = b.take(0, None)?;
                let curve_point: [f64; 3] = b.take(1, None)?;
                let leader_length: f64 = b.take(2, None)?;
                b.finish()?;
                Self::Radius {
                    center,
                    curve_point,
                    leader_length,
                }
            }
            5 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "first_diameter_point"),
                        (1, "second_diameter_point"),
                        (2, "leader_length"),
                    ],
                )?;
                let first_diameter_point: [f64; 3] = b.take(0, None)?;
                let second_diameter_point: [f64; 3] = b.take(1, None)?;
                let leader_length: f64 = b.take(2, None)?;
                b.finish()?;
                Self::Diameter {
                    first_diameter_point,
                    second_diameter_point,
                    leader_length,
                }
            }
            6 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "origin"),
                        (1, "feature_point"),
                        (2, "axis_angle"),
                        (3, "leader_end"),
                        (4, "value_mode"),
                    ],
                )?;
                let origin: [f64; 3] = b.take(0, None)?;
                let feature_point: [f64; 3] = b.take(1, None)?;
                let axis_angle: f64 = b.take(2, None)?;
                let leader_end: [f64; 2] = b.take(3, None)?;
                let value_mode: DimensionOrdinateMode = b.take(4, None)?;
                b.finish()?;
                Self::Ordinate {
                    origin,
                    feature_point,
                    axis_angle,
                    leader_end,
                    value_mode,
                }
            }
            7 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "radius"),
                        (2, "start_parameter"),
                        (3, "sweep"),
                        (4, "dimension_arc_point"),
                        (5, "leader"),
                    ],
                )?;
                let center: [f64; 3] = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let start_parameter: f64 = b.take(2, None)?;
                let sweep: DimensionArcSpan = b.take(3, None)?;
                let dimension_arc_point: [f64; 2] = b.take(4, None)?;
                let leader: Option<DimensionArcLeader> = b.take(5, Some(None))?;
                b.finish()?;
                Self::ArcLength {
                    center,
                    radius,
                    start_parameter,
                    sweep,
                    dimension_arc_point,
                    leader,
                }
            }
            8 => {
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "curve_point"),
                        (2, "display_center"),
                        (3, "jog_point"),
                        (4, "jog_angle"),
                    ],
                )?;
                let center: [f64; 3] = b.take(0, None)?;
                let curve_point: [f64; 3] = b.take(1, None)?;
                let display_center: [f64; 2] = b.take(2, None)?;
                let jog_point: [f64; 2] = b.take(3, None)?;
                let jog_angle: f64 = b.take(4, None)?;
                b.finish()?;
                Self::JoggedRadius {
                    center,
                    curve_point,
                    display_center,
                    jog_point,
                    jog_angle,
                }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(v)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Linear {
                    first_point,
                    second_point,
                    direction_angle,
                    dimension_line_point,
                    extension_direction_angle,
                },
                Self::Linear {
                    first_point: first_point_other,
                    second_point: second_point_other,
                    direction_angle: direction_angle_other,
                    dimension_line_point: dimension_line_point_other,
                    extension_direction_angle: extension_direction_angle_other,
                },
            ) => {
                first_point.exact(first_point_other)
                    && second_point.exact(second_point_other)
                    && direction_angle.exact(direction_angle_other)
                    && dimension_line_point.exact(dimension_line_point_other)
                    && extension_direction_angle.exact(extension_direction_angle_other)
            }
            (
                Self::Aligned {
                    first_point,
                    second_point,
                    dimension_line_point,
                    extension_direction_angle,
                },
                Self::Aligned {
                    first_point: first_point_other,
                    second_point: second_point_other,
                    dimension_line_point: dimension_line_point_other,
                    extension_direction_angle: extension_direction_angle_other,
                },
            ) => {
                first_point.exact(first_point_other)
                    && second_point.exact(second_point_other)
                    && dimension_line_point.exact(dimension_line_point_other)
                    && extension_direction_angle.exact(extension_direction_angle_other)
            }
            (
                Self::AngularTwoLines {
                    line1_start,
                    line1_end,
                    line2_start,
                    line2_end,
                    ray1_sign,
                    ray2_sign,
                    traversal,
                    dimension_arc_point,
                },
                Self::AngularTwoLines {
                    line1_start: line1_start_other,
                    line1_end: line1_end_other,
                    line2_start: line2_start_other,
                    line2_end: line2_end_other,
                    ray1_sign: ray1_sign_other,
                    ray2_sign: ray2_sign_other,
                    traversal: traversal_other,
                    dimension_arc_point: dimension_arc_point_other,
                },
            ) => {
                line1_start.exact(line1_start_other)
                    && line1_end.exact(line1_end_other)
                    && line2_start.exact(line2_start_other)
                    && line2_end.exact(line2_end_other)
                    && ray1_sign.exact(ray1_sign_other)
                    && ray2_sign.exact(ray2_sign_other)
                    && traversal.exact(traversal_other)
                    && dimension_arc_point.exact(dimension_arc_point_other)
            }
            (
                Self::AngularThreePoints {
                    vertex,
                    first_point,
                    second_point,
                    traversal,
                    dimension_arc_point,
                },
                Self::AngularThreePoints {
                    vertex: vertex_other,
                    first_point: first_point_other,
                    second_point: second_point_other,
                    traversal: traversal_other,
                    dimension_arc_point: dimension_arc_point_other,
                },
            ) => {
                vertex.exact(vertex_other)
                    && first_point.exact(first_point_other)
                    && second_point.exact(second_point_other)
                    && traversal.exact(traversal_other)
                    && dimension_arc_point.exact(dimension_arc_point_other)
            }
            (
                Self::Radius {
                    center,
                    curve_point,
                    leader_length,
                },
                Self::Radius {
                    center: center_other,
                    curve_point: curve_point_other,
                    leader_length: leader_length_other,
                },
            ) => {
                center.exact(center_other)
                    && curve_point.exact(curve_point_other)
                    && leader_length.exact(leader_length_other)
            }
            (
                Self::Diameter {
                    first_diameter_point,
                    second_diameter_point,
                    leader_length,
                },
                Self::Diameter {
                    first_diameter_point: first_diameter_point_other,
                    second_diameter_point: second_diameter_point_other,
                    leader_length: leader_length_other,
                },
            ) => {
                first_diameter_point.exact(first_diameter_point_other)
                    && second_diameter_point.exact(second_diameter_point_other)
                    && leader_length.exact(leader_length_other)
            }
            (
                Self::Ordinate {
                    origin,
                    feature_point,
                    axis_angle,
                    leader_end,
                    value_mode,
                },
                Self::Ordinate {
                    origin: origin_other,
                    feature_point: feature_point_other,
                    axis_angle: axis_angle_other,
                    leader_end: leader_end_other,
                    value_mode: value_mode_other,
                },
            ) => {
                origin.exact(origin_other)
                    && feature_point.exact(feature_point_other)
                    && axis_angle.exact(axis_angle_other)
                    && leader_end.exact(leader_end_other)
                    && value_mode.exact(value_mode_other)
            }
            (
                Self::ArcLength {
                    center,
                    radius,
                    start_parameter,
                    sweep,
                    dimension_arc_point,
                    leader,
                },
                Self::ArcLength {
                    center: center_other,
                    radius: radius_other,
                    start_parameter: start_parameter_other,
                    sweep: sweep_other,
                    dimension_arc_point: dimension_arc_point_other,
                    leader: leader_other,
                },
            ) => {
                center.exact(center_other)
                    && radius.exact(radius_other)
                    && start_parameter.exact(start_parameter_other)
                    && sweep.exact(sweep_other)
                    && dimension_arc_point.exact(dimension_arc_point_other)
                    && leader.exact(leader_other)
            }
            (
                Self::JoggedRadius {
                    center,
                    curve_point,
                    display_center,
                    jog_point,
                    jog_angle,
                },
                Self::JoggedRadius {
                    center: center_other,
                    curve_point: curve_point_other,
                    display_center: display_center_other,
                    jog_point: jog_point_other,
                    jog_angle: jog_angle_other,
                },
            ) => {
                center.exact(center_other)
                    && curve_point.exact(curve_point_other)
                    && display_center.exact(display_center_other)
                    && jog_point.exact(jog_point_other)
                    && jog_angle.exact(jog_angle_other)
            }
            _ => false,
        }
    }
}
impl CborValue for DimensionTextPlacement {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "StyleDriven"), (1, "Explicit")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::StyleDriven => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Explicit {
                position,
                rotation,
                attachment,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if position.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !position.exact(d))
                {
                    __cbor_fields += 1;
                }
                if rotation.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !rotation.exact(d))
                {
                    __cbor_fields += 1;
                }
                if attachment.present()
                    && (None as Option<MTextAttachment>)
                        .as_ref()
                        .is_none_or(|d| !attachment.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if position.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !position.exact(d))
                {
                    w.u64(0)?;
                    position.encode(w)?;
                }
                if rotation.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !rotation.exact(d))
                {
                    w.u64(1)?;
                    rotation.encode(w)?;
                }
                if attachment.present()
                    && (None as Option<MTextAttachment>)
                        .as_ref()
                        .is_none_or(|d| !attachment.exact(d))
                {
                    w.u64(2)?;
                    attachment.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::StyleDriven,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[(0, "position"), (1, "rotation"), (2, "attachment")],
                )?;
                let position: [f64; 2] = b.take(0, None)?;
                let rotation: f64 = b.take(1, None)?;
                let attachment: MTextAttachment = b.take(2, None)?;
                b.finish()?;
                Self::Explicit {
                    position,
                    rotation,
                    attachment,
                }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::StyleDriven, Self::StyleDriven) => true,
            (
                Self::Explicit {
                    position,
                    rotation,
                    attachment,
                },
                Self::Explicit {
                    position: position_other,
                    rotation: rotation_other,
                    attachment: attachment_other,
                },
            ) => {
                position.exact(position_other)
                    && rotation.exact(rotation_other)
                    && attachment.exact(attachment_other)
            }
            _ => false,
        }
    }
}
impl CborValue for DimensionLabel {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Generated"), (1, "Suppressed"), (2, "Template")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Generated => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Suppressed => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Template(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<Vec<DimensionLabelPart>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<Vec<DimensionLabelPart>>)
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
            0 => Self::Generated,
            1 => Self::Suppressed,
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Vec<DimensionLabelPart> = b.take(0, None)?;
                b.finish()?;
                Self::Template(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Generated, Self::Generated) => true,
            (Self::Suppressed, Self::Suppressed) => true,
            (Self::Template(value_0), Self::Template(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
record!(IfccadDimensionAssociation;
0 => role: DimensionRole = None,
1 => source_entity_id: u64 = None,
2 => selector: DimensionPointSelector = None,
);
impl CborValue for DimensionLabelPart {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Literal"), (1, "Measurement")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Literal(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<Vec<TextRun>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<Vec<TextRun>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Measurement => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
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
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Vec<TextRun> = b.take(0, None)?;
                b.finish()?;
                Self::Literal(value_0)
            }
            1 => Self::Measurement,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Literal(value_0), Self::Literal(value_0_other)) => value_0.exact(value_0_other),
            (Self::Measurement, Self::Measurement) => true,
            _ => false,
        }
    }
}
unit_enum!(DimensionRole;0=>FirstPoint,1=>SecondPoint,2=>Center,3=>CurvePoint,4=>FirstDiameterPoint,5=>SecondDiameterPoint,6=>Line1Start,7=>Line1End,8=>Line2Start,9=>Line2End,10=>Vertex,11=>FirstLegPoint,12=>SecondLegPoint,13=>Origin,14=>FeaturePoint,15=>ArcStart,16=>ArcEnd,);
impl CborValue for DimensionPointSelector {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Position"),
            (1, "Start"),
            (2, "End"),
            (3, "Center"),
            (4, "Vertex"),
            (5, "CirclePoint"),
            (6, "ArcPoint"),
            (7, "SegmentPoint"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Position => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Start => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::End => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Center => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(3)?;
                Ok(())
            }
            Self::Vertex { index } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                index.encode(w)?;
                Ok(())
            }
            Self::CirclePoint { angle } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                angle.encode(w)?;
                Ok(())
            }
            Self::ArcPoint { fraction } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                fraction.encode(w)?;
                Ok(())
            }
            Self::SegmentPoint { index, fraction } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(7)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                index.encode(w)?;
                w.u64(1)?;
                fraction.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let v = match code {
            0 => Self::Position,
            1 => Self::Start,
            2 => Self::End,
            3 => Self::Center,
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "index")])?;
                let index: u32 = b.take(0, None)?;
                b.finish()?;
                Self::Vertex { index }
            }
            5 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "angle")])?;
                let angle: f64 = b.take(0, None)?;
                b.finish()?;
                Self::CirclePoint { angle }
            }
            6 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "fraction")])?;
                let fraction: f64 = b.take(0, None)?;
                b.finish()?;
                Self::ArcPoint { fraction }
            }
            7 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "index"), (1, "fraction")])?;
                let index: u32 = b.take(0, None)?;
                let fraction: f64 = b.take(1, None)?;
                b.finish()?;
                Self::SegmentPoint { index, fraction }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(v)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Position, Self::Position) => true,
            (Self::Start, Self::Start) => true,
            (Self::End, Self::End) => true,
            (Self::Center, Self::Center) => true,
            (Self::Vertex { index }, Self::Vertex { index: index_other }) => {
                index.exact(index_other)
            }
            (Self::CirclePoint { angle }, Self::CirclePoint { angle: angle_other }) => {
                angle.exact(angle_other)
            }
            (
                Self::ArcPoint { fraction },
                Self::ArcPoint {
                    fraction: fraction_other,
                },
            ) => fraction.exact(fraction_other),
            (
                Self::SegmentPoint { index, fraction },
                Self::SegmentPoint {
                    index: index_other,
                    fraction: fraction_other,
                },
            ) => index.exact(index_other) && fraction.exact(fraction_other),
            _ => false,
        }
    }
}
