use super::{inventory, recipes, Result};
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{CadDocument, DwgReader, DxfReader, Handle};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    hint::black_box,
    io::{Cursor, Write},
    path::Path,
    time::Instant,
};
pub const CASES: [&str; 6] = [
    "lines-10000",
    "polylines-10000",
    "text-1000",
    "hatches-1000",
    "practice-foundation",
    "practice-test-dxf",
];
pub fn selection(text: &str) -> Result<Vec<String>> {
    let mut chosen = Vec::new();
    for id in text.split(',') {
        if !CASES.contains(&id) || chosen.iter().any(|s| s == id) {
            return Err(format!("unknown, empty or duplicate case {id}").into());
        }
        chosen.push(id.to_string());
    }
    Ok(chosen)
}
pub fn compact_json(input: &[u8]) -> Result<Vec<u8>> {
    let _: Value = serde_json::from_slice(input)?;
    let (mut string, mut escaped) = (false, false);
    let mut out = Vec::with_capacity(input.len());
    for &c in input {
        if string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                string = false;
            }
        } else if c == b'"' {
            string = true;
            out.push(c);
        } else if !matches!(c, b' ' | b'\n' | b'\r' | b'\t') {
            out.push(c);
        }
    }
    Ok(out)
}
// Only the production JSON mapping's documented unordered collection normalization.
fn normalize(d: &mut IfccadDocument) {
    d.text_styles.sort_by_key(|s| s.id);
    d.layers.sort_by_key(|l| l.id.to_string());
    d.blocks.sort_by_key(|b| b.id.to_string());
    d.line_patterns.sort_by_key(|p| p.id.0.to_string());
    d.symbol_sources.sort_by_key(|s| s.id);
    d.paper_layouts.sort_by_key(|p| p.tab_index);
    if let Some(p) = &mut d.preservation {
        p.records.sort_by_key(|r| r.id.0);
    }
    for e in d
        .model
        .entities
        .iter_mut()
        .chain(d.paper_layouts.iter_mut().flat_map(|p| &mut p.entities))
        .chain(d.blocks.iter_mut().flat_map(|b| &mut b.entities))
    {
        if let Some(IfccadNativeEntity {
            kind: IfccadEntityKind::Viewport(v),
            ..
        }) = e.as_native_mut()
        {
            v.layer_overrides.sort_by_key(|r| r.layer_id);
        }
    }
}
fn exact(a: &IfccadDocument, b: &IfccadDocument) -> Result<()> {
    if a != b || encode_ifccad_cbor_document(a)?.bytes() != encode_ifccad_cbor_document(b)?.bytes()
    {
        return Err("typed content or numeric bits differ".into());
    }
    Ok(())
}
pub fn freeze(d: &IfccadDocument) -> Result<IfccadDocument> {
    validate_ifccad_document(d)?;
    let out = encode_ifccad_document(d)?;
    let loaded = load_ifccad_bytes(out.bytes(), Default::default())?.into_document();
    let mut expected = d.clone();
    normalize(&mut expected);
    exact(&expected, &loaded)?;
    Ok(loaded)
}
pub fn artifacts(d: &IfccadDocument) -> Result<[Vec<u8>; 3]> {
    let json = encode_ifccad_document(d)?.into_bytes();
    let compact = compact_json(&json)?;
    let cbor = encode_ifccad_cbor_document(d)?.into_bytes();
    Ok([json, compact, cbor])
}
pub fn qualify(d: &IfccadDocument, variants: &[Vec<u8>; 3]) -> Result<()> {
    for b in &variants[..2] {
        let loaded = load_ifccad_bytes(b, Default::default())?;
        exact(d, loaded.document())?;
    }
    let loaded = load_ifccad_cbor_bytes(&variants[2], Default::default())?;
    exact(d, loaded.document())?;
    Ok(())
}
pub fn statistics(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"samplesMs":samples,"medianMs":sorted[sorted.len()/2],"minMs":sorted[0],"maxMs":sorted[sorted.len()-1]})
}
fn timings(d: &IfccadDocument, variants: &[Vec<u8>; 3]) -> Result<Value> {
    let mut read: [Vec<f64>; 3] = Default::default();
    let mut write: [Vec<f64>; 2] = Default::default();
    for round in 0..6 {
        for step in 0..3 {
            let k = (step + round) % 3;
            let input = black_box(variants[k].as_slice());
            let start = Instant::now();
            let elapsed = if k == 2 {
                let loaded = load_ifccad_cbor_bytes(input, Default::default())?;
                let ms = start.elapsed().as_secs_f64() * 1000.;
                if black_box(loaded.document()) != d {
                    return Err("sample read changed document".into());
                }
                drop(loaded);
                ms
            } else {
                let loaded = load_ifccad_bytes(input, Default::default())?;
                let ms = start.elapsed().as_secs_f64() * 1000.;
                if black_box(loaded.document()) != d {
                    return Err("sample read changed document".into());
                }
                drop(loaded);
                ms
            };
            if round > 0 {
                read[k].push(elapsed);
            }
        }
        for step in 0..2 {
            let k = (step + round) % 2;
            let input = black_box(d);
            let start = Instant::now();
            let out = if k == 0 {
                encode_ifccad_document(input)?.into_bytes()
            } else {
                encode_ifccad_cbor_document(input)?.into_bytes()
            };
            let elapsed = start.elapsed().as_secs_f64() * 1000.;
            if black_box(out.as_slice()) != variants[if k == 0 { 0 } else { 2 }] {
                return Err("sample writer bytes changed".into());
            }
            drop(out);
            if round > 0 {
                write[k].push(elapsed);
            }
        }
    }
    Ok(
        json!({"jsonRead":statistics(&read[0]),"compactJsonRead":statistics(&read[1]),"cborRead":statistics(&read[2]),"jsonWrite":statistics(&write[0]),"cborWrite":statistics(&write[1]),"warmups":1,"samples":5,"excludesFilesystemAndReturnedValueDisposal":true}),
    )
}
fn save(path: &Path, value: &Value) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    f.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn location_handle(location: &str) -> Option<Handle> {
    let text = location
        .strip_prefix("entity/0x")?
        .split(['/', ':', '.', ' '])
        .next()?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let handle = Handle::new(u64::from_str_radix(text, 16).ok()?);
    (!handle.is_null()).then_some(handle)
}
fn source_handle(source: &IfccadGeometryEntitySource) -> Option<Handle> {
    match source {
        IfccadGeometryEntitySource::CadEntity { handle, .. } => Some(*handle),
        IfccadGeometryEntitySource::BlockOccurrence { leaf, .. } => source_handle(leaf),
        IfccadGeometryEntitySource::NativeEntity { .. } => None,
    }
}
pub fn entity_failure_handle(error: &IfccadConversionError) -> Option<Handle> {
    match error {
        IfccadConversionError::Geometry(failure) => source_handle(&failure.source),
        IfccadConversionError::DimensionPreparation { location, .. }
        | IfccadConversionError::HatchPreparation { location, .. }
        | IfccadConversionError::TextPreparation { location, .. }
        | IfccadConversionError::InvalidStructure(location) => location_handle(location),
        IfccadConversionError::Unsupported(issues) => issues
            .iter()
            .find(|d| matches!(d.code, "rounding" | "scale-clamped" | "precision"))
            .and_then(|d| location_handle(&d.location)),
        _ => None,
    }
}
pub fn dimension_failure_handle(failure: &IfccadGeometryFailure) -> Option<Handle> {
    if failure.domain != IfccadGeometryDomain::Drawing
        || failure.stage != IfccadGeometryStage::DeviationAssessment
        || failure.vertex_index != Some(2)
        || failure.reason != IfccadGeometryFailureReason::ProvenExceedance
        || failure.requested_tolerance != IfccadGeometryTolerance::default()
    {
        return None;
    }
    match &failure.source {
        IfccadGeometryEntitySource::CadEntity { handle, kind } if kind == "DIMENSION" => {
            Some(*handle)
        }
        _ => None,
    }
}
pub fn exclude_entity(cad: &mut CadDocument, handle: Handle) -> Result<Value> {
    let entity = cad
        .remove_entity(handle)
        .ok_or("explicit exclusion handle not found")?;
    // remove_entity deliberately retains owner membership for undo; this input
    // snapshot has no undo and must not contain a dangling owner membership.
    for block in cad.block_records.iter_mut() {
        block.entity_handles.retain(|h| *h != handle);
    }
    Ok(
        json!({"handle":format!("0x{handle:X}"),"kind":entity.as_entity().entity_type(),
        "ownerHandle":entity.common().owner_handle.to_string(),
        "reason":"explicit user-authorized benchmark exclusion after conversion failure; original source unchanged"}),
    )
}
pub fn apply_exclusions(cad: &mut CadDocument, records: &[Value]) -> Result<()> {
    for record in records {
        let handle = Handle::new(u64::from_str_radix(
            record["handle"]
                .as_str()
                .ok_or("missing exclusion handle")?
                .strip_prefix("0x")
                .ok_or("invalid handle prefix")?,
            16,
        )?);
        let entity = cad.get_entity(handle).ok_or("exclusion entity not found")?;
        if record["kind"].as_str() != Some(entity.as_entity().entity_type())
            || record["ownerHandle"].as_str()
                != Some(entity.common().owner_handle.to_string().as_str())
        {
            return Err("exclusion kind/owner differs from recorded source".into());
        }
        exclude_entity(cad, handle)?;
    }
    Ok(())
}
pub fn exclude_invalid_polylines(cad: &mut CadDocument) -> Result<Vec<Value>> {
    let candidates =
        cad.entities()
            .filter_map(|entity| {
                let opencadcodec::EntityType::LwPolyline(p) = entity else {
                    return None;
                };
                let n = p.vertices.len();
                let active = if p.is_closed { n } else { n.saturating_sub(1) };
                (n >= 2
                    && p.vertices.iter().take(active).enumerate().any(|(i, v)| {
                        v.bulge != 0. && v.location == p.vertices[(i + 1) % n].location
                    }))
                .then_some(p.common.handle)
            })
            .collect::<Vec<_>>();
    let mut records = Vec::new();
    for handle in candidates {
        // The cheap candidate check never authorizes removal by itself. Use the
        // same production primitive preparation that the converter calls.
        if let Err(error @ cad_geometry_convert::CadPreparationError::InvalidGeometry) =
            cad_geometry_convert::prepare_from_cad(cad.get_entity(handle).unwrap())
        {
            let mut record = exclude_entity(cad, handle)?;
            record["failure"] = json!(format!("shared production primitive preparation: {error}"));
            record["failureDebug"] = json!(format!("{error:?}"));
            records.push(record);
        }
    }
    Ok(records)
}
fn prepare(
    id: &str,
    options: &BTreeMap<String, String>,
    directory: &Path,
    repeat: bool,
    seed: Option<&[Value]>,
) -> Result<(IfccadDocument, Value)> {
    if !id.starts_with("practice-") {
        let (family, n) = id.rsplit_once('-').ok_or("invalid recipe id")?;
        let d = recipes::drawing(family, n.parse()?)?;
        return Ok((d, json!({"family":family,"count":n.parse::<usize>()?})));
    }
    let key = if id == "practice-foundation" {
        "--foundation"
    } else {
        "--test-dxf"
    };
    let path = options.get(key).ok_or("missing original practice source")?;
    let input = fs::read(path)?;
    let start = Instant::now();
    let mut cad = if id == "practice-foundation" {
        DwgReader::from_stream(Cursor::new(input)).read()?
    } else {
        DxfReader::from_reader(Cursor::new(input))?.read()?
    };
    let read_ms = start.elapsed().as_secs_f64() * 1000.;
    let source = inventory::cad(&cad);
    let mut exclusions = Vec::new();
    if let Some(records) = seed {
        apply_exclusions(&mut cad, records)?;
        exclusions.extend_from_slice(records);
    } else if options.contains_key("--omit-test-dxf-entity-errors") {
        if let Some(path) = options.get("--exclusions") {
            let manifest: Value = serde_json::from_slice(&fs::read(path)?)?;
            let records = manifest["exclusions"]
                .as_array()
                .ok_or("missing exclusion records")?;
            apply_exclusions(&mut cad, records)?;
            exclusions.extend_from_slice(records);
        }
        exclusions.extend(exclude_invalid_polylines(&mut cad)?);
    }
    if !repeat {
        for (i, record) in exclusions.iter().enumerate() {
            save(
                &directory.join(format!("exclusion-{:04}.json", i + 1)),
                record,
            )?;
        }
        if !exclusions.is_empty() {
            println!(
                "Applied {} verified exclusions in one preparation pass",
                exclusions.len()
            );
        }
    }
    if seed.is_none() && id == "practice-test-dxf" && options.contains_key("--omit-test-dxf-entity")
    {
        let text = &options["--omit-test-dxf-entity"];
        if text != "EF354" {
            return Err("only the explicitly authorized EF354 exclusion is supported".into());
        }
        let handle = Handle::new(0xEF354);
        let entity = cad
            .get_entity(handle)
            .ok_or("authorized dimension not found")?;
        if !entity.as_entity().entity_type().starts_with("DIMENSION") {
            return Err("authorized exclusion is not a dimension".into());
        }
        exclusions.push(exclude_entity(&mut cad, handle)?);
    }
    let start = Instant::now();
    let result = loop {
        let attempt = cad_document_to_ifccad_document(
            &cad,
            IfccadTargetMetadata {
                header: recipes::header(),
                drawing_id: 7,
            },
            CadToIfccadOptions {
                preservation: IfccadPreservationCapture::SupportedTyped,
                ..Default::default()
            },
        );
        match attempt {
            Err(error) => {
                let handle = if id == "practice-test-dxf"
                    && options.contains_key("--omit-test-dxf-entity-errors")
                {
                    entity_failure_handle(&error)
                } else if id == "practice-test-dxf"
                    && options.contains_key("--omit-test-dxf-dimension-errors")
                {
                    match &error {
                        IfccadConversionError::Geometry(failure) => {
                            dimension_failure_handle(failure)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let Some(handle) = handle else {
                    break Err(error);
                };
                // Removal must make progress; never loop on an inferred/missing ID.
                if cad.get_entity(handle).is_none() {
                    break Err(error);
                }
                let mut record = exclude_entity(&mut cad, handle)?;
                record["failure"] = json!(error.to_string());
                record["failureDebug"] = json!(format!("{error:?}"));
                if !repeat {
                    save(
                        &directory.join(format!("exclusion-{:04}.json", exclusions.len() + 1)),
                        &record,
                    )?;
                    println!(
                        "Excluded {} 0x{handle:X} after its conversion failure",
                        record["kind"]
                    );
                }
                exclusions.push(record);
            }
            other => break other,
        }
    };
    let conversion_ms = start.elapsed().as_secs_f64() * 1000.;
    let filtered = inventory::cad(&cad);
    if !repeat {
        save(
            &directory.join("source-inventory.json"),
            &json!({"sourceInventory":source,"conversionInputInventory":filtered,"exclusions":exclusions,"cadReadMs":read_ms}),
        )?;
    }
    let result = result?;
    let mut diagnostics=result.diagnostics().iter().map(|d|json!({"code":d.code,"location":d.location,"message":d.message,"action":format!("{:?}",d.action)})).collect::<Vec<_>>();
    diagnostics.sort_by_cached_key(Value::to_string);
    let mut actions: BTreeMap<String, usize> = BTreeMap::new();
    let mut codes: BTreeMap<String, usize> = BTreeMap::new();
    for diag in &diagnostics {
        *actions
            .entry(diag["action"].as_str().unwrap().into())
            .or_default() += 1;
        *codes
            .entry(diag["code"].as_str().unwrap().into())
            .or_default() += 1;
    }
    let evidence = json!({"sourceInventory":source,"conversionInputInventory":filtered,"exclusions":exclusions,"diagnostics":diagnostics,"diagnosticActions":actions,"diagnosticCodes":codes,"cadReadMs":read_ms,"cadConversionMs":conversion_ms,"lossPolicy":"Allow","preservation":"SupportedTyped"});
    Ok((result.into_document(), evidence))
}
fn case(id: &str, options: &BTreeMap<String, String>, dir: &Path) -> Result<Value> {
    let (raw, preparation) = prepare(id, options, dir, false, None)?;
    let raw_inventory = inventory::native(&raw);
    save(
        &dir.join("preparation.json"),
        &json!({"conversion":preparation,"retainedBeforeNormalization":raw_inventory}),
    )?;
    let d = freeze(&raw)?;
    let normalization_changed = raw != d;
    drop(raw);
    let variants = artifacts(&d)?;
    qualify(&d, &variants)?;
    let mut files = Vec::new();
    for (i, name) in [
        "drawing.ifcx",
        "drawing.compact.ifcx",
        "drawing.ifccad.cbor",
    ]
    .iter()
    .enumerate()
    {
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))?;
        f.write_all(&variants[i])?;
        files.push(json!({"file":name,"bytes":variants[i].len()}));
    }
    let timing = timings(&d, &variants)?;
    let (repeated, repeat_evidence) = prepare(
        id,
        options,
        dir,
        true,
        preparation
            .get("exclusions")
            .and_then(Value::as_array)
            .map(Vec::as_slice),
    )?;
    let repeated = freeze(&repeated)?;
    exact(&d, &repeated)?;
    let second = artifacts(&repeated)?;
    if variants != second {
        return Err("untimed repeat artifacts differ".into());
    }
    if preparation.get("diagnostics") != repeat_evidence.get("diagnostics")
        || preparation.get("sourceInventory") != repeat_evidence.get("sourceInventory")
        || preparation.get("conversionInputInventory")
            != repeat_evidence.get("conversionInputInventory")
        || preparation.get("exclusions") != repeat_evidence.get("exclusions")
    {
        return Err("untimed repeat source inventory/diagnostics differ".into());
    }
    Ok(
        json!({"id":id,"status":"passed","normalizationChangedOrder":normalization_changed,"artifacts":files,"inventory":inventory::native(&d),"timing":timing,"productionReadback":"strict-exact","untimedRepeat":"exact","cborToCompactJson":variants[2].len() as f64/variants[1].len() as f64}),
    )
}
pub fn run(args: Vec<String>) -> Result<()> {
    if args == ["--help"] {
        println!("cbor_baseline --out NEW_DIRECTORY --cases CASE,CASE [--foundation ORIGINAL_DWG] [--test-dxf ORIGINAL_DXF]\nManual timings only; tests use tiny correctness fixtures. Cases: {}",CASES.join(","));
        return Ok(());
    }
    if args.is_empty() || !args.len().is_multiple_of(2) {
        return Err("expected option/value pairs; use --help".into());
    }
    let mut options = BTreeMap::new();
    for pair in args.as_chunks::<2>().0 {
        if ![
            "--out",
            "--cases",
            "--foundation",
            "--test-dxf",
            "--omit-test-dxf-entity",
            "--omit-test-dxf-dimension-errors",
            "--omit-test-dxf-entity-errors",
            "--exclusions",
        ]
        .contains(&pair[0].as_str())
            || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err("unknown or duplicate option".into());
        }
    }
    let chosen = selection(
        options
            .get("--cases")
            .ok_or("explicit case selection required")?,
    )?;
    if let Some(policy) = options.get("--omit-test-dxf-dimension-errors") {
        if policy != "point2-proven-exceedance" || chosen != ["practice-test-dxf"] {
            return Err(
                "dimension exclusion requires only practice-test-dxf and the authorized policy"
                    .into(),
            );
        }
    }
    if let Some(policy) = options.get("--omit-test-dxf-entity-errors") {
        if policy != "identified-entity" || chosen != ["practice-test-dxf"] {
            return Err(
                "entity exclusion requires only practice-test-dxf and the authorized policy".into(),
            );
        }
    }
    if options.contains_key("--exclusions")
        && !options.contains_key("--omit-test-dxf-entity-errors")
    {
        return Err("exclusion manifest requires identified-entity policy".into());
    }
    let root = Path::new(
        options
            .get("--out")
            .ok_or("fresh output directory required")?,
    );
    fs::create_dir(root)?;
    let mut results = Vec::new();
    let mut failed = false;
    for id in chosen {
        let dir = root.join(&id);
        fs::create_dir(&dir)?;
        println!("Starting {id}");
        let result = match case(&id, &options, &dir) {
            Ok(v) => v,
            Err(e) => {
                failed = true;
                json!({"id":id,"status":"failed","error":e.to_string(),"detail":format!("{e:?}")})
            }
        };
        save(&dir.join("case.json"), &result)?;
        println!("{}: {}", id, result["status"]);
        results.push(result);
    }
    save(
        &root.join("results.json"),
        &json!({"version":1,"mappingRevision":2,"cases":results}),
    )?;
    if failed {
        return Err("one or more cases failed; detailed records retained".into());
    }
    Ok(())
}
