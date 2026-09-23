use super::physical::validate_physical;
use crate::diagnostic::{PackageDiagnostic, PackageDiagnosticSeverity};
use crate::ifcdr::geometry::{BlockTransformComponents, PlanePlacementComponents};
use crate::ifcdr::logical::*;
use crate::ifcdr::read::decoded::*;
use crate::ifcdr::{
    BlockScaling, Bounds3d, IfcdrLengthUnit, PlanePlacement, Point2, Point3, Scale3, Vector3,
};
use crate::ResourceId;
use serde_json::Value;

fn error(uri: &str, location: String, code: &str, message: &str) -> PackageDiagnostic {
    PackageDiagnostic {
        category: crate::diagnostic::PackageDiagnosticCategory::ContractViolation,
        code: code.into(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(uri.into()),
        location: Some(location),
        context: Default::default(),
        message: message.into(),
    }
}
pub(crate) fn decode_json(
    uri: &str,
    value: &Value,
) -> Result<DecodedIfcdrResource, Vec<PackageDiagnostic>> {
    let errors = validate_physical(uri, value);
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut errors = Vec::new();
    let id = ResourceId::new(
        value["header"]["resourceId"]
            .as_str()
            .expect("physical string"),
    );
    if id.is_err() {
        errors.push(error(
            uri,
            "/header/resourceId".into(),
            IFCCAD_IFCDR_STRUCTURE_INVALID,
            "resource identity must not be empty",
        ));
    }
    let unit = length_unit(value["header"]["unit"].as_str().expect("physical string"));
    if unit.is_none() {
        errors.push(error(
            uri,
            "/header/unit".into(),
            "IFCCAD_IFCDR_UNIT_UNSUPPORTED",
            "unsupported length unit",
        ));
    }
    let overrides = rows(value, "appearanceOverrides")
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let color = if v["color"].is_null() {
                None
            } else {
                let c = color(&v["color"]);
                if c.is_none() {
                    errors.push(error(
                        uri,
                        format!("/appearanceOverrides/{i}/color"),
                        IFCCAD_IFCDR_APPEARANCE_INVALID,
                        "color requires three RGB bytes and typed metadata",
                    ));
                }
                c
            };
            IfcdrAppearanceOverride {
                id: u32v(&v["id"]),
                color,
                opacity: v["opacity"].as_f64(),
                ifcx_line_pattern: v["ifcxLinePattern"].as_str().map(String::from),
                line_weight: v["lineWeight"].as_f64(),
            }
        })
        .collect();
    let block_definitions = rows(value, "blockDefinitionTable")
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let insertion_unit = v
                .get("insertionUnit")
                .map_or(Some(IfcdrLengthUnit::Unitless), |unit| {
                    length_unit(unit.as_str().expect("physical unit string"))
                });
            if insertion_unit.is_none() {
                errors.push(error(
                    uri,
                    format!("/blockDefinitionTable/{i}/insertionUnit"),
                    "IFCCAD_IFCDR_UNIT_UNSUPPORTED",
                    "unsupported insertion unit",
                ));
            }
            IfcdrBlockDefinition {
                scope_id: u32v(&v["scopeId"]),
                name: v["name"].as_str().unwrap().into(),
                base_point: v
                    .get("basePoint")
                    .map_or(Point3::new(0., 0., 0.), point_record),
                description: v["description"].as_str().unwrap_or("").into(),
                anonymous: v["anonymous"].as_bool().unwrap_or(false),
                insertion_unit: insertion_unit.unwrap_or(IfcdrLengthUnit::Unitless),
                explodable: v["explodable"].as_bool().unwrap_or(true),
                scaling: match v["scaling"].as_u64().unwrap_or(0) {
                    0 => BlockScaling::Any,
                    1 => BlockScaling::Uniform,
                    _ => unreachable!("physical scaling code"),
                },
            }
        })
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    let l = &value["streams"]["lineStream"];
    let instances = &value["streams"]["blockInstanceStream"];
    let instance_entities = entity(instances);
    let p = &value["streams"]["polylineStream"];
    let o = &value["streams"]["entityOrderStream"];
    let entries: Vec<u64> = values(
        &value["streams"]["entityOrderEntryStream"]["entityId"],
        |v| v.as_u64().unwrap(),
    );
    let orders = (0..o["count"].as_u64().unwrap_or(0) as usize)
        .map(|i| {
            let start = u32v(&o["entryOffset"][i]) as usize;
            let count = u32v(&o["entryCount"][i]) as usize;
            IfcdrScopeOrder {
                scope_id: u32v(&o["scopeId"][i]),
                entities: entries[start..start + count].to_vec(),
            }
        })
        .collect();
    let viewport_stream = &value["streams"]["viewportStream"];
    let override_stream = &value["streams"]["viewportLayerOverrideStream"];
    let viewport_entities = entity(viewport_stream);
    let viewport_overrides = (0..override_stream["count"].as_u64().unwrap_or(0) as usize)
        .map(|i| ViewportLayerOverride {
            layer_id: u32v(&override_stream["layerId"][i]),
            frozen: override_stream["frozen"][i].as_bool().unwrap(),
            appearance_override_id: override_stream["appearanceOverrideId"][i]
                .as_u64()
                .map(|id| id as u32),
        })
        .collect::<Vec<_>>();
    Ok(DecodedIfcdrResource {
        id: id.unwrap(),
        version: value["header"]["version"]
            .as_str()
            .expect("physical version")
            .into(),
        unit: unit.unwrap(),
        next: value["header"]["nextEntityId"].as_u64().unwrap(),
        block_definitions,
        block_instances: (0..instance_entities.ids.len())
            .map(|i| IfcdrBlockInstanceRow {
                entity: instance_entities.get(i).expect("physical entity row"),
                definition_scope_id: u32v(&instances["definitionScopeId"][i]),
                transform: transform_record(&instances["transform"][i]),
            })
            .collect(),
        viewports: (0..viewport_entities.ids.len())
            .map(|i| {
                let start = u32v(&viewport_stream["layerOverrideOffset"][i]) as usize;
                let count = u32v(&viewport_stream["layerOverrideCount"][i]) as usize;
                IfcdrViewportRow {
                    entity: viewport_entities.get(i).expect("physical viewport row"),
                    view_scope_id: u32v(&viewport_stream["viewScopeId"][i]),
                    frame: viewport_frame_record(&viewport_stream["frame"][i]),
                    view: view_record(&viewport_stream["view"][i]),
                    render_mode: render_mode(u32v(&viewport_stream["renderMode"][i])),
                    view_enabled: viewport_stream["viewEnabled"][i].as_bool().unwrap(),
                    view_locked: viewport_stream["viewLocked"][i].as_bool().unwrap(),
                    paper_clip: PaperClip {
                        enabled: viewport_stream["paperClip"][i]["enabled"]
                            .as_bool()
                            .unwrap(),
                        boundary_entity_id: viewport_stream["paperClip"][i]["boundaryEntityId"]
                            .as_u64(),
                    },
                    plot_shading_override: (!viewport_stream["plotShadingOverride"][i].is_null())
                        .then(|| shaded_plot_record(&viewport_stream["plotShadingOverride"][i])),
                    layer_overrides: viewport_overrides[start..start + count].to_vec(),
                }
            })
            .collect(),
        scopes: rows(value, "scopeTable")
            .iter()
            .map(|v| IfcdrScope {
                id: u32v(&v["id"]),
                kind: match u32v(&v["kind"]) {
                    0 => IfcdrScopeKind::ModelSpace,
                    1 => IfcdrScopeKind::PaperSpace,
                    2 => IfcdrScopeKind::BlockDefinition,
                    _ => unreachable!("physical scope kind"),
                },
                bounds: (!v["bounds"].is_null()).then(|| Bounds3d {
                    min: Point3::new(
                        num(&v["bounds"]["minX"]),
                        num(&v["bounds"]["minY"]),
                        num(&v["bounds"]["minZ"]),
                    ),
                    max: Point3::new(
                        num(&v["bounds"]["maxX"]),
                        num(&v["bounds"]["maxY"]),
                        num(&v["bounds"]["maxZ"]),
                    ),
                }),
            })
            .collect(),
        layers: rows(value, "layerBindings")
            .iter()
            .map(|v| IfcdrLayerBinding {
                id: u32v(&v["id"]),
                ifcx_layer: v["ifcxLayer"].as_str().unwrap().into(),
            })
            .collect(),
        appearances: rows(value, "appearanceBindings")
            .iter()
            .map(|v| IfcdrAppearanceBinding {
                id: u32v(&v["id"]),
                ifcx_appearance: v["ifcxAppearance"].as_str().map(String::from),
                modes: [
                    "colorMode",
                    "opacityMode",
                    "linePatternMode",
                    "lineWeightMode",
                ]
                .map(|key| u32v(&v[key])),
                override_id: v["overrideId"].as_u64().map(|v| v as u32),
            })
            .collect(),
        overrides,
        orders,
        lines: LineColumns {
            entity: entity(l),
            x1: values(&l["x1"], num),
            y1: values(&l["y1"], num),
            z1: values(&l["z1"], num),
            x2: values(&l["x2"], num),
            y2: values(&l["y2"], num),
            z2: values(&l["z2"], num),
        },
        polylines: PolylineColumns {
            entity: entity(p),
            offsets: values(&p["vertexOffset"], |v| u32v(v) as usize),
            counts: values(&p["vertexCount"], |v| u32v(v) as usize),
            closed: values(&p["closed"], |v| v.as_bool().unwrap()),
            placements: values(&p["placement"], |v| {
                (!v.is_null()).then(|| PlanePlacementComponents {
                    origin: Point3::new(
                        num(&v["origin"]["x"]),
                        num(&v["origin"]["y"]),
                        num(&v["origin"]["z"]),
                    ),
                    x: Vector3::new(num(&v["X"]["x"]), num(&v["X"]["y"]), num(&v["X"]["z"])),
                    y: Vector3::new(num(&v["Y"]["x"]), num(&v["Y"]["y"]), num(&v["Y"]["z"])),
                })
            }),
            x: values(&p["x"], num),
            y: values(&p["y"], num),
        },
    })
}
fn point2_record(v: &Value) -> Point2 {
    Point2::new(num(&v["x"]), num(&v["y"]))
}
fn vector3_record(v: &Value) -> Vector3 {
    Vector3::new(num(&v["x"]), num(&v["y"]), num(&v["z"]))
}
fn viewport_frame_record(v: &Value) -> ViewportFrame {
    ViewportFrame {
        center: point2_record(&v["center"]),
        width: num(&v["width"]),
        height: num(&v["height"]),
    }
}
fn view_record(v: &Value) -> ViewDefinition {
    ViewDefinition {
        center: point2_record(&v["center"]),
        target: point_record(&v["target"]),
        direction: vector3_record(&v["direction"]),
        height: num(&v["height"]),
        twist: num(&v["twist"]),
        projection: match u32v(&v["projection"]) {
            0 => ProjectionMode::Orthographic,
            1 => ProjectionMode::Perspective,
            _ => unreachable!("physical projection code"),
        },
        lens_length: v["lensLength"].as_f64(),
        front_clip: FrontClip {
            mode: match u32v(&v["frontClip"]["mode"]) {
                0 => FrontClipMode::Disabled,
                1 => FrontClipMode::AtCamera,
                2 => FrontClipMode::AtDistance,
                _ => unreachable!("physical front clip code"),
            },
            distance: v["frontClip"]["distance"].as_f64(),
        },
        back_clip: BackClip {
            mode: match u32v(&v["backClip"]["mode"]) {
                0 => BackClipMode::Disabled,
                1 => BackClipMode::AtDistance,
                _ => unreachable!("physical back clip code"),
            },
            distance: v["backClip"]["distance"].as_f64(),
        },
    }
}
fn render_mode(value: u32) -> ViewportRenderMode {
    match value {
        0 => ViewportRenderMode::TwoDimensional,
        1 => ViewportRenderMode::Wireframe,
        2 => ViewportRenderMode::HiddenLine,
        3 => ViewportRenderMode::FlatShadedWithoutEdges,
        4 => ViewportRenderMode::FlatShadedWithEdges,
        5 => ViewportRenderMode::SmoothShadedWithoutEdges,
        6 => ViewportRenderMode::SmoothShadedWithEdges,
        _ => unreachable!("physical render code"),
    }
}
fn shaded_plot_record(v: &Value) -> ShadedPlot {
    ShadedPlot {
        mode: match u32v(&v["mode"]) {
            0 => ShadedPlotMode::AsDisplayed,
            1 => ShadedPlotMode::Wireframe,
            2 => ShadedPlotMode::Hidden,
            3 => ShadedPlotMode::Rendered,
            _ => unreachable!("physical shaded plot code"),
        },
        quality: ShadedPlotQuality {
            mode: match u32v(&v["quality"]["mode"]) {
                0 => ShadedPlotQualityMode::Draft,
                1 => ShadedPlotQualityMode::Preview,
                2 => ShadedPlotQualityMode::Normal,
                3 => ShadedPlotQualityMode::Presentation,
                4 => ShadedPlotQualityMode::Maximum,
                5 => ShadedPlotQualityMode::Custom,
                _ => unreachable!("physical shaded plot quality code"),
            },
            dpi: v["quality"]["dpi"].as_u64().map(|dpi| dpi as u32),
        },
    }
}
fn point_record(v: &Value) -> Point3 {
    Point3::new(num(&v["x"]), num(&v["y"]), num(&v["z"]))
}
fn transform_record(v: &Value) -> BlockTransformComponents {
    BlockTransformComponents {
        placement: v.get("placement").map_or_else(
            || PlanePlacement::default().components(),
            |p| PlanePlacementComponents {
                origin: point_record(&p["origin"]),
                x: Vector3::new(num(&p["X"]["x"]), num(&p["X"]["y"]), num(&p["X"]["z"])),
                y: Vector3::new(num(&p["Y"]["x"]), num(&p["Y"]["y"]), num(&p["Y"]["z"])),
            },
        ),
        rotation: v.get("rotation").map_or(0., num),
        scale: v.get("scale").map_or_else(Scale3::default, |s| {
            Scale3::new(num(&s["x"]), num(&s["y"]), num(&s["z"]))
        }),
    }
}
fn rows<'a>(root: &'a Value, key: &str) -> &'a [Value] {
    root[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn values<T>(value: &Value, convert: impl Fn(&Value) -> T) -> Vec<T> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(convert)
        .collect()
}
fn num(value: &Value) -> f64 {
    value.as_f64().expect("physical number")
}
fn u32v(value: &Value) -> u32 {
    u32::try_from(value.as_u64().expect("physical unsigned integer")).expect("physical u32")
}
fn entity(value: &Value) -> EntityColumns {
    EntityColumns {
        ids: values(&value["entityId"], |v| v.as_u64().unwrap()),
        scopes: values(&value["scopeId"], u32v),
        layers: values(&value["layerId"], u32v),
        appearances: values(&value["appearanceId"], u32v),
        visible: value
            .get("visible")
            .map(|v| values(v, |v| v.as_bool().unwrap()))
            .unwrap_or_else(|| vec![true; value["count"].as_u64().unwrap_or(0) as usize]),
    }
}
pub(crate) fn color(value: &Value) -> Option<IfcdrColor> {
    let rgb = value["rgb"].as_array()?;
    if rgb.len() != 3 {
        return None;
    }
    let indexed = if let Some(v) = value.get("indexedColor") {
        Some(IfcdrIndexedColor {
            system: v["system"].as_str()?.into(),
            index: v["index"].as_u64()?,
        })
    } else {
        None
    };
    let named = if let Some(v) = value.get("namedColor") {
        Some(IfcdrNamedColor {
            catalog: v["catalog"].as_str()?.into(),
            name: v["name"].as_str()?.into(),
        })
    } else {
        None
    };
    Some(IfcdrColor {
        rgb: [
            rgb_channel(rgb[0].as_u64()?)?,
            rgb_channel(rgb[1].as_u64()?)?,
            rgb_channel(rgb[2].as_u64()?)?,
        ],
        indexed,
        named,
    })
}
