use super::recipe::{Appearance, Drawing, Geometry, Property};
use ocdraw::ocdraw::{
    AppearanceSelection, CoordinateFrame3, DrawingBuilder, DrawingColor, DrawingGeometry,
    DrawingOptions, EncodedDrawing, EntityAppearance, GeometricEntityDefinition, LayerDefinition,
    Point3, Vector3,
};
use ocdraw_convert::cadcodec::{
    CadDocument, Color, DxfVersion, EntityType, Layer, Line, LineWeight, LwPolyline, Transparency,
    Vector2,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn selection<T: Clone>(value: &Property<T>) -> AppearanceSelection<T> {
    match value {
        Property::ByLayer => AppearanceSelection::ByLayer,
        Property::ByBlock => AppearanceSelection::ByBlock,
        Property::Explicit(v) => AppearanceSelection::Explicit(v.clone()),
    }
}
fn native_appearance(value: &Appearance) -> EntityAppearance {
    EntityAppearance {
        color: match &value.color {
            Property::ByLayer => AppearanceSelection::ByLayer,
            Property::ByBlock => AppearanceSelection::ByBlock,
            Property::Explicit([r, g, b]) => {
                AppearanceSelection::Explicit(DrawingColor::rgb(*r, *g, *b))
            }
        },
        opacity: selection(&value.opacity),
        line_pattern: match &value.pattern {
            Property::Explicit(v) if v.eq_ignore_ascii_case("continuous") => {
                AppearanceSelection::Explicit("Continuous".into())
            }
            other => selection(other),
        },
        line_weight: selection(&value.weight),
    }
}
pub fn drawing(recipe: &Drawing) -> Result<EncodedDrawing> {
    let mut builder = DrawingBuilder::new(DrawingOptions::new("size-baseline", "mm"))?;
    let mut layers = std::collections::BTreeMap::new();
    for source in &recipe.layers {
        let Property::Explicit([r, g, b]) = source.appearance.color else {
            return Err("recipe layer color must be explicit".into());
        };
        let mut layer = LayerDefinition::new(&source.name, DrawingColor::rgb(r, g, b));
        layer.visible = source.visible;
        let Property::Explicit(weight) = source.appearance.weight else {
            return Err("recipe layer weight must be explicit".into());
        };
        layer.line_weight = weight;
        let Property::Explicit(opacity) = source.appearance.opacity else {
            return Err("recipe layer opacity must be explicit".into());
        };
        layer.opacity = opacity;
        layers.insert(source.name.clone(), builder.add_layer(layer)?);
    }
    for entity in &recipe.entities {
        let geometry = match &entity.geometry {
            Geometry::Line { start, end } => DrawingGeometry::Line {
                start: *start,
                end: *end,
            },
            Geometry::Polyline {
                points,
                closed,
                origin,
                x_axis,
                y_axis,
            } => DrawingGeometry::PlanarPolyline {
                placement: CoordinateFrame3::try_new(
                    Point3::new(origin[0], origin[1], origin[2]),
                    Vector3::new(x_axis[0], x_axis[1], x_axis[2]),
                    Vector3::new(y_axis[0], y_axis[1], y_axis[2]),
                )?,
                vertices: points.iter().map(|p| [p[0], p[1], 0.]).collect(),
                closed: *closed,
            },
        };
        let mut record = GeometricEntityDefinition::new(0, layers[&entity.layer], geometry);
        record.visible = entity.visible;
        record.appearance = native_appearance(&entity.appearance);
        builder.add_geometric_entity(record)?;
    }
    Ok(builder.finish()?)
}

pub fn fix_cad_metadata(document: &mut CadDocument) {
    document.version = DxfVersion::AC1032;
    document.header.create_date_julian = 0.;
    document.header.update_date_julian = 0.;
    document.header.total_editing_time = 0.;
    document.header.user_elapsed_time = 0.;
    // Table::add does not allocate handles. DWG references layers by handle,
    // unlike textual DXF. Allocate only missing technical IDs before writing.
    let missing: Vec<_> = document
        .layers
        .iter()
        .filter(|layer| layer.handle.is_null())
        .map(|layer| layer.name.clone())
        .collect();
    for name in missing {
        let handle = document.allocate_handle();
        document.layers.get_mut(&name).unwrap().handle = handle;
    }
}

fn color(property: &Property<[u8; 3]>) -> Color {
    match property {
        Property::ByLayer => Color::ByLayer,
        Property::ByBlock => Color::ByBlock,
        Property::Explicit([r, g, b]) => Color::from_rgb(*r, *g, *b),
    }
}
fn weight(property: &Property<f64>) -> LineWeight {
    match property {
        Property::ByLayer => LineWeight::ByLayer,
        Property::ByBlock => LineWeight::ByBlock,
        Property::Explicit(0.25) => LineWeight::W0_25,
        Property::Explicit(0.5) => LineWeight::W0_50,
        _ => unreachable!("frozen recipe weight"),
    }
}
fn pattern(property: &Property<String>) -> String {
    match property {
        Property::ByLayer => String::new(),
        Property::ByBlock => "ByBlock".into(),
        Property::Explicit(value) => value.clone(),
    }
}
fn opacity(property: &Property<f64>) -> Transparency {
    match property {
        Property::ByLayer => Transparency::BY_LAYER,
        Property::ByBlock => Transparency::BY_BLOCK,
        Property::Explicit(1.0) => Transparency::OPAQUE,
        _ => unreachable!("frozen recipe opacity"),
    }
}
fn apply(common: &mut ocdraw_convert::cadcodec::entities::EntityCommon, appearance: &Appearance) {
    common.color = color(&appearance.color);
    common.line_weight = weight(&appearance.weight);
    common.linetype = pattern(&appearance.pattern);
    common.transparency = opacity(&appearance.opacity);
}
pub fn cad(recipe: &Drawing) -> Result<CadDocument> {
    let mut document = CadDocument::new();
    fix_cad_metadata(&mut document);
    document.header.insertion_units = 4;
    document.header.measurement = 1;
    for layer in &recipe.layers {
        if layer.name != "0" {
            let mut target = Layer::new(&layer.name);
            target.handle = document.allocate_handle();
            document.layers.add(target)?;
        }
        let target = document
            .layers
            .get_mut(&layer.name)
            .ok_or("missing CAD layer")?;
        target.flags.off = !layer.visible;
        target.color = color(&layer.appearance.color);
        target.line_weight = weight(&layer.appearance.weight);
        target.line_type = "Continuous".into();
        target.transparency = opacity(&layer.appearance.opacity);
    }
    for entity in &recipe.entities {
        let mut target = match &entity.geometry {
            Geometry::Line { start, end } => EntityType::Line(Line::from_coords(
                start[0], start[1], start[2], end[0], end[1], end[2],
            )),
            Geometry::Polyline {
                points,
                closed,
                origin,
                x_axis,
                y_axis,
            } => {
                let mut polyline = LwPolyline::from_points(
                    points.iter().map(|p| Vector2::new(p[0], p[1])).collect(),
                );
                if *x_axis == [0., 1., 0.] && *y_axis == [0., 0., 1.] {
                    polyline.normal = ocdraw_convert::cadcodec::Vector3::UNIT_X;
                    polyline.elevation = origin[0];
                    for v in &mut polyline.vertices {
                        v.location.x += origin[1];
                        v.location.y += origin[2];
                    }
                } else if *x_axis == [1., 0., 0.] && *y_axis == [0., 1., 0.] {
                    polyline.elevation = origin[2];
                    for v in &mut polyline.vertices {
                        v.location.x += origin[0];
                        v.location.y += origin[1];
                    }
                } else {
                    return Err("recipe has unsupported reference frame".into());
                }
                polyline.is_closed = *closed;
                EntityType::LwPolyline(polyline)
            }
        };
        let common = target.common_mut();
        common.layer = entity.layer.clone();
        common.invisible = !entity.visible;
        apply(common, &entity.appearance);
        document.add_entity(target)?;
    }
    Ok(document)
}
