use ocdraw::ocdraw::*;
use ocdraw_convert::opencadcodec::{
    self as cad, entities::*, CadDocument, EntityType, Handle, Vector2,
};

pub const KINDS: [&str; 4] = ["circle", "ellipse", "polyline", "legacy"];
pub fn source(kind: &str, forward: bool, bulge: f64) -> (CadDocument, Handle, Handle) {
    let mut d = CadDocument::new();
    d.header.insertion_units = 4;
    let layout = d
        .objects
        .values()
        .find_map(|o| {
            if let cad::objects::ObjectType::Layout(l) = o {
                (l.name == "Layout1").then_some(l.block_record)
            } else {
                None
            }
        })
        .unwrap();
    d.header.show_model_space = false;
    d.header.paper_space_block_handle = layout;
    let mut canvas = Viewport::new();
    canvas.id = 1;
    canvas.view_height = 35.;
    d.add_entity_to_layout(EntityType::Viewport(canvas), "Layout1")
        .unwrap();
    let mut boundary = match kind {
        "circle" => {
            let mut c = Circle::new();
            c.radius = 2.;
            EntityType::Circle(c)
        }
        "ellipse" => EntityType::Ellipse(Ellipse::from_center_axes(
            cad::Vector3::ZERO,
            cad::Vector3::new(3., 4., 0.),
            0.5,
        )),
        "legacy" => {
            let mut p = Polyline2D::new();
            p.flags.set_closed(true);
            p.vertices = vec![
                Vertex2D::new(cad::Vector3::new(-1., 0., 0.)).with_bulge(bulge),
                Vertex2D::new(cad::Vector3::new(1., 0., 0.)).with_bulge(bulge),
            ];
            EntityType::Polyline2D(p)
        }
        _ => {
            let mut p = LwPolyline::new();
            p.is_closed = true;
            p.vertices = vec![
                LwVertex::with_bulge(Vector2::new(-1., 0.), bulge),
                LwVertex::with_bulge(Vector2::new(1., 0.), bulge),
            ];
            EntityType::LwPolyline(p)
        }
    };
    boundary.common_mut().invisible = true;
    let mut v = Viewport::new();
    v.id = 2;
    v.width = 12.;
    v.height = 12.;
    v.view_height = 20.;
    v.status.locked = true;
    v.status = ViewportStatusFlags::from_bits(v.status.to_bits() | 0x10000);
    d.add_entity_to_layout(
        EntityType::Line(Line::from_coords(-5., 0., 0., -4., 0., 0.)),
        "Layout1",
    )
    .unwrap();
    let (b, h) = if forward {
        let h = d
            .add_entity_to_layout(EntityType::Viewport(v), "Layout1")
            .unwrap();
        let b = d.add_entity_to_layout(boundary, "Layout1").unwrap();
        (b, h)
    } else {
        let b = d.add_entity_to_layout(boundary, "Layout1").unwrap();
        let h = d
            .add_entity_to_layout(EntityType::Viewport(v), "Layout1")
            .unwrap();
        (b, h)
    };
    let EntityType::Viewport(v) = d.get_entity_mut(h).unwrap() else {
        unreachable!()
    };
    v.clip_boundary_handle = b;
    d.add_entity_to_layout(
        EntityType::Line(Line::from_coords(4., 0., 0., 5., 0., 0.)),
        "Layout1",
    )
    .unwrap();
    (d, b, h)
}
pub fn native(kind: &str, forward: bool) -> OcdrawDocument {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../../../../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    let geometry = match kind {
        "circle" => DrawingGeometry::Circle {
            placement: CoordinateFrame3::default(),
            radius: 2.,
        },
        "ellipse" => DrawingGeometry::Ellipse {
            placement: CoordinateFrame3::default(),
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            arc: None,
        },
        _ => DrawingGeometry::PlanarPolyline {
            placement: CoordinateFrame3::default(),
            vertices: vec![[-1., 0., 1.], [1., 0., 1.]],
            closed: true,
            line_pattern_generation: LinePatternGeneration::PerSegment,
        },
    };
    d.geometric_entities.push(DrawingGeometricEntity {
        id: 2,
        layer_id: 0,
        visible: false,
        appearance: EntityAppearance::default(),
        geometry,
    });
    d.scopes[1].entities = if forward { vec![1, 2] } else { vec![2, 1] };
    d.scopes[1].bounds = Some(Bounds3d::new(
        Point3::new(-10., -10., 0.),
        Point3::new(10., 10., 0.),
    ));
    d.next_entity_id = 3;
    d.viewports[0].paper_clip = DrawingPaperClip {
        enabled: true,
        boundary_entity_id: Some(2),
    };
    d.paper_canvases.push(DrawingPaperCanvas {
        scope_id: 1,
        view: d.viewports[0].view,
        grid: DrawingGrid {
            enabled: false,
            spacing: Point2::new(1., 1.),
            style: DrawingGridStyle::Lines,
            major_line_frequency: 1,
            beyond_limits: false,
            adaptive: false,
            subdivision: false,
            follows_workplane: false,
        },
        snap: DrawingSnap {
            enabled: false,
            base: Point2::new(0., 0.),
            spacing: Point2::new(1., 1.),
            angle: 0.,
            style: DrawingSnapStyle::Rectangular,
            isometric_plane: DrawingIsometricPlane::Left,
        },
        stored_ucs: DrawingUcsSelection::World,
        current_ucs: DrawingUcsSelection::World,
        active_context: DrawingPaperContext::Canvas,
    });
    d
}
