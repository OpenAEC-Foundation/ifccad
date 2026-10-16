use super::*;
use crate::text::MTextAttachment;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "mode",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
#[derive(Default)]
pub enum DimensionAppearanceChoice<T> {
    #[default]
    InheritEntity,
    ByLayer,
    ByBlock,
    Explicit(T),
}

impl<C, P> Default for DimensionComponentAppearance<C, P> {
    fn default() -> Self {
        Self {
            color: Default::default(),
            opacity: Default::default(),
            line_pattern: Default::default(),
            line_weight: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionMarker<B> {
    ClosedFilled,
    ClosedOpen,
    Open,
    Dot,
    Tick,
    None,
    LocalBlock { block: B },
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionCenterMark {
    None,
    Mark { size: f64 },
    Lines { size: f64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionExtensionLength {
    Automatic,
    Fixed { length: f64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionTextOrientation {
    Aligned,
    Horizontal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionTextMovement {
    MoveDimensionLine,
    AddLeader,
    Free,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionLineSpacing {
    AtLeast { factor: f64 },
    Exact { factor: f64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionPadding {
    Absolute { distance: f64 },
    Relative { factor: f64 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DimensionBackground<C> {
    None,
    Color {
        color: C,
        opacity: f64,
        padding: DimensionPadding,
    },
    Canvas {
        opacity: f64,
        padding: DimensionPadding,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionFitPlacement {
    TextInside,
    ArrowsInside,
    BestFit,
}

// The paired sparse records retain authored presence at each setting leaf.
macro_rules! group {
    ($base:ident, $over:ident $(<$($g:ident),+>)? $bound:literal { $(@appearance: $app:ty => $overapp:ty;)? $($(#[$attr:meta])* $field:ident:$ty:ty),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all="camelCase", deny_unknown_fields)]
        pub struct $base $(<$($g),+>)? { $(pub appearance:$app,)? $($(#[$attr])* pub $field:$ty),+ }
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all="camelCase", deny_unknown_fields)]
        #[serde(bound(deserialize=$bound))]
        pub struct $over $(<$($g),+>)? { $(#[serde(default)] pub appearance:$overapp,)? $(
            #[serde(default, skip_serializing_if="Option::is_none", deserialize_with="super::present_option")]
            pub $field:Option<$ty>
        ),+ }
        impl $(<$($g),+>)? Default for $over $(<$($g),+>)? { fn default()->Self { Self { $(appearance:<$overapp>::default(),)? $($field:None),+ } } }
        impl $(<$($g:Clone),+>)? $over $(<$($g),+>)? {
            fn apply(&self, target:&mut $base $(<$($g),+>)?) { $(let _: &$app = &target.appearance; self.appearance.apply(&mut target.appearance);)? $(if let Some(value)=&self.$field { target.$field=value.clone(); })+ }
        }
    }
}
group!(DimensionComponentAppearance,DimensionComponentOverrides<C,P> "C: Deserialize<'de>, P: Deserialize<'de>" {
    color:DimensionAppearanceChoice<C>,opacity:DimensionAppearanceChoice<f64>,line_pattern:DimensionAppearanceChoice<P>,line_weight:DimensionAppearanceChoice<f64>
});
group!(DimensionLineSettings,DimensionLineOverrides<C,P> "C: Deserialize<'de>, P: Deserialize<'de>" {
    @appearance:DimensionComponentAppearance<C,P> => DimensionComponentOverrides<C,P>; extension:f64, suppress_first:bool,suppress_second:bool
});
group!(DimensionExtensionSettings,DimensionExtensionOverrides<C,P> "C: Deserialize<'de>, P: Deserialize<'de>" {
    @appearance:DimensionComponentAppearance<C,P> => DimensionComponentOverrides<C,P>; offset:f64,extension:f64,suppressed:bool,length:DimensionExtensionLength
});
group!(DimensionArrowSettings,DimensionArrowOverrides<B> "B: Deserialize<'de>" {
    first:DimensionMarker<B>,second:DimensionMarker<B>,size:f64,tick_size:f64,flip_first:bool,flip_second:bool,center_mark:DimensionCenterMark
});
group!(DimensionTextSettings,DimensionTextOverrides<S,C,P> "S: Deserialize<'de>, C: Deserialize<'de>, P: Deserialize<'de>" {
    @appearance:DimensionComponentAppearance<C,P> => DimensionComponentOverrides<C,P>; style:S,height:f64,attachment:MTextAttachment,
    inside_orientation:DimensionTextOrientation,outside_orientation:DimensionTextOrientation,gap:f64,
    movement:DimensionTextMovement,line_spacing:DimensionLineSpacing,background:DimensionBackground<C>
});
group!(DimensionFitSettings,DimensionFitOverrides "" { preferred_placement:DimensionFitPlacement,force_text_inside:bool,keep_dimension_line_outside:bool });

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionExtensionLines<C, P> {
    pub first: DimensionExtensionSettings<C, P>,
    pub second: DimensionExtensionSettings<C, P>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    bound(deserialize = "C: Deserialize<'de>, P: Deserialize<'de>")
)]
pub struct DimensionExtensionLinesOverrides<C, P> {
    #[serde(default)]
    pub first: DimensionExtensionOverrides<C, P>,
    #[serde(default)]
    pub second: DimensionExtensionOverrides<C, P>,
}
impl<C, P> Default for DimensionExtensionLinesOverrides<C, P> {
    fn default() -> Self {
        Self {
            first: Default::default(),
            second: Default::default(),
        }
    }
}

group!(DimensionPrimaryNumberSettings,DimensionPrimaryNumberOverrides "" {
    measurement_factor:f64, application:DimensionFactorApplication, format:DimensionDistanceFormat,
    angle_format:DimensionAngleFormat, precision:u8,decimal_separator:char,rounding_increment:f64,
    zero_suppression:DimensionZeroSuppression,prefix:String,suffix:String,arc_symbol:DimensionArcSymbol,
    #[serde(default)] fraction_layout:DimensionFractionLayout,
    #[serde(default="super::default_fraction_scale")] fraction_scale:f64
});

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "settings",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DimensionAlternateChoice {
    None,
    Enabled(DimensionAlternateNumberSettings),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[derive(Default)]
pub struct DimensionNumberOverrides {
    #[serde(default)]
    pub primary: DimensionPrimaryNumberOverrides,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::present_option"
    )]
    pub alternate: Option<DimensionAlternateChoice>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::present_option"
    )]
    pub tolerance: Option<DimensionToleranceSettings>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DimensionStyleSettings<S, B, C, P> {
    pub dimension_line: DimensionLineSettings<C, P>,
    pub extension_lines: DimensionExtensionLines<C, P>,
    pub arrows: DimensionArrowSettings<B>,
    pub text: DimensionTextSettings<S, C, P>,
    pub fit: DimensionFitSettings,
    pub numbers: DimensionNumberSettings,
    pub presentation_scale: f64,
}
impl<S, B, C, P> DimensionStyleSettings<S, B, C, P> {
    pub fn new(style: S) -> Self {
        let extension = || DimensionExtensionSettings {
            appearance: Default::default(),
            offset: 0.625,
            extension: 1.25,
            suppressed: false,
            length: DimensionExtensionLength::Automatic,
        };
        Self {
            dimension_line: DimensionLineSettings {
                appearance: Default::default(),
                extension: 0.0,
                suppress_first: false,
                suppress_second: false,
            },
            extension_lines: DimensionExtensionLines {
                first: extension(),
                second: extension(),
            },
            arrows: DimensionArrowSettings {
                first: DimensionMarker::ClosedFilled,
                second: DimensionMarker::ClosedFilled,
                size: 2.5,
                tick_size: 0.0,
                flip_first: false,
                flip_second: false,
                center_mark: DimensionCenterMark::None,
            },
            text: DimensionTextSettings {
                style,
                height: 2.5,
                appearance: Default::default(),
                attachment: MTextAttachment::MiddleCenter,
                inside_orientation: DimensionTextOrientation::Aligned,
                outside_orientation: DimensionTextOrientation::Horizontal,
                gap: 0.625,
                movement: DimensionTextMovement::AddLeader,
                line_spacing: DimensionLineSpacing::AtLeast { factor: 1.0 },
                background: DimensionBackground::None,
            },
            fit: DimensionFitSettings {
                preferred_placement: DimensionFitPlacement::BestFit,
                force_text_inside: false,
                keep_dimension_line_outside: true,
            },
            numbers: Default::default(),
            presentation_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    bound(
        deserialize = "S: Deserialize<'de>, B: Deserialize<'de>, C: Deserialize<'de>, P: Deserialize<'de>"
    )
)]
pub struct DimensionStyleOverrides<S, B, C, P> {
    #[serde(default)]
    pub dimension_line: DimensionLineOverrides<C, P>,
    #[serde(default)]
    pub extension_lines: DimensionExtensionLinesOverrides<C, P>,
    #[serde(default)]
    pub arrows: DimensionArrowOverrides<B>,
    #[serde(default)]
    pub text: DimensionTextOverrides<S, C, P>,
    #[serde(default)]
    pub fit: DimensionFitOverrides,
    #[serde(default)]
    pub numbers: DimensionNumberOverrides,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::present_option"
    )]
    pub presentation_scale: Option<f64>,
}
impl<S, B, C, P> Default for DimensionStyleOverrides<S, B, C, P> {
    fn default() -> Self {
        Self {
            dimension_line: Default::default(),
            extension_lines: Default::default(),
            arrows: Default::default(),
            text: Default::default(),
            fit: Default::default(),
            numbers: Default::default(),
            presentation_scale: None,
        }
    }
}
impl<S: Clone, B: Clone, C: Clone, P: Clone> DimensionStyleOverrides<S, B, C, P> {
    pub fn apply(&self, target: &mut DimensionStyleSettings<S, B, C, P>) {
        self.dimension_line.apply(&mut target.dimension_line);
        self.extension_lines
            .first
            .apply(&mut target.extension_lines.first);
        self.extension_lines
            .second
            .apply(&mut target.extension_lines.second);
        self.arrows.apply(&mut target.arrows);
        self.text.apply(&mut target.text);
        self.fit.apply(&mut target.fit);
        self.numbers.primary.apply(&mut target.numbers.primary);
        if let Some(v) = &self.numbers.alternate {
            target.numbers.alternate = match v {
                DimensionAlternateChoice::None => None,
                DimensionAlternateChoice::Enabled(v) => Some(v.clone()),
            };
        }
        if let Some(v) = &self.numbers.tolerance {
            target.numbers.tolerance = v.clone();
        }
        if let Some(v) = self.presentation_scale {
            target.presentation_scale = v;
        }
    }
}

pub fn resolve_dimension_settings<S: Clone, B: Clone, C: Clone, P: Clone>(
    base: &DimensionStyleSettings<S, B, C, P>,
    kind: DimensionKind,
    types: &BTreeMap<DimensionKind, DimensionStyleOverrides<S, B, C, P>>,
    local: &DimensionStyleOverrides<S, B, C, P>,
) -> DimensionStyleSettings<S, B, C, P> {
    let mut output = base.clone();
    if let Some(overrides) = types.get(&kind) {
        overrides.apply(&mut output);
    }
    local.apply(&mut output);
    output
}
