use ifccad_convert::*;
use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter};
use std::io::Cursor;
fn drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/hatch-gradient.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
fn hatch(d: &IfccadDocument) -> &IfccadHatch {
    d.model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h)
            } else {
                None
            }
        })
        .unwrap()
}
fn hatch_mut(d: &mut IfccadDocument) -> &mut IfccadHatch {
    d.model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                Some(h)
            } else {
                None
            }
        })
        .unwrap()
}
fn exchange(d: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
#[test]
fn nine_gradient_definitions_authorship_modes_and_area_rules_survive_actual_files() {
    for (index, kind) in [
        HatchGradientType::Linear,
        HatchGradientType::Cylinder,
        HatchGradientType::InverseCylinder,
        HatchGradientType::Spherical,
        HatchGradientType::InverseSpherical,
        HatchGradientType::Hemispherical,
        HatchGradientType::InverseHemispherical,
        HatchGradientType::Curved,
        HatchGradientType::InverseCurved,
    ]
    .into_iter()
    .enumerate()
    {
        for one in [false, true] {
            let mut d = drawing();
            let h = hatch_mut(&mut d);
            h.area_rule = [
                HatchAreaRule::Normal,
                HatchAreaRule::Outer,
                HatchAreaRule::Ignore,
            ][index % 3];
            let g = h.gradient.as_mut().unwrap();
            g.gradient_type = Some(kind);
            g.color_mode = if one {
                HatchGradientColorMode::OneColor
            } else {
                HatchGradientColorMode::TwoColor
            };
            if !one {
                g.colors = Some([
                    IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1).unwrap(),
                    IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1).unwrap(),
                ]);
            }
            let expected = h.gradient.clone();
            let rule = h.area_rule;
            let association = h
                .loops
                .iter()
                .filter(|l| !l.source_entity_ids.is_empty())
                .count();
            let mut target = ifccad_document_to_cad_document(&d, Default::default())
                .unwrap()
                .into_document();
            target.version = DxfVersion::AC1032;
            for dwg in [false, true] {
                let file = exchange(&target, dwg);
                let back = cad_document_to_ifccad_document(
                    &file,
                    IfccadTargetMetadata {
                        header: d.header.clone(),
                        drawing_id: d.drawing_id,
                    },
                    Default::default(),
                )
                .unwrap();
                let actual = hatch(back.document());
                assert_eq!(actual.fill, HatchFill::Gradient);
                assert_eq!(
                    actual.gradient, expected,
                    "kind {kind:?}, one {one}, dwg {dwg}"
                );
                assert_eq!(actual.area_rule, rule);
                assert_eq!(
                    actual
                        .loops
                        .iter()
                        .filter(|l| !l.source_entity_ids.is_empty())
                        .count(),
                    association
                );
                load_ifccad_bytes(
                    encode_ifccad_document(back.document()).unwrap().bytes(),
                    Default::default(),
                )
                .unwrap();
                assert!(!back.geometry_assessment().is_complete());
            }
        }
    }
}
#[test]
fn hand_authored_gradient_tags_have_independent_native_expectations() {
    let d = drawing();
    let source = DxfReader::from_reader(Cursor::new(
        include_bytes!("../../cad-geometry-convert/tests/fixtures/hatch-gradient-reference.dxf")
            .to_vec(),
    ))
    .unwrap()
    .read()
    .unwrap();
    let back = cad_document_to_ifccad_document(
        &source,
        IfccadTargetMetadata {
            header: d.header.clone(),
            drawing_id: d.drawing_id,
        },
        Default::default(),
    )
    .unwrap();
    let g = hatch(back.document()).gradient.as_ref().unwrap();
    assert_eq!(g.gradient_type, Some(HatchGradientType::InverseSpherical));
    assert_eq!((g.rotation, g.shift, g.tint), (0.75, 0.5, 0.8));
    assert_eq!(g.color_mode, HatchGradientColorMode::OneColor);
    assert_eq!(
        g.colors.as_ref().unwrap()[0].concrete_rgb().unwrap(),
        [220, 80, 30]
    );
    assert_eq!(
        g.colors.as_ref().unwrap()[1].concrete_rgb().unwrap(),
        [30, 60, 150]
    );
    load_ifccad_bytes(
        encode_ifccad_document(back.document()).unwrap().bytes(),
        Default::default(),
    )
    .unwrap();
}
#[test]
fn pinned_disabled_state_is_retained_in_dwg_but_omitted_by_dxf() {
    let mut d = drawing();
    hatch_mut(&mut d).fill = HatchFill::Solid;
    let expected = hatch(&d).gradient.clone();
    let mut target = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    target.version = DxfVersion::AC1032;
    for dwg in [false, true] {
        let file = exchange(&target, dwg);
        let back = cad_document_to_ifccad_document(
            &file,
            IfccadTargetMetadata {
                header: d.header.clone(),
                drawing_id: d.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        assert_eq!(
            hatch(back.document()).gradient,
            if dwg { expected.clone() } else { None }
        );
    }
}

#[test]
fn tilted_basis_with_exact_target_axes_retains_gradient_parameters() {
    let mut d = drawing();
    let h = hatch_mut(&mut d);
    h.placement = IfccadPlacement {
        origin: [0., 0., 4.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 0., 1.],
    };
    let expected = h.gradient.clone();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let target = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    for dwg in [false, true] {
        let file = exchange(&target, dwg);
        let back = cad_document_to_ifccad_document(
            &file,
            IfccadTargetMetadata {
                header: d.header.clone(),
                drawing_id: d.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        assert_eq!(hatch(back.document()).gradient, expected);
    }
}

#[allow(dead_code)]
#[path = "../../../tests/support/hatch.rs"]
mod contours;
#[test]
fn nested_nonuniform_block_gradients_keep_local_authored_parameters() {
    let d = drawing();
    let grad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .document()
        .entities()
        .find_map(|e| {
            if let opencadcodec::EntityType::Hatch(h) = e {
                Some(h.gradient_color.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut cad = contours::cad_nonuniform_block();
    for e in cad.entities_mut() {
        if let opencadcodec::EntityType::Hatch(h) = e {
            h.gradient_color = grad.clone();
        }
    }
    for dwg in [false, true] {
        let file = exchange(&cad, dwg);
        let back = cad_document_to_ifccad_document(
            &file,
            IfccadTargetMetadata {
                header: d.header.clone(),
                drawing_id: d.drawing_id,
            },
            CadToIfccadOptions {
                geometry_tolerance: IfccadGeometryTolerance::drawing_units(1e-8).unwrap(),
                ..Default::default()
            },
        )
        .unwrap();
        let g = back
            .document()
            .blocks
            .iter()
            .flat_map(|b| &b.entities)
            .filter_map(IfccadEntity::as_native)
            .find_map(|e| {
                if let IfccadEntityKind::Hatch(h) = &e.kind {
                    h.gradient.as_ref()
                } else {
                    None
                }
            })
            .unwrap();
        load_ifccad_bytes(
            encode_ifccad_document(back.document()).unwrap().bytes(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(g.gradient_type, Some(HatchGradientType::InverseSpherical));
        assert_eq!((g.rotation, g.shift, g.tint), (0.75, 0.5, 0.8));
        assert_eq!(g.color_mode, HatchGradientColorMode::OneColor);
        assert!(!back.geometry_assessment().is_complete());
    }
}
