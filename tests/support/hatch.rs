use ocdraw::geometry_kernel::hatch::{HatchBoundary2 as B, HatchEdge2 as E};
pub fn contours() -> Vec<B> {
    vec![
        B::Circle {
            center: [0., 0.],
            radius: 1.,
        },
        B::Circle {
            center: [12., 0.],
            radius: 3.,
        },
        B::Circle {
            center: [0., 0.],
            radius: 3.,
        },
        B::Circle {
            center: [0., 0.],
            radius: 0.5,
        },
        B::Polyline {
            vertices: vec![[20., 0.], [22., 0.]],
            bulges: vec![1., 1.],
        },
        B::Edges(vec![
            E::CircularArc {
                center: [26., 0.],
                radius: 2.,
                start_parameter: std::f64::consts::PI,
                sweep_parameter: -std::f64::consts::PI,
            },
            E::CircularArc {
                center: [26., 0.],
                radius: 2.,
                start_parameter: 0.,
                sweep_parameter: -std::f64::consts::PI,
            },
        ]),
        B::Ellipse {
            center: [32., 0.],
            x_axis: [1., 0.],
            semi_major_radius: 3.,
            semi_minor_radius: 1.5,
        },
        B::Edges(vec![
            E::EllipticArc {
                center: [40., 0.],
                x_axis: [1., 0.],
                semi_major_radius: 3.,
                semi_minor_radius: 1.5,
                start_parameter: 0.,
                sweep_parameter: -std::f64::consts::PI,
            },
            E::Line {
                start: [37., 0.],
                end: [38., 0.],
            },
            E::CircularArc {
                center: [40., 0.],
                radius: 2.,
                start_parameter: -std::f64::consts::PI,
                sweep_parameter: std::f64::consts::PI,
            },
            E::Line {
                start: [42., 0.],
                end: [43., 0.],
            },
        ]),
    ]
}
pub fn tolerant_loop() -> B {
    B::Edges(vec![
        E::Line {
            start: [0., 0.],
            end: [2., 0.],
        },
        E::Line {
            start: [2. + 5e-10, 0.],
            end: [2., 2.],
        },
        E::Line {
            start: [2., 2.],
            end: [0., 2.],
        },
        E::Line {
            start: [0., 2.],
            end: [0., 0.],
        },
    ])
}
pub fn cad_nonuniform_block() -> opencadcodec::CadDocument {
    use opencadcodec::{BlockRecord, CadDocument, EntityType, Vector3};
    let mut d = CadDocument::new();
    let mut record = BlockRecord::new("HatchBlock");
    record.handle = d.allocate_handle();
    let owner = record.handle;
    d.block_records.add(record).unwrap();
    let mut h = cad_geometry_convert::hatch::prepare_hatch_to_cad(
        Default::default(),
        &contours(),
        ocdraw::geometry_kernel::hatch::HatchAreaRule::Normal,
        1e-9,
    )
    .unwrap()
    .hatch;
    h.common.owner_handle = owner;
    d.add_entity(EntityType::Hatch(h)).unwrap();
    let mut record = BlockRecord::new("NestedHatch");
    record.handle = d.allocate_handle();
    let nested = record.handle;
    d.block_records.add(record).unwrap();
    let mut insert = opencadcodec::entities::Insert::new("HatchBlock", Vector3::new(2., 3., 0.));
    insert.common.owner_handle = nested;
    insert.set_x_scale(-2.);
    insert.set_y_scale(3.);
    insert.rotation = 0.3;
    d.add_entity(EntityType::Insert(insert)).unwrap();
    let mut insert = opencadcodec::entities::Insert::new("NestedHatch", Vector3::new(10., -4., 2.));
    insert.set_x_scale(4.);
    insert.set_y_scale(2.);
    insert.rotation = -0.2;
    d.add_entity(EntityType::Insert(insert)).unwrap();
    d
}
