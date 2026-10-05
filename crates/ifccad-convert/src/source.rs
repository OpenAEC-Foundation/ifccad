use crate::diagnostics::diagnostic;
use crate::{IfccadConversionError as Error, IfccadDiagnostic};
use opencadcodec::objects::ObjectType;
use opencadcodec::{
    CadDocument, EntityType, Handle, SemanticEntityV1, SemanticNodeV1, SemanticObjectV1,
    SemanticPartV1, SemanticReferenceV1, SemanticRelationshipKindV1, SemanticTableRecordV1,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub(crate) struct Inspection {
    pub model_layout: Handle,
    pub entities: Vec<Handle>,
    pub blocks: Vec<Handle>,
    pub papers: Vec<InspectedPaperLayout>,
    pub issues: Vec<IfccadDiagnostic>,
    pub recoveries: Vec<IfccadDiagnostic>,
}
pub(crate) struct InspectedPaperLayout {
    pub layout_handle: Handle,
    pub block_handle: Handle,
    pub tab_index: u32,
    pub entity_handles: Vec<Handle>,
}

pub(crate) fn same_graph(a: &Value, b: &Value) -> bool {
    fn normalize(v: &Value) -> Value {
        let mut v = v.clone();
        if let Some(data) = v.get_mut("data").and_then(Value::as_array_mut) {
            data.sort_by_key(|n| n["path"].as_str().unwrap_or("").to_string());
        }
        v
    }
    fn number(n: &serde_json::Number) -> Option<num_rational::BigRational> {
        n.as_u64()
            .map(|v| num_rational::BigRational::from_integer(v.into()))
            .or_else(|| {
                n.as_i64()
                    .map(|v| num_rational::BigRational::from_integer(v.into()))
            })
            .or_else(|| n.as_f64().and_then(num_rational::BigRational::from_float))
    }
    fn same(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => number(a) == number(b),
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len() && a.iter().all(|(k, a)| b.get(k).is_some_and(|b| same(a, b)))
            }
            _ => a == b,
        }
    }
    // Compare exact values so 1 and 1.0 agree but large integer distinctions survive.
    same(&normalize(a), &normalize(b))
}

/// Diagnose each changed field after explicitly removing mapped or derived fields.
pub(crate) fn residual<T: Serialize>(
    actual: &T,
    default: &T,
    ignored: &[&str],
    location: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    let a = serde_json::to_value(actual).expect("pinned semantic model serializes");
    let b = serde_json::to_value(default).expect("pinned semantic model serializes");
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        for (key, value) in a {
            if !ignored.contains(&key.as_str()) && b.get(key) != Some(value) {
                issues.push(diagnostic(
                    "source-field",
                    format!("{location}.{key}"),
                    "field is outside the supported IFCCAD projection",
                ));
            }
        }
    } else if a != b {
        issues.push(diagnostic(
            "source-field",
            location,
            "unsupported semantic payload",
        ));
    }
}

pub(crate) fn inspect(doc: &CadDocument) -> Result<Inspection, Error> {
    let invalid = |s: &str| Error::InvalidStructure(s.to_string());
    let models = doc
        .block_records
        .iter()
        .filter(|b| b.is_model_space())
        .collect::<Vec<_>>();
    if models.len() != 1 || models[0].handle.is_null() {
        return Err(invalid("exactly one non-null model block is required"));
    }
    let model = models[0];
    let mut all = Vec::new();
    doc.semantic_inventory_v1().visit(|part| match part {
        SemanticPartV1::Entity(SemanticEntityV1::Typed(e)) => all.push(e),
        SemanticPartV1::Entity(SemanticEntityV1::Unsupported { common, .. }) => {
            if let Some(e) = doc.get_entity(common.handle) {
                all.push(e);
            }
        }
        _ => {}
    });
    let layouts = doc
        .objects
        .values()
        .filter_map(|o| match o {
            ObjectType::Layout(l) if l.block_record == model.handle => Some(l),
            _ => None,
        })
        .collect::<Vec<_>>();
    if layouts.len() != 1 || (!model.layout.is_null() && model.layout != layouts[0].handle) {
        return Err(invalid("model block and exactly one layout must agree"));
    }
    let mut recoveries = vec![];
    let layout_dictionary = crate::layout_references::dictionary(doc, &mut recoveries)?;
    if doc.header.model_space_block_handle != model.handle {
        if doc.header.model_space_block_handle.is_null()
            || doc
                .block_records
                .iter()
                .any(|b| b.handle == doc.header.model_space_block_handle)
        {
            return Err(invalid("model cache points to a conflicting owner"));
        }
        recoveries.push(diagnostic(
            "model-cache-recovered",
            "header.model_space_block_handle",
            "unique model block/layout agreement repairs a stale cache",
        ));
    }
    let mut identities = BTreeSet::new();
    macro_rules! ids {
        ($items:expr) => {
            for item in $items {
                if item.handle.is_null() || !identities.insert(item.handle) {
                    return Err(invalid("null or duplicate semantic handle"));
                }
            }
        };
    }
    ids!(doc.layers.iter());
    ids!(doc.block_records.iter());
    ids!(doc.line_types.iter());
    ids!(doc.text_styles.iter());
    ids!(doc.dim_styles.iter());
    ids!(doc.app_ids.iter());
    ids!(doc.views.iter());
    ids!(doc.vports.iter());
    ids!(doc.ucss.iter());
    ids!(doc.vx_table.iter());
    for (h, o) in &doc.objects {
        if h.is_null() || !identities.insert(*h) {
            return Err(invalid("duplicate object identity"));
        }
        if let ObjectType::Layout(l) = o {
            if *h != l.handle {
                return Err(invalid("layout key/handle mismatch"));
            }
        }
    }
    for e in &all {
        if e.common().handle.is_null() || !identities.insert(e.common().handle) {
            return Err(invalid("duplicate entity identity"));
        }
    }
    let mut layer_names = BTreeSet::new();
    for l in doc.layers.iter() {
        if !layer_names.insert(l.name.to_uppercase()) {
            return Err(invalid("ambiguous duplicate layer name"));
        }
    }
    let mut owned = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut issues = vec![];
    let mut blocks = vec![];
    let mut layout_blocks = BTreeSet::new();
    let mut layout_tabs = BTreeSet::new();
    let mut layout_names = BTreeSet::new();
    let mut paper_sources = Vec::new();
    for object in doc.objects.values() {
        if let ObjectType::Layout(l) = object {
            let record = doc
                .block_records
                .iter()
                .find(|b| b.handle == l.block_record)
                .ok_or_else(|| invalid("layout references missing block"))?;
            if !record.layout.is_null() && record.layout != l.handle {
                return Err(invalid("layout/block link disagrees"));
            }
            if (!record.is_model_space() && !record.is_paper_space())
                || !layout_blocks.insert(record.handle)
            {
                return Err(invalid("each layout needs a unique Model/Paper block"));
            }
            if l.tab_order < 0
                || !layout_tabs.insert(l.tab_order)
                || (record.is_model_space() && l.tab_order != 0)
            {
                return Err(invalid("ambiguous or invalid layout tab order"));
            }
            if !layout_names.insert(l.name.to_uppercase()) {
                return Err(invalid("ambiguous duplicate layout name"));
            }
            if !matches!(doc.objects.get(&layout_dictionary),Some(ObjectType::Dictionary(d)) if l.owner==d.handle && d.entries.iter().filter(|(name,h)| name==&l.name && *h==l.handle).count()==1)
            {
                return Err(invalid(
                    "layout dictionary membership or ownership disagrees",
                ));
            }
            if record.is_paper_space() {
                paper_sources.push(l);
            }
            crate::layout_references::viewports(doc, l, record.is_model_space(), &mut issues)?;
        }
        let entries = match object {
            ObjectType::Dictionary(d) => Some(&d.entries),
            ObjectType::DictionaryWithDefault(d) => Some(&d.entries),
            _ => None,
        };
        if let Some(entries) = entries {
            for (_, h) in entries {
                if h.is_null() || doc.semantic_inventory_v1().resolve(*h).is_none() {
                    return Err(invalid("dictionary references missing content"));
                }
            }
        }
    }
    for b in doc.block_records.iter() {
        if b.is_paper_space() && !layout_blocks.contains(&b.handle) {
            return Err(invalid("Paper block needs exactly one layout"));
        }
        if !names.insert(b.name.to_uppercase()) {
            return Err(invalid("ambiguous duplicate block name"));
        }
        if !b.is_model_space() && !b.is_paper_space() {
            blocks.push(b.handle);
        }
        for h in &b.entity_handles {
            if !owned.insert(*h) {
                return Err(invalid("duplicate owner membership"));
            }
            let e = doc
                .get_entity(*h)
                .ok_or_else(|| invalid("owner references missing entity"))?;
            if e.common().owner_handle != b.handle {
                return Err(invalid("entity owner disagrees with owner membership"));
            }
            if matches!(e, EntityType::Block(_) | EntityType::BlockEnd(_)) {
                return Err(invalid("structural marker appears in draw order"));
            }
            if doc.layers.get(&e.common().layer).is_none() {
                return Err(invalid("entity references missing layer"));
            }
            if b.is_paper_space()
                && matches!(e, EntityType::Viewport(_))
                && !overall_scaffold(doc, e)
                && matches!(e, EntityType::Viewport(v) if crate::viewports::overall_canvas(doc,v))
            {
                issues.push(diagnostic(
                    "paper",
                    format!("entity/{h}"),
                    "authored Paper viewport/canvas state is deferred",
                ));
            }
        }
    }
    for e in all {
        match e {
            EntityType::Block(marker) => {
                let b = doc
                    .block_records
                    .iter()
                    .find(|b| b.handle == marker.common.owner_handle)
                    .ok_or_else(|| invalid("BLOCK owner missing"))?;
                if marker.common.handle != b.block_entity_handle
                    || marker.name != b.name
                    || marker.base_point != b.base_point
                {
                    return Err(invalid(&format!(
                        "BLOCK marker {} ({:?}, {:?}) conflicts with block record {} ({:?}, {:?})",
                        marker.common.handle,
                        marker.name,
                        marker.base_point,
                        b.block_entity_handle,
                        b.name,
                        b.base_point
                    )));
                }
                residual(
                    marker,
                    &opencadcodec::entities::Block::new(&b.name, b.base_point),
                    &["common", "name", "base_point"],
                    &format!("block-marker/{}", b.name),
                    &mut issues,
                );
                marker_common(&marker.common, &mut issues);
            }
            EntityType::BlockEnd(marker) => {
                let b = doc
                    .block_records
                    .iter()
                    .find(|b| b.handle == marker.common.owner_handle)
                    .ok_or_else(|| invalid("ENDBLK owner missing"))?;
                if marker.common.handle != b.block_end_handle {
                    return Err(invalid("ENDBLK marker conflicts with record"));
                }
                marker_common(&marker.common, &mut issues);
            }
            _ if !owned.contains(&e.common().handle) => {
                return Err(invalid("entity missing from owner membership"))
            }
            _ => {}
        }
    }
    // Acyclic even when definitions are unused. Edges refer to validated names.
    fn visit(
        h: Handle,
        doc: &CadDocument,
        active: &mut BTreeSet<Handle>,
        done: &mut BTreeSet<Handle>,
    ) -> Result<(), Error> {
        if done.contains(&h) {
            return Ok(());
        }
        if !active.insert(h) {
            return Err(Error::InvalidStructure("cyclic block graph".into()));
        }
        let b = doc.block_records.iter().find(|b| b.handle == h).unwrap();
        for e in &b.entity_handles {
            if let Some(EntityType::Insert(i)) = doc.get_entity(*e) {
                let target = doc
                    .block_records
                    .get(&i.block_name)
                    .filter(|b| !b.is_model_space() && !b.is_paper_space())
                    .ok_or_else(|| {
                        Error::InvalidStructure("missing/local block target required".into())
                    })?;
                visit(target.handle, doc, active, done)?;
            }
        }
        active.remove(&h);
        done.insert(h);
        Ok(())
    }
    let mut done = BTreeSet::new();
    for h in blocks
        .iter()
        .chain(std::iter::once(&model.handle))
        .chain(paper_sources.iter().map(|l| &l.block_record))
    {
        visit(*h, doc, &mut BTreeSet::new(), &mut done)?;
    }
    paper_sources.sort_by_key(|l| l.tab_order);
    let mut papers = Vec::new();
    for layout in paper_sources {
        let block = doc
            .block_records
            .iter()
            .find(|b| b.handle == layout.block_record)
            .unwrap();
        if untouched_paper(doc, layout) {
            continue;
        }
        let tab_index =
            u32::try_from(papers.len() + 1).map_err(|_| invalid("too many paper layouts"))?;
        if i64::from(layout.tab_order) != i64::from(tab_index) {
            let mut recovery = diagnostic(
                "layout-tabs-normalized",
                format!("layout/{}.tabOrder", layout.name),
                "source tab gaps normalized without changing relative order",
            );
            recovery.action = crate::IfccadDiagnosticAction::Recovery;
            recoveries.push(recovery);
        }
        papers.push(InspectedPaperLayout {
            layout_handle: layout.handle,
            block_handle: layout.block_record,
            tab_index,
            entity_handles: block
                .entity_handles
                .iter()
                .filter(|h| !overall_scaffold(doc, doc.get_entity(**h).unwrap()))
                .copied()
                .collect(),
        });
    }
    scan(doc, layout_dictionary, &mut issues)?;
    for e in doc.entities() {
        crate::geometry::validate_source(e)?;
    }
    for recovery in &mut recoveries {
        recovery.action = crate::IfccadDiagnosticAction::Recovery;
    }
    Ok(Inspection {
        model_layout: layouts[0].handle,
        entities: model.entity_handles.clone(),
        blocks,
        papers,
        issues,
        recoveries,
    })
}
fn marker_common(c: &opencadcodec::entities::EntityCommon, issues: &mut Vec<IfccadDiagnostic>) {
    let mut r = c.clone();
    let b = opencadcodec::entities::EntityCommon::new();
    r.handle = b.handle;
    r.owner_handle = b.owner_handle;
    r.raw_record = None;
    r.entity_mode = None;
    r.linetype_handle = None;
    if r != b {
        issues.push(diagnostic(
            "marker-common",
            format!("entity/{}", c.handle),
            "authored block delimiter properties are unsupported",
        ));
    }
}
fn overall_scaffold(doc: &CadDocument, e: &EntityType) -> bool {
    let EntityType::Viewport(v) = e else {
        return false;
    };
    // DWG does not store DXF viewport numbers; the pinned reader assigns them
    // only to the active sheet. Its unnumbered overall viewport is identified
    // by the layout link and the same complete default-state comparison.
    if !(v.id==1 || (doc.dwg_source_version.is_some() && v.id==0)) || !doc.objects.values().any(|o|matches!(o,ObjectType::Layout(l) if l.block_record==v.common.owner_handle && l.viewport==v.common.handle && doc.block_records.iter().any(|b|b.handle==l.block_record && b.is_paper_space()))) {return false;}
    let mut a = v.clone();
    let mut b = opencadcodec::entities::Viewport::new();
    b.id = 1;
    a.id = b.id;
    a.common.handle = b.common.handle;
    a.common.owner_handle = b.common.owner_handle;
    a.common.entity_mode = b.common.entity_mode;
    a.common.raw_record = None;
    a.common.linetype_handle = None;
    a == b
}

fn untouched_paper(doc: &CadDocument, layout: &opencadcodec::objects::Layout) -> bool {
    // Only the initial runtime sheet is scaffold; extra empty sheets are authored.
    if layout.name != "Layout1" {
        return false;
    }
    let fresh = CadDocument::new();
    let Some(ObjectType::Layout(b)) = fresh
        .objects
        .values()
        .find(|o| matches!(o,ObjectType::Layout(l) if l.name=="Layout1"))
    else {
        return false;
    };
    let mut a = layout.clone();
    a.handle = b.handle;
    a.owner = b.owner;
    a.block_record = b.block_record;
    a.viewport = b.viewport;
    a.viewports = b.viewports.clone();
    a.min_extents = b.min_extents;
    a.max_extents = b.max_extents;
    a.raw_plot_settings_codes = None;
    a == *b
        && doc
            .block_records
            .iter()
            .find(|r| r.handle == layout.block_record)
            .is_some_and(|r| {
                r.entity_handles
                    .iter()
                    .all(|h| doc.get_entity(*h).is_some_and(|e| overall_scaffold(doc, e)))
            })
}

fn scan(
    doc: &CadDocument,
    layout_dictionary: Handle,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), Error> {
    static DWG_DEFAULT: OnceLock<Result<CadDocument, String>> = OnceLock::new();
    let fresh = CadDocument::new();
    let baseline = if doc.dwg_source_version.is_some() {
        DWG_DEFAULT
            .get_or_init(|| {
                let bytes = opencadcodec::DwgWriter::write_to_vec(&CadDocument::new())
                    .map_err(|e| e.to_string())?;
                opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
                    .read()
                    .map_err(|e| e.to_string())
            })
            .as_ref()
            .map_err(|e| {
                Error::CadConstruction(format!("cannot establish pinned DWG scaffold: {e}"))
            })?
    } else {
        &fresh
    };
    let object_roles = object_roles(doc, baseline);
    let mut unresolved = None;
    doc.semantic_inventory_v1().visit(|part| match part {
        SemanticPartV1::Header(h) => residual(
            h,
            &baseline.header,
            &[
                "insertion_units",
                "linetype_scale",
                "model_space_extents_min",
                "model_space_extents_max",
                "paper_space_extents_min",
                "paper_space_extents_max",
                "handle_seed",
                "current_layer_handle",
                "current_text_style_handle",
                "current_linetype_handle",
                "current_dimstyle_handle",
                "current_multiline_style_handle",
                "current_material_handle",
                "dim_text_style_handle",
                "dim_linetype_handle",
                "dim_linetype1_handle",
                "dim_linetype2_handle",
                "dim_arrow_block_handle",
                "dim_arrow_block1_handle",
                "dim_arrow_block2_handle",
                "block_control_handle",
                "layer_control_handle",
                "style_control_handle",
                "linetype_control_handle",
                "view_control_handle",
                "ucs_control_handle",
                "vport_control_handle",
                "appid_control_handle",
                "dimstyle_control_handle",
                "vpent_hdr_control_handle",
                "current_vx_handle",
                "named_objects_dict_handle",
                "acad_group_dict_handle",
                "acad_mlinestyle_dict_handle",
                "acad_layout_dict_handle",
                "acad_plotsettings_dict_handle",
                "acad_plotstylename_dict_handle",
                "acad_material_dict_handle",
                "acad_color_dict_handle",
                "acad_visualstyle_dict_handle",
                "model_space_block_handle",
                "paper_space_block_handle",
                "bylayer_linetype_handle",
                "byblock_linetype_handle",
                "continuous_linetype_handle",
            ],
            "header",
            issues,
        ),
        SemanticPartV1::TableRecord(r) => {
            macro_rules! table {
                ($r:expr,$t:expr) => {
                    if let Some(b) = $t.get(&$r.name) {
                        residual($r, b, &["handle"], &format!("table/{}", $r.name), issues);
                    } else {
                        issues.push(diagnostic(
                            "table",
                            format!("table/{}", $r.name),
                            "non-scaffold table record",
                        ));
                    }
                };
            }
            match r {
                SemanticTableRecordV1::Layer(l) => residual(
                    l,
                    &opencadcodec::Layer::new(&l.name),
                    &[
                        "handle",
                        "name",
                        "color",
                        "line_type",
                        "line_weight",
                        "transparency",
                    ],
                    &format!("layer/{}", l.name),
                    issues,
                ),
                SemanticTableRecordV1::BlockRecord(b) => {
                    let mut ignored = vec![
                        "handle",
                        "block_entity_handle",
                        "block_end_handle",
                        "name",
                        "entity_handles",
                        "insert_handles",
                    ];
                    if b.is_model_space() || b.is_paper_space() {
                        ignored.push("layout");
                    } else {
                        ignored.extend(["units", "base_point"]);
                    }
                    residual(
                        b,
                        &opencadcodec::BlockRecord::new(&b.name),
                        &ignored,
                        &format!("block/{}", b.name),
                        issues,
                    );
                }
                SemanticTableRecordV1::LineType(_) => {} // handled by checked native pattern conversion
                SemanticTableRecordV1::TextStyle(r) => table!(r, baseline.text_styles),
                SemanticTableRecordV1::DimStyle(r) => table!(r, baseline.dim_styles),
                SemanticTableRecordV1::AppId(r) => table!(r, baseline.app_ids),
                SemanticTableRecordV1::View(r) => table!(r, baseline.views),
                SemanticTableRecordV1::VPort(r) => table!(r, baseline.vports),
                SemanticTableRecordV1::Ucs(r) => table!(r, baseline.ucss),
                SemanticTableRecordV1::Vx(r) => table!(r, baseline.vx_table),
            }
        }
        SemanticPartV1::Class(c) => {
            let same = baseline.classes.iter().any(|b| {
                let mut a = c.clone();
                a.class_number = b.class_number;
                a.instance_count = b.instance_count;
                a.dwg_version = b.dwg_version;
                a.maintenance_version = b.maintenance_version;
                a == *b
            });
            if !same {
                issues.push(diagnostic("class", &c.dxf_name, "non-scaffold class"));
            }
        }
        SemanticPartV1::Entity(_) => {} // typed geometry and common-field scan follows
        SemanticPartV1::Object(SemanticObjectV1::Typed(ObjectType::Layout(l))) => {
            let expected = baseline.objects.values().find_map(|o| match o {
                ObjectType::Layout(b) if b.name == l.name => Some(b),
                _ => None,
            });
            let default = opencadcodec::objects::Layout::new(&l.name);
            let b = expected.unwrap_or(&default);
            {
                let mut actual = l.clone();
                // The DXF writer derives this role bit from the Model layout
                // name. Ownership was already checked; the bit adds no setting.
                if l.name == "Model"
                    && doc
                        .block_records
                        .iter()
                        .any(|record| record.is_model_space() && record.handle == l.block_record)
                {
                    actual.plot_flags.model_type = b.plot_flags.model_type;
                }
                let paper = doc.block_records.iter().any(|record|record.is_paper_space() && record.handle==l.block_record);
                let mut ignored=vec!["handle","owner","block_record","viewport","viewports","min_extents","max_extents"];
                if paper {
                    // Unit/mapping support is classified separately, including unknown physical intent.
                    ignored.extend(["name","tab_order","paper_width","paper_height","plot_paper_units","plot_scale_type","plot_scale_numerator","plot_scale_denominator","plot_scale_factor"]);
                    actual.plot_flags.use_standard_scale=b.plot_flags.use_standard_scale;
                }
                residual(
                    &actual,
                    b,
                    &ignored,
                    &format!("layout/{}", l.name),
                    issues,
                );
            }
        }
        SemanticPartV1::Object(SemanticObjectV1::Typed(o)) => {
            if !scaffold_object(o, doc, baseline, &object_roles, layout_dictionary) {
                let h = doc
                    .objects
                    .iter()
                    .find(|(_, v)| std::ptr::eq(*v, o))
                    .map(|(h, _)| *h)
                    .expect("inventory object");
                issues.push(diagnostic(
                    "object",
                    format!("object/{h}"),
                    "non-scaffold object",
                ));
            }
        }
        SemanticPartV1::Object(SemanticObjectV1::Unsupported { type_name }) => {
            issues.push(diagnostic("object", type_name, "unsupported object"))
        }
        SemanticPartV1::SummaryInfo(i) => {
            if i != &baseline.summary_info {
                issues.push(diagnostic(
                    "summary",
                    "summary_info",
                    "authored document metadata",
                ));
            }
        }
        SemanticPartV1::Preview(_) => issues.push(diagnostic(
            "preview",
            "preview",
            "preview is not represented",
        )),
        SemanticPartV1::Relationship {
            kind,
            source,
            target,
        } => {
            let missing = matches!(source, SemanticReferenceV1::Unresolved)
                || matches!(target, SemanticReferenceV1::Unresolved);
            let (source_path, source_name, owner) = relationship_source(doc, source);
            let property = match kind {
                SemanticRelationshipKindV1::Ownership => "owner",
                SemanticRelationshipKindV1::ExtensionDictionary => "extensionDictionary",
                SemanticRelationshipKindV1::Reactor => "reactor",
            };
            let location = format!("{source_path}.{property}");
            // Entity and Layout ownership is essential. Other objects are outside
            // the CAD projection and their ownership is omitted with the object.
            let metadata_owner = matches!(source,
                SemanticReferenceV1::Resolved(SemanticNodeV1::Object(object))
                if !matches!(object, ObjectType::Layout(_)));
            if missing && kind == SemanticRelationshipKindV1::Ownership && !metadata_owner {
                unresolved.get_or_insert_with(|| {
                    format!("dangling essential ownership relationship at {location}")
                });
            } else if missing {
                let owner_detail = if kind == SemanticRelationshipKindV1::Ownership {
                    owner.map(|h| format!(" (owner {h})")).unwrap_or_default()
                } else {
                    String::new()
                };
                issues.push(diagnostic(
                    "unresolved-relationship",
                    location,
                    format!("unresolved {kind:?} relationship on {source_name}{owner_detail} omitted outside the CAD projection"),
                ));
            } else if kind != SemanticRelationshipKindV1::Ownership {
                issues.push(diagnostic(
                    "relationship",
                    location,
                    "reactor/extension relationship is unsupported",
                ));
            }
        }
        SemanticPartV1::NonEntityExtendedData { owner, application, values } => {
            // The codec retains this layer opacity encoding as EED after DWG
            // readback. Accept only an exact duplicate of the mapped typed value.
            let duplicate = layer_transparency_duplicate(
                owner, application.map(|a| a.name.as_str()), values.as_deref(),
            );
            if !duplicate {
                issues.push(diagnostic(
                    "xdata", "inventory", "non-entity extended data is unsupported",
                ));
            }
        }
    });
    if let Some(problem) = unresolved {
        return Err(Error::InvalidStructure(problem));
    }
    Ok(())
}

fn layer_transparency_duplicate(
    owner: SemanticReferenceV1<'_>,
    application: Option<&str>,
    values: Option<&[opencadcodec::xdata::XDataValue]>,
) -> bool {
    match (owner, application, values) {
        (
            SemanticReferenceV1::Resolved(SemanticNodeV1::TableRecord(
                SemanticTableRecordV1::Layer(layer),
            )),
            Some("AcCmTransparency"),
            Some([opencadcodec::xdata::XDataValue::Integer32(value)]),
        ) => layer.transparency.is_explicit() && *value == layer.transparency.to_dxf_value(),
        _ => false,
    }
}

fn relationship_source(
    doc: &CadDocument,
    source: SemanticReferenceV1<'_>,
) -> (String, &'static str, Option<Handle>) {
    match source {
        SemanticReferenceV1::Resolved(SemanticNodeV1::Entity(entity)) => (
            format!("entity/{}", entity.common().handle),
            "entity",
            Some(entity.common().owner_handle),
        ),
        SemanticReferenceV1::Resolved(SemanticNodeV1::Object(object)) => {
            let handle = doc
                .objects
                .iter()
                .find(|(_, candidate)| std::ptr::eq(*candidate, object))
                .map(|(handle, _)| *handle)
                .expect("inventory object");
            let name = match object {
                ObjectType::ClassObject(class) => class.dxf_name(),
                ObjectType::Layout(_) => "Layout",
                _ => "object",
            };
            (format!("object/{handle}"), name, doc.object_owner(handle))
        }
        _ => ("inventory".into(), "metadata", None),
    }
}

// Adapted from the existing converter's bootstrap-role comparison. Dictionary
// paths establish roles; arbitrary handle coincidence never establishes one.
fn object_roles(doc: &CadDocument, base: &CadDocument) -> BTreeMap<Handle, Handle> {
    let mut map = BTreeMap::new();
    let mut pending = vec![(
        doc.header.named_objects_dict_handle,
        base.header.named_objects_dict_handle,
    )];
    while let Some((a, b)) = pending.pop() {
        if map.insert(a, b).is_some() {
            continue;
        }
        match (doc.objects.get(&a), base.objects.get(&b)) {
            (Some(ObjectType::Dictionary(a)), Some(ObjectType::Dictionary(b))) => {
                for (name, h) in &a.entries {
                    if let Some((_, v)) = b.entries.iter().find(|(key, _)| key == name) {
                        pending.push((*h, *v));
                    }
                }
            }
            (
                Some(ObjectType::DictionaryWithDefault(a)),
                Some(ObjectType::DictionaryWithDefault(b)),
            ) => {
                pending.push((a.default_handle, b.default_handle));
                for (name, h) in &a.entries {
                    if let Some((_, v)) = b.entries.iter().find(|(key, _)| key == name) {
                        pending.push((*h, *v));
                    }
                }
            }
            _ => {}
        }
    }
    map
}
fn scaffold_object(
    o: &ObjectType,
    doc: &CadDocument,
    base: &CadDocument,
    map: &BTreeMap<Handle, Handle>,
    layout_dictionary: Handle,
) -> bool {
    let Some((&h, _)) = doc.objects.iter().find(|(_, v)| std::ptr::eq(*v, o)) else {
        return false;
    };
    let mapped = |h| map.get(&h).copied().unwrap_or(h);
    let Some(b) = map.get(&h).and_then(|h| base.objects.get(h)) else {
        return false;
    };
    match (o, b) {
        (ObjectType::Dictionary(a), ObjectType::Dictionary(b)) => {
            let mut a = a.clone();
            let mut b = b.clone();
            if h == layout_dictionary {
                a.entries.retain(|(name,h)|!matches!(doc.objects.get(h),Some(ObjectType::Layout(l)) if l.name==*name));
                b.entries.retain(|(_,h)|matches!(base.objects.get(h),Some(o) if !matches!(o,ObjectType::Layout(_))));
            }
            a.handle = mapped(a.handle);
            a.owner = mapped(a.owner);
            for (_, h) in &mut a.entries {
                *h = mapped(*h);
            }
            a == b
        }
        (ObjectType::DictionaryWithDefault(a), ObjectType::DictionaryWithDefault(b)) => {
            let mut a = a.clone();
            a.handle = mapped(a.handle);
            a.owner = mapped(a.owner);
            a.default_handle = mapped(a.default_handle);
            for (_, h) in &mut a.entries {
                *h = mapped(*h);
            }
            a == *b
        }
        (ObjectType::PlaceHolder(a), ObjectType::PlaceHolder(b)) => {
            let mut a = a.clone();
            a.handle = mapped(a.handle);
            a.owner = mapped(a.owner);
            a == *b
        }
        (ObjectType::MLineStyle(a), ObjectType::MLineStyle(b)) => {
            let mut a = a.clone();
            a.handle = mapped(a.handle);
            a.owner = mapped(a.owner);
            a == *b
        }
        (ObjectType::MultiLeaderStyle(a), ObjectType::MultiLeaderStyle(b)) => {
            let mut a = a.clone();
            a.handle = mapped(a.handle);
            a.owner_handle = mapped(a.owner_handle);
            if a.line_type_handle == doc.line_types.get("ByLayer").map(|l| l.handle) {
                a.line_type_handle = b.line_type_handle;
            }
            a == *b
        }
        (ObjectType::TableStyle(a), ObjectType::TableStyle(b)) => {
            let mut a = a.clone();
            a.handle = mapped(a.handle);
            a.owner_handle = mapped(a.owner_handle);
            for (row, base) in [
                (&mut a.data_row_style, &b.data_row_style),
                (&mut a.header_row_style, &b.header_row_style),
                (&mut a.title_row_style, &b.title_row_style),
            ] {
                if row.text_style_handle.is_none() && row.text_style_name == base.text_style_name {
                    row.text_style_handle = base.text_style_handle;
                }
            }
            a.raw_dxf_codes = b.raw_dxf_codes.clone();
            a == *b
        }
        _ => o == b,
    }
}

#[cfg(test)]
mod transparency_tests {
    use super::*;
    use opencadcodec::xdata::XDataValue;

    #[test]
    fn transparency_encoding_requires_exact_owner_application_and_payload() {
        let mut layer = opencadcodec::tables::Layer::new("example");
        layer.transparency = opencadcodec::Transparency::Explicit(128);
        let owner = SemanticReferenceV1::Resolved(SemanticNodeV1::TableRecord(
            SemanticTableRecordV1::Layer(&layer),
        ));
        let valid = [XDataValue::Integer32(33_554_559)];
        assert!(layer_transparency_duplicate(
            owner,
            Some("AcCmTransparency"),
            Some(&valid)
        ));
        for payload in [
            None,
            Some(vec![]),
            Some(vec![XDataValue::Integer32(33_554_483)]),
            Some(vec![XDataValue::String("33554559".into())]),
            Some(vec![valid[0].clone(), XDataValue::Integer32(1)]),
            // The DWG packed flag is not the canonical layer XDATA flag.
            Some(vec![XDataValue::Integer32(50_331_775)]),
        ] {
            assert!(!layer_transparency_duplicate(
                owner,
                Some("AcCmTransparency"),
                payload.as_deref()
            ));
        }
        for application in [None, Some("ACAD"), Some("accmtransparency")] {
            assert!(!layer_transparency_duplicate(
                owner,
                application,
                Some(&valid)
            ));
        }
        for wrong_owner in [
            SemanticReferenceV1::Unresolved,
            SemanticReferenceV1::Resolved(SemanticNodeV1::Document),
        ] {
            assert!(!layer_transparency_duplicate(
                wrong_owner,
                Some("AcCmTransparency"),
                Some(&valid)
            ));
        }
    }
}
