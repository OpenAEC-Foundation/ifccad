//! Required local acceptance with the original external drawing, not a benchmark.
use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::*;
use ifccad_convert::{opencadcodec::*, *};
use sha2::{Digest, Sha256};
use std::{io::Cursor, path::PathBuf};
fn read(format: &str, bytes: &[u8]) -> CadDocument {
    if format == "dwg" {
        DwgReader::from_stream(Cursor::new(bytes.to_vec()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(bytes.to_vec()))
            .unwrap()
            .read()
            .unwrap()
    }
}
#[test]
#[ignore = "external original AC1024 files; required explicit local Hatch B acceptance"]
fn original_ac1024_foundation_hatches_survive_every_offered_target() {
    let root = PathBuf::from(
        std::env::var_os("IFCCAD_HATCH_FOUNDATION_INPUT_ROOT")
            .expect("set original foundation input directory"),
    );
    let stem = "2705_model Funderingsherstel - Constructie - Sheet - CP-21 - Constructietekening";
    for (input, hash, length, control) in [
        (
            "dwg",
            "24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5",
            239012,
            "9A7",
        ),
        (
            "dxf",
            "054317ae053be20d9c0f9c8650b5070f76826a7eede9690cd9c572700ea6149c",
            1314182,
            "9AA",
        ),
    ] {
        let path = root.join(format!("{stem}.{input}"));
        let bytes = std::fs::read(&path).expect("original input required");
        assert_eq!(bytes.len(), length);
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), hash);
        let source = read(input, &bytes);
        assert_eq!(
            source.dwg_source_version.unwrap_or(source.version),
            DxfVersion::AC1024
        );
        let source_hatches = source
            .entities()
            .filter(|e| matches!(e, EntityType::Hatch(_)))
            .count();
        assert!(source.entities().any(|e| matches!(e, EntityType::Hatch(_))
            && e.common().handle.value() == u64::from_str_radix(control, 16).unwrap()));
        let template = load_ifccad_bytes(
            include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
            Default::default(),
        )
        .unwrap()
        .into_document();
        let native = cad_document_to_ifccad_document(
            &source,
            IfccadTargetMetadata {
                header: template.header,
                drawing_id: template.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        let encoded = encode_ifccad_document(native.document()).unwrap();
        let checked = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        let expected =
            ifccad_document_to_cad_document(checked.document(), Default::default()).unwrap();
        assert!(
            !expected
                .diagnostics()
                .iter()
                .any(|d| d.code == "hatch-classic-settings-loss"),
            "original model recipe omission: {:?}",
            expected.diagnostics()
        );
        let hatches = expected
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Hatch(_)))
            .count();
        let deferred = native
            .diagnostics()
            .iter()
            .filter(|d| d.code == "hatch-unsupported")
            .collect::<Vec<_>>();
        assert_eq!(
            deferred.len(),
            1,
            "unexpected unsupported Hatch: {deferred:?}"
        );
        assert!(
            deferred[0].location.ends_with("/paths.edges.spline"),
            "{deferred:?}"
        );
        let token = deferred[0]
            .location
            .split('/')
            .nth(1)
            .unwrap()
            .trim_start_matches("0x");
        let skipped = source
            .get_entity(Handle::new(u64::from_str_radix(token, 16).unwrap()))
            .unwrap();
        assert!(
            matches!(skipped,EntityType::Hatch(h) if h.paths.iter().any(|p|p.edges.iter().any(|e|matches!(e,opencadcodec::entities::hatch::BoundaryEdge::Spline(_)))))
        );
        assert_eq!(source_hatches, 85);
        assert_eq!(hatches, 84);
        // Compare original semantic groups through confirmed import/export maps,
        // independently of the physical writer/reader's mutual agreement.
        for e in source.entities() {
            if let EntityType::Hatch(original) = e {
                if original.common.handle == skipped.common().handle {
                    continue;
                }
                let id = native
                    .mappings()
                    .entities
                    .ifccad_id(original.common.handle)
                    .unwrap();
                let handle = expected.mappings().entities.cad_handle(id).unwrap();
                let EntityType::Hatch(mapped) = expected.document().get_entity(handle).unwrap()
                else {
                    panic!()
                };
                if !original.is_solid && !original.gradient_color.enabled {
                    assert_eq!(
                        mapped.pattern, original.pattern,
                        "raw source recipe {}",
                        original.common.handle
                    );
                    assert_eq!(
                        mapped.pattern_angle.to_bits(),
                        original.pattern_angle.to_bits()
                    );
                    assert_eq!(
                        mapped.pattern_scale.to_bits(),
                        original.pattern_scale.to_bits()
                    );
                    assert_eq!(mapped.is_double, original.is_double);
                    assert_eq!(mapped.pattern_type, original.pattern_type);
                    assert_eq!(
                        mapped.stored_pattern_origin(),
                        original.stored_pattern_origin(),
                        "origin presence {}",
                        original.common.handle
                    );
                    let native_hatch = checked
                        .document()
                        .model
                        .entities
                        .iter()
                        .chain(checked.document().blocks.iter().flat_map(|b| &b.entities))
                        .chain(
                            checked
                                .document()
                                .paper_layouts
                                .iter()
                                .flat_map(|p| &p.entities),
                        )
                        .filter_map(IfccadEntity::as_native)
                        .find(|e| e.id == id)
                        .unwrap();
                    let IfccadEntityKind::Hatch(h) = &native_hatch.kind else {
                        panic!()
                    };
                    let c = h.classic_settings.as_ref().unwrap();
                    for (a, b) in c
                        .definition_lines
                        .as_ref()
                        .unwrap()
                        .iter()
                        .zip(&original.pattern.lines)
                    {
                        assert_eq!(a.angle.to_bits(), b.angle.to_bits());
                        assert_eq!(
                            a.base_point.map(f64::to_bits),
                            [b.base_point.x, b.base_point.y].map(f64::to_bits)
                        );
                        assert_eq!(
                            a.offset_vector.map(f64::to_bits),
                            [b.offset.x, b.offset.y].map(f64::to_bits)
                        );
                        assert_eq!(
                            a.dash_lengths
                                .iter()
                                .map(|v| v.to_bits())
                                .collect::<Vec<_>>(),
                            b.dash_lengths
                                .iter()
                                .map(|v| v.to_bits())
                                .collect::<Vec<_>>()
                        );
                    }
                }
                assert_eq!(mapped.is_associative, original.is_associative);
                assert_eq!(mapped.paths.len(), original.paths.len());
                for (index, (a, b)) in original.paths.iter().zip(&mapped.paths).enumerate() {
                    assert_eq!(a.flags.bits() & !2, b.flags.bits() & !2);
                    let sources = a
                        .boundary_handles
                        .iter()
                        .map(|h| {
                            native
                                .mappings()
                                .entities
                                .ifccad_id(*h)
                                .and_then(|id| expected.mappings().entities.cad_handle(id))
                        })
                        .collect::<Option<Vec<_>>>();
                    if let Some(sources) = sources {
                        assert_eq!(sources, b.boundary_handles);
                    } else {
                        assert!(b.boundary_handles.is_empty());
                        assert!(native
                            .diagnostics()
                            .iter()
                            .chain(expected.diagnostics())
                            .any(|d| d.code == "hatch-source-group-loss"
                                && d.location == format!("entity/{id}/loops/{index}/sources")));
                    }
                }
            }
        }
        let dimensions = expected
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Dimension(_)))
            .count();
        assert_eq!(
            dimensions,
            source
                .entities()
                .filter(|e| matches!(e, EntityType::Dimension(_)))
                .count()
        );
        for output in ["dxf", "dwg"] {
            for version in ["AC1015", "AC1018", "AC1021", "AC1024", "AC1027", "AC1032"] {
                // The original-version Explorer path is exercised end-to-end.
                // Other cells reuse the prepared complete model and invoke the
                // same production writers/readers, without repeated preparation.
                let out = if version == "AC1024" {
                    let out = viewer::export_ifccad_bytes_with_options(
                        "foundation.ifcx",
                        encoded.bytes(),
                        output,
                        version,
                        "{}",
                    );
                    assert!(
                        out["failure"].is_null(),
                        "{input}→{output}/{version}: {}",
                        out["failure"]
                    );
                    Some(out)
                } else {
                    None
                };
                let data = if let Some(out) = &out {
                    STANDARD
                        .decode(
                            out["export"]["download"]["base64"]
                                .as_str()
                                .expect("download required"),
                        )
                        .unwrap()
                } else {
                    let mut target = expected.document().clone();
                    target.version = DxfVersion::from_version_string(version);
                    if output == "dxf" {
                        DxfWriter::new(&target).write_to_vec().unwrap()
                    } else {
                        DwgWriter::write_to_vec(&target).unwrap()
                    }
                };
                let actual = read(output, &data);
                assert_eq!(
                    actual.dwg_source_version.unwrap_or(actual.version).as_str(),
                    version
                );
                let mut count = 0;
                let mut physical_recipe_differences = Vec::new();
                for e in expected.document().entities() {
                    if let EntityType::Hatch(w) = e {
                        let EntityType::Hatch(r) = actual
                            .get_entity(w.common.handle)
                            .expect("confirmed target Hatch identity")
                        else {
                            panic!("target kind changed")
                        };
                        assert_eq!(r.paths.len(), w.paths.len());
                        assert_eq!(r.is_solid, w.is_solid);
                        assert_eq!(r.gradient_color.enabled, w.gradient_color.enabled);
                        assert_eq!(r.style, w.style);
                        assert_eq!(r.pattern_type, w.pattern_type);
                        assert_eq!(r.stored_pattern_origin(), w.stored_pattern_origin());
                        assert_eq!(r.pattern_scale.to_bits(), w.pattern_scale.to_bits());
                        assert_eq!(r.is_double, w.is_double);
                        assert_eq!(r.pattern.lines.len(), w.pattern.lines.len());
                        let mut angle_check = |field: String, a: f64, b: f64| {
                            if a.to_bits() != b.to_bits() {
                                assert_eq!(output, "dxf", "DWG recipe scalar changed: {field}");
                                assert_eq!(
                                    b.to_bits(),
                                    a.to_degrees().to_radians().to_bits(),
                                    "new unclassified physical change: {field}"
                                );
                                physical_recipe_differences.push(serde_json::json!({"handle":w.common.handle.to_string(),"field":field,"expected":a,"actual":b,"code":"TARGET_CODEC_HATCH_CLASSIC_SETTINGS_LOSS"}));
                            }
                        };
                        angle_check("rotation".into(), w.pattern_angle, r.pattern_angle);
                        for (i, (a, b)) in w.pattern.lines.iter().zip(&r.pattern.lines).enumerate()
                        {
                            assert_eq!(
                                [a.base_point.x, a.base_point.y].map(f64::to_bits),
                                [b.base_point.x, b.base_point.y].map(f64::to_bits)
                            );
                            assert_eq!(
                                [a.offset.x, a.offset.y].map(f64::to_bits),
                                [b.offset.x, b.offset.y].map(f64::to_bits)
                            );
                            assert_eq!(
                                a.dash_lengths
                                    .iter()
                                    .map(|v| v.to_bits())
                                    .collect::<Vec<_>>(),
                                b.dash_lengths
                                    .iter()
                                    .map(|v| v.to_bits())
                                    .collect::<Vec<_>>()
                            );
                            angle_check(format!("definitionLines/{i}/angle"), a.angle, b.angle);
                        }
                        assert_eq!(r.pattern.name, w.pattern.name);
                        assert_eq!(r.seed_points, w.seed_points);
                        assert_eq!(
                            r.pixel_size,
                            if w.paths.iter().any(|p| p.flags.is_derived()) {
                                w.pixel_size
                            } else {
                                0.
                            }
                        );
                        assert_eq!(
                            r.is_associative,
                            w.is_associative
                                && w.paths.iter().any(|p| !p.boundary_handles.is_empty())
                        );
                        for (a, b) in w.paths.iter().zip(&r.paths) {
                            assert_eq!(a.flags, b.flags);
                            assert_eq!(a.boundary_handles, b.boundary_handles);
                        }
                        count += 1;
                    }
                }
                assert_eq!(count, hatches);
                assert_eq!(
                    actual
                        .entities()
                        .filter(|e| matches!(e, EntityType::Dimension(_)))
                        .count(),
                    dimensions
                );
                for e in expected.document().entities() {
                    if let EntityType::Dimension(w) = e {
                        let EntityType::Dimension(r) =
                            actual.get_entity(w.base().common.handle).unwrap()
                        else {
                            panic!("Dimension missing")
                        };
                        // Anonymous picture names may be renumbered by DWG.
                        // Follow each Dimension's reference and compare content.
                        if !w.base().block_name.is_empty() {
                            let a = expected
                                .document()
                                .block_records
                                .get(&w.base().block_name)
                                .unwrap();
                            let b = actual.block_records.get(&r.base().block_name).unwrap();
                            assert_eq!(
                                a.entity_handles.len(),
                                b.entity_handles.len(),
                                "Dimension picture {}",
                                w.base().block_name
                            );
                            let kinds = |d: &CadDocument, handles: &[Handle]| {
                                let mut kinds = handles
                                    .iter()
                                    .map(|h| {
                                        format!(
                                            "{:?}",
                                            d.get_entity(*h).unwrap().as_entity().entity_type()
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                kinds.sort();
                                kinds
                            };
                            assert_eq!(
                                kinds(expected.document(), &a.entity_handles),
                                kinds(&actual, &b.entity_handles)
                            );
                        }
                    }
                }
                // Export already production-imported, encoded and strict-read
                // these exact output bytes; do not repeat that expensive proof.
                let readback_diagnostics = if let Some(out) = &out {
                    assert_eq!(out["export"]["fileCheck"]["cadReadback"], true);
                    assert_eq!(out["export"]["fileCheck"]["ifccadStrictReadback"], true);
                    out["export"]["diagnostics"].clone()
                } else {
                    let back = cad_document_to_ifccad_document(
                        &actual,
                        IfccadTargetMetadata {
                            header: checked.document().header.clone(),
                            drawing_id: checked.document().drawing_id,
                        },
                        Default::default(),
                    )
                    .unwrap();
                    let bytes = encode_ifccad_document(back.document()).unwrap();
                    load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
                    serde_json::to_value(back.diagnostics()).unwrap()
                };
                println!(
                    "{}",
                    serde_json::json!({"source":input,"format":output,"requestedVersion":version,"actualVersion":actual.dwg_source_version.unwrap_or(actual.version).as_str(),"hatches":count,"dimensions":dimensions,"deferredSplineHatch":deferred[0].location,"bytes":data.len(),"nativeImportDiagnostics":native.diagnostics(),"modelExportDiagnostics":expected.diagnostics(),"readbackDiagnostics":readback_diagnostics,"physicalRecipeDifferences":physical_recipe_differences,"strictReadback":true})
                );
            }
        }
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
            hash
        );
    }
}
