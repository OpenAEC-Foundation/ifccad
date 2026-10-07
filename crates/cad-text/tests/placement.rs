use cad_text::*;
use ocdraw::geometry_kernel::{CoordinateFrame3, Point3};
use ocdraw::text::*;
use opencadcodec::{
    entities::{TextHorizontalAlignment as H, TextVerticalAlignment as V},
    EntityType, MText, Text, Vector3,
};

#[test]
fn text_ocs_active_anchor_and_mtext_wcs_are_separate() {
    let mut source = Text::new();
    source.value = "TILTED".into();
    source.normal = Vector3::UNIT_Y;
    source.insertion_point = Vector3::new(3., 4., 5.);
    let prepared = prepare_text_from_cad(&source).unwrap();
    assert_eq!(prepared.placement.origin().components(), [-3., 5., 4.]);
    assert_eq!(prepared.placement.x_axis().components(), [-1., 0., 0.]);
    source.horizontal_alignment = H::Right;
    source.vertical_alignment = V::Top;
    source.alignment_point = Some(Vector3::new(10., 20., 7.));
    assert_eq!(
        prepare_text_from_cad(&source)
            .unwrap()
            .placement
            .origin()
            .components(),
        [-10., 7., 20.]
    );
    let mut mtext = MText::new();
    mtext.value = "WCS".into();
    mtext.insertion_point = Vector3::new(100., 50., 7.);
    mtext.normal = Vector3::UNIT_Y;
    mtext.dwg_x_direction = Some(Vector3::UNIT_Z);
    mtext.rotation = 0.;
    let prepared = prepare_mtext_from_cad(&mtext).unwrap();
    assert_eq!(prepared.placement.origin().components(), [100., 50., 7.]);
    assert_eq!(prepared.placement.x_axis().components(), [0., 0., 1.]);
    assert_eq!(prepared.rotation, 0.);
}

#[test]
fn aligned_fit_and_whole_middle_are_closed_layouts() {
    let mut source = Text::new();
    source.value = "AB".into();
    source.insertion_point = Vector3::new(0., 0., 7.);
    source.alignment_point = Some(Vector3::new(3., 4., 7.));
    source.horizontal_alignment = H::Aligned;
    let p = prepare_text_from_cad(&source).unwrap();
    assert_eq!(
        p.layout,
        TextLayout::Aligned {
            length: 5.,
            width_factor: 1.
        }
    );
    assert_eq!(p.rotation, 0.);
    source.horizontal_alignment = H::Fit;
    source.height = 2.5;
    assert_eq!(
        prepare_text_from_cad(&source).unwrap().layout,
        TextLayout::Fit {
            length: 5.,
            height: 2.5
        }
    );
    source.horizontal_alignment = H::Middle;
    assert_eq!(
        prepare_text_from_cad(&source).unwrap().layout,
        TextLayout::WholeTextMiddle {
            height: 2.5,
            width_factor: 1.
        }
    );
    source.horizontal_alignment = H::Aligned;
    source.alignment_point = Some(Vector3::new(3., 4., 8.));
    assert!(prepare_text_from_cad(&source).is_err());
    source.alignment_point = Some(source.insertion_point);
    assert!(prepare_text_from_cad(&source).is_err());
}

#[test]
fn source_anchor_proof_contains_exact_residuals_not_a_glyph_certificate() {
    let mut source = Text::new();
    source.value = "A".into();
    source.normal = Vector3::new(1., 2., 3.);
    source.insertion_point = Vector3::new(100., 50., 7.);
    let p = prepare_text_from_cad(&source).unwrap();
    assert!(p.glyph_geometry_unassessed);
    assert!(!p.pair.points.is_empty());
    assert!(p.pair.curves.is_empty());
    let mut m = MText::new();
    m.value = "A".into();
    m.insertion_point = Vector3::new(100., 50., 7.);
    let m = prepare_mtext_from_cad(&m).unwrap();
    assert!(m.glyph_geometry_unassessed);
    assert_eq!(m.placement.origin(), Point3::new(100., 50., 7.));
    assert_eq!(m.pair.points[0].1.to_string(), "0");
}

#[test]
fn canonical_text_target_retains_mirrors_style_and_content() {
    let content = [TextRun {
        text: "AB".into(),
        ..Default::default()
    }];
    let input = TextCadInput {
        style_name: "Labels",
        placement: CoordinateFrame3::default(),
        rotation: 0.3,
        backward: true,
        upside_down: true,
        oblique_angle: 0.,
        thickness: -2.,
        layout: TextLayout::Fit {
            length: 8.,
            height: 2.5,
        },
        content: &content,
    };
    let target = prepare_text_to_cad(&input).unwrap();
    let opencadcodec::EntityType::Text(text) = &target.entity else {
        panic!()
    };
    assert_eq!(text.generation_flags, 6);
    assert_eq!(text.style, "Labels");
    assert_eq!(text.height, 2.5);
    assert_eq!(text.thickness, -2.);
    assert_eq!(text.rotation, 0.3);
    assert!(target.glyph_geometry_unassessed);
}

#[test]
fn mtext_target_keeps_wcs_direction_and_declines_independent_mirrors() {
    let content = [MTextParagraph {
        inlines: vec![MTextInline::Run {
            text: "WCS".into(),
            character_format: Default::default(),
        }],
        ..Default::default()
    }];
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let mut input = MTextCadInput {
        style_name: "Labels",
        placement: CoordinateFrame3::try_from_normal_arbitrary_axis(
            Point3::new(100., 50., 7.),
            ocdraw::geometry_kernel::Vector3::new(0., 1., 0.),
        )
        .unwrap(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        height: 2.5,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: Some(80.),
        columns: None,
        background: None,
        content: &content,
        character_format: &c,
        paragraph_format: &p,
    };
    let target = prepare_mtext_to_cad(&input).unwrap();
    let EntityType::MText(m) = &target.entity else {
        panic!()
    };
    assert_eq!(m.insertion_point, Vector3::new(100., 50., 7.));
    assert!(m.dwg_x_direction.is_some());
    let restored = prepare_mtext_from_cad(m).unwrap();
    assert_eq!(restored.placement.origin().components(), [100., 50., 7.]);
    input.backward = true;
    assert!(matches!(
        prepare_mtext_to_cad(&input),
        Err(CadTextError::Unsupported(_))
    ));
}
