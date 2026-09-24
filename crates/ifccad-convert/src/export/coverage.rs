use super::conversion::ExportContext;
use super::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::xdata::XDataValue;
use cadcodec::{
    CadDocument, SemanticNodeV1, SemanticObjectV1, SemanticPartV1, SemanticReferenceV1,
    SemanticRelationshipKindV1, SemanticTableRecordV1,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

static DWG_BOOTSTRAP: OnceLock<Option<CadDocument>> = OnceLock::new();

fn dwg_bootstrap() -> Option<&'static CadDocument> {
    DWG_BOOTSTRAP
        .get_or_init(|| {
            let bytes = cadcodec::DwgWriter::write_to_vec(&CadDocument::new()).ok()?;
            cadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
                .read()
                .ok()
        })
        .as_ref()
}

pub(crate) fn scan_document_semantics(document: &CadDocument, context: &mut ExportContext) {
    let baseline = CadDocument::new();
    let content_baseline = if document.dwg_source_version.is_some() {
        dwg_bootstrap().unwrap_or(&baseline)
    } else {
        &baseline
    };
    let bootstrap_handles = bootstrap_handle_map(document, content_baseline);
    let object_handles = document
        .objects
        .iter()
        .map(|(handle, object)| (object as *const _, *handle))
        .collect::<HashMap<_, _>>();
    let mut baseline_classes = HashMap::<String, Vec<&cadcodec::classes::DxfClass>>::new();
    for class in content_baseline.classes.iter() {
        baseline_classes
            .entry(class.dxf_name.to_ascii_uppercase())
            .or_default()
            .push(class);
    }
    let mut class_positions = HashMap::<String, usize>::new();
    let mut table_counts = BTreeMap::<&'static str, usize>::new();
    let mut table_positions = HashMap::<(&'static str, String), usize>::new();
    let mut collection_counts = BTreeMap::<&'static str, usize>::new();
    let mut relationships = 0;
    let mut extended_data = 0;
    let mut unresolved_typed = HashMap::<(*const cadcodec::EntityType, u8), usize>::new();
    let inventory = document.semantic_inventory_v1();
    // Keep this match exhaustive: a new upstream category must be classified.
    // Typed exporters still own their field-level and geometry diagnostics.
    inventory.visit(|part| match part {
        SemanticPartV1::Header(header) => scan_header(header, &baseline, context),
        SemanticPartV1::TableRecord(record) => {
            if let Some(kind) = unsupported_table_record(record, &baseline, context, &mut table_positions) {
                *table_counts.entry(kind).or_default() += 1;
            }
        }
        SemanticPartV1::Class(class) => {
            let name = class.dxf_name.to_ascii_uppercase();
            let position = class_positions.entry(name.clone()).or_default();
            let original = baseline_classes.get(&name).and_then(|classes| classes.get(*position)).copied();
            *position += 1;
            let changed = original.is_none_or(|original| {
                let mut candidate = class.clone();
                candidate.class_number = original.class_number;
                candidate.instance_count = original.instance_count;
                candidate.dwg_version = original.dwg_version;
                candidate.maintenance_version = original.maintenance_version;
                candidate != *original
            });
            *collection_counts.entry("classes").or_default() += usize::from(changed);
        }
        SemanticPartV1::Entity(_) => {}
        SemanticPartV1::Object(object) => {
            *collection_counts.entry("objects").or_default() +=
                usize::from(unsupported_object(object, document, content_baseline, &bootstrap_handles, &object_handles));
        }
        SemanticPartV1::SummaryInfo(info) => {
            if info != &baseline.summary_info {
                context.diagnostics.push(ExportDiagnostic::loss(
                    ExportDiagnosticSource::DocumentField { name: "summary_info".to_owned() },
                    ExportAction::Skipped,
                    vec![ExportLossReason::DocumentSummaryInformation],
                ));
            }
        }
        SemanticPartV1::Preview(_) => {
            *collection_counts.entry("preview").or_default() += 1;
        }
        SemanticPartV1::Relationship { kind, source, target } => match kind {
            // Typed owner fields are checked by the existing entity/object scan.
            SemanticRelationshipKindV1::Ownership => {}
            SemanticRelationshipKindV1::ExtensionDictionary
            | SemanticRelationshipKindV1::Reactor => {
                let already_reported = match source {
                    SemanticReferenceV1::Resolved(SemanticNodeV1::Entity(entity)) => {
                        let common = entity.common();
                        let matches_target = |handle| match (inventory.resolve(handle), target) {
                            (Some(node), SemanticReferenceV1::Resolved(other)) =>
                                same_semantic_node(node, other),
                            _ => false,
                        };
                        if matches!(target, SemanticReferenceV1::Unresolved) {
                            let kind_key = match kind {
                                SemanticRelationshipKindV1::ExtensionDictionary => 1,
                                SemanticRelationshipKindV1::Reactor => 2,
                                SemanticRelationshipKindV1::Ownership => unreachable!(),
                            };
                            let remaining = unresolved_typed
                                .entry((entity as *const _, kind_key))
                                .or_insert_with(|| match kind {
                                    SemanticRelationshipKindV1::ExtensionDictionary => usize::from(
                                        common.xdictionary_handle.is_some_and(|handle| inventory.resolve(handle).is_none()),
                                    ),
                                    SemanticRelationshipKindV1::Reactor => common.reactors.iter().filter(|handle| inventory.resolve(**handle).is_none()).count(),
                                    SemanticRelationshipKindV1::Ownership => unreachable!(),
                                });
                            if *remaining > 0 {
                                *remaining -= 1;
                                true
                            } else {
                                false
                            }
                        } else {
                            match kind {
                                SemanticRelationshipKindV1::ExtensionDictionary => common.xdictionary_handle.is_some_and(matches_target),
                                SemanticRelationshipKindV1::Reactor => common.reactors.iter().copied().any(matches_target),
                                SemanticRelationshipKindV1::Ownership => unreachable!(),
                            }
                        }
                    }
                    _ => false,
                };
                relationships += usize::from(!already_reported);
            }
        },
        SemanticPartV1::NonEntityExtendedData { owner, application, values } => {
            // These two standard payloads duplicate typed layer properties already
            // mapped or diagnosed. Extra values and undecodable payloads remain loss.
            let represented = match (owner, application, values.as_deref()) {
                (SemanticReferenceV1::Resolved(SemanticNodeV1::TableRecord(SemanticTableRecordV1::Layer(layer))), Some(app), Some(values)) => {
                    if app.name.eq_ignore_ascii_case("AcCmTransparency") {
                        matches!(values, [XDataValue::Integer32(value)] if *value == layer.transparency.to_alpha_value())
                    } else if app.name.eq_ignore_ascii_case(cadcodec::tables::layer::LAYER_DESCRIPTION_APP) {
                        matches!(values, [XDataValue::String(prefix), XDataValue::String(description)]
                            if prefix.is_empty() && description == &layer.description)
                    } else {
                        false
                    }
                }
                _ => false,
            };
            extended_data += usize::from(!represented);
        }
    });
    for (kind, count) in table_counts {
        record_table(context, kind, count);
    }
    for (kind, count) in collection_counts {
        record_collection(context, kind, count);
    }
    record_collection(context, "inventory.additional_relationships", relationships);
    record_collection(context, "inventory.non_entity_extended_data", extended_data);
}

fn same_semantic_node(a: SemanticNodeV1<'_>, b: SemanticNodeV1<'_>) -> bool {
    match (a, b) {
        (SemanticNodeV1::Document, SemanticNodeV1::Document) => true,
        (SemanticNodeV1::Entity(a), SemanticNodeV1::Entity(b)) => std::ptr::eq(a, b),
        (SemanticNodeV1::Object(a), SemanticNodeV1::Object(b)) => std::ptr::eq(a, b),
        (SemanticNodeV1::TableRecord(a), SemanticNodeV1::TableRecord(b)) => match (a, b) {
            (SemanticTableRecordV1::Layer(a), SemanticTableRecordV1::Layer(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::LineType(a), SemanticTableRecordV1::LineType(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::TextStyle(a), SemanticTableRecordV1::TextStyle(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::BlockRecord(a), SemanticTableRecordV1::BlockRecord(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::DimStyle(a), SemanticTableRecordV1::DimStyle(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::AppId(a), SemanticTableRecordV1::AppId(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::View(a), SemanticTableRecordV1::View(b)) => std::ptr::eq(a, b),
            (SemanticTableRecordV1::VPort(a), SemanticTableRecordV1::VPort(b)) => {
                std::ptr::eq(a, b)
            }
            (SemanticTableRecordV1::Ucs(a), SemanticTableRecordV1::Ucs(b)) => std::ptr::eq(a, b),
            (SemanticTableRecordV1::Vx(a), SemanticTableRecordV1::Vx(b)) => std::ptr::eq(a, b),
            _ => false,
        },
        _ => false,
    }
}

fn scan_header(
    header: &cadcodec::document::HeaderVariables,
    baseline: &CadDocument,
    context: &mut ExportContext,
) {
    if !header.project_name.is_empty() {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.project_name".to_owned(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::UnsupportedHeaderField {
                name: "project_name".to_owned(),
            }],
        ));
    }
    let mut remaining = header.clone();
    let mut original = baseline.header.clone();
    remaining.project_name.clear();
    original.project_name.clear();
    remaining.insertion_units = original.insertion_units;
    remaining.measurement = original.measurement;
    remaining.paper_space_linetype_scaling = original.paper_space_linetype_scaling;
    remaining.plotstyle_mode = original.plotstyle_mode;
    normalize_header_bookkeeping(&mut remaining, &original);
    if remaining != original {
        context.diagnostics.push(ExportDiagnostic::loss(
            ExportDiagnosticSource::DocumentField {
                name: "header.other_semantics".to_owned(),
            },
            ExportAction::Skipped,
            vec![ExportLossReason::UnsupportedHeaderField {
                name: "other_header_semantics".to_owned(),
            }],
        ));
    }
}

fn unsupported_table_record(
    record: SemanticTableRecordV1<'_>,
    baseline: &CadDocument,
    context: &ExportContext,
    positions: &mut HashMap<(&'static str, String), usize>,
) -> Option<&'static str> {
    let (kind, name) = match &record {
        SemanticTableRecordV1::Layer(_) => return None,
        SemanticTableRecordV1::LineType(record) => ("line_types", &record.name),
        SemanticTableRecordV1::TextStyle(record) => ("text_styles", &record.name),
        SemanticTableRecordV1::BlockRecord(record) => ("block_records", &record.name),
        SemanticTableRecordV1::DimStyle(record) => ("dim_styles", &record.name),
        SemanticTableRecordV1::AppId(record) => ("app_ids", &record.name),
        SemanticTableRecordV1::View(record) => ("views", &record.name),
        SemanticTableRecordV1::VPort(record) => ("vports", &record.name),
        SemanticTableRecordV1::Ucs(record) => ("ucss", &record.name),
        SemanticTableRecordV1::Vx(record) => ("vx_table", &record.name),
    };
    let position = positions.entry((kind, name.to_uppercase())).or_default();
    if *position > 0 {
        *position += 1;
        return Some(kind);
    }
    *position += 1;
    macro_rules! changed {
        ($record:expr, $table:expr) => {{
            let record = $record;
            $table.get(&record.name).is_none_or(|original| {
                let mut candidate = record.clone();
                candidate.handle = original.handle;
                candidate != *original
            })
        }};
    }
    match record {
        SemanticTableRecordV1::Layer(_) => None,
        SemanticTableRecordV1::LineType(record) => {
            let supported = matches!(
                record.name.as_str(),
                "Continuous" | "ByLayer" | "ByBlock" | "Dashed"
            );
            let changed_dashed =
                if record.name == "Dashed" && baseline.line_types.get(&record.name).is_none() {
                    let mut candidate = record.clone();
                    candidate.handle = cadcodec::Handle::NULL;
                    candidate != cadcodec::LineType::dashed()
                } else {
                    false
                };
            (record.xref_block_record_handle != cadcodec::Handle::NULL
                || !supported
                || changed_dashed
                || (baseline.line_types.get(&record.name).is_some()
                    && changed!(record, baseline.line_types)))
            .then_some("line_types")
        }
        SemanticTableRecordV1::TextStyle(record) => {
            changed!(record, baseline.text_styles).then_some("text_styles")
        }
        SemanticTableRecordV1::BlockRecord(record) => {
            let changed = baseline
                .block_records
                .get(&record.name)
                .is_none_or(|original| {
                    let mut candidate = record.clone();
                    candidate.handle = original.handle;
                    candidate.block_entity_handle = original.block_entity_handle;
                    candidate.block_end_handle = original.block_end_handle;
                    candidate.layout = original.layout;
                    candidate.entity_handles = original.entity_handles.clone();
                    candidate.insert_handles = original.insert_handles.clone();
                    candidate != *original
                });
            (!context.blocks.contains_key(&record.handle)
                && !context.paper_scopes.contains_key(&record.handle)
                && changed)
                .then_some("block_records")
        }
        SemanticTableRecordV1::DimStyle(record) => {
            changed!(record, baseline.dim_styles).then_some("dim_styles")
        }
        SemanticTableRecordV1::AppId(record) => {
            changed!(record, baseline.app_ids).then_some("app_ids")
        }
        SemanticTableRecordV1::View(record) => changed!(record, baseline.views).then_some("views"),
        SemanticTableRecordV1::VPort(record) => {
            changed!(record, baseline.vports).then_some("vports")
        }
        SemanticTableRecordV1::Ucs(record) => changed!(record, baseline.ucss).then_some("ucss"),
        SemanticTableRecordV1::Vx(record) => {
            changed!(record, baseline.vx_table).then_some("vx_table")
        }
    }
}

fn unsupported_object(
    object: SemanticObjectV1<'_>,
    document: &CadDocument,
    baseline: &CadDocument,
    handles: &HashMap<cadcodec::Handle, cadcodec::Handle>,
    object_handles: &HashMap<*const cadcodec::objects::ObjectType, cadcodec::Handle>,
) -> bool {
    let SemanticObjectV1::Typed(object) = object else {
        return true;
    };
    if matches!(object, cadcodec::objects::ObjectType::Layout(_)) {
        return false;
    }
    let handle = object_handles.get(&(object as *const _));
    handle.is_none_or(|handle| {
        let role = handles.get(handle).unwrap_or(handle);
        baseline.objects.get(role).is_none_or(|original| {
            !same_bootstrap_object(object, original, document, baseline, handles, *role)
        })
    })
}

fn bootstrap_handle_map(
    document: &CadDocument,
    baseline: &CadDocument,
) -> HashMap<cadcodec::Handle, cadcodec::Handle> {
    use cadcodec::objects::ObjectType;
    let mut handles = HashMap::new();
    let mut pending = vec![(
        document.header.named_objects_dict_handle,
        baseline.header.named_objects_dict_handle,
    )];
    while let Some((actual, original)) = pending.pop() {
        if handles.insert(actual, original).is_some() {
            continue;
        }
        match (
            document.objects.get(&actual),
            baseline.objects.get(&original),
        ) {
            (Some(ObjectType::Dictionary(actual)), Some(ObjectType::Dictionary(original))) => {
                for (key, target) in &actual.entries {
                    if let Some((_, original_target)) =
                        original.entries.iter().find(|(name, _)| name == key)
                    {
                        pending.push((*target, *original_target));
                    }
                }
            }
            (
                Some(ObjectType::DictionaryWithDefault(actual)),
                Some(ObjectType::DictionaryWithDefault(original)),
            ) => {
                pending.push((actual.default_handle, original.default_handle));
                for (key, target) in &actual.entries {
                    if let Some((_, original_target)) =
                        original.entries.iter().find(|(name, _)| name == key)
                    {
                        pending.push((*target, *original_target));
                    }
                }
            }
            _ => {}
        }
    }
    handles
}

fn same_bootstrap_object(
    object: &cadcodec::objects::ObjectType,
    original: &cadcodec::objects::ObjectType,
    document: &CadDocument,
    baseline: &CadDocument,
    handles: &HashMap<cadcodec::Handle, cadcodec::Handle>,
    role: cadcodec::Handle,
) -> bool {
    use cadcodec::objects::ObjectType;
    let mapped = |handle| handles.get(&handle).copied().unwrap_or(handle);
    match (object, original) {
        (ObjectType::Dictionary(actual), ObjectType::Dictionary(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner = mapped(actual.owner);
            if role == baseline.header.acad_layout_dict_handle {
                actual.entries.retain(|(name, target)| {
                    default.entries.iter().any(|(key, _)| key == name)
                        || !matches!(document.objects.get(target), Some(ObjectType::Layout(layout)) if layout.name == *name)
                });
            }
            for (_, target) in &mut actual.entries {
                *target = mapped(*target);
            }
            actual == *default
        }
        (ObjectType::DictionaryWithDefault(actual), ObjectType::DictionaryWithDefault(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner = mapped(actual.owner);
            actual.default_handle = mapped(actual.default_handle);
            for (_, target) in &mut actual.entries {
                *target = mapped(*target);
            }
            actual == *default
        }
        (ObjectType::PlaceHolder(actual), ObjectType::PlaceHolder(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner = mapped(actual.owner);
            actual == *default
        }
        (ObjectType::MLineStyle(actual), ObjectType::MLineStyle(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner = mapped(actual.owner);
            actual == *default
        }
        (ObjectType::MultiLeaderStyle(actual), ObjectType::MultiLeaderStyle(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner_handle = mapped(actual.owner_handle);
            // The DXF writer materializes the standard style's implicit
            // ByLayer linetype reference as a handle.
            if actual.line_type_handle
                == document
                    .line_types
                    .get("ByLayer")
                    .map(|line_type| line_type.handle)
            {
                actual.line_type_handle = default.line_type_handle;
            }
            actual == *default
        }
        (ObjectType::TableStyle(actual), ObjectType::TableStyle(default)) => {
            let mut actual = actual.clone();
            actual.handle = mapped(actual.handle);
            actual.owner_handle = mapped(actual.owner_handle);
            // DXF stores style names; the reader cannot recover the optional
            // cached handle. The typed name remains part of the comparison.
            for (row, original_row) in [
                (&mut actual.data_row_style, &default.data_row_style),
                (&mut actual.header_row_style, &default.header_row_style),
                (&mut actual.title_row_style, &default.title_row_style),
            ] {
                if row.text_style_handle.is_none()
                    && row.text_style_name == original_row.text_style_name
                {
                    row.text_style_handle = original_row.text_style_handle;
                }
            }
            actual.raw_dxf_codes = default.raw_dxf_codes.clone();
            actual == *default
        }
        _ => object == original,
    }
}

fn normalize_header_bookkeeping(
    header: &mut cadcodec::document::HeaderVariables,
    baseline: &cadcodec::document::HeaderVariables,
) {
    macro_rules! normalize {
        ($($field:ident),+ $(,)?) => {
            $(header.$field = baseline.$field;)+
        };
    }
    normalize!(
        // Geometry-derived caches are recomputed from emitted entities. Drawing
        // limits and current defaults below remain independently meaningful.
        model_space_extents_min,
        model_space_extents_max,
        paper_space_extents_min,
        paper_space_extents_max,
        handle_seed,
        ucs_ortho_ref,
        paper_ucs_ortho_ref,
        current_layer_handle,
        current_text_style_handle,
        current_linetype_handle,
        current_dimstyle_handle,
        current_multiline_style_handle,
        current_material_handle,
        dim_text_style_handle,
        dim_linetype_handle,
        dim_linetype1_handle,
        dim_linetype2_handle,
        dim_arrow_block_handle,
        dim_arrow_block1_handle,
        dim_arrow_block2_handle,
        block_control_handle,
        layer_control_handle,
        style_control_handle,
        linetype_control_handle,
        view_control_handle,
        ucs_control_handle,
        vport_control_handle,
        appid_control_handle,
        dimstyle_control_handle,
        vpent_hdr_control_handle,
        current_vx_handle,
        named_objects_dict_handle,
        acad_group_dict_handle,
        acad_mlinestyle_dict_handle,
        acad_layout_dict_handle,
        acad_plotsettings_dict_handle,
        acad_plotstylename_dict_handle,
        acad_material_dict_handle,
        acad_color_dict_handle,
        acad_visualstyle_dict_handle,
        model_space_block_handle,
        paper_space_block_handle,
        bylayer_linetype_handle,
        byblock_linetype_handle,
        continuous_linetype_handle,
    );
}

fn record_table(context: &mut ExportContext, kind: &str, count: usize) {
    if count == 0 {
        return;
    }
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Table {
            kind: kind.to_owned(),
        },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedTableRecords {
            kind: kind.to_owned(),
            count,
        }],
    ));
}

fn record_collection(context: &mut ExportContext, kind: &str, count: usize) {
    if count == 0 {
        return;
    }
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Collection {
            kind: kind.to_owned(),
            count,
        },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedCollection {
            kind: kind.to_owned(),
            count,
        }],
    ));
}
