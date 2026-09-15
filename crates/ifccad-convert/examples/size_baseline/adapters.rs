use super::recipe::{Appearance, Drawing, Geometry, Property};
use ifccad::ifcdr::{IfcdrLengthUnit, Point2};
use ifccad::package::{
    AppearanceColor, AppearanceDefinition, DrawingOptions, DrawingResourceStorage, EncodedPackage,
    EntityAppearance, LayerDefinition, LineDefinition, LinePatternDefinition, PackageBuilder,
    PackageOptions, PolylineDefinition,
};
use ifccad::{PackageId, ResourceId};
use ifccad_convert::cadcodec::{
    CadDocument, Color, DxfVersion, EntityType, Layer, Line, LineWeight, LwPolyline, Transparency,
    Vector2,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("size-baseline").unwrap(),
        data_version: "1".into(),
        author: "IFCCAD size baseline".into(),
        timestamp: "2026-09-11T00:00:00Z".into(),
    }
}

fn definition(alternate: bool) -> AppearanceDefinition {
    let color = if alternate {
        [180, 40, 60]
    } else {
        [10, 20, 30]
    };
    AppearanceDefinition {
        name: if alternate { "Explicit" } else { "Layer" }.into(),
        color: AppearanceColor::rgb(color[0], color[1], color[2]),
        opacity: 1.0,
        line_pattern: LinePatternDefinition::named("continuous"),
        line_weight: if alternate { 0.5 } else { 0.25 },
    }
}

pub fn package(recipe: &Drawing, inline: bool) -> Result<EncodedPackage> {
    let mut package = PackageBuilder::new(options())?;
    let mut drawing = package.add_drawing(DrawingOptions {
        model_layout_name: "Model".into(),
        representation_resource_id: ResourceId::new("drawing")?,
        length_unit: IfcdrLengthUnit::Millimetre,
    })?;
    drawing.set_resource_storage(if inline {
        DrawingResourceStorage::Inline
    } else {
        DrawingResourceStorage::External
    });
    let appearance = drawing.appearances().add(definition(false))?;
    let explicit = if recipe
        .entities
        .iter()
        .any(|e| matches!(e.appearance.color, Property::Explicit(_)))
    {
        Some(drawing.appearances().add(definition(true))?)
    } else {
        None
    };
    let mut layers = std::collections::BTreeMap::new();
    for layer in &recipe.layers {
        layers.insert(
            layer.name.clone(),
            drawing.layers().add(LayerDefinition {
                name: layer.name.clone(),
                visible: layer.visible,
                appearance,
            })?,
        );
    }
    for entity in &recipe.entities {
        let layer = layers[&entity.layer];
        let appearance = if matches!(entity.appearance.color, Property::ByLayer) {
            EntityAppearance::by_layer()
        } else {
            EntityAppearance::explicit(explicit.ok_or("missing recipe appearance")?)
        };
        let visible = entity.visible;
        match &entity.geometry {
            Geometry::Line { start, end } => {
                drawing.model_space().add_line(LineDefinition {
                    start: ifccad::ifcdr::Point3::new(start[0], start[1], start[2]),
                    end: ifccad::ifcdr::Point3::new(end[0], end[1], end[2]),
                    layer,
                    appearance,
                    visible,
                })?;
            }
            Geometry::Polyline {
                points,
                closed,
                origin,
                x_axis,
                y_axis,
            } => {
                drawing.model_space().add_polyline(PolylineDefinition {
                    placement: ifccad::ifcdr::PlanePlacement::try_new(
                        ifccad::ifcdr::Point3::new(origin[0], origin[1], origin[2]),
                        ifccad::ifcdr::Vector3::new(x_axis[0], x_axis[1], x_axis[2]),
                        ifccad::ifcdr::Vector3::new(y_axis[0], y_axis[1], y_axis[2]),
                    )?,
                    points: points.iter().map(|p| Point2::new(p[0], p[1])).collect(),
                    closed: *closed,
                    layer,
                    appearance,
                    visible,
                })?;
            }
        }
    }
    Ok(package.finish()?)
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
fn apply(common: &mut ifccad_convert::cadcodec::entities::EntityCommon, appearance: &Appearance) {
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
                    polyline.normal = ifccad_convert::cadcodec::Vector3::UNIT_X;
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
