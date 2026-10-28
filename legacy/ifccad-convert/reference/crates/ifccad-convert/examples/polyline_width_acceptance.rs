#[path = "polyline_width_acceptance/selection.rs"]
mod selection;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Handle,
    Polyline2DWriteMode,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Cursor, path::PathBuf};
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn metadata() -> IfccadTargetMetadata {
    IfccadTargetMetadata {
        drawing_id: 7,
        header: IfccadHeader {
            id: "polyline-width-original-acceptance".into(),
            data_version: "0.1.0".into(),
            author: "OpenAEC".into(),
            timestamp: "2026-10-23T00:00:00Z".into(),
        },
    }
}
fn facts(d: &CadDocument) -> serde_json::Value {
    let dim = d
        .entities()
        .find_map(|e| {
            if let EntityType::Dimension(v) = e {
                Some(v)
            } else {
                None
            }
        })
        .unwrap();
    let picture = d.block_records.get(&dim.base().block_name).unwrap();
    assert_eq!(picture.entity_handles.len(), 9);
    let parts: Vec<_> = picture
        .entity_handles
        .iter()
        .map(|h| d.get_entity(*h).unwrap())
        .collect();
    let inserts: Vec<_> = parts
        .iter()
        .filter_map(|e| {
            if let EntityType::Insert(i) = e {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(inserts.len(), 2);
    for i in &inserts {
        let marker = d.block_records.get(&i.block_name).unwrap();
        let wide: Vec<_> = marker
            .entity_handles
            .iter()
            .filter_map(|h| {
                if let EntityType::LwPolyline(p) = d.get_entity(*h).unwrap() {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(wide.len(), 1);
        assert_eq!(wide[0].constant_width.to_bits(), 0.15f64.to_bits());
        assert_eq!(wide[0].common.layer, "0");
        assert_eq!(wide[0].common.color, opencadcodec::Color::ByBlock);
        assert_eq!(
            wide[0].common.line_weight,
            opencadcodec::LineWeight::ByLayer
        );
    }
    let text = parts
        .iter()
        .find_map(|e| {
            if let EntityType::MText(m) = e {
                Some(m)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(text.rectangle_width, 0.);
    assert!(text.value.contains("\\S"));
    json!({"dimension":dim,"pictureParts":parts,"markerOccurrences":inserts,"mText":text})
}
// Independent semantic comparison of anchors and stored occurrence operands.
fn compare_facts(source: &serde_json::Value, target: &serde_json::Value) {
    fn close(a: &serde_json::Value, b: &serde_json::Value) {
        match a {
            serde_json::Value::Number(n) => assert!(
                (n.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-10,
                "anchor/operand drift: {a} -> {b}"
            ),
            serde_json::Value::Object(o) => {
                for (k, v) in o {
                    close(v, &b[k]);
                }
            }
            _ => assert_eq!(a, b),
        }
    }
    let sd = source["dimension"].as_object().unwrap();
    let (kind, dim) = sd.iter().next().unwrap();
    let returned = &target["dimension"][kind];
    assert!(!returned.is_null());
    for field in ["first_point", "second_point", "definition_point"] {
        close(&dim[field], &returned[field]);
    }
    for field in ["definition_point", "text_middle_point", "normal"] {
        close(&dim["base"][field], &returned["base"][field]);
    }
    for (a, b) in source["markerOccurrences"]
        .as_array()
        .unwrap()
        .iter()
        .zip(target["markerOccurrences"].as_array().unwrap())
    {
        for field in [
            "insert_point",
            "normal",
            "rotation",
            "x_scale",
            "y_scale",
            "z_scale",
        ] {
            close(&a[field], &b[field]);
        }
        assert_eq!(a["common"]["layer"], b["common"]["layer"]);
        assert_eq!(a["common"]["color"], b["common"]["color"]);
    }
    let a = cad_text::parse_mtext(
        source["mText"]["value"].as_str().unwrap(),
        Default::default(),
    )
    .unwrap();
    let b = cad_text::parse_mtext(
        target["mText"]["value"].as_str().unwrap(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(a.content, b.content);
    assert_eq!(a.character_format, b.character_format);
    assert_eq!(a.paragraph_format, b.paragraph_format);
    close(
        &source["mText"]["insertion_point"],
        &target["mText"]["insertion_point"],
    );
    for (a, b) in source["pictureParts"]
        .as_array()
        .unwrap()
        .iter()
        .zip(target["pictureParts"].as_array().unwrap())
    {
        assert_eq!(
            a.as_object().unwrap().keys().next(),
            b.as_object().unwrap().keys().next()
        );
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.windows(2)
            .find(|p| p[0] == flag)
            .map(|p| p[1].clone())
            .ok_or_else(|| format!("missing {flag}"))
    };
    let source = PathBuf::from(value("--source")?);
    let output = PathBuf::from(value("--output-dir")?);
    let h = value("--dimension-handle")?;
    let handle = Handle::new(u64::from_str_radix(h.trim_start_matches("0x"), 16)?);
    let bytes = fs::read(&source)?;
    let sha = hash(&bytes);
    let dwg = source.extension().is_some_and(|e| e == "dwg");
    let expected = if dwg {
        "0e8faaca949c9429c92240082d9ea4d3524aca5baf7c7815ec439245c321b528"
    } else {
        "c2934f616ecd7097e773b85e5d6e0548d21ccc635abcc8e7276f7d4a585a7ae2"
    };
    if sha != expected {
        return Err("original source SHA-256 mismatch".into());
    }
    fs::create_dir(&output)?;
    let original = if dwg {
        DwgReader::from_stream(Cursor::new(bytes)).read()?
    } else {
        DxfReader::from_reader(Cursor::new(bytes))?.read()?
    };
    let (selected, manifest) = selection::select_dimension_source_closure(&original, handle)?;
    assert_eq!(manifest.pictures, ["*D8"]);
    assert!(
        manifest.entities.contains(&"660".into())
            && manifest.entities.contains(&"661".into())
            && manifest.entities.contains(&"4A6".into())
            && manifest.entities.contains(&"662".into())
    );
    let source_facts = facts(&selected);
    let imported = cad_document_to_encoded_ifccad(&selected, metadata(), Default::default())?;
    let native = imported.validated_source().document();
    let encoded = imported.encoded();
    fs::write(output.join("selected.ifcx"), encoded.bytes())?;
    let strict = load_ifccad_bytes(encoded.bytes(), Default::default())?;
    assert_eq!(native, strict.document());
    let cbor = encode_ifccad_cbor_document(native)?;
    fs::write(output.join("selected.ifccad.cbor"), cbor.bytes())?;
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())?.document(),
        native
    );
    #[cfg(feature = "ifccad-cbor-zstd")]
    {
        let z = encode_ifccad_cbor_zstd_document(native, Default::default())?;
        fs::write(output.join("selected.ifccad.cbor.zst"), z.bytes())?;
        assert_eq!(
            load_ifccad_cbor_zstd_bytes(z.bytes(), Default::default())?.document(),
            native
        );
    }
    let converted = ifccad_document_to_cad_document(native, Default::default())?;
    let mut target = converted.document().clone();
    target.version = opencadcodec::DxfVersion::AC1032;
    let mut outputs = vec![];
    for format in ["ascii.dxf", "binary.dxf", "dwg"] {
        let b = if format == "dwg" {
            DwgWriter::write_to_vec(&target)?
        } else {
            let mut w = DxfWriter::new(&target)
                .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
            w.set_binary(format == "binary.dxf");
            w.write_to_vec()?
        };
        fs::write(output.join(format!("roundtrip.{format}")), &b)?;
        let back = if format == "dwg" {
            assert_eq!(&b[..6], b"AC1032");
            DwgReader::from_stream(Cursor::new(b.clone())).read()?
        } else {
            DxfReader::from_reader(Cursor::new(b.clone()))?.read()?
        };
        assert_eq!(back.version, opencadcodec::DxfVersion::AC1032);
        let returned_facts = facts(&back);
        compare_facts(&source_facts, &returned_facts);
        let read = cad_document_to_encoded_ifccad(&back, metadata(), Default::default())?;
        load_ifccad_bytes(read.encoded().bytes(), Default::default())?;
        let wide: Vec<_> = read
            .geometry_assessment()
            .polyline_widths()
            .iter()
            .filter(|a| a.effective_widths.iter().flatten().any(|w| *w > 0.))
            .collect();
        assert!(wide.len() >= 3);
        assert!(wide.iter().all(|a| a
            .segments
            .iter()
            .all(|s| s.band_coverage == IfccadPolylineWidthBandCoverage::Proven)));
        outputs.push(json!({"format":format,"sha256":hash(&b),"strictNativeReadback":true,"facts":returned_facts,"widthAssessment":format!("{:?}",wide),"dimensionAssessment":format!("{:?}",read.dimension_assessment()),"diagnostics":read.diagnostics()}));
    }
    let native_hashes = [
        "selected.ifcx",
        "selected.ifccad.cbor",
        "selected.ifccad.cbor.zst",
    ]
    .into_iter()
    .filter_map(|name| fs::read(output.join(name)).ok().map(|b| (name, hash(&b))))
    .collect::<std::collections::BTreeMap<_, _>>();
    let report = json!({"source":source.file_name().unwrap().to_string_lossy(),"sourceSha256":sha,"nativeSha256":native_hashes,"selection":manifest,"resourceSelection":"all source table records retained conservatively, plus their marker block closure; unrelated entity instances excluded","sourceFacts":source_facts,"importDiagnostics":imported.diagnostics(),"exportDiagnostics":converted.diagnostics(),"widthAssessment":format!("{:?}",imported.geometry_assessment().polyline_widths()),"dimensionAssessment":format!("{:?}",imported.dimension_assessment()),"outputs":outputs,"hostObservation":"pending; no host rendering qualification inferred from semantic checks"});
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("passed original storage acceptance; source={sha}");
    Ok(())
}
#[cfg(test)]
mod tests {
    #[path = "../../tests/support/dimension_graphics.rs"]
    mod fixture;
    #[test]
    fn real_source_selection_keeps_complete_picture_and_resources() {
        let mut source = fixture::picture(true);
        let h = source
            .entities()
            .find(|e| matches!(e, opencadcodec::EntityType::Dimension(_)))
            .unwrap()
            .common()
            .handle;
        let unrelated = source
            .add_entity(opencadcodec::EntityType::Ray(
                opencadcodec::entities::Ray::new(Default::default(), opencadcodec::Vector3::UNIT_X),
            ))
            .unwrap();
        let (d, m) = super::selection::select_dimension_source_closure(&source, h).unwrap();
        assert!(d.get_entity(unrelated).is_none());
        for id in &m.entities {
            let id = opencadcodec::Handle::new(u64::from_str_radix(id, 16).unwrap());
            assert_eq!(d.get_entity(id), source.get_entity(id));
        }
        assert_eq!(
            d.block_records.get("*DimensionPicture"),
            source.block_records.get("*DimensionPicture")
        );
    }
}
