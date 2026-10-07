//! Author standalone text without a CAD runtime or font engine.
use ocdraw::{ocdraw::*, text::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: write_text <new-output.ocdraw.json>")?;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("explorer-ocdraw-text", "mm"))?;
    let continuous = b.ensure_continuous_line_pattern()?;
    let layer = b.add_layer(LayerDefinition::new(
        "Text",
        RgbColor::new(35, 90, 160),
        continuous,
    ))?;
    let mut properties = TextStyleProperties::new(FontRequest {
        cad_font_name: Some("txt.shx".into()),
        ..Default::default()
    });
    properties.last_used_height = Some(2.5);
    let style = b.add_text_style(TextStyleDefinition {
        name: "Requested CAD font".into(),
        properties,
    })?;
    b.add_text(TextEntityDefinition::new(
        layer,
        style,
        2.5,
        vec![TextRun {
            text: "OCDraw Text · é🙂".into(),
            underline: true,
            ..Default::default()
        }],
    ))?;
    let mut middle = TextEntityDefinition::new(
        layer,
        style,
        2.5,
        vec![TextRun {
            text: "Whole-text middle".into(),
            ..Default::default()
        }],
    );
    middle.placement = CoordinateFrame3::try_new(
        Point3::new(50., -10., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )?;
    middle.layout = TextLayout::WholeTextMiddle {
        height: 2.5,
        width_factor: 1.,
    };
    b.add_text(middle)?;
    let mut mtext = MTextEntityDefinition::new(
        layer,
        style,
        2.5,
        vec![
            MTextParagraph {
                character_format: CharacterFormat {
                    underline: Some(true),
                    ..Default::default()
                },
                inlines: vec![MTextInline::Run {
                    text: "MText met alinea’s".into(),
                    character_format: Default::default(),
                }],
                ..Default::default()
            },
            MTextParagraph {
                inlines: vec![MTextInline::Run {
                    text: "Letterlijke codes: \\P en %<veld>.".into(),
                    character_format: Default::default(),
                }],
                ..Default::default()
            },
            MTextParagraph {
                inlines: vec![
                    MTextInline::Run {
                        text: "Breuk: ".into(),
                        character_format: Default::default(),
                    },
                    MTextInline::Stack(TextStack {
                        upper: "1".into(),
                        lower: "2".into(),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            MTextParagraph::default(),
        ],
    );
    mtext.placement = CoordinateFrame3::try_new(
        Point3::new(0., -20., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )?;
    mtext.wrap_width = Some(60.);
    mtext.background = Some(MTextBackground {
        fill: MTextFill::Canvas,
        ..Default::default()
    });
    b.add_mtext(mtext)?;
    b.finish()?.write_file(path)?;
    Ok(())
}
