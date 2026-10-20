use super::*;
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
);
impl CborValue for DimensionDefinition {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Linear"), (1, "Aligned")]
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
                let mut __cbor_fields = 0_u64;
                if first_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !first_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if second_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !second_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if direction_angle.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !direction_angle.exact(d))
                {
                    __cbor_fields += 1;
                }
                if dimension_line_point.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !dimension_line_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if extension_direction_angle.present()
                    && (Some(None) as Option<Option<f64>>)
                        .as_ref()
                        .is_none_or(|d| !extension_direction_angle.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if first_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !first_point.exact(d))
                {
                    w.u64(0)?;
                    first_point.encode(w)?;
                }
                if second_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !second_point.exact(d))
                {
                    w.u64(1)?;
                    second_point.encode(w)?;
                }
                if direction_angle.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !direction_angle.exact(d))
                {
                    w.u64(2)?;
                    direction_angle.encode(w)?;
                }
                if dimension_line_point.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !dimension_line_point.exact(d))
                {
                    w.u64(3)?;
                    dimension_line_point.encode(w)?;
                }
                if extension_direction_angle.present()
                    && (Some(None) as Option<Option<f64>>)
                        .as_ref()
                        .is_none_or(|d| !extension_direction_angle.exact(d))
                {
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
                let mut __cbor_fields = 0_u64;
                if first_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !first_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if second_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !second_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if dimension_line_point.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !dimension_line_point.exact(d))
                {
                    __cbor_fields += 1;
                }
                if extension_direction_angle.present()
                    && (Some(None) as Option<Option<f64>>)
                        .as_ref()
                        .is_none_or(|d| !extension_direction_angle.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if first_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !first_point.exact(d))
                {
                    w.u64(0)?;
                    first_point.encode(w)?;
                }
                if second_point.present()
                    && (None as Option<[f64; 3]>)
                        .as_ref()
                        .is_none_or(|d| !second_point.exact(d))
                {
                    w.u64(1)?;
                    second_point.encode(w)?;
                }
                if dimension_line_point.present()
                    && (None as Option<[f64; 2]>)
                        .as_ref()
                        .is_none_or(|d| !dimension_line_point.exact(d))
                {
                    w.u64(2)?;
                    dimension_line_point.encode(w)?;
                }
                if extension_direction_angle.present()
                    && (Some(None) as Option<Option<f64>>)
                        .as_ref()
                        .is_none_or(|d| !extension_direction_angle.exact(d))
                {
                    w.u64(3)?;
                    extension_direction_angle.encode(w)?;
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
                let body = f.span(1)?;
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
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
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
unit_enum!(DimensionRole;
0 => FirstPoint,
1 => SecondPoint,
);
impl CborValue for DimensionPointSelector {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Position"),
            (1, "Start"),
            (2, "End"),
            (3, "Center"),
            (4, "Vertex"),
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
                let mut __cbor_fields = 0_u64;
                if index.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !index.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if index.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !index.exact(d))
                {
                    w.u64(0)?;
                    index.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
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
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
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
            _ => false,
        }
    }
}
