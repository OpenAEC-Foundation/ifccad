use ocdraw::{ocdraw::*, text::*};

fn framed(paper: bool, unit: &str) -> OcdrawDocument {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("frame-text", unit)).unwrap();
    let p = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "Text",
            RgbColor::new(255, 255, 255),
            p,
        ))
        .unwrap();
    let style = b
        .add_text_style(TextStyleDefinition {
            name: "Text".into(),
            properties: TextStyleProperties::new(FontRequest::family("Requested")),
        })
        .unwrap();
    let scope = if paper {
        b.add_paper_layout("Sheet").unwrap()
    } else {
        0
    };
    let mut t = MTextEntityDefinition::new(
        layer,
        style,
        2.,
        vec![MTextParagraph {
            inlines: vec![MTextInline::Run {
                text: "Box".into(),
                character_format: Default::default(),
            }],
            ..Default::default()
        }],
    )
    .in_scope(scope);
    t.background = Some(MTextBackground {
        frame: true,
        ..Default::default()
    });
    b.add_mtext(t).unwrap();
    b.build_document().unwrap()
}

#[test]
fn frame_stroke_resolves_in_drawing_coordinates_but_paper_does_not_borrow_the_unit() {
    let model = framed(false, "mm");
    assert_eq!(
        model.scopes[0].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
    assert!(model.scopes[0].bounds.is_some());
    let mut paper = framed(true, "mm");
    assert!(paper.scopes[1].bounds.is_none());
    assert_eq!(
        assess_ocdraw_document_bounds(&paper).unwrap()[1].quality,
        Some(OcdrawBoundsQuality::Partial)
    );
    paper.layouts[1].settings.media = Some(LayoutMedia {
        unit: MediaUnit::Physical(DrawingLengthUnit::Millimetre),
        width: 210.,
        height: 297.,
    });
    recompute_ocdraw_document_bounds(&mut paper).unwrap();
    assert!(paper.scopes[1].bounds.is_none());
    paper.mtext_entities[0].appearance.line_weight = AppearanceSelection::Explicit(0.);
    recompute_ocdraw_document_bounds(&mut paper).unwrap();
    assert_eq!(
        paper.scopes[1].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
}

#[test]
fn partial_coverage_does_not_hide_numeric_failure_and_recompute_is_atomic() {
    let mut d = framed(false, "unitless");
    assert!(d.scopes[0].bounds.is_none());
    let before = d.scopes.clone();
    d.geometric_entities.push(DrawingGeometricEntity {
        id: 2,
        layer_id: d.layers[0].id,
        visible: true,
        appearance: Default::default(),
        geometry: DrawingGeometry::Circle {
            placement: CoordinateFrame3::try_new(
                Point3::new(f64::MAX, 0., 0.),
                Vector3::new(1., 0., 0.),
                Vector3::new(0., 1., 0.),
            )
            .unwrap(),
            radius: f64::MAX,
        },
    });
    d.next_entity_id = 3;
    d.scopes[0].entities.push(2);
    for reverse in [false, true] {
        if reverse {
            d.scopes[0].entities.reverse();
        }
        assert!(recompute_ocdraw_document_bounds(&mut d).is_err());
        assert_eq!(d.scopes[0].bounds, before[0].bounds);
        assert_eq!(d.scopes[0].bounds_quality, before[0].bounds_quality);
    }
}

#[test]
fn supplied_enclosing_claim_is_preserved_without_promoting_glyph_evidence() {
    let mut d = framed(false, "mm");
    d.scopes[0].bounds = Some(Bounds3d::new(
        Point3::new(-100., -100., -100.),
        Point3::new(100., 100., 100.),
    ));
    d.scopes[0].bounds_quality = Some(OcdrawBoundsQuality::Enclosing);
    let derived = assess_ocdraw_document_bounds(&d).unwrap();
    assert_eq!(derived[0].quality, Some(OcdrawBoundsQuality::Estimated));
    assert!(!derived[0].enclosure_verified);
    let output = encode_ocdraw_document(&d).unwrap();
    assert_eq!(
        load_ocdraw_bytes(output.bytes()).unwrap().scopes(),
        d.scopes
    );
    d.scopes[0].bounds_quality = Some(OcdrawBoundsQuality::Partial);
    assert!(validate_ocdraw_document(&d).is_err());
}
