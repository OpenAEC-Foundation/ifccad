use super::*;
impl CborValue for IfccadNativeEntity {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "id"),
            (1, "layer_id"),
            (2, "kind"),
            (3, "geometry"),
            (4, "appearance"),
            (5, "visible"),
            (6, "line_pattern_scale"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let Self {
            id,
            visible,
            layer_id,
            appearance,
            line_pattern_scale,
            kind,
        } = self;
        let appearance_default = appearance.exact(&default_appearance());
        let scale_default = line_pattern_scale.to_bits() == 1.0_f64.to_bits();
        w.map(
            4 + u64::from(!appearance_default) + u64::from(!*visible) + u64::from(!scale_default),
        )?;
        w.u64(0)?;
        id.encode(w)?;
        w.u64(1)?;
        layer_id.encode(w)?;
        w.u64(2)?;
        w.u64(kind.code())?;
        w.u64(3)?;
        kind.encode_body(w)?;
        if !appearance_default {
            w.u64(4)?;
            appearance.encode(w)?;
        }
        if !*visible {
            w.u64(5)?;
            visible.encode(w)?;
        }
        if !scale_default {
            w.u64(6)?;
            line_pattern_scale.encode(w)?;
        }
        Ok(())
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(
            r,
            &[
                (0, "id"),
                (1, "layer_id"),
                (2, "kind"),
                (3, "geometry"),
                (4, "appearance"),
                (5, "visible"),
                (6, "line_pattern_scale"),
            ],
        )?;
        let code = f.take(2, None)?;
        let kind = IfccadEntityKind::decode_body(code, &mut f.span(3)?)?;
        let out = Self {
            id: f.take(0, None)?,
            layer_id: f.take(1, None)?,
            kind,
            appearance: f.take(4, Some(default_appearance()))?,
            visible: f.take(5, Some(true))?,
            line_pattern_scale: f.take(6, Some(1.0))?,
        };
        f.finish()?;
        Ok(out)
    }
    fn exact(&self, o: &Self) -> bool {
        let Self {
            id,
            visible,
            layer_id,
            appearance,
            line_pattern_scale,
            kind,
        } = self;
        id.exact(&o.id)
            && visible.exact(&o.visible)
            && layer_id.exact(&o.layer_id)
            && appearance.exact(&o.appearance)
            && line_pattern_scale.exact(&o.line_pattern_scale)
            && kind.exact(&o.kind)
    }
}
