#![allow(dead_code)]
use opencadcodec::{
    entities::{Block, BlockEnd, Dimension, DimensionAligned, Insert},
    *,
};
pub fn block(d: &mut CadDocument, name: &str, parts: Vec<EntityType>) -> Handle {
    let mut b = BlockRecord::new(name);
    b.handle = d.allocate_handle();
    b.block_entity_handle = d.allocate_handle();
    b.block_end_handle = d.allocate_handle();
    b.flags.anonymous = name.starts_with('*');
    let h = b.handle;
    let mut start = Block::new(name, Vector3::ZERO);
    start.common.handle = b.block_entity_handle;
    start.common.owner_handle = h;
    let mut end = BlockEnd::new();
    end.common.handle = b.block_end_handle;
    end.common.owner_handle = h;
    d.block_records.add(b).unwrap();
    d.add_entity(EntityType::Block(start)).unwrap();
    d.add_entity(EntityType::BlockEnd(end)).unwrap();
    for mut e in parts {
        e.common_mut().owner_handle = h;
        d.add_entity(e).unwrap();
    }
    h
}
pub fn dimension(d: &mut CadDocument, name: &str) -> Handle {
    let mut v = DimensionAligned::new(Vector3::new(100., 0., 0.), Vector3::new(140., 0., 0.));
    v.definition_point = Vector3::new(100., 10., 0.);
    v.base.definition_point = v.definition_point;
    v.base.block_name = name.into();
    d.add_entity(EntityType::Dimension(Box::new(Dimension::Aligned(v))))
        .unwrap()
}
pub fn picture(wide: bool) -> CadDocument {
    let mut d = CadDocument::new();
    let mut layer = Layer::new("Defpoints");
    layer.handle = d.allocate_handle();
    layer.is_plottable = false;
    d.layers.add(layer).unwrap();
    let line = Line::from_points(Vector3::new(100., 10., 0.), Vector3::new(140., 10., 0.));
    let mut point = Point::from_coords(100., 0., 0.);
    point.common.layer = "Defpoints".into();
    let solid = entities::Solid::new(
        Vector3::new(100., 10., 0.),
        Vector3::new(103., 11., 0.),
        Vector3::new(103., 9., 0.),
        Vector3::new(103., 9., 0.),
    );
    let mut parts = vec![
        EntityType::Line(line),
        EntityType::Point(point),
        EntityType::Solid(solid),
    ];
    if wide {
        let mut path = LwPolyline::new();
        path.add_vertex(entities::LwVertex::new(Vector2::new(0., 0.)));
        path.add_vertex(entities::LwVertex::new(Vector2::new(1., 1.)));
        path.constant_width = 0.15;
        parts.push(EntityType::LwPolyline(path));
    }
    block(&mut d, "*DimensionPicture", parts);
    dimension(&mut d, "*DimensionPicture");
    d
}
pub fn ordinary_insert(d: &mut CadDocument, name: &str) -> Handle {
    d.add_entity(EntityType::Insert(Insert::new(name, Vector3::ZERO)))
        .unwrap()
}
