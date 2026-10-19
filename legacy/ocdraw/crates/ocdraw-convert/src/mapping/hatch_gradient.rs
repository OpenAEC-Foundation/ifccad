//! Format-owned concrete gradient colors and target identity losses.
use cad_geometry_convert::hatch::HatchSourceLoss;
use cad_presentation_convert::{color_to_cad, explicit_color_from_cad, CadColorValue};
use ocdraw::geometry_kernel::hatch::HatchGradient;
use ocdraw::ocdraw::{DrawingColor, DrawingHatchGradient};
use opencadcodec::Color;

pub(super) fn gradient_from_cad(g: HatchGradient<Color>) -> DrawingHatchGradient {
    let colors = g.colors.map(|cs| {
        cs.map(|color| {
            let CadColorValue::Concrete {
                rgb,
                indexed,
                named,
            } = explicit_color_from_cad(color).expect("qualified concrete gradient color")
            else {
                unreachable!("symbolic endpoint excluded by preparation")
            };
            DrawingColor::Concrete {
                rgb,
                indexed,
                named,
            }
        })
    });
    HatchGradient {
        gradient_type: g.gradient_type,
        colors,
        rotation: g.rotation,
        shift: g.shift,
        color_mode: g.color_mode,
        tint: g.tint,
    }
}

pub(super) fn gradient_to_cad(
    g: &DrawingHatchGradient,
) -> (HatchGradient<Color>, Vec<HatchSourceLoss>) {
    let mut losses = Vec::new();
    let colors=g.colors.as_ref().map(|cs| {
        let mut mapped=[Color::ByLayer;2];
        for (index,c) in cs.iter().enumerate() {
            let field=if index==0 {"gradient/colors/0"}else{"gradient/colors/1"};
            let DrawingColor::Concrete { rgb, indexed, named } = c
                else { unreachable!("validated concrete gradient color") };
            let color=color_to_cad(&CadColorValue::Concrete{rgb:*rgb,indexed:indexed.clone(),named:named.clone()})
                .expect("validated native gradient color");
            for loss in color.losses {
                losses.push(HatchSourceLoss{field,detail:format!("gradient indexed identity omitted ({loss:?}); RGB retained")});
            }
            if color.named.is_some() {
                losses.push(HatchSourceLoss{field,detail:"pinned gradient model cannot retain named color identity; concrete color retained".into()});
            }
            mapped[index]=color.color;
        }
        mapped
    });
    (
        HatchGradient {
            gradient_type: g.gradient_type,
            colors,
            rotation: g.rotation,
            shift: g.shift,
            color_mode: g.color_mode,
            tint: g.tint,
        },
        losses,
    )
}
