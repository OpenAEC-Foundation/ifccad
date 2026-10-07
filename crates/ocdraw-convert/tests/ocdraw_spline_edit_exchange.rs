//! Native editing across durable OCDraw saves and real DXF/AC1032 DWG readback.
use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::entities::{Insert, Spline};
use opencadcodec::{
    BlockRecord, CadDocument, Color, DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter,
    EntityType, Handle, Layer, Line, LineType, LineWeight, Transparency, Vector3 as CadVector,
};
use std::io::Cursor;

fn cubic() -> Spline {
    let mut s = Spline::new();
    s.control_points = vec![
        CadVector::new(0., 0., 0.),
        CadVector::new(1., 2., 0.),
        CadVector::new(2., 2., 0.),
        CadVector::new(3., 0., 0.),
    ];
    s.knots = vec![0., 0., 0., 0., 1., 1., 1., 1.];
    s
}

fn physical(cad: &CadDocument, dwg: bool) -> CadDocument {
    let mut cad = cad.clone();
    cad.version = DxfVersion::AC1032;
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&cad).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(&cad).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}

fn saved(doc: &OcdrawDocument) -> OcdrawDocument {
    validate_ocdraw_document(doc).unwrap();
    let encoded = encode_ocdraw_document(doc).unwrap();
    load_ocdraw_bytes(encoded.bytes()).unwrap().into_document()
}

fn add_members(source: &mut CadDocument, owner: Handle, absent_appearance: bool) {
    let mut line = Line::from_coords(0., 0., 0., 1., 0., 0.);
    line.common.owner_handle = owner;
    source.add_entity(EntityType::Line(line)).unwrap();
    let mut spline = cubic();
    spline.common.owner_handle = owner;
    if absent_appearance {
        spline.common.line_weight = LineWeight::Default;
    }
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let mut line = Line::from_coords(3., 0., 0., 4., 0., 0.);
    line.common.owner_handle = owner;
    source.add_entity(EntityType::Line(line)).unwrap();
}

fn captured(absent_appearance: bool) -> OcdrawDocument {
    let mut source = CadDocument::new();
    source.header.insertion_units = 4;
    let mut pattern = LineType::new("EditDash");
    pattern.handle = source.allocate_handle();
    pattern.elements = [4., -2.]
        .into_iter()
        .map(|length| opencadcodec::tables::LineTypeElement {
            length,
            complex: None,
        })
        .collect();
    pattern.pattern_length = 6.;
    source.line_types.add(pattern).unwrap();
    let mut layer = Layer::new("Alternate");
    layer.handle = source.allocate_handle();
    layer.color = Color::Rgb {
        r: 80,
        g: 90,
        b: 100,
    };
    layer.line_weight = LineWeight::Value(25);
    layer.transparency = Transparency::Explicit(0);
    source.layers.add(layer).unwrap();
    let model = source.header.model_space_block_handle;
    add_members(&mut source, model, absent_appearance);
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
    add_members(&mut source, paper, absent_appearance);
    let mut leaf = BlockRecord::new("EditLeaf");
    leaf.handle = source.allocate_handle();
    leaf.base_point = CadVector::new(1., 0., 0.);
    let leaf_owner = leaf.handle;
    source.block_records.add(leaf).unwrap();
    add_members(&mut source, leaf_owner, absent_appearance);
    let mut parent = BlockRecord::new("EditParent");
    parent.handle = source.allocate_handle();
    let parent_owner = parent.handle;
    source.block_records.add(parent).unwrap();
    let mut inner = Insert::new("EditLeaf", CadVector::new(5., 0., 0.));
    inner.common.owner_handle = parent_owner;
    source.add_entity(EntityType::Insert(inner)).unwrap();
    source
        .add_entity(EntityType::Insert(Insert::new(
            "EditParent",
            CadVector::new(10., 0., 0.),
        )))
        .unwrap();
    let mut unused = BlockRecord::new("EditUnused");
    unused.handle = source.allocate_handle();
    let unused_owner = unused.handle;
    source.block_records.add(unused).unwrap();
    add_members(&mut source, unused_owner, absent_appearance);
    let outcome = cad_document_to_encoded_ocdraw(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    drop(source);
    let doc = load_ocdraw_bytes(outcome.encoded().bytes())
        .unwrap()
        .into_document();
    assert_eq!(doc.opaque_entities.len(), 4);
    doc
}

fn assert_parameters(actual: &Spline, expected: &Spline) {
    let bits = |v: &[f64]| v.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    let vectors = |v: &[CadVector]| {
        v.iter()
            .map(|v| [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()])
            .collect::<Vec<_>>()
    };
    assert_eq!(actual.degree, expected.degree);
    assert_eq!(actual.flags, expected.flags);
    assert_eq!(bits(&actual.knots), bits(&expected.knots));
    assert_eq!(bits(&actual.weights), bits(&expected.weights));
    assert_eq!(
        vectors(&actual.control_points),
        vectors(&expected.control_points)
    );
    assert_eq!(vectors(&actual.fit_points), vectors(&expected.fit_points));
    assert_eq!(
        vectors(&[actual.normal, actual.begin_tangent, actual.end_tangent]),
        vectors(&[
            expected.normal,
            expected.begin_tangent,
            expected.end_tangent
        ])
    );
    assert_eq!(
        bits(&[
            actual.knot_tolerance,
            actual.control_tolerance,
            actual.fit_tolerance
        ]),
        bits(&[
            expected.knot_tolerance,
            expected.control_tolerance,
            expected.fit_tolerance
        ])
    );
    assert_eq!(
        (
            actual.knot_parameterization,
            actual.cv_frame_visible,
            actual.dwg_flags1,
            actual.dxf_flags
        ),
        (
            expected.knot_parameterization,
            expected.cv_frame_visible,
            expected.dwg_flags1,
            expected.dxf_flags
        )
    );
}

fn scope_owner(doc: &OcdrawDocument, cad: &CadDocument, scope: &DrawingScope) -> Handle {
    match scope.kind {
        DrawingScopeKind::Model => cad.header.model_space_block_handle,
        DrawingScopeKind::Paper => {
            let name = &doc
                .layouts
                .iter()
                .find(|l| l.scope_id == scope.id)
                .unwrap()
                .name;
            cad.objects
                .values()
                .find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l) if &l.name == name => {
                        Some(l.block_record)
                    }
                    _ => None,
                })
                .unwrap()
        }
        DrawingScopeKind::Block => {
            cad.block_records
                .get(
                    &doc.block_definitions
                        .iter()
                        .find(|b| b.scope_id == scope.id)
                        .unwrap()
                        .name,
                )
                .unwrap()
                .handle
        }
    }
}

fn assert_order(doc: &OcdrawDocument, cad: &CadDocument) {
    assert!(
        doc.viewports.is_empty(),
        "fixture has no authored viewport entities"
    );
    for scope in &doc.scopes {
        let expected = scope
            .entities
            .iter()
            .map(|id| {
                if doc.opaque_entities.iter().any(|e| e.id == *id) {
                    "SPLINE"
                } else if doc.geometric_entities.iter().any(|e| {
                    e.id == *id && matches!(e.geometry, DrawingGeometry::BlockInstance { .. })
                }) {
                    "INSERT"
                } else {
                    "LINE"
                }
            })
            .collect::<Vec<_>>();
        let owner = scope_owner(doc, cad, scope);
        let record = cad
            .block_records
            .iter()
            .find(|r| r.handle == owner)
            .unwrap();
        let actual = record
            .entity_handles
            .iter()
            .filter_map(|h| cad.get_entity(*h))
            .filter(|e| !matches!(e, EntityType::Block(_) | EntityType::BlockEnd(_)))
            .filter(|e| {
                if matches!(e,EntityType::Viewport(v) if v.id==1) {
                    assert!(
                        scope.kind == DrawingScopeKind::Paper
                            && doc.paper_canvases.iter().any(|c| c.scope_id == scope.id),
                        "overall viewport must correspond to saved paper workspace"
                    );
                    false
                } else {
                    true
                }
            })
            .map(|e| e.as_entity().entity_type())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "scope {}", scope.id);
    }
}

#[derive(Clone, Copy, Debug)]
enum Edit {
    LayerReassign,
    LayerDefaults,
    PatternDefinition,
    PatternRename,
    GlobalScale,
    EntityAppearance,
    SupplyAbsentAppearance,
    NativeGeometry,
    BlockBase,
    InstanceTransform,
    LayoutAndOrder,
    DeleteLayerReassign,
}

fn appearance() -> EntityAppearance {
    EntityAppearance {
        color: AppearanceSelection::Explicit(DrawingColor::rgb(5, 10, 15)),
        opacity: AppearanceSelection::Explicit(0.8),
        line_pattern: AppearanceSelection::ByLayer,
        line_weight: AppearanceSelection::Explicit(0.5),
        line_pattern_scale: 2.,
    }
}

fn apply(doc: &mut OcdrawDocument, edit: Edit) {
    let alternate = doc
        .layers
        .iter()
        .find(|l| l.name == "Alternate")
        .unwrap()
        .id;
    let original = doc.layers.iter().find(|l| l.name == "0").unwrap().id;
    let dash = doc
        .line_patterns
        .iter()
        .find(|p| p.name == "EditDash")
        .unwrap()
        .id;
    match edit {
        Edit::LayerReassign => {
            for e in &mut doc.opaque_entities {
                e.layer_id = Some(alternate);
            }
        }
        Edit::LayerDefaults => {
            let l = doc.layers.iter_mut().find(|l| l.id == original).unwrap();
            l.color = DrawingColor::rgb(20, 40, 60);
            l.opacity = 0.8;
            l.line_weight = 0.5;
            l.line_pattern_id = dash;
        }
        Edit::PatternDefinition | Edit::PatternRename => {
            for e in &mut doc.opaque_entities {
                e.appearance.as_mut().unwrap().line_pattern = AppearanceSelection::Explicit(dash);
            }
            let p = doc.line_patterns.iter_mut().find(|p| p.id == dash).unwrap();
            if matches!(edit, Edit::PatternDefinition) {
                p.pattern = vec![6., -3., 0., -1.];
            } else {
                p.name = "RenamedDash".into();
            }
        }
        Edit::GlobalScale => doc.line_pattern_scale = 3.5,
        Edit::EntityAppearance | Edit::SupplyAbsentAppearance => {
            for e in &mut doc.opaque_entities {
                e.appearance = Some(appearance());
                e.visible = false;
            }
        }
        Edit::NativeGeometry => {
            let model = doc
                .scopes
                .iter()
                .find(|s| s.kind == DrawingScopeKind::Model)
                .unwrap();
            let e = doc
                .geometric_entities
                .iter_mut()
                .find(|e| {
                    model.entities.contains(&e.id)
                        && matches!(e.geometry, DrawingGeometry::Line { .. })
                })
                .unwrap();
            e.geometry = DrawingGeometry::Line {
                start: [0., 0., 0.],
                end: [8., 2., 0.],
            };
        }
        Edit::BlockBase => {
            doc.block_definitions
                .iter_mut()
                .find(|d| d.name == "EditLeaf")
                .unwrap()
                .base_point = [2., 1., 0.]
        }
        Edit::InstanceTransform => {
            let model = doc
                .scopes
                .iter()
                .find(|s| s.kind == DrawingScopeKind::Model)
                .unwrap();
            let e = doc
                .geometric_entities
                .iter_mut()
                .find(|e| {
                    model.entities.contains(&e.id)
                        && matches!(e.geometry, DrawingGeometry::BlockInstance { .. })
                })
                .unwrap();
            if let DrawingGeometry::BlockInstance { transform, .. } = &mut e.geometry {
                let frame = CoordinateFrame3::try_new(
                    Point3::new(30., 4., 2.),
                    Vector3::new(1., 0., 0.),
                    Vector3::new(0., 1., 0.),
                )
                .unwrap();
                *transform = BlockTransform::try_new(frame, 0., Scale3::new(2., 2., 2.)).unwrap();
            }
        }
        Edit::LayoutAndOrder => {
            for scope in &mut doc.scopes {
                scope.entities.reverse();
            }
            doc.layouts
                .iter_mut()
                .find(|l| l.kind == DrawingLayoutKind::Paper)
                .unwrap()
                .name = "RenamedSheet".into();
        }
        Edit::DeleteLayerReassign => {
            for e in &mut doc.opaque_entities {
                e.layer_id = Some(alternate);
            }
            for e in &mut doc.geometric_entities {
                e.layer_id = alternate;
            }
            if let Some(w) = &mut doc.workspace_state {
                if w.current_layer_id == Some(original) {
                    w.current_layer_id = Some(alternate);
                }
            }
            doc.layers.retain(|l| l.id != original);
        }
    }
}

fn assert_edit(cad: &CadDocument, edit: Edit) {
    let splines = cad
        .entities()
        .filter_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(splines.len(), 4, "{edit:?}");
    for spline in splines {
        match edit {
            Edit::LayerReassign | Edit::DeleteLayerReassign => {
                assert_eq!(spline.common.layer, "Alternate")
            }
            Edit::LayerDefaults => {
                assert_eq!(spline.common.color, Color::ByLayer);
                assert_eq!(spline.common.line_weight, LineWeight::ByLayer);
            }
            Edit::PatternDefinition => assert_eq!(spline.common.linetype, "EditDash"),
            Edit::PatternRename => assert_eq!(spline.common.linetype, "RenamedDash"),
            Edit::EntityAppearance | Edit::SupplyAbsentAppearance => {
                assert_eq!(spline.common.color, Color::Rgb { r: 5, g: 10, b: 15 });
                assert_eq!(spline.common.transparency, Transparency::Explicit(51));
                assert_eq!(spline.common.line_weight, LineWeight::Value(50));
                assert_eq!(spline.common.linetype_scale.to_bits(), 2_f64.to_bits());
                assert!(spline.common.invisible);
            }
            _ => {}
        }
    }
    match edit {
        Edit::LayerDefaults => {
            let l = cad.layers.get("0").unwrap();
            assert_eq!(
                l.color,
                Color::Rgb {
                    r: 20,
                    g: 40,
                    b: 60
                }
            );
            assert_eq!(l.transparency, Transparency::Explicit(51));
            assert_eq!(l.line_weight, LineWeight::Value(50));
            assert_eq!(l.line_type, "EditDash");
        }
        Edit::PatternDefinition => assert_eq!(
            cad.line_types
                .get("EditDash")
                .unwrap()
                .elements
                .iter()
                .map(|e| e.length)
                .collect::<Vec<_>>(),
            vec![6., -3., 0., -1.]
        ),
        Edit::PatternRename => assert!(cad.line_types.get("RenamedDash").is_some()),
        Edit::GlobalScale => assert_eq!(cad.header.linetype_scale.to_bits(), 3.5_f64.to_bits()),
        Edit::NativeGeometry => assert!(cad
            .entities()
            .any(|e| matches!(e,EntityType::Line(l) if l.end==CadVector::new(8.,2.,0.)))),
        Edit::BlockBase => assert_eq!(
            cad.block_records.get("EditLeaf").unwrap().base_point,
            CadVector::new(2., 1., 0.)
        ),
        Edit::InstanceTransform => {
            let i = cad
                .entities()
                .find_map(|e| match e {
                    EntityType::Insert(i) if i.block_name == "EditParent" => Some(i),
                    _ => None,
                })
                .unwrap();
            assert_eq!(i.insert_point, CadVector::new(30., 4., 2.));
            assert_eq!((i.x_scale(), i.y_scale(), i.z_scale()), (2., 2., 2.));
        }
        _ => {}
    }
}

fn verify(edit: Edit) {
    let mut doc = captured(matches!(edit, Edit::SupplyAbsentAppearance));
    if matches!(edit, Edit::SupplyAbsentAppearance) {
        assert!(doc.opaque_entities.iter().all(|e| e.appearance.is_none()));
    }
    let original = doc.preservation.clone();
    apply(&mut doc, edit);
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    let restored = saved(&doc);
    assert_eq!(
        restored.preservation, original,
        "snapshot/baseline mutated by {edit:?}"
    );
    assert_eq!(restored.opaque_entities, doc.opaque_entities);
    assert_eq!(restored.scopes, doc.scopes);
    let outcome =
        ocdraw_document_to_cad_document(&restored, OcdrawToCadOptions::default()).unwrap();
    assert_eq!(outcome.preservation_report().entries().len(), 4);
    assert!(
        outcome
            .preservation_report()
            .entries()
            .iter()
            .all(|e| e.result == OcdrawPreservationResult::RestoredTyped),
        "{:?}",
        outcome.preservation_report()
    );
    assert_edit(outcome.document(), edit);
    assert_order(&restored, outcome.document());
    for s in outcome.document().entities().filter_map(|e| {
        if let EntityType::Spline(s) = e {
            Some(s)
        } else {
            None
        }
    }) {
        assert_parameters(s, &cubic());
    }
    for dwg in [false, true] {
        // The direct source chain is independent of preservation and characterizes
        // only codec normalization of the unchanged spline parameter carrier.
        let mut direct = CadDocument::new();
        direct.add_entity(EntityType::Spline(cubic())).unwrap();
        let baseline = physical(&direct, dwg);
        let expected = baseline
            .entities()
            .find_map(|e| {
                if let EntityType::Spline(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap();
        let cad = physical(outcome.document(), dwg);
        assert_edit(&cad, edit);
        assert_order(&restored, &cad);
        for s in cad.entities().filter_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        }) {
            assert_parameters(s, expected);
        }
    }
}

#[test]
fn layer_reassignment_survives_edit_save_reopen_dwg() {
    verify(Edit::LayerReassign);
}
#[test]
fn layer_defaults_survive_edit_save_reopen_dwg() {
    verify(Edit::LayerDefaults);
}
#[test]
fn pattern_definition_edits_survive_edit_save_reopen_dwg() {
    verify(Edit::PatternDefinition);
}
#[test]
fn pattern_rename_survives_edit_save_reopen_dwg() {
    verify(Edit::PatternRename);
}
#[test]
fn global_pattern_scale_survives_edit_save_reopen_dwg() {
    verify(Edit::GlobalScale);
}
#[test]
fn edited_common_properties_survive_edit_save_reopen_dwg() {
    verify(Edit::EntityAppearance);
}
#[test]
fn native_appearance_supplied_after_absence_survives_edit_save_reopen_dwg() {
    verify(Edit::SupplyAbsentAppearance);
}
#[test]
fn unrelated_native_geometry_edits_survive_edit_save_reopen_dwg() {
    verify(Edit::NativeGeometry);
}
#[test]
fn block_basepoint_edits_survive_edit_save_reopen_dwg() {
    verify(Edit::BlockBase);
}
#[test]
fn instance_transform_edits_survive_edit_save_reopen_dwg() {
    verify(Edit::InstanceTransform);
}
#[test]
fn layout_names_and_draworder_survive_edit_save_reopen_dwg() {
    verify(Edit::LayoutAndOrder);
}
#[test]
fn deleted_layer_reassignment_survives_edit_save_reopen_dwg() {
    verify(Edit::DeleteLayerReassign);
}

#[test]
fn clearing_native_layer_after_deletion_keeps_saved_snapshot_and_refuses_restore() {
    let mut doc = captured(false);
    let original = doc.preservation.clone();
    let layer = doc.layers.iter().find(|l| l.name == "0").unwrap().id;
    doc.layers.retain(|l| l.id != layer);
    assert!(
        validate_ocdraw_document(&doc).is_err(),
        "present native references remain hard"
    );
    let alternate = doc.layers[0].id;
    for e in &mut doc.opaque_entities {
        e.layer_id = None;
    }
    for e in &mut doc.geometric_entities {
        e.layer_id = alternate;
    }
    if let Some(w) = &mut doc.workspace_state {
        w.current_layer_id = Some(alternate);
    }
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    let restored = saved(&doc);
    assert_eq!(restored.preservation, original);
    let out = ocdraw_document_to_cad_document(&restored, OcdrawToCadOptions::default()).unwrap();
    assert_eq!(out.preservation_report().entries().len(), 4);
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .all(|e| e.reason == Some(OcdrawPreservationReason::MissingDependency)));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &restored,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
    for dwg in [false, true] {
        let cad = physical(out.document(), dwg);
        assert!(!cad.entities().any(|e| matches!(e, EntityType::Spline(_))));
    }
}

#[derive(Clone, Copy, Debug)]
enum InvalidCondition {
    MissingEntity,
    MissingCoordinate,
    DuplicateEntity,
    FutureVersion,
    WrongEntityTarget,
    WrongRecordBaseline,
    MalformedBaseline,
    InvalidUnitBaseline,
    ReferenceTargetMismatch,
    ReferenceSlotMismatch,
    ReferenceKeyMismatch,
    ReferenceRoleMismatch,
    MissingRequiredReference,
    FuturePayload,
}

#[test]
fn condition_failures_are_rechecked_after_save_and_reopen() {
    use OcdrawPreservationReason::*;
    let cases = [
        (InvalidCondition::MissingEntity, UnsupportedContext),
        (InvalidCondition::MissingCoordinate, UnsupportedContext),
        (InvalidCondition::DuplicateEntity, UnsupportedContext),
        (InvalidCondition::FutureVersion, UnsupportedPredicate),
        (InvalidCondition::WrongEntityTarget, UnsupportedContext),
        (InvalidCondition::WrongRecordBaseline, ChangedDependency),
        (InvalidCondition::MalformedBaseline, MalformedPayload),
        (InvalidCondition::InvalidUnitBaseline, MalformedPayload),
        (
            InvalidCondition::ReferenceTargetMismatch,
            UnsupportedContext,
        ),
        (InvalidCondition::ReferenceSlotMismatch, UnsupportedContext),
        (InvalidCondition::ReferenceKeyMismatch, UnsupportedContext),
        (InvalidCondition::ReferenceRoleMismatch, UnsupportedContext),
        (
            InvalidCondition::MissingRequiredReference,
            UnsupportedContext,
        ),
        (InvalidCondition::FuturePayload, UnsupportedPayload),
    ];
    const ENTITY: &str = "openaec.ocdraw.splineEntityBinding";
    const COORDINATE: &str = "openaec.ocdraw.splineCoordinateContext";
    const REFERENCE: &str = "openaec.ocdraw.sourceReferenceBinding";
    for (case, reason) in cases {
        let mut doc = captured(false);
        let target = doc.opaque_entities[0].id;
        let record_id = doc.opaque_entities[0].preservation_record_id;
        if matches!(case, InvalidCondition::MissingRequiredReference) {
            doc.opaque_entities[0].layer_id = None;
        }
        let record = doc
            .preservation
            .as_mut()
            .unwrap()
            .records
            .iter_mut()
            .find(|r| r.id == record_id)
            .unwrap();
        record.dependency_coverage = OcdrawPreservationDependencyCoverage::Qualified;
        match case {
            InvalidCondition::MissingEntity => record.conditions.retain(|c| c.predicate != ENTITY),
            InvalidCondition::MissingCoordinate => {
                record.conditions.retain(|c| c.predicate != COORDINATE)
            }
            InvalidCondition::DuplicateEntity => record.conditions.push(
                record
                    .conditions
                    .iter()
                    .find(|c| c.predicate == ENTITY)
                    .unwrap()
                    .clone(),
            ),
            InvalidCondition::FutureVersion => {
                record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == ENTITY)
                    .unwrap()
                    .version = 2
            }
            InvalidCondition::WrongEntityTarget => {
                record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == ENTITY)
                    .unwrap()
                    .target = OcdrawPreservationTarget::Layer(0)
            }
            InvalidCondition::WrongRecordBaseline => {
                let c = record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == ENTITY)
                    .unwrap();
                let mut b: serde_json::Value = serde_json::from_slice(&c.baseline).unwrap();
                b["preservationRecordId"] = serde_json::json!(999);
                c.baseline = serde_json::to_vec(&b).unwrap();
            }
            InvalidCondition::MalformedBaseline => {
                record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == ENTITY)
                    .unwrap()
                    .baseline = vec![0xff]
            }
            InvalidCondition::InvalidUnitBaseline => {
                let c = record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == COORDINATE)
                    .unwrap();
                let mut b: serde_json::Value = serde_json::from_slice(&c.baseline).unwrap();
                b["coordinateUnit"] = serde_json::json!("invented-unit");
                c.baseline = serde_json::to_vec(&b).unwrap();
            }
            InvalidCondition::ReferenceTargetMismatch => {
                record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == REFERENCE)
                    .unwrap()
                    .target = OcdrawPreservationTarget::Layer(999)
            }
            InvalidCondition::ReferenceSlotMismatch
            | InvalidCondition::ReferenceKeyMismatch
            | InvalidCondition::ReferenceRoleMismatch => {
                let c = record
                    .conditions
                    .iter_mut()
                    .find(|c| c.predicate == REFERENCE)
                    .unwrap();
                let mut b: serde_json::Value = serde_json::from_slice(&c.baseline).unwrap();
                match case {
                    InvalidCondition::ReferenceSlotMismatch => {
                        b["slot"] = serde_json::json!("not-a-slot")
                    }
                    InvalidCondition::ReferenceKeyMismatch => {
                        b["sourceKey"] = serde_json::json!("not-the-source")
                    }
                    InvalidCondition::ReferenceRoleMismatch => {
                        b["targetRole"] = serde_json::json!("entity")
                    }
                    _ => unreachable!(),
                }
                c.baseline = serde_json::to_vec(&b).unwrap();
            }
            InvalidCondition::MissingRequiredReference => {
                record.conditions.retain(|c| c.predicate != REFERENCE)
            }
            InvalidCondition::FuturePayload => record.payload.version = 3,
        }
        let reopened = saved(&doc);
        assert_eq!(
            reopened.preservation, doc.preservation,
            "condition bytes changed for {case:?}"
        );
        let out =
            ocdraw_document_to_cad_document(&reopened, OcdrawToCadOptions::default()).unwrap();
        assert!(
            out.preservation_report()
                .entries()
                .iter()
                .any(|e| e.entity_id == Some(target) && e.reason == Some(reason)),
            "{case:?}: {:?}",
            out.preservation_report()
        );
        assert_eq!(
            out.document()
                .entities()
                .filter(|e| matches!(e, EntityType::Spline(_)))
                .count(),
            3,
            "only the affected spline must be skipped: {case:?}"
        );
        assert!(
            matches!(
                ocdraw_document_to_cad_document(
                    &reopened,
                    OcdrawToCadOptions {
                        loss_policy: OcdrawLossPolicy::Reject,
                        ..Default::default()
                    }
                ),
                Err(OcdrawToCadError::LossRejected { .. })
            ),
            "{case:?}"
        );
    }
}

#[test]
fn changed_coordinate_unit_remains_ineligible_across_saves_and_reverting_restores_dwg() {
    let mut doc = captured(false);
    let original = doc.preservation.clone();
    doc.unit = "cm".into();
    let reopened = saved(&doc);
    assert_eq!(reopened.preservation, original);
    let out = ocdraw_document_to_cad_document(&reopened, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .all(|e| e.reason == Some(OcdrawPreservationReason::ChangedDependency)));
    let mut reverted = reopened;
    reverted.unit = "mm".into();
    let reopened = saved(&reverted);
    assert_eq!(reopened.preservation, original);
    let out = ocdraw_document_to_cad_document(&reopened, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .all(|e| e.result == OcdrawPreservationResult::RestoredTyped));
    assert_eq!(
        physical(out.document(), true)
            .entities()
            .filter(|e| matches!(e, EntityType::Spline(_)))
            .count(),
        4
    );
}

#[test]
fn edited_native_file_is_closed_reopened_from_disk_and_exported_to_dwg() {
    let mut doc = captured(false);
    let original = doc.preservation.clone();
    apply(&mut doc, Edit::EntityAppearance);
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "ocdraw-spline-edit-{}-{nonce}.ocdraw.json",
        std::process::id()
    ));
    encoded.write_file(&path).unwrap(); // Production storage refuses overwrites.
    drop(encoded);
    drop(doc);
    let read = load_ocdraw_file(&path);
    std::fs::remove_file(&path).unwrap(); // Only the file created above is removed.
    let doc = read.unwrap().into_document();
    assert_eq!(doc.preservation, original);
    let outcome = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let cad = physical(outcome.document(), true);
    assert_edit(&cad, Edit::EntityAppearance);
    assert_order(&doc, &cad);
}
