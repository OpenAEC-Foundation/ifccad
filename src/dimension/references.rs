use super::*;

fn choice<A, Z>(
    v: &DimensionAppearanceChoice<A>,
    map: &impl Fn(&A) -> Z,
) -> DimensionAppearanceChoice<Z> {
    match v {
        DimensionAppearanceChoice::InheritEntity => DimensionAppearanceChoice::InheritEntity,
        DimensionAppearanceChoice::ByLayer => DimensionAppearanceChoice::ByLayer,
        DimensionAppearanceChoice::ByBlock => DimensionAppearanceChoice::ByBlock,
        DimensionAppearanceChoice::Explicit(v) => DimensionAppearanceChoice::Explicit(map(v)),
    }
}
fn appearance<C, P, D, Q>(
    v: &DimensionComponentAppearance<C, P>,
    color: &impl Fn(&C) -> D,
    pattern: &impl Fn(&P) -> Q,
) -> DimensionComponentAppearance<D, Q> {
    DimensionComponentAppearance {
        color: choice(&v.color, color),
        opacity: v.opacity.clone(),
        line_pattern: choice(&v.line_pattern, pattern),
        line_weight: v.line_weight.clone(),
    }
}
fn appearance_override<C, P, D, Q>(
    v: &DimensionComponentOverrides<C, P>,
    color: &impl Fn(&C) -> D,
    pattern: &impl Fn(&P) -> Q,
) -> DimensionComponentOverrides<D, Q> {
    DimensionComponentOverrides {
        color: v.color.as_ref().map(|v| choice(v, color)),
        opacity: v.opacity.clone(),
        line_pattern: v.line_pattern.as_ref().map(|v| choice(v, pattern)),
        line_weight: v.line_weight.clone(),
    }
}
fn marker<B, D>(v: &DimensionMarker<B>, map: &impl Fn(&B) -> D) -> DimensionMarker<D> {
    match v {
        DimensionMarker::ClosedFilled => DimensionMarker::ClosedFilled,
        DimensionMarker::ClosedOpen => DimensionMarker::ClosedOpen,
        DimensionMarker::Open => DimensionMarker::Open,
        DimensionMarker::Dot => DimensionMarker::Dot,
        DimensionMarker::Tick => DimensionMarker::Tick,
        DimensionMarker::None => DimensionMarker::None,
        DimensionMarker::LocalBlock { block } => DimensionMarker::LocalBlock { block: map(block) },
    }
}
fn background<C, D>(v: &DimensionBackground<C>, map: &impl Fn(&C) -> D) -> DimensionBackground<D> {
    match v {
        DimensionBackground::None => DimensionBackground::None,
        DimensionBackground::Color {
            color,
            opacity,
            padding,
        } => DimensionBackground::Color {
            color: map(color),
            opacity: *opacity,
            padding: *padding,
        },
        DimensionBackground::Canvas { opacity, padding } => DimensionBackground::Canvas {
            opacity: *opacity,
            padding: *padding,
        },
    }
}
fn extension<C, P, D, Q>(
    v: &DimensionExtensionSettings<C, P>,
    c: &impl Fn(&C) -> D,
    p: &impl Fn(&P) -> Q,
) -> DimensionExtensionSettings<D, Q> {
    DimensionExtensionSettings {
        appearance: appearance(&v.appearance, c, p),
        offset: v.offset,
        extension: v.extension,
        suppressed: v.suppressed,
        length: v.length,
    }
}
fn extension_override<C, P, D, Q>(
    v: &DimensionExtensionOverrides<C, P>,
    c: &impl Fn(&C) -> D,
    p: &impl Fn(&P) -> Q,
) -> DimensionExtensionOverrides<D, Q> {
    DimensionExtensionOverrides {
        appearance: appearance_override(&v.appearance, c, p),
        offset: v.offset,
        extension: v.extension,
        suppressed: v.suppressed,
        length: v.length,
    }
}
impl<S, B, C, P> DimensionStyleSettings<S, B, C, P> {
    /// Rebinds references and concrete presentation values without flattening settings.
    pub fn map_references<T, D, E, Q>(
        &self,
        s: impl Fn(&S) -> T,
        b: impl Fn(&B) -> D,
        c: impl Fn(&C) -> E,
        p: impl Fn(&P) -> Q,
    ) -> DimensionStyleSettings<T, D, E, Q> {
        let a = &self.arrows;
        let t = &self.text;
        let l = &self.dimension_line;
        DimensionStyleSettings {
            dimension_line: DimensionLineSettings {
                appearance: appearance(&l.appearance, &c, &p),
                extension: l.extension,
                suppress_first: l.suppress_first,
                suppress_second: l.suppress_second,
            },
            extension_lines: DimensionExtensionLines {
                first: extension(&self.extension_lines.first, &c, &p),
                second: extension(&self.extension_lines.second, &c, &p),
            },
            arrows: DimensionArrowSettings {
                first: marker(&a.first, &b),
                second: marker(&a.second, &b),
                size: a.size,
                tick_size: a.tick_size,
                flip_first: a.flip_first,
                flip_second: a.flip_second,
                center_mark: a.center_mark,
            },
            text: DimensionTextSettings {
                style: s(&t.style),
                height: t.height,
                appearance: appearance(&t.appearance, &c, &p),
                attachment: t.attachment,
                inside_orientation: t.inside_orientation,
                outside_orientation: t.outside_orientation,
                gap: t.gap,
                movement: t.movement,
                line_spacing: t.line_spacing,
                background: background(&t.background, &c),
            },
            fit: self.fit.clone(),
            numbers: self.numbers.clone(),
            presentation_scale: self.presentation_scale,
        }
    }
}
impl<S, B, C, P> DimensionStyleOverrides<S, B, C, P> {
    /// Retains explicit resets and unused type settings while rebinding local domains.
    pub fn map_references<T, D, E, Q>(
        &self,
        s: impl Fn(&S) -> T,
        b: impl Fn(&B) -> D,
        c: impl Fn(&C) -> E,
        p: impl Fn(&P) -> Q,
    ) -> DimensionStyleOverrides<T, D, E, Q> {
        let a = &self.arrows;
        let t = &self.text;
        let l = &self.dimension_line;
        DimensionStyleOverrides {
            dimension_line: DimensionLineOverrides {
                appearance: appearance_override(&l.appearance, &c, &p),
                extension: l.extension,
                suppress_first: l.suppress_first,
                suppress_second: l.suppress_second,
            },
            extension_lines: DimensionExtensionLinesOverrides {
                first: extension_override(&self.extension_lines.first, &c, &p),
                second: extension_override(&self.extension_lines.second, &c, &p),
            },
            arrows: DimensionArrowOverrides {
                first: a.first.as_ref().map(|v| marker(v, &b)),
                second: a.second.as_ref().map(|v| marker(v, &b)),
                size: a.size,
                tick_size: a.tick_size,
                flip_first: a.flip_first,
                flip_second: a.flip_second,
                center_mark: a.center_mark,
            },
            text: DimensionTextOverrides {
                style: t.style.as_ref().map(s),
                height: t.height,
                appearance: appearance_override(&t.appearance, &c, &p),
                attachment: t.attachment,
                inside_orientation: t.inside_orientation,
                outside_orientation: t.outside_orientation,
                gap: t.gap,
                movement: t.movement,
                line_spacing: t.line_spacing,
                background: t.background.as_ref().map(|v| background(v, &c)),
            },
            fit: self.fit.clone(),
            numbers: self.numbers.clone(),
            presentation_scale: self.presentation_scale,
        }
    }
}
