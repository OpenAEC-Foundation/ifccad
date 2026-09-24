use ifccad::ifcdr::{AppearanceId, IfcdrEntityRef, IfcdrLengthUnit, Point2, Point3};
use ifccad::package::{
    load_directory_package, AppearanceColor, AppearanceDefinition, AppearanceMode,
    AppearanceProperty, ArcDefinition, CircleDefinition, DrawingLayoutKind, DrawingOptions,
    EllipseArcDefinition, EllipseDefinition, EntityAppearance, LayerDefinition, LineDefinition,
    LinePatternDefinition, LinePatternRef, PackageBuilder, PackageOptions,
    PlanarPolylineDefinition, PointDefinition, PointDisplay, PointGlyph, PointSize,
    SpatialPolylineDefinition,
};
use ifccad::{PackageId, ResourceId};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

#[test]
fn spatial_polyline_roundtrips_with_xyz_and_closure() {
    let root = TempRoot::new();
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("spatial-polyline-package").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-24T10:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("spatial-polyline-drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap();
    let appearance = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Black".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.0,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance,
        })
        .unwrap();
    let points = vec![
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(4.0, 5.0, -6.0),
        Point3::new(7.0, 8.0, 9.0),
    ];
    drawing
        .model_space()
        .add_spatial_polyline(SpatialPolylineDefinition {
            points: points.clone(),
            closed: true,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    let target = root.0.join("spatial-polyline-package");
    package.finish().unwrap().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    let entity = resource.entities(layout.scope().id()).next().unwrap();
    let IfcdrEntityRef::SpatialPolyline(polyline) = entity else {
        panic!("spatial polyline")
    };
    assert_eq!(polyline.points(), points);
    assert!(polyline.closed());
    let bounds = layout.scope().bounds().unwrap();
    assert_eq!(bounds.min(), Point3::new(1.0, 2.0, -6.0));
    assert_eq!(bounds.max(), Point3::new(7.0, 8.0, 9.0));
}

#[test]
fn circular_family_retains_kinds_and_signed_sweep_through_strict_readback() {
    let root = TempRoot::new();
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("circle-arc-package").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-24T10:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("circle-arc-drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap();
    let appearance = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Black".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.0,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance,
        })
        .unwrap();
    let placement = ifccad::ifcdr::PlanePlacement::try_new(
        Point3::new(4.0, 5.0, 6.0),
        ifccad::ifcdr::Vector3::new(1.0, 0.0, 0.0),
        ifccad::ifcdr::Vector3::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    drawing
        .model_space()
        .add_circle(CircleDefinition {
            placement,
            radius: 3.0,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    drawing
        .model_space()
        .add_arc(ArcDefinition {
            placement,
            radius: 2.0,
            start_parameter: 0.25,
            sweep_parameter: -std::f64::consts::PI,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    let target = root.0.join("circle-arc-package");
    package.finish().unwrap().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package
        .drawing_sets()
        .next()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    let mut entities = resource.entities(layout.scope().id());
    assert!(
        matches!(entities.next().unwrap(), IfcdrEntityRef::Circle(circle) if circle.radius() == 3.0)
    );
    assert!(
        matches!(entities.next().unwrap(), IfcdrEntityRef::Arc(arc) if arc.radius() == 2.0 && arc.sweep_parameter() == -std::f64::consts::PI)
    );
}

#[test]
fn elliptic_family_retains_equal_radii_and_nearly_full_arc_identity() {
    let root = TempRoot::new();
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("ellipse-package").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-24T10:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("ellipse-drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap();
    let appearance = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Black".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.0,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance,
        })
        .unwrap();
    let placement = ifccad::ifcdr::PlanePlacement::default();
    drawing
        .model_space()
        .add_ellipse(EllipseDefinition {
            placement,
            semi_major_radius: 2.0,
            semi_minor_radius: 2.0,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    drawing
        .model_space()
        .add_ellipse_arc(EllipseArcDefinition {
            placement,
            semi_major_radius: 3.0,
            semi_minor_radius: 1.0,
            start_parameter: 0.0,
            sweep_parameter: std::f64::consts::TAU.next_down(),
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    let target = root.0.join("ellipse-package");
    package.finish().unwrap().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let scope = drawing.layouts().next().unwrap().scope().id();
    let resource = drawing.representation().resource();
    let mut entities = resource.entities(scope);
    assert!(
        matches!(entities.next().unwrap(), IfcdrEntityRef::Ellipse(ellipse) if ellipse.semi_minor_radius() == 2.0)
    );
    assert!(
        matches!(entities.next().unwrap(), IfcdrEntityRef::EllipseArc(arc) if arc.sweep_parameter() == std::f64::consts::TAU.next_down())
    );
}

#[test]
fn point_builder_writes_a_distinct_strict_readable_entity() {
    let root = TempRoot::new();
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("point-package").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-24T10:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("point-drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap();
    drawing
        .set_point_display(PointDisplay {
            glyph: PointGlyph::Plus,
            circle: true,
            square: false,
            size: PointSize::ViewportPercent(5.0),
        })
        .unwrap();
    let appearance = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Black".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.0,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance,
        })
        .unwrap();
    let placement = ifccad::ifcdr::PlanePlacement::try_new(
        Point3::new(4.0, 5.0, 6.0),
        ifccad::ifcdr::Vector3::new(1.0, 0.0, 0.0),
        ifccad::ifcdr::Vector3::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    drawing
        .model_space()
        .add_point(PointDefinition {
            placement,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    let target = root.0.join("point-package");
    package.finish().unwrap().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package
        .drawing_sets()
        .next()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    assert_eq!(
        drawing.point_display(),
        PointDisplay {
            glyph: PointGlyph::Plus,
            circle: true,
            square: false,
            size: PointSize::ViewportPercent(5.0),
        }
    );
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    let entity = resource.entities(layout.scope().id()).next().unwrap();
    assert!(
        matches!(entity, IfcdrEntityRef::Point(point) if point.position() == Point3::new(4.0, 5.0, 6.0))
    );
}

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ifccad-writer-roundtrip-{}-{nonce}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn representative_builder() -> PackageBuilder {
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("roundtrip-package").unwrap(),
        data_version: "42".to_owned(),
        author: "Writer integration test".to_owned(),
        timestamp: "2026-09-03T14:15:16.125+00:00".to_owned(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model layout".to_owned(),
            representation_resource_id: ResourceId::new("geometry-main").unwrap(),
            length_unit: IfcdrLengthUnit::Millimetre,
        })
        .unwrap();
    let solid = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Solid black".to_owned(),
            color: AppearanceColor::rgb(0, 0, 0).with_indexed("ACI", 7),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.25,
        })
        .unwrap();
    let dashed = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Dashed red".to_owned(),
            color: AppearanceColor::rgb(255, 0, 0).with_named("RAL", "Traffic red"),
            opacity: 0.5,
            line_pattern: LinePatternDefinition::named("dashed"),
            line_weight: 0.18,
        })
        .unwrap();
    let layer_0 = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".to_owned(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance: solid,
        })
        .unwrap();
    let walls = drawing
        .layers()
        .add(LayerDefinition {
            name: "A-WALL".to_owned(),
            visible: false,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance: dashed,
        })
        .unwrap();

    drawing
        .model_space()
        .add_line(LineDefinition {
            start: ifccad::ifcdr::Point3::new(0.0, 0.0, 0.0),
            end: ifccad::ifcdr::Point3::new(10.0, 5.0, 0.0),
            layer: layer_0,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    drawing
        .model_space()
        .add_planar_polyline(PlanarPolylineDefinition {
            placement: ifccad::ifcdr::PlanePlacement::default(),
            bulges: vec![0.0, 0.5],
            points: vec![Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)],
            closed: false,
            layer: walls,
            appearance: EntityAppearance::explicit(solid),
            visible: false,
        })
        .unwrap();
    drawing
        .model_space()
        .add_line(LineDefinition {
            start: ifccad::ifcdr::Point3::new(1.0, 2.0, 0.0),
            end: ifccad::ifcdr::Point3::new(3.0, 4.0, 0.0),
            layer: walls,
            appearance: EntityAppearance::by_block(),
            visible: false,
        })
        .unwrap();
    drawing
        .model_space()
        .add_planar_polyline(PlanarPolylineDefinition {
            placement: ifccad::ifcdr::PlanePlacement::default(),
            bulges: vec![0.0, 0.0, 0.25],
            points: vec![
                Point2::new(2.0, 2.0),
                Point2::new(8.0, 8.0),
                Point2::new(5.0, -1.0),
            ],
            closed: true,
            layer: layer_0,
            appearance: EntityAppearance::explicit(dashed),
            visible: true,
        })
        .unwrap();
    drawing
        .model_space()
        .add_line(LineDefinition {
            start: ifccad::ifcdr::Point3::new(0.0, 1.0, 0.0),
            end: ifccad::ifcdr::Point3::new(1.0, 2.0, 0.0),
            layer: layer_0,
            appearance: EntityAppearance {
                appearance: Some(dashed),
                color_mode: AppearanceMode::ByLayer,
                opacity_mode: AppearanceMode::Explicit,
                line_pattern_mode: AppearanceMode::ByBlock,
                line_weight_mode: AppearanceMode::Explicit,
            },
            visible: true,
        })
        .unwrap();
    package
}

#[test]
fn typed_layout_settings_roundtrip_with_independent_paper_values() {
    use ifccad::ifcdr::{ShadedPlot, ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode};
    use ifccad::package::*;
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("plot-layouts").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-23T00:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("geometry").unwrap(),
            length_unit: IfcdrLengthUnit::Metre,
        })
        .unwrap();
    drawing.set_plot_style_mode(PlotStyleMode::Named);
    let paper_a = drawing.add_paper_space("A".into()).unwrap();
    let paper_b = drawing.add_paper_space("B".into()).unwrap();
    let media_rect = PlotRect {
        min_x: 5.0,
        min_y: 5.0,
        max_x: 205.0,
        max_y: 292.0,
    };
    let settings = PlotSettings {
        media: PlotMedia {
            unit: PlotUnit::Millimetre,
            width: 210.0,
            height: 297.0,
            printable_area: media_rect,
            rotation: PlotRotation::None,
            device_name: None,
            media_name: None,
        },
        area: PlotArea::Layout,
        mapping: PlotMapping {
            scale: PlotScale::Fixed {
                output_length: 1.0,
                scope_length: 1.0,
            },
            placement: PlotPlacement::Offset {
                reference: PlotOffsetReference::Media,
                x: 0.0,
                y: 0.0,
            },
        },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode: ShadedPlotMode::AsDisplayed,
                quality: ShadedPlotQuality {
                    mode: ShadedPlotQualityMode::Normal,
                    dpi: None,
                },
            },
            apply_plot_styles: true,
            plot_style_table_name: Some("styles.stb".into()),
        },
        options: PlotOptions {
            plot_viewport_borders: false,
            plot_paper_space_last: true,
            hide_paper_space_objects: false,
            plot_line_weights: true,
            scale_line_weights: false,
            plot_transparency: false,
        },
    };
    let mut second_settings = settings.clone();
    second_settings.media.unit = PlotUnit::Inch;
    second_settings.media.width = 11.0;
    second_settings.media.height = 17.0;
    second_settings.media.printable_area = PlotRect {
        min_x: 0.25,
        min_y: 0.25,
        max_x: 10.75,
        max_y: 16.75,
    };
    second_settings.area = PlotArea::Extents;
    second_settings.mapping.scale = PlotScale::FitToArea;
    second_settings.mapping.placement = PlotPlacement::Centered;
    second_settings.output.apply_plot_styles = false;
    second_settings.output.plot_style_table_name = None;
    drawing
        .set_paper_layout_settings(
            paper_a,
            LayoutSettings {
                plot_settings: Some(settings),
                ..Default::default()
            },
        )
        .unwrap();
    drawing
        .set_paper_layout_settings(
            paper_b,
            LayoutSettings {
                limits_checking: true,
                plot_settings: Some(second_settings),
                ..Default::default()
            },
        )
        .unwrap();
    let artifact = package.finish().unwrap();
    if let Ok(destination) = std::env::var("IFCCAD_CONFORMANCE_OUTPUT") {
        artifact.write_directory(destination).unwrap();
    }
    let root = TempRoot::new();
    let target = root.0.join("package");
    artifact.write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(
        loaded.validated_package().is_some(),
        "{:?}",
        loaded.report()
    );
    let validated = loaded.validated_package().unwrap();
    let drawing = validated.drawings().next().unwrap();
    let layouts = drawing.layouts().collect::<Vec<_>>();
    assert_eq!(
        layouts[1].settings().plot_settings.unwrap().area,
        PlotArea::Layout
    );
    assert_eq!(
        layouts[2].settings().plot_settings.unwrap().area,
        PlotArea::Extents
    );
    let entry: serde_json::Value =
        serde_json::from_slice(artifact.file("package.ifcx.json").unwrap()).unwrap();
    assert_eq!(entry["data"][1]["attributes"]["plotStyleMode"], "named");
    assert_eq!(
        entry["data"][3]["attributes"]["plotSettings"]["area"]["mode"],
        "Layout"
    );
    assert_eq!(
        entry["data"][4]["attributes"]["plotSettings"]["area"]["mode"],
        "Extents"
    );
    assert_eq!(
        entry["data"][4]["attributes"]["plotSettings"]["media"]["unit"],
        "in"
    );
}

#[test]
fn writer_roundtrip_retains_model_limits_and_paper_window_plot_modes() {
    use ifccad::ifcdr::{ShadedPlot, ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode};
    use ifccad::package::*;

    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("plot-modes").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-23T00:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("geometry").unwrap(),
            length_unit: IfcdrLengthUnit::Metre,
        })
        .unwrap();
    let paper = drawing.add_paper_space("Window sheet".into()).unwrap();
    let mut plot = PlotSettings {
        media: PlotMedia {
            unit: PlotUnit::Millimetre,
            width: 210.0,
            height: 297.0,
            printable_area: PlotRect {
                min_x: 5.0,
                min_y: 5.0,
                max_x: 205.0,
                max_y: 292.0,
            },
            rotation: PlotRotation::None,
            device_name: None,
            media_name: None,
        },
        area: PlotArea::Limits,
        mapping: PlotMapping {
            scale: PlotScale::Fixed {
                output_length: 1.0,
                scope_length: 1.0,
            },
            placement: PlotPlacement::Offset {
                reference: PlotOffsetReference::Media,
                x: 0.0,
                y: 0.0,
            },
        },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode: ShadedPlotMode::AsDisplayed,
                quality: ShadedPlotQuality {
                    mode: ShadedPlotQualityMode::Normal,
                    dpi: None,
                },
            },
            apply_plot_styles: false,
            plot_style_table_name: None,
        },
        options: PlotOptions {
            plot_viewport_borders: false,
            plot_paper_space_last: true,
            hide_paper_space_objects: false,
            plot_line_weights: true,
            scale_line_weights: false,
            plot_transparency: false,
        },
    };
    drawing.set_model_layout_settings(LayoutSettings {
        limits: Some(PlotRect {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        }),
        plot_settings: Some(plot.clone()),
        ..Default::default()
    });
    plot.area = PlotArea::Window(PlotRect {
        min_x: 1.0,
        min_y: 2.0,
        max_x: 5.0,
        max_y: 8.0,
    });
    plot.mapping.scale = PlotScale::FitToArea;
    plot.mapping.placement = PlotPlacement::Centered;
    drawing
        .set_paper_layout_settings(
            paper,
            LayoutSettings {
                plot_settings: Some(plot),
                ..Default::default()
            },
        )
        .unwrap();

    let root = TempRoot::new();
    package
        .finish()
        .unwrap()
        .write_directory(root.0.join("package"))
        .unwrap();
    let outcome = load_directory_package(root.0.join("package")).unwrap();
    let package = outcome.validated_package().expect("strict writer readback");
    let drawing = package.drawings().next().unwrap();
    let layouts = drawing.layouts().collect::<Vec<_>>();
    assert_eq!(
        layouts[0].settings().plot_settings.unwrap().area,
        PlotArea::Limits
    );
    let paper_plot = layouts[1].settings().plot_settings.unwrap();
    assert_eq!(
        paper_plot.area,
        PlotArea::Window(PlotRect {
            min_x: 1.0,
            min_y: 2.0,
            max_x: 5.0,
            max_y: 8.0,
        })
    );
    assert_eq!(paper_plot.mapping.scale, PlotScale::FitToArea);
}

#[test]
fn writer_emits_paper_viewport_with_frozen_layer_override() {
    use ifccad::ifcdr::*;
    use ifccad::package::*;
    let mut package = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("viewport-package").unwrap(),
        data_version: "1".into(),
        author: "test".into(),
        timestamp: "2026-09-23T00:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = package
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("geometry").unwrap(),
            length_unit: IfcdrLengthUnit::Metre,
        })
        .unwrap();
    let style = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Style".into(),
            color: AppearanceColor::rgb(1, 2, 3),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.25,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance: style,
        })
        .unwrap();
    let notes_layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "Notes".into(),
            visible: true,
            frozen: false,
            locked: false,
            plottable: true,
            frozen_in_new_viewports: false,
            description: None,
            appearance: style,
        })
        .unwrap();
    let paper = drawing.add_paper_space("Sheet".into()).unwrap();
    drawing
        .set_paper_layout_settings(
            paper,
            LayoutSettings {
                plot_settings: Some(PlotSettings {
                    media: PlotMedia {
                        unit: PlotUnit::Millimetre,
                        width: 210.0,
                        height: 297.0,
                        printable_area: PlotRect {
                            min_x: 5.0,
                            min_y: 5.0,
                            max_x: 205.0,
                            max_y: 292.0,
                        },
                        rotation: PlotRotation::None,
                        device_name: None,
                        media_name: Some("A4".into()),
                    },
                    area: PlotArea::Layout,
                    mapping: PlotMapping {
                        scale: PlotScale::Fixed {
                            output_length: 1.0,
                            scope_length: 1.0,
                        },
                        placement: PlotPlacement::Offset {
                            reference: PlotOffsetReference::Media,
                            x: 0.0,
                            y: 0.0,
                        },
                    },
                    output: PlotOutput {
                        shaded_plot: ShadedPlot {
                            mode: ShadedPlotMode::AsDisplayed,
                            quality: ShadedPlotQuality {
                                mode: ShadedPlotQualityMode::Normal,
                                dpi: None,
                            },
                        },
                        apply_plot_styles: false,
                        plot_style_table_name: None,
                    },
                    options: PlotOptions {
                        plot_viewport_borders: true,
                        plot_paper_space_last: true,
                        hide_paper_space_objects: false,
                        plot_line_weights: true,
                        scale_line_weights: false,
                        plot_transparency: false,
                    },
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let definition = ViewportDefinition {
        frame: ViewportFrame {
            center: Point2::new(10.0, 20.0),
            width: 4.0,
            height: 2.0,
        },
        view: ViewDefinition {
            center: Point2::new(0.0, 0.0),
            target: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(0.0, 0.0, 1.0),
            height: 10.0,
            twist: 0.0,
            projection: ProjectionMode::Orthographic,
            lens_length: Some(35.0),
            front_clip: FrontClip {
                mode: FrontClipMode::Disabled,
                distance: Some(1.0),
            },
            back_clip: BackClip {
                mode: BackClipMode::Disabled,
                distance: Some(50.0),
            },
        },
        render_mode: ViewportRenderMode::Wireframe,
        view_enabled: true,
        view_locked: true,
        paper_clip: PaperClip {
            enabled: false,
            boundary_entity_id: None,
        },
        plot_shading_override: None,
        layer,
        appearance: EntityAppearance::by_layer(),
        visible: false,
        layer_overrides: vec![
            ViewportLayerOverrideDefinition {
                layer,
                frozen: true,
                appearance: Some(AppearancePatch {
                    color: Some(AppearanceColor::rgb(40, 50, 60)),
                    opacity: Some(0.5),
                    line_pattern: None,
                    line_weight: Some(0.35),
                }),
            },
            ViewportLayerOverrideDefinition {
                layer: notes_layer,
                frozen: false,
                appearance: Some(AppearancePatch {
                    color: None,
                    opacity: Some(0.75),
                    line_pattern: None,
                    line_weight: None,
                }),
            },
        ],
    };
    let id = drawing
        .paper_space(paper)
        .unwrap()
        .add_viewport(definition.clone())
        .unwrap();
    let boundary_id = drawing
        .paper_space(paper)
        .unwrap()
        .add_planar_polyline(PlanarPolylineDefinition {
            bulges: Vec::new(),

            points: vec![
                Point2::new(28.5, 19.2),
                Point2::new(31.5, 20.8),
                Point2::new(28.5, 20.8),
                Point2::new(31.5, 19.2),
            ],
            placement: PlanePlacement::default(),
            closed: true,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    let mut clipped = definition;
    clipped.frame = ViewportFrame {
        center: Point2::new(30.0, 20.0),
        width: 4.0,
        height: 2.0,
    };
    clipped.paper_clip = PaperClip {
        enabled: true,
        boundary_entity_id: Some(boundary_id.get()),
    };
    clipped.layer_overrides.clear();
    drawing
        .paper_space(paper)
        .unwrap()
        .add_viewport(clipped)
        .unwrap();
    let artifact = package.finish().unwrap();
    if let Ok(destination) = std::env::var("IFCCAD_CONFORMANCE_OUTPUT") {
        artifact.write_directory(destination).unwrap();
    }
    let root = TempRoot::new();
    let target = root.0.join("package");
    artifact.write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let paper = drawing.layouts().nth(1).unwrap();
    let representation = paper.representation();
    let resource = representation.resource();
    let entities = resource.entities(paper.scope().id()).collect::<Vec<_>>();
    assert_eq!(entities.len(), 3);
    let IfcdrEntityRef::Viewport(viewport) = &entities[0] else {
        panic!("viewport expected")
    };
    assert_eq!(viewport.entity_id(), id);
    assert_eq!(viewport.frame().width, 4.0);
    assert_eq!(viewport.layer_overrides().len(), 2);
    assert!(viewport.layer_overrides()[0].frozen);
    assert!(!viewport.visible());
    assert!(viewport.view_enabled());
    assert!(viewport.view_locked());
    assert_eq!(viewport.view().lens_length, Some(35.0));
    assert_eq!(viewport.view().front_clip.distance, Some(1.0));
    assert_eq!(viewport.view().back_clip.distance, Some(50.0));
    let patch = resource
        .appearance_override(
            viewport.layer_overrides()[0]
                .appearance_override_id
                .unwrap(),
        )
        .unwrap();
    assert_eq!(patch.color().unwrap().rgb_components(), [40, 50, 60]);
    assert_eq!(patch.opacity(), Some(0.5));
    assert_eq!(patch.line_weight(), Some(0.35));
    let second_patch = resource
        .appearance_override(
            viewport.layer_overrides()[1]
                .appearance_override_id
                .unwrap(),
        )
        .unwrap();
    assert_eq!(second_patch.color(), None);
    assert_eq!(second_patch.opacity(), Some(0.75));
    assert!(matches!(&entities[1], IfcdrEntityRef::PlanarPolyline(polyline) if polyline.closed()));
    assert!(
        matches!(&entities[2], IfcdrEntityRef::Viewport(clipped) if clipped.paper_clip().enabled && clipped.paper_clip().boundary_entity_id == Some(boundary_id.get()))
    );
}

#[test]
fn writer_output_reloads_without_diagnostics_and_preserves_semantics() {
    let root = TempRoot::new();
    let target = root.0.join("project");
    representative_builder()
        .finish()
        .unwrap()
        .write_directory(&target)
        .unwrap();

    let bytes = std::fs::read(target.join("resources/drawing.ifcdr.json")).unwrap();
    let resource: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(resource["header"]["version"], "0.11.0");
    assert!(resource.get("namedUcsBindings").is_none());
    assert!(resource.get("dimensionOverrideTable").is_none());
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().expect("strict writer output");
    assert_eq!(package.header().package_id().as_str(), "roundtrip-package");
    assert_eq!(package.header().data_version(), "42");
    assert_eq!(package.header().author(), "Writer integration test");
    assert_eq!(package.header().timestamp(), "2026-09-03T14:15:16.125Z");

    let drawing_set = package.drawing_sets().next().expect("drawing set");
    assert_eq!(package.drawing_sets().count(), 1);
    assert_eq!(drawing_set.path(), "drawing-set-0");
    let drawing = drawing_set.drawings().next().expect("drawing");
    assert_eq!(drawing.path(), "drawing-0");
    let layout = drawing.layouts().next().expect("layout");
    assert_eq!(drawing.layouts().count(), 1);
    assert_eq!(layout.path(), "layout-0");
    assert_eq!(layout.name(), "Model layout");
    assert_eq!(layout.kind(), DrawingLayoutKind::Model);
    assert_eq!(layout.scope().id().get(), 0);
    assert!(matches!(
        layout.scope(),
        ifccad::ifcdr::ScopeRef::ModelSpace(_)
    ));

    let representation = drawing.representation();
    assert_eq!(representation.path(), "representation-0");
    assert_eq!(representation.role(), "drawing");
    assert_eq!(representation.resource_id().as_str(), "geometry-main");
    assert_eq!(
        representation.external_uri(),
        Some("resources/drawing.ifcdr.json")
    );
    assert_eq!(
        representation.resource().unit(),
        IfcdrLengthUnit::Millimetre
    );
    let bounds = layout.scope().bounds().unwrap();
    assert_eq!(bounds.min(), Point3::new(-2.0, -5.0, 0.0));
    assert_eq!(bounds.max(), Point3::new(10.0, 8.0, 0.0));

    let layers = representation.layers().collect::<Vec<_>>();
    assert_eq!(layers.len(), 2);
    assert_eq!(
        (layers[0].id().get(), layers[0].name(), layers[0].visible()),
        (0, "0", true)
    );
    assert_eq!(
        (layers[1].id().get(), layers[1].name(), layers[1].visible()),
        (1, "A-WALL", false)
    );
    assert_eq!(layers[0].appearance().unwrap().name(), "Solid black");
    assert_eq!(layers[1].appearance().unwrap().name(), "Dashed red");

    let by_layer = representation.appearance(AppearanceId::from(0)).unwrap();
    assert!(matches!(by_layer.color(), AppearanceProperty::ByLayer));
    assert!(matches!(by_layer.opacity(), AppearanceProperty::ByLayer));
    let by_block = representation.appearance(AppearanceId::from(1)).unwrap();
    assert!(matches!(by_block.color(), AppearanceProperty::ByBlock));
    let solid = representation.appearance(AppearanceId::from(2)).unwrap();
    assert_eq!(solid.ifcx_definition().unwrap().name(), "Solid black");
    assert!(matches!(solid.opacity(), AppearanceProperty::Explicit(1.0)));
    assert!(matches!(
        solid.line_pattern(),
        AppearanceProperty::Explicit(LinePatternRef::Name("continuous"))
    ));
    let dashed = representation.appearance(AppearanceId::from(3)).unwrap();
    assert!(matches!(
        dashed.opacity(),
        AppearanceProperty::Explicit(0.5)
    ));
    let mixed = representation.appearance(AppearanceId::from(4)).unwrap();
    assert!(matches!(mixed.color(), AppearanceProperty::ByLayer));
    assert!(matches!(mixed.opacity(), AppearanceProperty::Explicit(0.5)));
    assert!(matches!(mixed.line_pattern(), AppearanceProperty::ByBlock));
    assert!(matches!(
        mixed.line_weight(),
        AppearanceProperty::Explicit(0.18)
    ));

    let resource = representation.resource();
    let entities = resource.entities(layout.scope().id()).collect::<Vec<_>>();
    assert_eq!(entities.len(), 5);
    let IfcdrEntityRef::Line(first) = &entities[0] else {
        panic!("entity 1 must be a line");
    };
    assert_eq!(first.entity_id().get(), 1);
    assert_eq!(first.start(), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(first.end(), Point3::new(10.0, 5.0, 0.0));
    assert_eq!(first.appearance_id().get(), 0);
    assert!(first.visible());
    let IfcdrEntityRef::PlanarPolyline(second) = &entities[1] else {
        panic!("entity 2 must be a polyline");
    };
    assert_eq!(second.entity_id().get(), 2);
    assert_eq!(
        second.local_points().collect::<Vec<_>>(),
        [Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)]
    );
    assert!(!second.closed());
    assert_eq!(second.bulges().collect::<Vec<_>>(), [0.0, 0.5]);
    assert!(!second.visible());
    assert_eq!(second.appearance_id().get(), 2);
    let IfcdrEntityRef::Line(third) = &entities[2] else {
        panic!("entity 3 must be a line");
    };
    assert_eq!(third.entity_id().get(), 3);
    assert_eq!(third.appearance_id().get(), 1);
    assert!(!third.visible());
    let IfcdrEntityRef::PlanarPolyline(fourth) = &entities[3] else {
        panic!("entity 4 must be a polyline");
    };
    assert_eq!(fourth.entity_id().get(), 4);
    assert!(fourth.closed());
    assert_eq!(fourth.bulges().collect::<Vec<_>>(), [0.0, 0.0, 0.25]);
    assert!(fourth.visible());
    assert_eq!(fourth.appearance_id().get(), 3);
    let IfcdrEntityRef::Line(fifth) = &entities[4] else {
        panic!("entity 5 must be a line");
    };
    assert_eq!(fifth.entity_id().get(), 5);
    assert_eq!(fifth.appearance_id().get(), 4);
}

#[test]
fn single_entity_families_reload_without_requiring_the_other_stream() {
    for polyline in [false, true] {
        let root = TempRoot::new();
        let target = root.0.join("single-family");
        let mut package = PackageBuilder::new(PackageOptions {
            package_id: PackageId::new("single-family").unwrap(),
            data_version: "1".into(),
            author: "Test".into(),
            timestamp: "2026-09-08T00:00:00Z".into(),
        })
        .unwrap();
        let mut drawing = package
            .add_drawing(DrawingOptions {
                model_layout_name: "Model".into(),
                representation_resource_id: ResourceId::new("drawing").unwrap(),
                length_unit: IfcdrLengthUnit::Millimetre,
            })
            .unwrap();
        let appearance = drawing
            .appearances()
            .add(AppearanceDefinition {
                name: "Default".into(),
                color: AppearanceColor::rgb(0, 0, 0),
                opacity: 1.0,
                line_pattern: LinePatternDefinition::named("continuous"),
                line_weight: 0.25,
            })
            .unwrap();
        let layer = drawing
            .layers()
            .add(LayerDefinition {
                name: "0".into(),
                visible: true,
                frozen: false,
                locked: false,
                plottable: true,
                frozen_in_new_viewports: false,
                description: None,
                appearance,
            })
            .unwrap();
        let points = vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)];
        if polyline {
            drawing
                .model_space()
                .add_planar_polyline(PlanarPolylineDefinition {
                    placement: ifccad::ifcdr::PlanePlacement::default(),
                    bulges: Vec::new(),
                    points: points.clone(),
                    closed: true,
                    layer,
                    appearance: EntityAppearance::by_layer(),
                    visible: true,
                })
                .unwrap();
        } else {
            drawing
                .model_space()
                .add_line(LineDefinition {
                    start: Point3::new(points[0].x(), points[0].y(), 0.0),
                    end: Point3::new(points[1].x(), points[1].y(), 0.0),
                    layer,
                    appearance: EntityAppearance::by_layer(),
                    visible: true,
                })
                .unwrap();
        }
        package.finish().unwrap().write_directory(&target).unwrap();
        let loaded = load_directory_package(&target).unwrap();
        assert!(loaded.report().is_empty(), "{:?}", loaded.report());
        let drawing = loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap();
        let layout = drawing.layouts().next().unwrap();
        let resource = layout.representation().resource();
        let mut entities = resource.entities(layout.scope().id());
        match entities.next().unwrap() {
            IfcdrEntityRef::Point(_) => panic!("line or polyline fixture"),
            IfcdrEntityRef::Circle(_) | IfcdrEntityRef::Arc(_) => {
                panic!("line or polyline fixture")
            }
            IfcdrEntityRef::Ellipse(_) | IfcdrEntityRef::EllipseArc(_) => {
                panic!("line or polyline fixture")
            }
            IfcdrEntityRef::BlockInstance(_) => panic!("primitive-only roundtrip fixture"),
            IfcdrEntityRef::Viewport(_) => panic!("primitive-only roundtrip fixture"),
            IfcdrEntityRef::SpatialPolyline(_) => panic!("primitive-only roundtrip fixture"),
            IfcdrEntityRef::Line(line) => {
                assert!(!polyline);
                assert_eq!(
                    vec![line.start(), line.end()],
                    points
                        .iter()
                        .map(|p| Point3::new(p.x(), p.y(), 0.0))
                        .collect::<Vec<_>>()
                );
            }
            IfcdrEntityRef::PlanarPolyline(line) => {
                assert!(polyline && line.closed());
                assert_eq!(line.local_points().collect::<Vec<_>>(), points);
            }
        }
        assert!(entities.next().is_none());
    }
}

#[test]
fn identical_input_is_byte_deterministic_across_builder_tokens() {
    let first = representative_builder().finish().unwrap();
    let second = representative_builder().finish().unwrap();

    let first_files = first
        .files()
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect::<Vec<_>>();
    let second_files = second
        .files()
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect::<Vec<_>>();
    assert_eq!(first_files, second_files);
    assert_eq!(first_files.len(), 2);
}

#[test]
fn empty_model_space_reloads_without_bounds_or_entities() {
    let root = TempRoot::new();
    let target = root.0.join("empty-project");
    let mut builder = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("empty-package").unwrap(),
        data_version: "1".to_owned(),
        author: "Writer integration test".to_owned(),
        timestamp: "2026-09-03T14:15:16Z".to_owned(),
    })
    .unwrap();
    builder
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".to_owned(),
            representation_resource_id: ResourceId::new("empty-geometry").unwrap(),
            length_unit: IfcdrLengthUnit::Unitless,
        })
        .unwrap();
    let artifact = builder.finish().unwrap();
    if let Ok(destination) = std::env::var("IFCCAD_CONFORMANCE_OUTPUT") {
        artifact.write_directory(destination).unwrap();
    }
    artifact.write_directory(&target).unwrap();

    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().expect("strict empty package");
    let drawing = package.drawings().next().unwrap();
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    assert!(resource.scopes().next().unwrap().bounds().is_none());
    assert_eq!(resource.entities(layout.scope().id()).count(), 0);
}
