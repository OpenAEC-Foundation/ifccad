use ocdraw::ocdraw::{encode_ocdraw_document, load_ocdraw_bytes, validate_ocdraw_document};
use ocdraw_convert::*;
use opencadcodec::entities::{Insert, Spline};
use opencadcodec::{
    BlockRecord, CadDocument, DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter, EntityType,
    Handle, Line, Vector3,
};
use std::io::Cursor;

fn cubic() -> Spline {
    let mut s = Spline::new();
    s.degree = 3;
    s.control_points = vec![
        Vector3::new(0., 0., 0.),
        Vector3::new(1., 2., 0.),
        Vector3::new(2., 2., 0.),
        Vector3::new(3., 0., 0.),
    ];
    s.knots = vec![0., 0., 0., 0., 1., 1., 1., 1.];
    s
}
fn physical(doc: &CadDocument, dwg: bool) -> Result<CadDocument, String> {
    let mut doc = doc.clone();
    doc.version = DxfVersion::AC1032;
    if dwg {
        DwgReader::from_stream(Cursor::new(
            DwgWriter::write_to_vec(&doc).map_err(|e| e.to_string())?,
        ))
        .read()
        .map_err(|e| e.to_string())
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(&doc)
                .write_to_vec()
                .map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())
    }
}
fn preserved(source: &CadDocument) -> CadDocument {
    let captured = cad_document_to_ocdraw_document(
        source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    validate_ocdraw_document(captured.document()).unwrap();
    let encoded = encode_ocdraw_document(captured.document()).unwrap();
    drop(captured);
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    let restored = ocdraw_source_to_cad_document(&loaded, OcdrawToCadOptions::default()).unwrap();
    assert!(
        restored
            .preservation_report()
            .entries()
            .iter()
            .all(|e| e.result == OcdrawPreservationResult::RestoredTyped),
        "{:?}",
        restored.preservation_report()
    );
    restored.into_document()
}
fn floats(v: &[f64]) -> Vec<u64> {
    v.iter().map(|v| v.to_bits()).collect()
}
fn spline_semantics(a: &Spline, b: &Spline) {
    assert_eq!(a.degree, b.degree);
    assert_eq!(a.flags, b.flags);
    assert_eq!(floats(&a.knots), floats(&b.knots));
    assert_eq!(floats(&a.weights), floats(&b.weights));
    for (a, b) in [
        (&a.control_points, &b.control_points),
        (&a.fit_points, &b.fit_points),
    ] {
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(b) {
            assert_eq!(
                [a.x.to_bits(), a.y.to_bits(), a.z.to_bits()],
                [b.x.to_bits(), b.y.to_bits(), b.z.to_bits()]
            );
        }
    }
    for (a, b) in [
        (a.normal, b.normal),
        (a.begin_tangent, b.begin_tangent),
        (a.end_tangent, b.end_tangent),
    ] {
        assert_eq!(
            [a.x.to_bits(), a.y.to_bits(), a.z.to_bits()],
            [b.x.to_bits(), b.y.to_bits(), b.z.to_bits()]
        );
    }
    assert_eq!(
        floats(&[a.knot_tolerance, a.control_tolerance, a.fit_tolerance]),
        floats(&[b.knot_tolerance, b.control_tolerance, b.fit_tolerance])
    );
    assert_eq!(
        (
            a.knot_parameterization,
            a.cv_frame_visible,
            a.dwg_flags1,
            a.dxf_flags
        ),
        (
            b.knot_parameterization,
            b.cv_frame_visible,
            b.dwg_flags1,
            b.dxf_flags
        )
    );
    assert_eq!(a.common.layer, b.common.layer);
    assert_eq!(a.common.color, b.common.color);
    assert_eq!(a.common.line_weight, b.common.line_weight);
    assert_eq!(a.common.transparency, b.common.transparency);
    assert_eq!(a.common.color_name, b.common.color_name);
    assert_eq!(a.common.invisible, b.common.invisible);
    assert_eq!(a.common.linetype, b.common.linetype);
    assert_eq!(
        a.common.linetype_scale.to_bits(),
        b.common.linetype_scale.to_bits()
    );
}
fn owner_entities<'a>(doc: &'a CadDocument, name: &str) -> Vec<&'a EntityType> {
    let r = if let Some(layout_name) = name.strip_prefix("paper:") {
        let owner = doc
            .objects
            .values()
            .find_map(|o| match o {
                opencadcodec::objects::ObjectType::Layout(l) if l.name == layout_name => {
                    Some(l.block_record)
                }
                _ => None,
            })
            .unwrap();
        doc.block_records
            .iter()
            .find(|r| r.handle == owner)
            .unwrap()
    } else {
        doc.block_records.get(name).unwrap_or_else(|| {
            panic!(
                "missing owner {name}; names {:?}",
                doc.block_records
                    .iter()
                    .map(|r| &r.name)
                    .collect::<Vec<_>>()
            )
        })
    };
    r.entity_handles
        .iter()
        .filter_map(|h| doc.get_entity(*h))
        .filter(|e| !matches!(e, EntityType::Block(_) | EntityType::BlockEnd(_)))
        .collect()
}
fn compare(direct: &CadDocument, via: &CadDocument, owner_names: &[&str]) {
    for name in owner_names {
        let a = owner_entities(direct, name);
        let b = owner_entities(via, name);
        assert_eq!(a.len(), b.len(), "{name}");
        for (a, b) in a.into_iter().zip(b) {
            assert_eq!(
                a.as_entity().entity_type(),
                b.as_entity().entity_type(),
                "{name}"
            );
            match (a, b) {
                (EntityType::Spline(a), EntityType::Spline(b)) => spline_semantics(a, b),
                (EntityType::Line(a), EntityType::Line(b)) => {
                    assert_eq!(a.start, b.start);
                    assert_eq!(a.end, b.end);
                }
                (EntityType::Insert(a), EntityType::Insert(b)) => {
                    assert_eq!(a.block_name, b.block_name);
                    assert_eq!(a.insert_point, b.insert_point);
                }
                _ => {}
            }
        }
    }
}

#[test]
fn literal_cubic_survives_ocdraw_and_both_real_codecs() {
    let source = DxfReader::from_reader(Cursor::new(
        include_bytes!("fixtures/splines/open-cubic.dxf").as_slice(),
    ))
    .unwrap()
    .read()
    .unwrap();
    let s = source
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(s.degree, 3);
    assert_eq!(s.control_points, cubic().control_points);
    assert_eq!(s.knots, vec![0., 0., 0., 0., 1., 1., 1., 1.]);
    let target = preserved(&source);
    for dwg in [false, true] {
        compare(
            &physical(&source, dwg).unwrap(),
            &physical(&target, dwg).unwrap(),
            &["*Model_Space"],
        );
    }
}

fn add_mixed(source: &mut CadDocument, owner: Handle) {
    let mut a = Line::from_coords(-1., 0., 0., 0., 0., 0.);
    a.common.owner_handle = owner;
    source.add_entity(EntityType::Line(a)).unwrap();
    let mut s = cubic();
    s.common.owner_handle = owner;
    source.add_entity(EntityType::Spline(s)).unwrap();
    let mut b = Line::from_coords(3., 0., 0., 4., 0., 0.);
    b.common.owner_handle = owner;
    source.add_entity(EntityType::Line(b)).unwrap();
}

#[test]
fn direct_vs_preservation_exchange_in_every_supported_owner_and_nested_occurrence() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 4;
    let model = source.header.model_space_block_handle;
    add_mixed(&mut source, model);
    source.add_layout("Sheet").unwrap();
    let paper = source
        .objects
        .values()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Sheet" => {
                Some(l.block_record)
            }
            _ => None,
        })
        .unwrap();
    add_mixed(&mut source, paper);
    source.add_layout("SecondSheet").unwrap();
    let second = source
        .objects
        .values()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "SecondSheet" => {
                Some(l.block_record)
            }
            _ => None,
        })
        .unwrap();
    add_mixed(&mut source, second);
    let mut leaf = BlockRecord::new("SplineLeaf");
    leaf.handle = source.allocate_handle();
    let leaf_owner = leaf.handle;
    source.block_records.add(leaf).unwrap();
    add_mixed(&mut source, leaf_owner);
    let mut parent = BlockRecord::new("SplineParent");
    parent.handle = source.allocate_handle();
    let parent_owner = parent.handle;
    source.block_records.add(parent).unwrap();
    let mut inner = Insert::new("SplineLeaf", Vector3::new(5., 0., 0.));
    inner.common.owner_handle = parent_owner;
    source.add_entity(EntityType::Insert(inner)).unwrap();
    for x in [0., 10.] {
        source
            .add_entity(EntityType::Insert(Insert::new(
                "SplineParent",
                Vector3::new(x, 0., 0.),
            )))
            .unwrap();
    }
    let mut unused = BlockRecord::new("UnusedSpline");
    unused.handle = source.allocate_handle();
    let unused_owner = unused.handle;
    source.block_records.add(unused).unwrap();
    add_mixed(&mut source, unused_owner);
    let target = preserved(&source);
    for dwg in [false, true] {
        compare(
            &physical(&source, dwg).unwrap(),
            &physical(&target, dwg).unwrap(),
            &[
                "*Model_Space",
                "paper:Sheet",
                "paper:SecondSheet",
                "SplineLeaf",
                "SplineParent",
                "UnusedSpline",
            ],
        );
    }
}

#[test]
fn additional_variants_compare_against_direct_codec_without_capture_whitelist() {
    let mut quadratic = cubic();
    quadratic.degree = 2;
    quadratic.control_points.pop();
    quadratic.knots = vec![0., 0., 0., 1., 1., 1.];
    let mut rational = cubic();
    rational.flags.rational = true;
    rational.weights = vec![1., 2., 0.5, 1.];
    let mut periodic = cubic();
    periodic.flags.closed = true;
    periodic.flags.periodic = true;
    let mut spatial = cubic();
    spatial.control_points[1].z = 2.;
    spatial.normal = Vector3::new(0., 1., 1.);
    let mut fit_only = cubic();
    fit_only.fit_points = fit_only.control_points.clone();
    fit_only.control_points.clear();
    fit_only.knots.clear();
    let mut mixed = cubic();
    mixed.fit_points = vec![Vector3::new(0., 0., 0.), Vector3::new(3., 0., 0.)];
    mixed.begin_tangent = Vector3::new(1., 2., 0.);
    mixed.end_tangent = Vector3::new(1., -2., 0.);
    mixed.fit_tolerance = 0.001;
    for (name, spline) in [
        ("quadratic", quadratic),
        ("rational", rational),
        ("closed-periodic", periodic),
        ("spatial", spatial),
        ("fit-only", fit_only),
        ("mixed", mixed),
    ] {
        let mut source = CadDocument::new();
        source.add_entity(EntityType::Spline(spline)).unwrap();
        let target = preserved(&source);
        for dwg in [false, true] {
            match physical(&source, dwg) {
                Err(error) => eprintln!(
                    "{name}/{} direct sourceCodecRejected: {error}",
                    if dwg { "DWG" } else { "DXF" }
                ),
                Ok(direct) => {
                    let via = physical(&target, dwg).unwrap_or_else(|e| {
                        panic!("{name}/{dwg}: direct succeeded but preservation chain failed: {e}")
                    });
                    compare(&direct, &via, &["*Model_Space"]);
                    eprintln!(
                        "{name}/{} direct-vs-preservation equivalent ({} splines read)",
                        if dwg { "DWG" } else { "DXF" },
                        direct
                            .entities()
                            .filter(|e| matches!(e, EntityType::Spline(_)))
                            .count()
                    );
                }
            }
        }
    }
}
