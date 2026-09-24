use super::mapping::registry_0_11;
use crate::ifcdr::logical::*;
use crate::ifcdr::{BlockScaling, Bounds3d, IfcdrLengthUnit, PlanePlacement, Scale3};
use crate::ResourceId;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub(crate) struct EncodedIfcdrResource {
    pub resource_id: ResourceId,
    pub bytes: Vec<u8>,
    pub value: Value,
    pub checksum: String,
}
#[derive(Debug, thiserror::Error)]
pub(crate) enum IfcdrEncodeError {
    #[error("{kind} count exceeds JSON packing range")]
    RangeExhausted { kind: &'static str },
    #[error("IFCDR serialization failed: {message}")]
    Serialization { message: String },
}
fn count(value: usize) -> Result<u32, IfcdrEncodeError> {
    u32::try_from(value).map_err(|_| IfcdrEncodeError::RangeExhausted {
        kind: "stream or pool",
    })
}
fn push(columns: &mut Map<String, Value>, name: &str, value: impl Into<Value>) {
    columns
        .entry(name)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap()
        .push(value.into());
}
fn common(columns: &mut Map<String, Value>, e: IfcdrEntityRow) {
    for (n, v) in [
        ("entityId", e.entity_id),
        ("scopeId", u64::from(e.scope_id)),
        ("layerId", u64::from(e.layer_id)),
        ("appearanceId", u64::from(e.appearance_id)),
    ] {
        push(columns, n, v);
    }
    push(columns, "visible", e.visible);
}
pub(crate) fn encode_json<R: IfcdrResourceAccess>(
    proof: &ValidatedIfcdr<R>,
) -> Result<EncodedIfcdrResource, IfcdrEncodeError> {
    let r = proof.loaded().resource();
    let registry = registry_0_11();
    let mut streams = Map::new();
    let mut directory = Vec::new();
    for schema in registry.streams() {
        let mut columns: Map<String, Value> = schema
            .columns
            .iter()
            .map(|c| (c.name.clone(), json!([])))
            .collect();
        let row_count;
        match schema.name() {
            "point" => {
                row_count = count(r.points().len())?;
                for point in r.points() {
                    common(&mut columns, point.entity);
                    push(&mut columns, "placement", placement_json(point.placement));
                }
            }
            "circle" => {
                row_count = count(r.circles().len())?;
                for circle in r.circles() {
                    common(&mut columns, circle.entity);
                    push(&mut columns, "placement", placement_json(circle.placement));
                    push(&mut columns, "radius", circle.radius);
                }
            }
            "arc" => {
                row_count = count(r.arcs().len())?;
                for arc in r.arcs() {
                    common(&mut columns, arc.entity);
                    push(&mut columns, "placement", placement_json(arc.placement));
                    push(&mut columns, "radius", arc.radius);
                    push(&mut columns, "startParameter", arc.start_parameter);
                    push(&mut columns, "sweepParameter", arc.sweep_parameter);
                }
            }
            "ellipse" => {
                row_count = count(r.ellipses().len())?;
                for ellipse in r.ellipses() {
                    common(&mut columns, ellipse.entity);
                    push(&mut columns, "placement", placement_json(ellipse.placement));
                    push(&mut columns, "semiMajorRadius", ellipse.semi_major_radius);
                    push(&mut columns, "semiMinorRadius", ellipse.semi_minor_radius);
                }
            }
            "ellipseArc" => {
                row_count = count(r.ellipse_arcs().len())?;
                for arc in r.ellipse_arcs() {
                    common(&mut columns, arc.entity);
                    push(&mut columns, "placement", placement_json(arc.placement));
                    push(&mut columns, "semiMajorRadius", arc.semi_major_radius);
                    push(&mut columns, "semiMinorRadius", arc.semi_minor_radius);
                    push(&mut columns, "startParameter", arc.start_parameter);
                    push(&mut columns, "sweepParameter", arc.sweep_parameter);
                }
            }
            "viewport" => {
                row_count = count(r.viewports().len())?;
                let mut offset = 0usize;
                for viewport in r.viewports() {
                    common(&mut columns, viewport.entity);
                    push(&mut columns, "viewScopeId", viewport.view_scope_id);
                    push(&mut columns, "frame", viewport_frame_json(viewport.frame));
                    push(&mut columns, "view", view_json(viewport.view));
                    push(
                        &mut columns,
                        "renderMode",
                        render_mode_code(viewport.render_mode),
                    );
                    push(&mut columns, "viewEnabled", viewport.view_enabled);
                    push(&mut columns, "viewLocked", viewport.view_locked);
                    let mut clip = json!({"enabled":viewport.paper_clip.enabled});
                    if let Some(id) = viewport.paper_clip.boundary_entity_id {
                        clip["boundaryEntityId"] = json!(id);
                    }
                    push(&mut columns, "paperClip", clip);
                    push(
                        &mut columns,
                        "plotShadingOverride",
                        viewport.plot_shading_override.map(shaded_plot_json),
                    );
                    push(&mut columns, "layerOverrideOffset", count(offset)?);
                    push(
                        &mut columns,
                        "layerOverrideCount",
                        count(viewport.layer_overrides.len())?,
                    );
                    offset = offset.checked_add(viewport.layer_overrides.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "viewport override entries",
                        },
                    )?;
                    count(offset)?;
                }
            }
            "viewportLayerOverride" => {
                let mut length = 0usize;
                for viewport in r.viewports() {
                    length = length.checked_add(viewport.layer_overrides.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "viewport override entries",
                        },
                    )?;
                    count(length)?;
                    for entry in &viewport.layer_overrides {
                        push(&mut columns, "layerId", entry.layer_id);
                        push(&mut columns, "frozen", entry.frozen);
                        push(
                            &mut columns,
                            "appearanceOverrideId",
                            entry.appearance_override_id,
                        );
                    }
                }
                row_count = count(length)?;
            }
            "blockInstance" => {
                let instances = r.block_instances();
                row_count = count(instances.len())?;
                for i in 0..instances.len() {
                    let instance = instances.get(i).expect("validated block instance row");
                    common(&mut columns, instance.entity);
                    push(
                        &mut columns,
                        "definitionScopeId",
                        instance.definition_scope_id,
                    );
                    let t = instance.transform;
                    let mut transform = Map::new();
                    if t.placement != PlanePlacement::default().components() {
                        let p = t.placement;
                        transform.insert("placement".into(),json!({"origin":{"x":p.origin.x(),"y":p.origin.y(),"z":p.origin.z()},"X":{"x":p.x.x(),"y":p.x.y(),"z":p.x.z()},"Y":{"x":p.y.x(),"y":p.y.y(),"z":p.y.z()}}));
                    }
                    if t.rotation != 0. {
                        transform.insert("rotation".into(), json!(t.rotation));
                    }
                    if t.scale != Scale3::default() {
                        transform.insert(
                            "scale".into(),
                            json!({"x":t.scale.x(),"y":t.scale.y(),"z":t.scale.z()}),
                        );
                    }
                    push(&mut columns, "transform", Value::Object(transform));
                }
            }
            "line" => {
                let lines = r.lines();
                row_count = count(lines.len())?;
                for i in 0..lines.len() {
                    let l = lines.get(i).expect("validated line row");
                    common(&mut columns, l.entity);
                    for (n, v) in [
                        ("x1", l.start.x()),
                        ("y1", l.start.y()),
                        ("z1", l.start.z()),
                        ("x2", l.end.x()),
                        ("y2", l.end.y()),
                        ("z2", l.end.z()),
                    ] {
                        push(&mut columns, n, v);
                    }
                }
            }
            "planarPolyline" => {
                let polylines = r.polylines();
                row_count = count(polylines.len())?;
                let mut offset = 0usize;
                for i in 0..polylines.len() {
                    let p = polylines.get(i).expect("validated polyline row");
                    common(&mut columns, p.entity());
                    push(&mut columns, "vertexOffset", count(offset)?);
                    push(&mut columns, "vertexCount", count(p.vertex_count())?);
                    push(&mut columns, "closed", p.closed());
                    let frame = p.placement();
                    let placement = if frame == PlanePlacement::default().components() {
                        Value::Null
                    } else if frame.x == PlanePlacement::default().components().x
                        && frame.y == PlanePlacement::default().components().y
                    {
                        json!({"origin":{"x":frame.origin.x(),"y":frame.origin.y(),"z":frame.origin.z()}})
                    } else {
                        json!({"origin":{"x":frame.origin.x(),"y":frame.origin.y(),"z":frame.origin.z()},
                        "X":{"x":frame.x.x(),"y":frame.x.y(),"z":frame.x.z()},
                        "Y":{"x":frame.y.x(),"y":frame.y.y(),"z":frame.y.z()}})
                    };
                    push(&mut columns, "placement", placement);
                    offset = offset.checked_add(p.vertex_count()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "vertex pool",
                        },
                    )?;
                    count(offset)?;
                    for j in 0..p.vertex_count() {
                        let point = p.vertex(j).expect("validated vertex");
                        push(&mut columns, "x", point.x());
                        push(&mut columns, "y", point.y());
                        push(
                            &mut columns,
                            "bulge",
                            p.bulge(j).expect("validated vertex bulge"),
                        );
                    }
                }
            }
            "spatialPolyline" => {
                row_count = count(r.spatial_polylines().len())?;
                let mut offset = 0usize;
                for polyline in r.spatial_polylines() {
                    common(&mut columns, polyline.entity);
                    push(&mut columns, "vertexOffset", count(offset)?);
                    push(&mut columns, "vertexCount", count(polyline.points.len())?);
                    push(&mut columns, "closed", polyline.closed);
                    offset = offset.checked_add(polyline.points.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "vertex pool",
                        },
                    )?;
                    count(offset)?;
                    for point in &polyline.points {
                        push(&mut columns, "x", point.x());
                        push(&mut columns, "y", point.y());
                        push(&mut columns, "z", point.z());
                    }
                }
            }
            "entityOrder" => {
                row_count = count(r.orders().len())?;
                let mut offset = 0usize;
                for order in r.orders() {
                    push(&mut columns, "scopeId", order.scope_id);
                    push(&mut columns, "entryOffset", count(offset)?);
                    push(&mut columns, "entryCount", count(order.entities.len())?);
                    offset = offset.checked_add(order.entities.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "order entries",
                        },
                    )?;
                    count(offset)?;
                }
            }
            "entityOrderEntry" => {
                let mut length = 0usize;
                for order in r.orders() {
                    length = length.checked_add(order.entities.len()).ok_or(
                        IfcdrEncodeError::RangeExhausted {
                            kind: "order entries",
                        },
                    )?;
                    count(length)?;
                    for id in &order.entities {
                        push(&mut columns, "entityId", *id);
                    }
                }
                row_count = count(length)?;
            }
            _ => unreachable!("registered base-profile stream"),
        }
        if columns
            .get("visible")
            .is_some_and(|v| v.as_array().unwrap().iter().all(|v| v == true))
        {
            columns.remove("visible");
        }
        for key in ["z1", "z2", "placement", "bulge"] {
            if columns.get(key).is_some_and(|v| {
                v.as_array().unwrap().iter().all(|v| {
                    if key == "placement" {
                        v.is_null()
                    } else {
                        v.as_f64() == Some(0.0)
                    }
                })
            }) {
                columns.remove(key);
            }
        }
        let names: Vec<_> = schema
            .columns
            .iter()
            .filter(|c| columns.contains_key(&c.name))
            .map(|c| c.name.as_str())
            .collect();
        let mut entry = json!({"name":schema.name(),"schema":schema.schema_id(),"role":match schema.role() { super::mapping::StreamRole::Object=>"object",super::mapping::StreamRole::Order=>"order",super::mapping::StreamRole::Child=>"child" },"count":row_count,"columns":names});
        if let Some(parent) = &schema.parent {
            entry["parent"] = json!(parent);
        }
        if !schema.children.is_empty() {
            entry["children"] = json!(schema.children);
        }
        directory.push(entry);
        columns.insert("count".into(), json!(row_count));
        streams.insert(schema.payload_key().into(), Value::Object(columns));
    }
    let root = json!({
        "header":{"format":"openaec.ifcdr","version":registry.ifcdr_version(),"resourceId":r.resource_id(),"unit":unit_name(r.unit()),"nextEntityId":r.next_entity_id()},
        "scopeTable":r.scopes().iter().map(|s| json!({"id":s.id,"kind":match s.kind {IfcdrScopeKind::ModelSpace=>0,IfcdrScopeKind::PaperSpace=>1,IfcdrScopeKind::BlockDefinition=>2},"bounds":s.bounds.map(bounds_json)})).collect::<Vec<_>>(),
        "blockDefinitionTable":r.block_definitions().iter().map(|d| json!({"scopeId":d.scope_id,"name":d.name,"basePoint":{"x":d.base_point.x(),"y":d.base_point.y(),"z":d.base_point.z()},"description":d.description,"anonymous":d.anonymous,"insertionUnit":unit_name(d.insertion_unit),"explodable":d.explodable,"scaling":match d.scaling {BlockScaling::Any=>0,BlockScaling::Uniform=>1}})).collect::<Vec<_>>(),
        "layerBindings":r.layers().iter().map(|l| json!({"id":l.id,"ifcxLayer":l.ifcx_layer})).collect::<Vec<_>>(),
        "appearanceBindings":r.appearances().iter().map(|a| json!({"id":a.id,"ifcxAppearance":a.ifcx_appearance,"colorMode":a.modes[0],"opacityMode":a.modes[1],"linePatternMode":a.modes[2],"lineWeightMode":a.modes[3],"overrideId":a.override_id})).collect::<Vec<_>>(),
        "appearanceOverrides":r.overrides().iter().map(|o| json!({"id":o.id,"color":o.color.as_ref().map(color_json),"opacity":o.opacity,"lineWeight":o.line_weight,"ifcxLinePattern":o.ifcx_line_pattern})).collect::<Vec<_>>(),
        "streamDirectory":{"version":"ifccad.ifcdr.streamDirectory.v1","streams":directory},"streams":streams
    });
    let bytes = serde_json::to_vec_pretty(&root).map_err(|e| IfcdrEncodeError::Serialization {
        message: e.to_string(),
    })?;
    let checksum = format!("sha256:{:x}", Sha256::digest(&bytes));
    Ok(EncodedIfcdrResource {
        resource_id: r.resource_id().clone(),
        bytes,
        checksum,
        value: root,
    })
}
fn point2_json(point: crate::ifcdr::Point2) -> Value {
    json!({"x":point.x(),"y":point.y()})
}
fn point3_json(point: crate::ifcdr::Point3) -> Value {
    json!({"x":point.x(),"y":point.y(),"z":point.z()})
}
fn placement_json(frame: crate::ifcdr::geometry::PlanePlacementComponents) -> Value {
    let standard = PlanePlacement::default().components();
    if frame == standard {
        Value::Null
    } else if frame.x == standard.x && frame.y == standard.y {
        json!({"origin":point3_json(frame.origin)})
    } else {
        json!({"origin":point3_json(frame.origin),
               "X":vector3_json(frame.x),"Y":vector3_json(frame.y)})
    }
}
fn vector3_json(vector: crate::ifcdr::Vector3) -> Value {
    json!({"x":vector.x(),"y":vector.y(),"z":vector.z()})
}
fn viewport_frame_json(frame: ViewportFrame) -> Value {
    json!({"center":point2_json(frame.center),"width":frame.width,"height":frame.height})
}
fn view_json(view: ViewDefinition) -> Value {
    let mut front = json!({"mode":match view.front_clip.mode {FrontClipMode::Disabled=>0,FrontClipMode::AtCamera=>1,FrontClipMode::AtDistance=>2}});
    if let Some(distance) = view.front_clip.distance {
        front["distance"] = json!(distance);
    }
    let mut back = json!({"mode":match view.back_clip.mode {BackClipMode::Disabled=>0,BackClipMode::AtDistance=>1}});
    if let Some(distance) = view.back_clip.distance {
        back["distance"] = json!(distance);
    }
    let mut value = json!({
        "center":point2_json(view.center),"target":point3_json(view.target),"direction":vector3_json(view.direction),
        "height":view.height,"twist":view.twist,"projection":match view.projection {ProjectionMode::Orthographic=>0,ProjectionMode::Perspective=>1},
        "frontClip":front,"backClip":back
    });
    if let Some(length) = view.lens_length {
        value["lensLength"] = json!(length);
    }
    value
}
fn render_mode_code(mode: ViewportRenderMode) -> u32 {
    match mode {
        ViewportRenderMode::TwoDimensional => 0,
        ViewportRenderMode::Wireframe => 1,
        ViewportRenderMode::HiddenLine => 2,
        ViewportRenderMode::FlatShadedWithoutEdges => 3,
        ViewportRenderMode::FlatShadedWithEdges => 4,
        ViewportRenderMode::SmoothShadedWithoutEdges => 5,
        ViewportRenderMode::SmoothShadedWithEdges => 6,
    }
}
fn shaded_plot_json(shading: ShadedPlot) -> Value {
    let mut quality = json!({"mode":match shading.quality.mode {
        ShadedPlotQualityMode::Draft=>0,ShadedPlotQualityMode::Preview=>1,ShadedPlotQualityMode::Normal=>2,
        ShadedPlotQualityMode::Presentation=>3,ShadedPlotQualityMode::Maximum=>4,ShadedPlotQualityMode::Custom=>5,
    }});
    if let Some(dpi) = shading.quality.dpi {
        quality["dpi"] = json!(dpi);
    }
    json!({"mode":match shading.mode {ShadedPlotMode::AsDisplayed=>0,ShadedPlotMode::Wireframe=>1,ShadedPlotMode::Hidden=>2,ShadedPlotMode::Rendered=>3},"quality":quality})
}
pub(crate) fn color_json(c: &IfcdrColor) -> Value {
    let mut value = json!({"rgb":c.rgb});
    if let Some(i) = &c.indexed {
        value["indexedColor"] = json!({"system":i.system,"index":i.index});
    }
    if let Some(n) = &c.named {
        value["namedColor"] = json!({"catalog":n.catalog,"name":n.name});
    }
    value
}
pub(crate) fn unit_name(unit: IfcdrLengthUnit) -> &'static str {
    match unit {
        IfcdrLengthUnit::Unitless => "unitless",
        IfcdrLengthUnit::Millimetre => "mm",
        IfcdrLengthUnit::Centimetre => "cm",
        IfcdrLengthUnit::Metre => "m",
        IfcdrLengthUnit::Kilometre => "km",
        IfcdrLengthUnit::Inch => "in",
        IfcdrLengthUnit::Foot => "ft",
        IfcdrLengthUnit::Mile => "mi",
        IfcdrLengthUnit::Microinch => "microin",
        IfcdrLengthUnit::Mil => "mil",
        IfcdrLengthUnit::Yard => "yd",
        IfcdrLengthUnit::Angstrom => "angstrom",
        IfcdrLengthUnit::Nanometre => "nm",
        IfcdrLengthUnit::Micrometre => "um",
        IfcdrLengthUnit::Decimetre => "dm",
        IfcdrLengthUnit::Decametre => "dam",
        IfcdrLengthUnit::Hectometre => "hm",
        IfcdrLengthUnit::Gigametre => "Gm",
        IfcdrLengthUnit::AstronomicalUnit => "au",
        IfcdrLengthUnit::LightYear => "ly",
        IfcdrLengthUnit::Parsec => "pc",
        IfcdrLengthUnit::UsSurveyFoot => "usSurveyFoot",
        IfcdrLengthUnit::UsSurveyInch => "usSurveyInch",
        IfcdrLengthUnit::UsSurveyYard => "usSurveyYard",
        IfcdrLengthUnit::UsSurveyMile => "usSurveyMile",
    }
}

fn bounds_json(b: Bounds3d) -> Value {
    json!({"minX":b.min().x(),"minY":b.min().y(),"minZ":b.min().z(),"maxX":b.max().x(),"maxY":b.max().y(),"maxZ":b.max().z()})
}
