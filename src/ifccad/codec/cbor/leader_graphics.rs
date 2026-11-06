//! Closed saved-picture value records; connected to file kinds by the leader migration.
use super::*;
record!(IfccadLeaderGraphics;
0 => origin: IfccadLeaderGraphicsOrigin = None,
1 => placement: IfccadPlacement = None,
2 => commands: Vec<IfccadLeaderGraphicCommand> = None,
);
record!(IfccadLeaderGraphicFont;
0 => family: String = None,
1 => font_file: String = None,
2 => big_font_file: String = None,
3 => bold: Option<bool> = Some(None),
4 => italic: Option<bool> = Some(None),
5 => charset: Option<u32> = Some(None),
6 => pitch_family: Option<u32> = Some(None),
);
record!(IfccadLeaderGraphicText;
0 => position: [f64;3] = None,
1 => normal: [f64;3] = None,
2 => direction: [f64;3] = None,
3 => text: String = None,
4 => processing: IfccadLeaderGraphicTextProcessing = None,
5 => height: f64 = None,
6 => width_factor: f64 = None,
7 => oblique_angle: f64 = None,
8 => tracking: f64 = None,
9 => backwards: bool = None,
10 => upside_down: bool = None,
11 => vertical: bool = None,
12 => underline: bool = None,
13 => overline: bool = None,
14 => font: Option<IfccadLeaderGraphicFont> = Some(None),
);
unit_enum!(IfccadLeaderGraphicsOrigin; 0=>SourceSnapshot,1=>Generated);
unit_enum!(IfccadLeaderGraphicTextProcessing; 0=>Interpreted,1=>Raw);
unit_enum!(IfccadLeaderGraphicArcClosure; 0=>Simple,1=>Sector,2=>Chord);
impl<T: CborValue> CborValue for IfccadLeaderGraphicInheritance<T> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Owner"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Owner => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
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
            0 => Self::Owner,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: T = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Owner, Self::Owner) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadLeaderGraphicLineWeight {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Owner"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "DrawingDefault"),
            (4, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Owner => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::DrawingDefault => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(3)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
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
            0 => Self::Owner,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => Self::DrawingDefault,
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Owner, Self::Owner) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::DrawingDefault, Self::DrawingDefault) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadLeaderGraphicCommand {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Draw"),
            (1, "StoredExtents"),
            (2, "SetColor"),
            (3, "SetLayer"),
            (4, "SetLinePattern"),
            (5, "SetSelectionMarker"),
            (6, "SetFill"),
            (7, "SetLineWeight"),
            (8, "SetLinePatternScale"),
            (9, "SetThickness"),
            (10, "PushTransform"),
            (11, "PopTransform"),
            (12, "CadCompatibility51Zero"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Draw(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::StoredExtents { min, max } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(u64::from(min.present()) + u64::from(max.present()))?;
                if min.present() {
                    w.u64(0)?;
                    min.encode(w)?;
                }
                if max.present() {
                    w.u64(1)?;
                    max.encode(w)?;
                }
                Ok(())
            }
            Self::SetColor(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetLayer(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetLinePattern(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetSelectionMarker(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetFill(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetLineWeight(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(7)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetLinePatternScale(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(8)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::SetThickness(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(9)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::PushTransform(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(10)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::PopTransform => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(11)?;
                Ok(())
            }
            Self::CadCompatibility51Zero => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(12)?;
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
                let value_0: IfccadLeaderGraphicPrimitive = b.take(0, None)?;
                b.finish()?;
                Self::Draw(value_0)
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "min"), (1, "max")])?;
                let min: [f64; 3] = b.take(0, None)?;
                let max: [f64; 3] = b.take(1, None)?;
                b.finish()?;
                Self::StoredExtents { min, max }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadLeaderGraphicInheritance<IfccadColor> = b.take(0, None)?;
                b.finish()?;
                Self::SetColor(value_0)
            }
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Option<u64> = b.take(0, Some(None))?;
                b.finish()?;
                Self::SetLayer(value_0)
            }
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadLeaderGraphicInheritance<IfccadLinePatternId> =
                    b.take(0, None)?;
                b.finish()?;
                Self::SetLinePattern(value_0)
            }
            5 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u32 = b.take(0, None)?;
                b.finish()?;
                Self::SetSelectionMarker(value_0)
            }
            6 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: bool = b.take(0, None)?;
                b.finish()?;
                Self::SetFill(value_0)
            }
            7 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadLeaderGraphicLineWeight = b.take(0, None)?;
                b.finish()?;
                Self::SetLineWeight(value_0)
            }
            8 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::SetLinePatternScale(value_0)
            }
            9 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::SetThickness(value_0)
            }
            10 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: [f64; 16] = b.take(0, None)?;
                b.finish()?;
                Self::PushTransform(value_0)
            }
            11 => Self::PopTransform,
            12 => Self::CadCompatibility51Zero,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Draw(value_0), Self::Draw(value_0_other)) => value_0.exact(value_0_other),
            (
                Self::StoredExtents { min, max },
                Self::StoredExtents {
                    min: min_other,
                    max: max_other,
                },
            ) => min.exact(min_other) && max.exact(max_other),
            (Self::SetColor(value_0), Self::SetColor(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetLayer(value_0), Self::SetLayer(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetLinePattern(value_0), Self::SetLinePattern(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetSelectionMarker(value_0), Self::SetSelectionMarker(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetFill(value_0), Self::SetFill(value_0_other)) => value_0.exact(value_0_other),
            (Self::SetLineWeight(value_0), Self::SetLineWeight(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetLinePatternScale(value_0), Self::SetLinePatternScale(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::SetThickness(value_0), Self::SetThickness(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::PushTransform(value_0), Self::PushTransform(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::PopTransform, Self::PopTransform) => true,
            (Self::CadCompatibility51Zero, Self::CadCompatibility51Zero) => true,
            _ => false,
        }
    }
}
impl CborValue for IfccadLeaderGraphicPrimitive {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Point"),
            (1, "Polyline"),
            (2, "Polygon"),
            (3, "Circle"),
            (4, "Circle3"),
            (5, "Arc"),
            (6, "Arc3"),
            (7, "Text"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Point(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Polyline { vertices, normal } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                w.map(u64::from(vertices.present()) + u64::from(normal.present()))?;
                if vertices.present() {
                    w.u64(0)?;
                    vertices.encode(w)?;
                }
                if normal.present() {
                    w.u64(1)?;
                    normal.encode(w)?;
                }
                Ok(())
            }
            Self::Polygon { vertices } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                w.map(u64::from(vertices.present()))?;
                if vertices.present() {
                    w.u64(0)?;
                    vertices.encode(w)?;
                }
                Ok(())
            }
            Self::Circle {
                center,
                radius,
                normal,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                w.map(
                    u64::from(center.present())
                        + u64::from(radius.present())
                        + u64::from(normal.present()),
                )?;
                if center.present() {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if radius.present() {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                if normal.present() {
                    w.u64(2)?;
                    normal.encode(w)?;
                }
                Ok(())
            }
            Self::Circle3 { points } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                w.map(u64::from(points.present()))?;
                if points.present() {
                    w.u64(0)?;
                    points.encode(w)?;
                }
                Ok(())
            }
            Self::Arc {
                center,
                radius,
                normal,
                start_direction,
                sweep_angle,
                closure,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(5)?;
                w.u64(1)?;
                w.map(
                    u64::from(center.present())
                        + u64::from(radius.present())
                        + u64::from(normal.present())
                        + u64::from(start_direction.present())
                        + u64::from(sweep_angle.present())
                        + u64::from(closure.present()),
                )?;
                if center.present() {
                    w.u64(0)?;
                    center.encode(w)?;
                }
                if radius.present() {
                    w.u64(1)?;
                    radius.encode(w)?;
                }
                if normal.present() {
                    w.u64(2)?;
                    normal.encode(w)?;
                }
                if start_direction.present() {
                    w.u64(3)?;
                    start_direction.encode(w)?;
                }
                if sweep_angle.present() {
                    w.u64(4)?;
                    sweep_angle.encode(w)?;
                }
                if closure.present() {
                    w.u64(5)?;
                    closure.encode(w)?;
                }
                Ok(())
            }
            Self::Arc3 { points, closure } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                w.map(u64::from(points.present()) + u64::from(closure.present()))?;
                if points.present() {
                    w.u64(0)?;
                    points.encode(w)?;
                }
                if closure.present() {
                    w.u64(1)?;
                    closure.encode(w)?;
                }
                Ok(())
            }
            Self::Text(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(7)?;
                w.u64(1)?;
                w.map(u64::from(value_0.present()))?;
                if value_0.present() {
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
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: [f64; 3] = b.take(0, None)?;
                b.finish()?;
                Self::Point(value_0)
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "vertices"), (1, "normal")])?;
                let vertices: Vec<[f64; 3]> = b.take(0, None)?;
                let normal: Option<[f64; 3]> = b.take(1, Some(None))?;
                b.finish()?;
                Self::Polyline { vertices, normal }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "vertices")])?;
                let vertices: Vec<[f64; 3]> = b.take(0, None)?;
                b.finish()?;
                Self::Polygon { vertices }
            }
            3 => {
                let body = f.span(1)?;
                let mut b =
                    RecordFields::new(&body, &[(0, "center"), (1, "radius"), (2, "normal")])?;
                let center: [f64; 3] = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let normal: [f64; 3] = b.take(2, None)?;
                b.finish()?;
                Self::Circle {
                    center,
                    radius,
                    normal,
                }
            }
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "points")])?;
                let points: [[f64; 3]; 3] = b.take(0, None)?;
                b.finish()?;
                Self::Circle3 { points }
            }
            5 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "center"),
                        (1, "radius"),
                        (2, "normal"),
                        (3, "start_direction"),
                        (4, "sweep_angle"),
                        (5, "closure"),
                    ],
                )?;
                let center: [f64; 3] = b.take(0, None)?;
                let radius: f64 = b.take(1, None)?;
                let normal: [f64; 3] = b.take(2, None)?;
                let start_direction: [f64; 3] = b.take(3, None)?;
                let sweep_angle: f64 = b.take(4, None)?;
                let closure: IfccadLeaderGraphicArcClosure = b.take(5, None)?;
                b.finish()?;
                Self::Arc {
                    center,
                    radius,
                    normal,
                    start_direction,
                    sweep_angle,
                    closure,
                }
            }
            6 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "points"), (1, "closure")])?;
                let points: [[f64; 3]; 3] = b.take(0, None)?;
                let closure: IfccadLeaderGraphicArcClosure = b.take(1, None)?;
                b.finish()?;
                Self::Arc3 { points, closure }
            }
            7 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: Box<IfccadLeaderGraphicText> = b.take(0, None)?;
                b.finish()?;
                Self::Text(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Point(value_0), Self::Point(value_0_other)) => value_0.exact(value_0_other),
            (
                Self::Polyline { vertices, normal },
                Self::Polyline {
                    vertices: vertices_other,
                    normal: normal_other,
                },
            ) => vertices.exact(vertices_other) && normal.exact(normal_other),
            (
                Self::Polygon { vertices },
                Self::Polygon {
                    vertices: vertices_other,
                },
            ) => vertices.exact(vertices_other),
            (
                Self::Circle {
                    center,
                    radius,
                    normal,
                },
                Self::Circle {
                    center: center_other,
                    radius: radius_other,
                    normal: normal_other,
                },
            ) => {
                center.exact(center_other)
                    && radius.exact(radius_other)
                    && normal.exact(normal_other)
            }
            (
                Self::Circle3 { points },
                Self::Circle3 {
                    points: points_other,
                },
            ) => points.exact(points_other),
            (
                Self::Arc {
                    center,
                    radius,
                    normal,
                    start_direction,
                    sweep_angle,
                    closure,
                },
                Self::Arc {
                    center: center_other,
                    radius: radius_other,
                    normal: normal_other,
                    start_direction: start_direction_other,
                    sweep_angle: sweep_angle_other,
                    closure: closure_other,
                },
            ) => {
                center.exact(center_other)
                    && radius.exact(radius_other)
                    && normal.exact(normal_other)
                    && start_direction.exact(start_direction_other)
                    && sweep_angle.exact(sweep_angle_other)
                    && closure.exact(closure_other)
            }
            (
                Self::Arc3 { points, closure },
                Self::Arc3 {
                    points: points_other,
                    closure: closure_other,
                },
            ) => points.exact(points_other) && closure.exact(closure_other),
            (Self::Text(value_0), Self::Text(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
