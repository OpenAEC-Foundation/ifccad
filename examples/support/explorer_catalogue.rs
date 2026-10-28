//! Extend SAMPLES with a named drawing function; the grid and sheet grow automatically.
use super::drawing::{diagonal_fill, plane, Drawing, Result, MODEL};
use serde_json::json;
use std::f64::consts::{FRAC_PI_2, PI};

pub struct Sample {
    pub title: &'static str,
    pub note: &'static str,
    pub draw: fn(&mut Drawing, f64, f64),
}

// Each sample uses local coordinates: x=0..82, y=0..42. Labels are outside that area.
// Appending entries preserves the reading order of existing examples.
pub const SAMPLES: &[Sample] = &[
    Sample {
        title: "Point",
        note: "Position + reference cross",
        draw: |d, x, y| {
            d.entity(
                MODEL,
                1,
                json!({"ifccad::geom::placement":plane(x+40.,y+20.),"ifccad::geom::point":{}}),
            );
            d.line(MODEL, 1, [x + 35., y + 20.], [x + 45., y + 20.]);
            d.line(MODEL, 1, [x + 40., y + 15.], [x + 40., y + 25.]);
        },
    },
    Sample {
        title: "Line segment",
        note: "Start and end points",
        draw: |d, x, y| {
            d.line(MODEL, 1, [x + 10., y + 8.], [x + 72., y + 32.]);
        },
    },
    Sample {
        title: "Circle",
        note: "Centre and radius",
        draw: |d, x, y| d.circle(MODEL, 1, x + 41., y + 21., 17.),
    },
    Sample {
        title: "Arc",
        note: "Positive and negative sweep",
        draw: |d, x, y| {
            d.arc(MODEL, 1, x + 22., y + 20., 17., 0., PI * 1.4);
            d.arc(MODEL, 1, x + 62., y + 20., 17., 0., -PI * 1.4);
        },
    },
    Sample {
        title: "Ellipse",
        note: "Major and minor radii",
        draw: |d, x, y| {
            d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+41.,y+21.),"ifccad::geom::ellipse":{"semiMajorRadius":32.,"semiMinorRadius":15.}}));
        },
    },
    Sample {
        title: "Ellipse arc",
        note: "Partial ellipse",
        draw: |d, x, y| {
            d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+41.,y+21.),"ifccad::geom::ellipseArc":{"semiMajorRadius":32.,"semiMinorRadius":15.,"startParameter":0.3,"sweepParameter":4.5}}));
        },
    },
    Sample {
        title: "Planar polyline",
        note: "Open, closed and bulged",
        draw: |d, x, y| {
            d.poly(
                MODEL,
                1,
                vec![[x + 4., y + 5.], [x + 18., y + 34.], [x + 30., y + 8.]],
                false,
            );
            d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+47.,y+7.),"ifccad::geom::planarPolyline":{"vertices":[[0.,0.],[26.,0.],[26.,24.],[0.,24.]],"bulges":[0.7,0.,0.,0.],"closed":true}}));
        },
    },
    Sample {
        title: "Spatial polyline",
        note: "XYZ path, viewed from above",
        draw: |d, x, y| {
            d.entity(MODEL,1,json!({"ifccad::geom::spatialPolyline":{"vertices":[[x+5.,y+5.,0.],[x+25.,y+30.,10.],[x+50.,y+10.,25.],[x+75.,y+35.,15.]],"closed":false}}));
        },
    },
    Sample {
        title: "Text",
        note: "Single-line text with a style",
        draw: |d, x, y| {
            d.text(MODEL, 1, x + 5., y + 27., 7., "IFCCAD");
            d.text(MODEL, 1, x + 5., y + 10., 4., "Text at an anchor point");
        },
    },
    Sample {
        title: "MText",
        note: "Paragraphs and formatting",
        draw: |d, x, y| {
            d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+5.,y+37.),"ifccad::mText":{"style":"/cad/d1/textStyle/0","height":4.,"wrapWidth":72.,"attachment":"topLeft","flow":"horizontal","content":[{"inlines":[{"kind":"run","text":"Multiple lines"}]},{"inlines":[{"kind":"run","text":"in a text frame"}]}]}}));
        },
    },
    Sample {
        title: "Solid hatch",
        note: "Filled region with an island",
        draw: |d, x, y| {
            d.hatch(
                1,
                vec![
                    [x + 8., y + 5.],
                    [x + 74., y + 5.],
                    [x + 74., y + 37.],
                    [x + 8., y + 37.],
                ],
                json!({"kind":"solid"}),
                None,
            );
            let last = d.nodes.last_mut().unwrap();
            last["attributes"]["ifccad::hatch"]["loops"]
                .as_array_mut()
                .unwrap()
                .push(json!({"boundary":{"kind":"circle","center":[x+41.,y+21.],"radius":10.}}));
        },
    },
    Sample {
        title: "Pattern hatch",
        note: "Explicit line family",
        draw: |d, x, y| {
            d.hatch(
                1,
                vec![
                    [x + 8., y + 5.],
                    [x + 74., y + 5.],
                    [x + 74., y + 37.],
                    [x + 8., y + 37.],
                ],
                diagonal_fill(5.),
                None,
            );
        },
    },
    Sample {
        title: "Gradient hatch",
        note: "Two-colour linear gradient",
        draw: |d, x, y| {
            d.hatch(1,vec![[x+8.,y+5.],[x+74.,y+5.],[x+74.,y+37.],[x+8.,y+37.]],json!({"kind":"gradient"}),Some(json!({"type":"linear","colors":[{"rgb":[30,100,180]},{"rgb":[90,200,170]}],"colorMode":"twoColor","rotation":0.,"shift":0.})));
        },
    },
    Sample {
        title: "Block instances",
        note: "Repeat, rotate and mirror",
        draw: |d, x, y| {
            let (owner, id) = d.block("Catalogue marker");
            d.poly(
                owner,
                0,
                vec![[0., 0.], [12., 0.], [12., 8.], [0., 0.]],
                false,
            );
            d.circle(owner, 0, 3., 3., 1.5);
            d.insert(1, id, x + 12., y + 12., 0., [1., 1., 1.]);
            d.insert(1, id, x + 43., y + 12., FRAC_PI_2, [1.5, 1.5, 1.]);
            d.insert(1, id, x + 74., y + 12., 0., [-1., 1., 1.]);
        },
    },
    Sample {
        title: "Line patterns",
        note: "Continuous, dashed and text",
        draw: |d, x, y| {
            d.line(MODEL, 1, [x + 5., y + 34.], [x + 77., y + 34.]);
            d.line(MODEL, 1, [x + 5., y + 21.], [x + 77., y + 21.]);
            d.nodes.last_mut().unwrap()["attributes"]["ifccad::entity"]["appearance"]
                ["linePattern"] = json!({"mode":"Explicit","value":"/cad/d1/linePattern/1"});
            d.line(MODEL, 1, [x + 5., y + 8.], [x + 77., y + 8.]);
            d.nodes.last_mut().unwrap()["attributes"]["ifccad::entity"]["appearance"]
                ["linePattern"] = json!({"mode":"Explicit","value":"/cad/d1/linePattern/2"});
        },
    },
    Sample {
        title: "Layers & appearance",
        note: "Layer colour and lineweight",
        draw: |d, x, y| {
            for (i, layer) in [1, 3, 4].iter().enumerate() {
                d.rect(MODEL, *layer, x + 8. + i as f64 * 24., y + 8., 18., 26.);
            }
        },
    },
    Sample {
        title: "Layouts & viewport",
        note: "Open the A3 sheet for the viewport",
        draw: |d, x, y| {
            d.rect(MODEL, 0, x + 12., y + 3., 58., 38.);
            d.rect(MODEL, 1, x + 17., y + 12., 48., 24.);
            d.text(MODEL, 0, x + 19., y + 5., 3., "MODEL > PAPER");
        },
    },
    Sample {
        title: "SOLID filled faces",
        note: "Filled face / arrowhead",
        draw: |d, x, y| {
            for corners in [
                [[4., 8.], [34., 8.], [4., 30.], [34., 30.]],
                [[48., 15.], [48., 23.], [76., 19.], [76., 19.]],
            ] {
                d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x,y),"ifccad::geom::solid":{"corners":corners}}));
            }
        },
    },
    Sample {
        title: "TRACE wide strokes",
        note: "Wide strip / joined strips",
        draw: |d, x, y| {
            for corners in [
                [[4., 12.], [4., 18.], [31., 12.], [31., 18.]],
                [[47., 9.], [47., 15.], [72., 9.], [66., 15.]],
                [[72., 9.], [66., 15.], [72., 30.], [66., 30.]],
            ] {
                d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x,y),"ifccad::geom::trace":{"corners":corners}}));
            }
        },
    },
];

pub fn build() -> Result<Vec<u8>> {
    build_with_samples(SAMPLES)
}

/// One grouped, authored illustration; not a production Dimension renderer.
fn dimension_section(d: &mut Drawing) -> Result<()> {
    use ifccad::dimension::*;
    use serde_json::Value;
    const SCALE: f64 = 3.;
    fn coordinates(value: &mut Value) {
        for component in value.as_array_mut().unwrap() {
            if component.is_array() {
                coordinates(component);
            } else {
                *component = json!(component.as_f64().unwrap() * SCALE);
            }
        }
    }
    fn length(value: &mut Value) {
        if let Some(number) = value.as_f64() {
            *value = json!(number * SCALE);
        }
    }
    d.rect(MODEL, 0, 0., 0., 388., 280.);
    d.text(MODEL, 0, 8., 261., 5., "DIMENSIONS / ALL NINE KINDS");
    d.text(
        MODEL,
        0,
        8.,
        248.,
        3.,
        "Definitions, measured values and stored graphics; select each dimension.",
    );
    // Reuse the reviewed native illustration as source material. This recipe
    // creates millimetre examples, with larger geometry and newly authored labels.
    let source: Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/ifccad/dimension-kinds-gallery.ifcx"
    ))?;
    let nodes = source["data"].as_array().unwrap();
    let mut numbers = DimensionNumberSettings::default();
    numbers.primary.zero_suppression.trailing = true;
    for (index, node) in nodes
        .iter()
        .filter(|n| n["attributes"].get("ifccad::dimension").is_some())
        .enumerate()
    {
        let old_path = node["path"].as_str().unwrap();
        let mut attributes = node["attributes"].clone();
        let dimension = &mut attributes["ifccad::dimension"];
        for (field, value) in dimension["definition"].as_object_mut().unwrap() {
            if value.is_array() {
                coordinates(value);
            } else if matches!(field.as_str(), "radius" | "leaderLength") {
                length(value);
            } else if field == "leader" {
                for point in value.as_object_mut().unwrap().values_mut() {
                    coordinates(point);
                }
            }
        }
        match dimension["definition"]["kind"].as_str().unwrap() {
            "linear" => {
                dimension["definition"]["secondPoint"] = json!([18., 12., 0.]);
                dimension["definition"]["dimensionLinePoint"] = json!([0., 21.]);
            }
            "angularTwoLines" => {
                dimension["definition"]["line1End"] = json!([27., 0., 0.]);
                dimension["definition"]["line2End"] = json!([0., 27., 0.]);
            }
            "angularThreePoints" => {
                dimension["definition"]["firstPoint"] = json!([27., 0., 0.]);
                dimension["definition"]["secondPoint"] = json!([0., 27., 0.]);
            }
            _ => {}
        }
        let definition: DimensionDefinition =
            serde_json::from_value(dimension["definition"].clone())?;
        let title = match definition.kind() {
            DimensionKind::Linear => "Linear",
            DimensionKind::Aligned => "Aligned",
            DimensionKind::Radius => "Radius",
            DimensionKind::Diameter => "Diameter",
            DimensionKind::AngularTwoLines => "Two-line angle",
            DimensionKind::AngularThreePoints => "Three-point angle",
            DimensionKind::Ordinate => "Ordinate",
            DimensionKind::ArcLength => "Arc length",
            DimensionKind::JoggedRadius => "Jogged radius",
        };
        let mut label = format_dimension_kind_label(
            definition.kind(),
            evaluate_dimension(&definition)?,
            &numbers,
            DimensionFormatContext { is_paper: false },
            &DimensionLabel::Generated,
        )?
        .runs
        .into_iter()
        .map(|r| r.text)
        .collect::<String>();
        if matches!(
            definition.kind(),
            DimensionKind::AngularTwoLines | DimensionKind::AngularThreePoints
        ) {
            label.push('°');
        }
        dimension["overrides"] = json!({"text":{"height":2.4,"gap":0.6},"arrows":{"size":0.9}});
        attributes["ifccad::geom::placement"] = plane(
            65. + (index % 3) as f64 * 120.,
            210. - (index / 3) as f64 * 75.,
        );
        let origin = [
            65. + (index % 3) as f64 * 120.,
            210. - (index / 3) as f64 * 75.,
        ];
        let id = d.entity(MODEL, 2, attributes);
        let dimension_node = d.nodes.len() - 1;
        let path = format!("/cad/d1/e{id}");
        d.nodes[dimension_node]["children"] = json!({"graphics":format!("{path}/graphics")});
        let mut label_written = false;
        let mut parts = Vec::new();
        let mut associations = Vec::new();
        for (part_index, part) in nodes
            .iter()
            .filter(|n| {
                n["path"]
                    .as_str()
                    .unwrap()
                    .starts_with(&format!("{old_path}/graphics/"))
            })
            .enumerate()
        {
            let mut a = part["attributes"].clone();
            if a.get("ifccad::geom::placement").is_some() {
                coordinates(&mut a["ifccad::geom::placement"]["origin"]);
            }
            if let Some(line) = a.get_mut("ifccad::geom::lineSegment") {
                coordinates(&mut line["start"]);
                coordinates(&mut line["end"]);
            }
            for kind in ["ifccad::geom::circle", "ifccad::geom::arc"] {
                if let Some(curve) = a.get_mut(kind) {
                    length(&mut curve["radius"]);
                }
            }
            if let Some(solid) = a.get_mut("ifccad::geom::solid") {
                coordinates(&mut solid["corners"]);
            }
            if definition.kind() == DimensionKind::Linear {
                match part_index {
                    0 => a["ifccad::geom::lineSegment"] = json!({"start":[0.,0.],"end":[18.,12.]}),
                    1 => a["ifccad::geom::lineSegment"] = json!({"start":[0.,0.],"end":[0.,21.]}),
                    2 => {
                        a["ifccad::geom::lineSegment"] = json!({"start":[18.,12.],"end":[18.,21.]})
                    }
                    3 => a["ifccad::geom::lineSegment"] = json!({"start":[0.,21.],"end":[18.,21.]}),
                    4 | 5 => {
                        for corner in a["ifccad::geom::solid"]["corners"].as_array_mut().unwrap() {
                            corner[0] = json!(
                                corner[0].as_f64().unwrap() - if part_index == 5 { 6. } else { 0. }
                            );
                            corner[1] = json!(corner[1].as_f64().unwrap() + 12.);
                        }
                    }
                    _ => {}
                }
            }
            let mut caption = false;
            if let Some(text) = a.get_mut("ifccad::text") {
                length(&mut text["layout"]["height"]);
                if !label_written {
                    text["content"][0]["text"] = json!(label);
                    label_written = true;
                } else {
                    text["content"][0]["text"] = json!(title);
                    caption = true;
                }
            }
            let reference = match definition.kind() {
                DimensionKind::AngularTwoLines | DimensionKind::AngularThreePoints => {
                    part_index < 2
                }
                _ => part_index == 0,
            };
            if caption || reference {
                // These are independent objects, so editing a DIM does not
                // erase the sample name or the measured reference geometry.
                a.as_object_mut()
                    .unwrap()
                    .remove("ifccad::dimensionGraphicsPart");
                if let Some(line) = a.get_mut("ifccad::geom::lineSegment") {
                    for field in ["start", "end"] {
                        let p = &line[field];
                        line[field] = json!([
                            p[0].as_f64().unwrap() + origin[0],
                            p[1].as_f64().unwrap() + origin[1],
                            0.
                        ]);
                    }
                } else {
                    let p = &mut a["ifccad::geom::placement"]["origin"];
                    p[0] = json!(p[0].as_f64().unwrap() + origin[0]);
                    p[1] = json!(p[1].as_f64().unwrap() + origin[1]);
                }
                let source = d.entity(MODEL, if caption { 0 } else { 1 }, a);
                if reference {
                    let mut bind = |role: DimensionRole, selector: DimensionPointSelector| {
                        associations.push(json!({"role":role,"selector":selector,"source":format!("/cad/d1/e{source}")}))
                    };
                    use DimensionPointSelector as P;
                    use DimensionRole as R;
                    match &definition {
                        DimensionDefinition::Linear { .. }
                        | DimensionDefinition::Aligned { .. } => {
                            bind(R::FirstPoint, P::Start);
                            bind(R::SecondPoint, P::End);
                        }
                        DimensionDefinition::AngularTwoLines { .. } if part_index == 0 => {
                            bind(R::Line1Start, P::Start);
                            bind(R::Line1End, P::End);
                        }
                        DimensionDefinition::AngularTwoLines { .. } => {
                            bind(R::Line2Start, P::Start);
                            bind(R::Line2End, P::End);
                        }
                        DimensionDefinition::AngularThreePoints { .. } if part_index == 0 => {
                            bind(R::Vertex, P::Start);
                            bind(R::FirstLegPoint, P::End);
                        }
                        DimensionDefinition::AngularThreePoints { .. } => {
                            bind(R::SecondLegPoint, P::End)
                        }
                        DimensionDefinition::Ordinate { .. } => {
                            bind(R::Origin, P::Start);
                            bind(R::FeaturePoint, P::End);
                        }
                        DimensionDefinition::Radius {
                            center,
                            curve_point,
                            ..
                        }
                        | DimensionDefinition::JoggedRadius {
                            center,
                            curve_point,
                            ..
                        } => {
                            bind(R::Center, P::Center);
                            bind(
                                R::CurvePoint,
                                P::CirclePoint {
                                    angle: (curve_point[1] - center[1])
                                        .atan2(curve_point[0] - center[0]),
                                },
                            );
                        }
                        DimensionDefinition::Diameter {
                            first_diameter_point: a,
                            second_diameter_point: b,
                            ..
                        } => {
                            bind(
                                R::FirstDiameterPoint,
                                P::CirclePoint {
                                    angle: a[1].atan2(a[0]),
                                },
                            );
                            bind(
                                R::SecondDiameterPoint,
                                P::CirclePoint {
                                    angle: b[1].atan2(b[0]),
                                },
                            );
                        }
                        DimensionDefinition::ArcLength { .. } => {
                            bind(R::Center, P::Center);
                            bind(R::ArcStart, P::Start);
                            bind(R::ArcEnd, P::End);
                        }
                    }
                }
            } else {
                parts.push(a);
            }
        }
        d.nodes[dimension_node]["attributes"]["ifccad::dimension"]["associations"] =
            json!(associations);
        d.nodes.push(json!({"path":format!("{path}/graphics"),"attributes":{"ifccad::dimensionGraphics":{"origin":"generated"}},"children":parts.iter().enumerate().map(|(i,_)|(i.to_string(),json!(format!("{path}/graphics/p{i}")))).collect::<serde_json::Map<_,_>>()}));
        for (i, a) in parts.into_iter().enumerate() {
            d.nodes
                .push(json!({"path":format!("{path}/graphics/p{i}"),"attributes":a}));
        }
        assert!(
            label_written,
            "reviewed illustration needs its stored label"
        );
    }
    Ok(())
}

pub fn build_with_samples(samples: &[Sample]) -> Result<Vec<u8>> {
    let mut d = Drawing::new("explorer-entity-catalogue");
    d.layer("Samples", Some([40, 140, 185]), 0.3);
    d.layer("Dimensions", None, 0.13);
    d.layer("Green accent", Some([55, 175, 120]), 0.5);
    d.layer("Orange accent", Some([220, 145, 55]), 0.7);
    d.pattern();
    d.nodes[0]["children"]["gas"] = json!("/cad/d1/linePattern/2");
    d.nodes[0]["attributes"]["ifccad::drawing"]["nextLinePatternId"] = json!(3);
    d.nodes.push(json!({"path":"/cad/d1/linePattern/2","attributes":{"ifccad::linePattern":{"name":"GAS","pattern":[10.,{"length":0.,"marker":{"kind":"text","text":"GAS","textStyle":"/cad/d1/textStyle/0","scale":2.,"offset":[0.,0.],"rotation":{"mode":"relative","angle":0.}}},-7.]}}}));
    let rows = samples.len().div_ceil(4);
    let total_height = rows as f64 * 78. + 296.;
    d.text(
        MODEL,
        0,
        0.,
        total_height + 30.,
        9.,
        "IFCCAD / ENTITY CATALOGUE",
    );
    d.text(
        MODEL,
        0,
        0.,
        total_height + 18.,
        3.7,
        "Select a sample to inspect its native properties.",
    );
    for (i, sample) in samples.iter().enumerate() {
        let x = (i % 4) as f64 * 98.;
        let y = total_height - (i / 4 + 1) as f64 * 78.;
        d.rect(MODEL, 0, x, y, 94., 74.);
        d.text(
            MODEL,
            0,
            x + 5.,
            y + 63.,
            4.,
            &format!("{:02}  {}", i + 1, sample.title),
        );
        d.text(MODEL, 0, x + 5., y + 54., 2.6, sample.note);
        (sample.draw)(&mut d, x + 5., y + 5.);
    }
    dimension_section(&mut d)?;
    d.text(
        MODEL,
        0,
        0.,
        -12.,
        3.1,
        "Dimensions are in millimetres. Select them to inspect styles and measured values.",
    );
    let paper = d.sheet(
        "A3 - Entity catalogue",
        [194., total_height / 2. + 10.],
        total_height + 70.,
        370.,
        235.,
    );
    d.rect(paper, 0, 10., 10., 400., 277.);
    d.text(paper, 0, 18., 22., 5., "IFCCAD / SUPPORTED ENTITIES");
    d.text(
        paper,
        0,
        18.,
        14.,
        2.8,
        "Geometry, annotation, reuse and presentation / experimental profile",
    );
    d.model_view([194., total_height / 2. + 15.], total_height + 190.);
    d.finish("explorer-entity-catalogue")
}
