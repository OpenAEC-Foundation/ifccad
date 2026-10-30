//! Explicit graphics records using the existing contextual primitive bodies.
use super::*;
unit_enum!(IfccadDimensionGraphicsOrigin;0=>SourceSnapshot,1=>Generated,);
unit_enum!(IfccadDimensionGraphicsPurpose;0=>Display,1=>ReferencePoint,);
record!(IfccadDimensionGraphics;
0=>origin:IfccadDimensionGraphicsOrigin=None,
1=>parts:Vec<IfccadDimensionGraphicsPart>=None,
);
impl CborValue for IfccadDimensionGraphicsKind {
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
            (10, "Text"),
            (11, "MText"),
            (12, "Hatch"),
            (14, "Solid"),
            (15, "Trace"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        IfccadEntityKind::from(self.clone()).encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        IfccadEntityKind::decode(r)?
            .try_into()
            .map_err(|e| r.error(e))
    }
    fn exact(&self, o: &Self) -> bool {
        IfccadEntityKind::from(self.clone()).exact(&o.clone().into())
    }
}
impl CborValue for IfccadDimensionGraphicsPart {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "layer_id"),
            (1, "kind"),
            (2, "geometry"),
            (3, "appearance"),
            (4, "visible"),
            (5, "line_pattern_scale"),
            (6, "purpose"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let appearance = !self.appearance.exact(&default_appearance());
        let visible = !self.visible;
        let scale = !self.line_pattern_scale.exact(&1.0);
        let purpose = self.purpose != IfccadDimensionGraphicsPurpose::Display;
        w.map(
            2 + u64::from(self.layer_id.is_some())
                + u64::from(appearance)
                + u64::from(visible)
                + u64::from(scale)
                + u64::from(purpose),
        )?;
        if let Some(id) = self.layer_id {
            w.u64(0)?;
            id.encode(w)?;
        }
        let kind = IfccadEntityKind::from(self.kind.clone());
        w.u64(1)?;
        w.u64(kind.code())?;
        w.u64(2)?;
        kind.encode_body(w)?;
        if appearance {
            w.u64(3)?;
            self.appearance.encode(w)?;
        }
        if visible {
            w.u64(4)?;
            self.visible.encode(w)?;
        }
        if scale {
            w.u64(5)?;
            self.line_pattern_scale.encode(w)?;
        }
        if purpose {
            w.u64(6)?;
            self.purpose.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(
            r,
            &[
                (0, "layer_id"),
                (1, "kind"),
                (2, "geometry"),
                (3, "appearance"),
                (4, "visible"),
                (5, "line_pattern_scale"),
                (6, "purpose"),
            ],
        )?;
        let code: u64 = f.take(1, None)?;
        if !matches!(code,0..=8|10..=12|14|15) {
            return Err(r.error("invalid Dimension graphics kind"));
        }
        let mut body = f.span(2)?;
        let kind = IfccadEntityKind::decode_body(code, &mut body)?
            .try_into()
            .map_err(|e| r.error(e))?;
        let out = Self {
            layer_id: f.take(0, Some(None))?,
            kind,
            appearance: f.take(3, Some(default_appearance()))?,
            visible: f.take(4, Some(true))?,
            line_pattern_scale: f.take(5, Some(1.0))?,
            purpose: f.take(6, Some(IfccadDimensionGraphicsPurpose::Display))?,
        };
        f.finish()?;
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        self.layer_id.exact(&o.layer_id)
            && self.kind.exact(&o.kind)
            && self.appearance.exact(&o.appearance)
            && self.visible.exact(&o.visible)
            && self.line_pattern_scale.exact(&o.line_pattern_scale)
            && self.purpose.exact(&o.purpose)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standalone_graphics_enum_reuses_registered_body_without_opening_forbidden_kinds() {
        for v in [
            IfccadDimensionGraphicsKind::LineSegment {
                start: [-0., 2., 0.],
                end: [4., 5., 0.],
                thickness: 0.,
                extrusion_direction: [0., 0., 1.],
            },
            IfccadDimensionGraphicsKind::Trace(IfccadFilledQuadrilateral {
                placement: IfccadPlacement {
                    origin: [0.; 3],
                    x_axis: [1., 0., 0.],
                    y_axis: [0., 1., 0.],
                },
                corners: [[0.; 2]; 4],
                thickness: -0.,
            }),
        ] {
            let mut w = CborWriter::new();
            v.encode(&mut w).unwrap();
            let b = w.finish().unwrap().bytes;
            let back =
                IfccadDimensionGraphicsKind::decode(&mut CborReader::new(&b, 0, "/kind".into()))
                    .unwrap();
            assert!(v.exact(&back));
        }
        for code in [9, 13, 16] {
            assert!(IfccadDimensionGraphicsKind::decode(&mut CborReader::new(
                &[0xa2, 0, code, 1, 0xa0],
                0,
                "/kind".into()
            ))
            .is_err());
        }
    }
}
