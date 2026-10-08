use opencadcodec::entities::Viewport;
use opencadcodec::{
    objects::ObjectType, CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType,
    Handle, Ucs, VPort, Vector2, Vector3,
};
use std::io::Cursor;

fn dxf(bytes: &[u8]) -> CadDocument {
    DxfReader::from_reader(Cursor::new(bytes.to_vec()))
        .unwrap()
        .read()
        .unwrap()
}
fn transfer(source: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(source).unwrap()))
            .read()
            .unwrap()
    } else {
        dxf(&DxfWriter::new(source).write_to_vec().unwrap())
    }
}
fn source() -> CadDocument {
    let mut doc = CadDocument::new();
    doc.add_layout("Second").unwrap();
    doc.vports.clear();
    for (rect, height) in [([0., 0., 0.5, 1.], 20.), ([0.5, 0., 1., 1.], 30.)] {
        let mut v = VPort::active();
        v.handle = doc.allocate_handle();
        v.lower_left = Vector2::new(rect[0], rect[1]);
        v.upper_right = Vector2::new(rect[2], rect[3]);
        v.view_height = height;
        v.grid_spacing = Vector2::new(2., 3.);
        v.snap_spacing = Vector2::ZERO;
        v.snap_on = false;
        v.snap_rotation = std::f64::consts::FRAC_PI_2;
        v.ucs_origin = Vector3::new(1., 2., 0.);
        v.ucs_per_viewport = false;
        doc.vports.add_allow_duplicate(v);
    }
    let mut unused = Ucs::new("Unused");
    unused.handle = doc.allocate_handle();
    unused.origin = Vector3::new(3., 4., 5.);
    unused.elevation = 6.;
    doc.ucss.add(unused).unwrap();
    let layouts: Vec<_> = doc
        .objects
        .values()
        .filter_map(|o| match o {
            ObjectType::Layout(l) if l.name != "Model" => {
                Some((l.handle, l.block_record, l.viewport))
            }
            _ => None,
        })
        .collect();
    for (layout, owner, existing) in layouts {
        let canvas = if matches!(doc.get_entity(existing), Some(EntityType::Viewport(_))) {
            existing
        } else {
            let mut v = Viewport::new();
            v.id = 1;
            v.common.owner_handle = owner;
            let h = doc.add_entity(EntityType::Viewport(v)).unwrap();
            if let ObjectType::Layout(l) = doc.objects.get_mut(&layout).unwrap() {
                l.viewport = h;
                l.viewports.insert(0, h);
            }
            h
        };
        if let Some(EntityType::Viewport(v)) = doc.get_entity_mut(canvas) {
            v.center = Vector3::new(0.25, 0.5, 0.);
            v.width = 10.;
            v.height = 8.;
            v.grid_spacing = Vector3::new(2., 3., 0.);
            v.snap_spacing = Vector3::ZERO;
            v.status.snap_on = false;
            v.ucs_origin = Vector3::new(1., 2., 0.);
            v.ucs_per_viewport = false;
        }
        let mut v = Viewport::new();
        v.id = 2;
        v.common.owner_handle = owner;
        v.grid_spacing = Vector3::new(2., 3., 0.);
        v.snap_spacing = Vector3::ZERO;
        v.status.snap_on = false;
        v.snap_angle = std::f64::consts::FRAC_PI_2;
        v.ucs_origin = Vector3::new(1., 2., 0.);
        v.ucs_per_viewport = false;
        v.status.iso_pair_top = true;
        v.status.iso_pair_right = true;
        let h = doc.add_entity(EntityType::Viewport(v)).unwrap();
        if let ObjectType::Layout(l) = doc.objects.get_mut(&layout).unwrap() {
            l.viewports.push(h);
        }
    }
    doc
}

#[test]
fn literal_dxf_workspace_fields_and_active_order_are_visible() {
    let doc = dxf(include_bytes!("fixtures/workspace-reference.dxf"));
    let windows: Vec<_> = doc
        .vports
        .iter()
        .filter(|v| v.name.eq_ignore_ascii_case("*ACTIVE"))
        .collect();
    assert_eq!(
        windows.iter().map(|v| v.view_height).collect::<Vec<_>>(),
        vec![20., 30.]
    );
    assert_eq!(windows[0].snap_spacing, Vector2::ZERO);
    assert_eq!(windows[0].grid_spacing, Vector2::new(2., 3.));
    let unused = doc.ucss.get("Unused").unwrap();
    assert_eq!(unused.origin, Vector3::new(3., 4., 5.));
    assert_eq!(unused.elevation, 6.);
    let v = match doc.get_entity(Handle::new(0x41)) {
        Some(EntityType::Viewport(v)) => v,
        _ => panic!("literal viewport"),
    };
    assert_eq!(v.snap_spacing, Vector3::ZERO);
    assert!(v.status.iso_pair_top && v.status.iso_pair_right);
}

#[test]
fn workspace_values_survive_dxf_and_ac1032_dwg() {
    let source = source();
    for dwg in [false, true] {
        let back = transfer(&source, dwg);
        let mut heights: Vec<_> = back
            .vports
            .iter()
            .filter(|v| v.name.eq_ignore_ascii_case("*ACTIVE"))
            .map(|v| v.view_height)
            .collect();
        heights.sort_by(f64::total_cmp);
        assert_eq!(heights, vec![20., 30.], "{dwg}");
        for v in back
            .vports
            .iter()
            .filter(|v| v.name.eq_ignore_ascii_case("*ACTIVE"))
        {
            assert_eq!(v.grid_spacing, Vector2::new(2., 3.), "{dwg}");
            assert_eq!(v.snap_spacing, Vector2::ZERO, "{dwg}");
            assert!(!v.snap_on);
            assert_eq!(v.ucs_origin, Vector3::new(1., 2., 0.), "{dwg}");
            assert!(!v.ucs_per_viewport, "{dwg}");
            assert_eq!(v.snap_rotation, std::f64::consts::FRAC_PI_2, "{dwg}");
        }
        let unused = back.ucss.get("Unused").unwrap();
        assert_eq!(unused.origin, Vector3::new(3., 4., 5.), "{dwg}");
        assert_eq!(unused.elevation, 6., "{dwg}");
        let viewports: Vec<_> = back
            .entities()
            .filter_map(|e| match e {
                EntityType::Viewport(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(viewports.len(), 4, "{dwg}");
        for v in viewports {
            assert_eq!(v.grid_spacing, Vector3::new(2., 3., 0.), "{dwg}");
            assert_eq!(v.snap_spacing, Vector3::ZERO, "{dwg}");
            assert!(!v.status.snap_on);
            assert_eq!(v.ucs_origin, Vector3::new(1., 2., 0.), "{dwg}");
            assert!(!v.ucs_per_viewport, "{dwg}");
            if v.width == 10. && v.height == 8. {
                assert_eq!(v.center, Vector3::new(0.25, 0.5, 0.), "{dwg}");
            }
        }
    }
}

#[test]
fn positive_viewport_stack_rank_is_not_a_current_identity() {
    let text = std::str::from_utf8(include_bytes!("fixtures/workspace-reference.dxf")).unwrap();
    let first = dxf(text.as_bytes());
    let third = dxf(text.replace("68\n1\n69", "68\n3\n69").as_bytes());
    let viewport = |d: &CadDocument| match d.get_entity(Handle::new(0x41)).unwrap() {
        EntityType::Viewport(v) => serde_json::to_value(v).unwrap(),
        _ => panic!("viewport"),
    };
    assert_eq!(viewport(&first), viewport(&third));
    assert!(!text.contains("$CVPORT"));
}

#[test]
fn isometric_pair_bits_are_preserved_by_the_codec() {
    use opencadcodec::entities::ViewportStatusFlags;
    for bits in [0, 0x1000, 0x2000, 0x3000] {
        let flags = ViewportStatusFlags::from_bits(bits);
        assert_eq!(flags.to_bits() & 0x3000, bits);
        assert_eq!(flags.iso_pair_top, bits & 0x1000 != 0);
        assert_eq!(flags.iso_pair_right, bits & 0x2000 != 0);
    }
}

#[test]
fn dwg_secondary_paper_markers_and_entities_keep_their_owner() {
    let mut doc = source();
    let line = doc
        .add_entity_to_layout(
            EntityType::Line(opencadcodec::Line::from_points(
                Vector3::ZERO,
                Vector3::new(10., 0., 0.),
            )),
            "Second",
        )
        .unwrap();
    let back = transfer(&doc, true);
    let extra = back.block_records.get("*Paper_Space0").unwrap();
    let Some(EntityType::Block(marker)) = back.get_entity(extra.block_entity_handle) else {
        panic!("paper marker")
    };
    assert_eq!(marker.common.owner_handle, extra.handle);
    assert_eq!(
        back.get_entity(line).unwrap().common().owner_handle,
        extra.handle
    );
}

#[test]
fn dwg_overall_role_uses_layout_link_instead_of_entity_order() {
    let mut doc = source();
    let paper = doc.block_records.get("*Paper_Space").unwrap().handle;
    let overall = doc
        .objects
        .values()
        .find_map(|o| match o {
            ObjectType::Layout(l) if l.block_record == paper => Some(l.viewport),
            _ => None,
        })
        .unwrap();
    doc.block_records
        .get_mut("*Paper_Space")
        .unwrap()
        .entity_handles
        .reverse();
    let back = transfer(&doc, true);
    let Some(EntityType::Viewport(v)) = back.get_entity(overall) else {
        panic!("overall")
    };
    assert_eq!(v.id, 1);
    assert!(back.entities().any(|e|matches!(e,EntityType::Viewport(v) if v.common.owner_handle==paper && v.common.handle!=overall && v.id>=2)));
}

#[test]
fn model_grid_behavior_survives_files_but_paper_viewport_behavior_does_not() {
    let mut source = source();
    let flags = opencadcodec::entities::GridFlags {
        beyond_limits: true,
        adaptive: true,
        subdivision: true,
        follow_dynamic: true,
    };
    for v in source.vports.iter_mut() {
        v.grid_flags = flags;
    }
    for v in source.entities_mut() {
        if let EntityType::Viewport(v) = v {
            v.grid_flags = flags;
        }
    }
    for dwg in [false, true] {
        let actual = transfer(&source, dwg);
        assert!(
            actual.vports.iter().all(|v| v.grid_flags == flags),
            "dwg={dwg}"
        );
        assert!(
            actual
                .entities()
                .filter_map(|e| match e {
                    EntityType::Viewport(v) => Some(v),
                    _ => None,
                })
                .all(|v| v.grid_flags == opencadcodec::entities::GridFlags::default()),
            "dwg={dwg}"
        );
    }
}
