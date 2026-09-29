//! Direct standalone drawing to CadDocument conversion.

use crate::{ConversionLossPolicy, ImportOptions};
use cadcodec::objects::ObjectType;
use cadcodec::{
    BlockRecord, CadDocument, Color, EntityType, Handle, Layer, Line, LineWeight, Transparency,
};
use ocdraw::drawing::ValidatedDrawing;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectImportDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DirectImportError {
    #[error("drawing conversion loss was rejected")]
    LossRejected {
        diagnostics: Vec<DirectImportDiagnostic>,
    },
    #[error("CAD construction failed: {0}")]
    Cad(String),
}

pub struct DirectImportOutcome {
    document: CadDocument,
    diagnostics: Vec<DirectImportDiagnostic>,
    entity_mapping: BTreeMap<u64, Handle>,
}

impl DirectImportOutcome {
    pub fn document(&self) -> &CadDocument {
        &self.document
    }
    pub fn diagnostics(&self) -> &[DirectImportDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<u64, Handle> {
        &self.entity_mapping
    }
    pub fn into_document(self) -> CadDocument {
        self.document
    }
}

fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> DirectImportDiagnostic {
    DirectImportDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn color(
    value: &Value,
    location: &str,
    diagnostics: &mut Vec<DirectImportDiagnostic>,
) -> (Color, Option<String>, Option<String>) {
    let rgb = value["rgb"].as_array().expect("validated color");
    let rgb = [
        rgb[0].as_u64().expect("validated RGB") as u8,
        rgb[1].as_u64().expect("validated RGB") as u8,
        rgb[2].as_u64().expect("validated RGB") as u8,
    ];
    let indexed = &value["indexedColor"];
    let mapped = if indexed.is_object() {
        let system = indexed["system"].as_str().expect("validated index system");
        let index = indexed["index"].as_u64().expect("validated index");
        if system.eq_ignore_ascii_case("ACI") && (1..=255).contains(&index) {
            Color::Index(index as u8)
        } else {
            diagnostics.push(diagnostic(
                "COLOR_INDEX",
                location,
                "indexed color metadata cannot be represented in CAD",
            ));
            Color::from_rgb(rgb[0], rgb[1], rgb[2])
        }
    } else {
        Color::from_rgb(rgb[0], rgb[1], rgb[2])
    };
    let named = &value["namedColor"];
    let (catalog, name) = if named.is_object() {
        (
            named["catalog"].as_str().map(str::to_owned),
            named["name"].as_str().map(str::to_owned),
        )
    } else {
        (None, None)
    };
    (mapped, catalog, name)
}

fn line_weight(
    value: f64,
    location: &str,
    diagnostics: &mut Vec<DirectImportDiagnostic>,
) -> LineWeight {
    let hundredths = (value * 100.0).round();
    let bounded = hundredths.clamp(0.0, i16::MAX as f64) as i16;
    if f64::from(bounded) / 100.0 != value {
        diagnostics.push(diagnostic(
            "LINE_WEIGHT_ROUNDED",
            location,
            "line weight was rounded to CAD hundredths of a millimetre",
        ));
    }
    LineWeight::from_value(bounded)
}

fn column<'a>(stream: &'a Value, name: &str, row: usize) -> &'a Value {
    &stream[name][row]
}

fn mode<'a>(stream: &'a Value, property: &str, row: usize) -> &'a str {
    let key = format!("{property}Mode");
    stream[&key][row].as_str().unwrap_or("ByLayer")
}

fn apply_common(
    stream: &Value,
    row: usize,
    kind: &str,
    common: &mut cadcodec::entities::EntityCommon,
    layer_name: &str,
    diagnostics: &mut Vec<DirectImportDiagnostic>,
) {
    common.layer = layer_name.into();
    common.invisible = !column(stream, "visible", row).as_bool().unwrap_or(true);
    common.color = match mode(stream, "color", row) {
        "ByLayer" => Color::ByLayer,
        "ByBlock" => Color::ByBlock,
        _ => {
            let (mapped, catalog, name) = color(
                column(stream, "color", row),
                &format!("/streams/{kind}Stream/color/{row}"),
                diagnostics,
            );
            if let (Some(catalog), Some(name)) = (catalog, name) {
                common.color_name = Some(format!("{catalog}${name}"));
            }
            mapped
        }
    };
    common.transparency = match mode(stream, "opacity", row) {
        "ByLayer" => Transparency::ByLayer,
        "ByBlock" => Transparency::ByBlock,
        _ => Transparency::from_percent(
            1.0 - column(stream, "opacity", row)
                .as_f64()
                .expect("validated explicit opacity"),
        ),
    };
    common.linetype = match mode(stream, "linePattern", row) {
        "ByLayer" => String::new(),
        "ByBlock" => "ByBlock".into(),
        _ => column(stream, "linePattern", row)
            .as_str()
            .expect("validated explicit line pattern")
            .into(),
    };
    common.line_weight = match mode(stream, "lineWeight", row) {
        "ByLayer" => LineWeight::ByLayer,
        "ByBlock" => LineWeight::ByBlock,
        _ => line_weight(
            column(stream, "lineWeight", row)
                .as_f64()
                .expect("validated explicit weight"),
            &format!("/streams/{kind}Stream/lineWeight/{row}"),
            diagnostics,
        ),
    };
}

fn planar_angle(placement: &Value) -> Option<f64> {
    if placement.is_null() {
        return Some(0.0);
    }
    let x = &placement["X"];
    let y = &placement["Y"];
    if x["z"].as_f64()? != 0.0 || y["z"].as_f64()? != 0.0 {
        return None;
    }
    let x0 = x["x"].as_f64()?;
    let x1 = x["y"].as_f64()?;
    let y0 = y["x"].as_f64()?;
    let y1 = y["y"].as_f64()?;
    if (x0 * y1 - x1 * y0 - 1.0).abs() > 1e-12 {
        return None;
    }
    Some(x1.atan2(x0))
}

fn simple_block_transform(transform: &Value) -> bool {
    if transform["rotation"]
        .as_f64()
        .is_some_and(|angle| angle != 0.0)
    {
        return false;
    }
    if let Some(scale) = transform.get("scale") {
        if ["x", "y", "z"]
            .into_iter()
            .any(|axis| scale[axis].as_f64() != Some(1.0))
        {
            return false;
        }
    }
    let placement = &transform["placement"];
    placement.is_null()
        || (["x", "y", "z"].map(|axis| placement["X"][axis].as_f64())
            == [Some(1.0), Some(0.0), Some(0.0)]
            && ["x", "y", "z"].map(|axis| placement["Y"][axis].as_f64())
                == [Some(0.0), Some(1.0), Some(0.0)])
}

pub fn ocdraw_to_cad_document(
    drawing: &ValidatedDrawing,
    options: ImportOptions,
) -> Result<DirectImportOutcome, DirectImportError> {
    let value = drawing.as_value();
    let mut document = CadDocument::new();
    let vector = |row: &Value| {
        cadcodec::Vector3::new(
            row["x"].as_f64().expect("validated coordinate"),
            row["y"].as_f64().expect("validated coordinate"),
            row["z"].as_f64().expect("validated coordinate"),
        )
    };
    for source in value["ucsDefinitions"].as_array().into_iter().flatten() {
        let mut target = cadcodec::Ucs::new(source["name"].as_str().expect("validated UCS name"));
        target.handle = document.allocate_handle();
        target.origin = vector(&source["frame"]["origin"]);
        target.x_axis = vector(&source["frame"]["X"]);
        target.y_axis = vector(&source["frame"]["Y"]);
        target.elevation = source["elevation"].as_f64().expect("validated elevation");
        document
            .ucss
            .add(target)
            .map_err(|error| DirectImportError::Cad(format!("UCS definition: {error}")))?;
    }
    let mut diagnostics = Vec::new();
    let units = crate::direct::UNIT_TOKENS;
    if let Some(unit) = value["header"]["unit"].as_str() {
        if let Some(code) = units.iter().position(|token| *token == unit) {
            document.header.insertion_units = code as i16;
        }
    }
    document.header.plotstyle_mode = drawing.plot_style_mode() == "colorDependent";
    let mut layer_names = BTreeMap::new();
    for (index, source) in drawing.layers().iter().enumerate() {
        let id = source["id"].as_u64().expect("validated layer ID");
        let name = source["name"].as_str().expect("validated layer name");
        let mut target = Layer::new(name);
        target.flags.off = !source["visible"].as_bool().expect("validated visible");
        target.flags.frozen = source["frozen"].as_bool().expect("validated frozen");
        target.flags.locked = source["locked"].as_bool().expect("validated locked");
        target.flags.frozen_in_new_viewport = source["frozenInNewViewports"]
            .as_bool()
            .expect("validated viewport flag");
        target.is_plottable = source["plottable"].as_bool().expect("validated plottable");
        target.description = source["description"].as_str().unwrap_or("").into();
        let (mapped_color, catalog, color_name) = color(
            &source["color"],
            &format!("/layers/{index}/color"),
            &mut diagnostics,
        );
        target.color = mapped_color;
        target.book_name = catalog;
        target.color_name = color_name;
        target.transparency = Transparency::from_percent(
            1.0 - source["opacity"].as_f64().expect("validated opacity"),
        );
        target.line_type = source["linePattern"]
            .as_str()
            .expect("validated line pattern")
            .into();
        target.line_weight = line_weight(
            source["lineWeight"]
                .as_f64()
                .expect("validated line weight"),
            &format!("/layers/{index}/lineWeight"),
            &mut diagnostics,
        );
        if name == "0" {
            *document.layers.get_mut("0").ok_or_else(|| {
                DirectImportError::Cad("fresh CAD document has no Layer 0".into())
            })? = target;
        } else {
            document
                .layers
                .add(target)
                .map_err(|error| DirectImportError::Cad(format!("Layer {name}: {error}")))?;
        }
        layer_names.insert(id, name.to_owned());
    }
    if layer_names.is_empty() {
        diagnostics.push(diagnostic(
            "CAD_SCAFFOLD_LAYER",
            "/layers",
            "CAD runtime creates its default Layer 0",
        ));
    }

    let mut scope_layouts = BTreeMap::new();
    let mut saved_linetype_scaling = None;
    for (index, layout) in drawing.layouts().iter().enumerate() {
        let scope_id = layout["scopeId"].as_u64().expect("validated scope ID");
        let name = layout["name"].as_str().expect("validated layout name");
        if layout["kind"] == "model" {
            scope_layouts.insert(scope_id, None::<String>);
            if name != "Model" {
                diagnostics.push(diagnostic(
                    "MODEL_LAYOUT_NAME",
                    format!("/layouts/{index}/name"),
                    "CAD model layout name is runtime-defined",
                ));
            }
        } else {
            if !document.objects.values().any(
                |object| matches!(object, ObjectType::Layout(existing) if existing.name == name),
            ) {
                document.add_layout(name).map_err(|error| {
                    DirectImportError::Cad(format!("paper layout {name}: {error}"))
                })?;
            }
            scope_layouts.insert(scope_id, Some(name.to_owned()));
        }
        let scaling = layout["paperSpaceLinetypeScaling"]
            .as_bool()
            .unwrap_or(true);
        if saved_linetype_scaling.is_some_and(|previous| previous != scaling) {
            diagnostics.push(diagnostic(
                "LAYOUT_SCALING",
                format!("/layouts/{index}/paperSpaceLinetypeScaling"),
                "CAD has one drawing-wide paper-space linetype scaling setting",
            ));
        } else {
            saved_linetype_scaling = Some(scaling);
            document.header.paper_space_linetype_scaling = scaling;
        }
        if let Some(ObjectType::Layout(target)) =
            document.objects.values_mut().find(|object| match object {
                ObjectType::Layout(target) if layout["kind"] == "model" => {
                    target.block_record == document.header.model_space_block_handle
                }
                ObjectType::Layout(target) => target.name == name,
                _ => false,
            })
        {
            if layout["limitsChecking"] == true {
                target.flags |= 2;
            } else {
                target.flags &= !2;
            }
            if let Some(limits) = layout["limits"].as_object() {
                target.min_limits = (
                    limits["minX"].as_f64().expect("validated limit"),
                    limits["minY"].as_f64().expect("validated limit"),
                );
                target.max_limits = (
                    limits["maxX"].as_f64().expect("validated limit"),
                    limits["maxY"].as_f64().expect("validated limit"),
                );
            }
            if let Some(plot) = layout.get("plotSettings") {
                crate::direct_layout::apply_plot_to_cad(target, plot);
            }
        }
        if layout["plotSettings"]["mapping"]["placement"]["reference"] == "PrintableArea" {
            diagnostics.push(diagnostic(
                "LAYOUT_SETTINGS",
                format!("/layouts/{index}/plotSettings/mapping/placement"),
                "CAD plot offset uses media origin",
            ));
        }
        if layout["plotSettings"]["options"]["plotTransparency"] == true {
            diagnostics.push(diagnostic(
                "LAYOUT_SETTINGS",
                format!("/layouts/{index}/plotSettings/options/plotTransparency"),
                "CAD plot transparency is unavailable",
            ));
        }
    }
    if let Some(workspace) = value.get("drawingWorkspaceState") {
        if let Some(id) = workspace["currentLayerId"].as_u64() {
            if let Some(name) = layer_names.get(&id) {
                document.header.current_layer_name = name.clone();
            }
        }
        if let Some(id) = workspace["activeLayoutId"].as_u64() {
            if let Some(layout) = drawing
                .layouts()
                .iter()
                .find(|layout| layout["id"].as_u64() == Some(id))
            {
                if layout["kind"] == "paper" {
                    document.header.show_model_space = false;
                    let name = layout["name"].as_str().expect("validated layout name");
                    if let Some(block) = document.objects.values().find_map(|object| match object {
                        ObjectType::Layout(target) if target.name == name => {
                            Some(target.block_record)
                        }
                        _ => None,
                    }) {
                        document.header.paper_space_block_handle = block;
                    }
                } else {
                    document.header.show_model_space = true;
                }
            }
        }
    }
    if let Some(display) = value.get("pointDisplay") {
        let glyph = match display["form"]["glyph"].as_str().expect("validated glyph") {
            "dot" => 0,
            "hidden" => 1,
            "plus" => 2,
            "cross" => 3,
            "shortLine" => 4,
            _ => unreachable!("validated glyph"),
        };
        document.header.point_display_mode = glyph
            + if display["form"]["circle"] == true {
                32
            } else {
                0
            }
            + if display["form"]["square"] == true {
                64
            } else {
                0
            };
        document.header.point_display_size = match display["size"]["kind"]
            .as_str()
            .expect("validated size kind")
        {
            "defaultFivePercent" => 0.0,
            "absolute" => display["size"]["value"].as_f64().expect("validated size"),
            "viewportPercent" => -display["size"]["value"].as_f64().expect("validated size"),
            _ => unreachable!("validated size kind"),
        };
    }
    if value.get("drawingViewState").is_some()
        || value.get("modelWindows").is_some()
        || value.get("paperCanvases").is_some()
        || value.get("viewportWorkspaces").is_some()
        || value.get("ucsDefinitions").is_some()
    {
        diagnostics.push(diagnostic(
            "DRAWING_STATE",
            "/",
            "point display or drawing view state is not yet transferred",
        ));
    }
    let mut block_handles = BTreeMap::new();
    let mut block_names = BTreeMap::new();
    if let Some(definitions) = value["blockDefinitions"].as_array() {
        for definition in definitions {
            let scope_id = definition["scopeId"]
                .as_u64()
                .expect("validated block scope");
            let name = definition["name"].as_str().expect("validated block name");
            let base = &definition["basePoint"];
            let mut record = BlockRecord::new(name);
            record.handle = document.allocate_handle();
            record.block_entity_handle = document.allocate_handle();
            record.block_end_handle = document.allocate_handle();
            record.base_point = cadcodec::Vector3::new(
                base["x"].as_f64().unwrap_or(0.0),
                base["y"].as_f64().unwrap_or(0.0),
                base["z"].as_f64().unwrap_or(0.0),
            );
            record.description = definition["description"].as_str().unwrap_or("").into();
            record.flags.anonymous = definition["anonymous"].as_bool().unwrap_or(false);
            record.explodable = definition["explodable"].as_bool().unwrap_or(true);
            record.scale_uniformly = definition["scaling"] == "Uniform";
            let unit = definition["insertionUnit"].as_str().unwrap_or("unitless");
            record.units = crate::direct::UNIT_TOKENS
                .iter()
                .position(|candidate| *candidate == unit)
                .unwrap_or(0) as i16;
            let mut marker = cadcodec::entities::Block::new(name, record.base_point);
            marker.description = record.description.clone();
            marker.common.handle = record.block_entity_handle;
            marker.common.owner_handle = record.handle;
            let mut end = cadcodec::entities::BlockEnd::new();
            end.common.handle = record.block_end_handle;
            end.common.owner_handle = record.handle;
            let handle = record.handle;
            document.block_records.add(record).map_err(|error| {
                DirectImportError::Cad(format!("block definition {name}: {error}"))
            })?;
            for entity in [EntityType::Block(marker), EntityType::BlockEnd(end)] {
                document.add_entity(entity).map_err(|error| {
                    DirectImportError::Cad(format!("block marker {name}: {error}"))
                })?;
            }
            block_handles.insert(scope_id, handle);
            block_names.insert(scope_id, name.to_owned());
        }
    }

    let mut entity_mapping = BTreeMap::new();
    for entry in value["streamDirectory"]["streams"]
        .as_array()
        .expect("validated directory")
    {
        if entry["role"] == "object"
            && !matches!(
                entry["name"].as_str(),
                Some(
                    "line"
                        | "point"
                        | "circle"
                        | "arc"
                        | "ellipse"
                        | "ellipseArc"
                        | "blockInstance"
                        | "planarPolyline"
                        | "spatialPolyline"
                )
            )
            && entry["count"].as_u64().unwrap_or(0) > 0
        {
            diagnostics.push(diagnostic(
                "ENTITY_KIND",
                "/streams",
                format!(
                    "{} entities are not yet transferred",
                    entry["name"].as_str().unwrap_or("unknown")
                ),
            ));
        }
    }
    let mut rows = BTreeMap::new();
    for kind in [
        "line",
        "point",
        "circle",
        "arc",
        "ellipse",
        "ellipseArc",
        "blockInstance",
        "planarPolyline",
        "spatialPolyline",
    ] {
        let payload = format!("{kind}Stream");
        if let Some(stream) = value["streams"].get(&payload) {
            let count = stream["count"].as_u64().expect("validated count") as usize;
            for row in 0..count {
                let id = column(stream, "entityId", row)
                    .as_u64()
                    .expect("validated ID");
                rows.insert(id, (kind, row));
            }
        }
    }
    if let Some(order) = value["streams"]["entityOrderEntryStream"]["entityId"].as_array() {
        for id in order.iter().filter_map(Value::as_u64) {
            let Some(&(kind, row)) = rows.get(&id) else {
                continue;
            };
            let payload = format!("{kind}Stream");
            let stream = &value["streams"][&payload];
            let scope_id = column(stream, "scopeId", row)
                .as_u64()
                .expect("validated scope");
            let layer_id = column(stream, "layerId", row)
                .as_u64()
                .expect("validated Layer");
            let Some(layer_name) = layer_names.get(&layer_id) else {
                continue;
            };
            if kind == "blockInstance" && !simple_block_transform(column(stream, "transform", row))
            {
                diagnostics.push(diagnostic(
                    "BLOCK_TRANSFORM",
                    format!("/streams/{payload}/transform/{row}"),
                    "block transform is not yet mapped to CAD",
                ));
                continue;
            }
            if matches!(kind, "planarPolyline" | "spatialPolyline")
                && !column(stream, "placement", row).is_null()
            {
                diagnostics.push(diagnostic(
                    "POLYLINE_FRAME",
                    format!("/streams/{payload}/placement/{row}"),
                    "polyline placement is not yet mapped to CAD",
                ));
                continue;
            }
            let angle = if matches!(kind, "line" | "blockInstance") {
                Some(0.0)
            } else {
                planar_angle(column(stream, "placement", row))
            };
            let Some(angle) = angle else {
                diagnostics.push(diagnostic(
                    "ENTITY_FRAME",
                    format!("/streams/{payload}/placement/{row}"),
                    "placed entity uses a plane that is not yet mapped to CAD",
                ));
                continue;
            };
            let mut entity = match kind {
                "line" => {
                    let start = ["x1", "y1", "z1"]
                        .map(|field| column(stream, field, row).as_f64().unwrap_or(0.0));
                    let end = ["x2", "y2", "z2"]
                        .map(|field| column(stream, field, row).as_f64().unwrap_or(0.0));
                    EntityType::Line(Line::from_coords(
                        start[0], start[1], start[2], end[0], end[1], end[2],
                    ))
                }
                "point" => {
                    let origin = &column(stream, "placement", row)["origin"];
                    let mut point = cadcodec::Point::from_coords(
                        origin["x"].as_f64().unwrap_or(0.0),
                        origin["y"].as_f64().unwrap_or(0.0),
                        origin["z"].as_f64().unwrap_or(0.0),
                    );
                    point.x_axis_angle = angle;
                    EntityType::Point(point)
                }
                "circle" => {
                    let origin = &column(stream, "placement", row)["origin"];
                    EntityType::Circle(cadcodec::Circle::from_center_radius(
                        cadcodec::Vector3::new(
                            origin["x"].as_f64().unwrap_or(0.0),
                            origin["y"].as_f64().unwrap_or(0.0),
                            origin["z"].as_f64().unwrap_or(0.0),
                        ),
                        column(stream, "radius", row)
                            .as_f64()
                            .expect("validated radius"),
                    ))
                }
                "arc" => {
                    let origin = &column(stream, "placement", row)["origin"];
                    let start = column(stream, "startParameter", row)
                        .as_f64()
                        .expect("validated arc start");
                    let sweep = column(stream, "sweepParameter", row)
                        .as_f64()
                        .expect("validated arc sweep");
                    EntityType::Arc(cadcodec::Arc::from_center_radius_angles(
                        cadcodec::Vector3::new(
                            origin["x"].as_f64().unwrap_or(0.0),
                            origin["y"].as_f64().unwrap_or(0.0),
                            origin["z"].as_f64().unwrap_or(0.0),
                        ),
                        column(stream, "radius", row)
                            .as_f64()
                            .expect("validated arc radius"),
                        start + angle,
                        start + angle + sweep,
                    ))
                }
                "ellipse" | "ellipseArc" => {
                    let origin = &column(stream, "placement", row)["origin"];
                    let major = column(stream, "semiMajorRadius", row)
                        .as_f64()
                        .expect("validated major radius");
                    let minor = column(stream, "semiMinorRadius", row)
                        .as_f64()
                        .expect("validated minor radius");
                    let x = &column(stream, "placement", row)["X"];
                    let mut ellipse = cadcodec::Ellipse::from_center_axes(
                        cadcodec::Vector3::new(
                            origin["x"].as_f64().unwrap_or(0.0),
                            origin["y"].as_f64().unwrap_or(0.0),
                            origin["z"].as_f64().unwrap_or(0.0),
                        ),
                        cadcodec::Vector3::new(
                            x["x"].as_f64().unwrap_or(1.0) * major,
                            x["y"].as_f64().unwrap_or(0.0) * major,
                            0.0,
                        ),
                        minor / major,
                    );
                    if kind == "ellipseArc" {
                        ellipse.start_parameter = column(stream, "startParameter", row)
                            .as_f64()
                            .expect("validated start");
                        ellipse.end_parameter = ellipse.start_parameter
                            + column(stream, "sweepParameter", row)
                                .as_f64()
                                .expect("validated sweep");
                    }
                    EntityType::Ellipse(ellipse)
                }
                "blockInstance" => {
                    let definition = column(stream, "definitionScopeId", row)
                        .as_u64()
                        .expect("validated definition scope");
                    let name = block_names
                        .get(&definition)
                        .expect("validated block definition");
                    let origin = &column(stream, "transform", row)["placement"]["origin"];
                    EntityType::Insert(cadcodec::entities::Insert::new(
                        name,
                        cadcodec::Vector3::new(
                            origin["x"].as_f64().unwrap_or(0.0),
                            origin["y"].as_f64().unwrap_or(0.0),
                            origin["z"].as_f64().unwrap_or(0.0),
                        ),
                    ))
                }
                "planarPolyline" => {
                    let offset = column(stream, "vertexOffset", row)
                        .as_u64()
                        .expect("validated offset") as usize;
                    let count = column(stream, "vertexCount", row)
                        .as_u64()
                        .expect("validated vertex count") as usize;
                    let points = (offset..offset + count)
                        .map(|index| {
                            cadcodec::Vector2::new(
                                stream["x"][index].as_f64().expect("validated x"),
                                stream["y"][index].as_f64().expect("validated y"),
                            )
                        })
                        .collect();
                    let mut polyline = cadcodec::LwPolyline::from_points(points);
                    polyline.is_closed = column(stream, "closed", row)
                        .as_bool()
                        .expect("validated closed");
                    for (index, vertex) in polyline.vertices.iter_mut().enumerate() {
                        vertex.bulge = stream["bulge"][offset + index]
                            .as_f64()
                            .expect("validated bulge");
                    }
                    EntityType::LwPolyline(polyline)
                }
                "spatialPolyline" => {
                    let offset = column(stream, "vertexOffset", row)
                        .as_u64()
                        .expect("validated offset") as usize;
                    let count = column(stream, "vertexCount", row)
                        .as_u64()
                        .expect("validated vertex count") as usize;
                    let points = (offset..offset + count)
                        .map(|index| {
                            cadcodec::Vector3::new(
                                stream["x"][index].as_f64().expect("validated x"),
                                stream["y"][index].as_f64().expect("validated y"),
                                stream["z"][index].as_f64().expect("validated z"),
                            )
                        })
                        .collect();
                    let mut polyline = cadcodec::entities::Polyline3D::from_points(points);
                    polyline.flags.closed = column(stream, "closed", row)
                        .as_bool()
                        .expect("validated closed");
                    EntityType::Polyline3D(polyline)
                }
                _ => unreachable!("known family"),
            };
            apply_common(
                stream,
                row,
                kind,
                entity.common_mut(),
                layer_name,
                &mut diagnostics,
            );
            let handle = match scope_layouts.get(&scope_id) {
                Some(None) => document.add_entity(entity),
                Some(Some(layout)) => document.add_entity_to_layout(entity, layout),
                None => {
                    entity.common_mut().owner_handle = block_handles[&scope_id];
                    document.add_entity(entity)
                }
            }
            .map_err(|error| DirectImportError::Cad(format!("entity {id}: {error}")))?;
            entity_mapping.insert(id, handle);
        }
    }
    if options.loss_policy == ConversionLossPolicy::Reject && !diagnostics.is_empty() {
        return Err(DirectImportError::LossRejected { diagnostics });
    }
    Ok(DirectImportOutcome {
        document,
        diagnostics,
        entity_mapping,
    })
}
