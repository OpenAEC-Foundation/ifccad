//! Explicit context records; canonical Dimension definitions remain separate.
use super::*;
record!(IfccadStoredBoolOverride;0=>active:bool=None,1=>value:bool=None,);
record!(IfccadDimensionContextLayout;
0=>line_outside:IfccadStoredBoolOverride=None,1=>suppress_arrows_outside:IfccadStoredBoolOverride=None,2=>fit_switch:IfccadStoredBoolOverride=None,3=>force_text_inside:IfccadStoredBoolOverride=None,4=>text_movement_switch:IfccadStoredBoolOverride=None,5=>second_arrow_present:bool=None,6=>flip_first:bool=None,7=>flip_second:bool=None,);
record!(IfccadDimensionAnnotationContext;
0=>text_location:[f64;2]=None,1=>user_text_location:bool=None,2=>text_rotation:f64=None,3=>layout:IfccadDimensionContextLayout=None,4=>geometry:IfccadDimensionContextGeometry=None,5=>graphics:Option<IfccadDimensionGraphics>=Some(None),);
impl CborValue for IfccadDimensionContextGeometry {
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
        w.map(2)?;
        w.u64(0)?;
        match self {
            Self::Linear {
                dimension_line_point,
            } => {
                w.u64(0)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                dimension_line_point.encode(w)?;
                Ok(())
            }
            Self::Aligned {
                dimension_line_point,
            } => {
                w.u64(1)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                dimension_line_point.encode(w)?;
                Ok(())
            }
            Self::AngularTwoLines {
                dimension_arc_point,
            } => {
                w.u64(2)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                dimension_arc_point.encode(w)?;
                Ok(())
            }
            Self::AngularThreePoints {
                dimension_arc_point,
            } => {
                w.u64(3)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                dimension_arc_point.encode(w)?;
                Ok(())
            }
            Self::Radius { curve_point } => {
                w.u64(4)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                curve_point.encode(w)?;
                Ok(())
            }
            Self::Diameter {
                first_diameter_point,
                second_diameter_point,
            } => {
                w.u64(5)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                first_diameter_point.encode(w)?;
                w.u64(1)?;
                second_diameter_point.encode(w)?;
                Ok(())
            }
            Self::Ordinate {
                feature_point,
                leader_end,
            } => {
                w.u64(6)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                feature_point.encode(w)?;
                w.u64(1)?;
                leader_end.encode(w)?;
                Ok(())
            }
            Self::ArcLength {
                dimension_arc_point,
            } => {
                w.u64(7)?;
                w.u64(1)?;
                w.map(1)?;
                w.u64(0)?;
                dimension_arc_point.encode(w)?;
                Ok(())
            }
            Self::JoggedRadius {
                display_center,
                jog_point,
            } => {
                w.u64(8)?;
                w.u64(1)?;
                w.map(2)?;
                w.u64(0)?;
                display_center.encode(w)?;
                w.u64(1)?;
                jog_point.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let kind: u64 = f.take(0, None)?;
        let body = f.span(1)?;
        f.finish()?;
        match kind {
            0 => {
                let mut b = RecordFields::new(&body, &[(0, "dimension_line_point")])?;
                let dimension_line_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Linear {
                    dimension_line_point,
                })
            }
            1 => {
                let mut b = RecordFields::new(&body, &[(0, "dimension_line_point")])?;
                let dimension_line_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Aligned {
                    dimension_line_point,
                })
            }
            2 => {
                let mut b = RecordFields::new(&body, &[(0, "dimension_arc_point")])?;
                let dimension_arc_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::AngularTwoLines {
                    dimension_arc_point,
                })
            }
            3 => {
                let mut b = RecordFields::new(&body, &[(0, "dimension_arc_point")])?;
                let dimension_arc_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::AngularThreePoints {
                    dimension_arc_point,
                })
            }
            4 => {
                let mut b = RecordFields::new(&body, &[(0, "curve_point")])?;
                let curve_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::Radius { curve_point })
            }
            5 => {
                let mut b = RecordFields::new(
                    &body,
                    &[(0, "first_diameter_point"), (1, "second_diameter_point")],
                )?;
                let first_diameter_point = b.take(0, None)?;
                let second_diameter_point = b.take(1, None)?;
                b.finish()?;
                Ok(Self::Diameter {
                    first_diameter_point,
                    second_diameter_point,
                })
            }
            6 => {
                let mut b = RecordFields::new(&body, &[(0, "feature_point"), (1, "leader_end")])?;
                let feature_point = b.take(0, None)?;
                let leader_end = b.take(1, None)?;
                b.finish()?;
                Ok(Self::Ordinate {
                    feature_point,
                    leader_end,
                })
            }
            7 => {
                let mut b = RecordFields::new(&body, &[(0, "dimension_arc_point")])?;
                let dimension_arc_point = b.take(0, None)?;
                b.finish()?;
                Ok(Self::ArcLength {
                    dimension_arc_point,
                })
            }
            8 => {
                let mut b = RecordFields::new(&body, &[(0, "display_center"), (1, "jog_point")])?;
                let display_center = b.take(0, None)?;
                let jog_point = b.take(1, None)?;
                b.finish()?;
                Ok(Self::JoggedRadius {
                    display_center,
                    jog_point,
                })
            }
            _ => Err(r.error("unknown Dimension annotation geometry kind")),
        }
    }
    fn exact(&self, o: &Self) -> bool {
        self.kind() == o.kind() && self.points().zip(o.points()).all(|(a, b)| a.exact(&b))
    }
}
