#[path = "ifccad_consistency.rs"]
mod consistency;
#[path = "explorer_drawing.rs"]
#[allow(dead_code)]
mod drawing;
use drawing::{plane, Drawing, MODEL};
use ifccad::*;
use serde_json::json;

pub fn document() -> IfccadDocument {
    build(false)
}

#[allow(dead_code)]
pub fn explorer_document() -> IfccadDocument {
    build(true)
}

fn build(explorer: bool) -> IfccadDocument {
    let mut d = Drawing::new("extrusion-thickness-gallery");
    let layer = d.layer("Extrusion", Some([30, 130, 220]), 0.18);
    d.text(
        MODEL,
        0,
        0.,
        20.,
        3.,
        "EXTRUSION THICKNESS / mm / profile 0.3.0",
    );
    for (col, (h, title)) in [(3., "+3 / +Z"), (-3., "-3 / +Z"), (3., "+3 / oblique")]
        .into_iter()
        .enumerate()
    {
        let x = col as f64 * 55.;
        d.text(MODEL, 0, x, 10., 2., title);
        for row in 0..5 {
            let y = -(row as f64) * 20.;
            let mut p = plane(x, y);
            p["origin"] = json!([x, y, 2.]);
            let n = if col == 2 {
                p["yAxis"] = json!([0., 0.8, 0.6]);
                [0., -0.6, 0.8]
            } else {
                [0., 0., 1.]
            };
            let (key, body, title) = match row {
                0 => ("point", json!({"thickness":h}), "POINT"),
                1 => (
                    "lineSegment",
                    json!({"start":[x,y,2.],"end":[x+12.,y+4.,5.],"thickness":h,"extrusionDirection":n}),
                    "LINE",
                ),
                2 => ("circle", json!({"radius":5.,"thickness":h}), "CIRCLE"),
                3 => (
                    "arc",
                    json!({"radius":5.,"startParameter":0.,"sweepParameter":2.5,"thickness":h}),
                    "ARC",
                ),
                _ => (
                    "planarPolyline",
                    json!({"vertices":[[0.,0.],[10.,0.],[15.,5.]],"bulges":[0.25,0.,0.],"widths":{"kind":"constant","width":0.75},"thickness":h}),
                    "POLYLINE width 0.75",
                ),
            };
            let mut attrs = json!({});
            attrs[format!("ifccad::geom::{key}")] = body;
            if row != 1 {
                attrs["ifccad::geom::placement"] = p;
            }
            d.entity(MODEL, layer, attrs);
            d.text(MODEL, 0, x, y - 8., 1.8, title);
        }
    }
    d.text(
        MODEL,
        0,
        0.,
        -103.,
        2.,
        "CLASSIC POLYLINE / effective widths retained",
    );
    let mut classic_plane = plane(0., -105.);
    classic_plane["origin"] = json!([0., -105., 1.]);
    d.entity(MODEL,layer,json!({"ifccad::geom::placement":classic_plane,"ifccad::geom::planarPolyline":{"vertices":[[0.,0.],[10.,0.],[15.,5.]],"bulges":[0.25,0.,0.],"thickness":4.,"widths":{"kind":"perVertex","widths":[[0.25,0.5],[0.,0.5],[0.25,0.]]}}}));
    d.text(MODEL, 0, 0., -118., 2., "WIDTH MODES / all thickness +3");
    for (i,widths) in [json!({"kind":"perVertex","widths":[[0.25,1.],[1.,0.],[0.,0.5]]}),json!({"kind":"defaulted","defaultStartWidth":0.25,"defaultEndWidth":0.5,"overrides":[{}, {"start":0.}, {"end":0.}]})].into_iter().enumerate(){
  d.entity(MODEL,layer,json!({"ifccad::geom::placement":plane(i as f64*55.,-130.),"ifccad::geom::planarPolyline":{"vertices":[[0.,0.],[10.,0.],[15.,5.]],"closed":true,"widths":widths,"thickness":3.}}));
 }
    d.entity(MODEL,layer,json!({"ifccad::geom::placement":plane(110.,-130.),"ifccad::geom::solid":{"corners":[[0.,0.],[8.,0.],[0.,5.],[8.,5.]],"thickness":3.}}));
    d.text(
        MODEL,
        0,
        0.,
        -150.,
        2.,
        "Existing TEXT thickness +2 control",
    );
    let idx = d.nodes.len() - 1;
    d.nodes[idx]["attributes"]["ifccad::text"]["thickness"] = json!(2.);
    d.dimension(layer, [60., -165.], [80., -165.], [70., -158.], false, 2.);
    if explorer {
        d.model_view([0., 0.], 260.);
        let view = &mut d.nodes.last_mut().unwrap()["attributes"]["ifccad::modelWindow"]["view"];
        view["target"] = json!([60., -75., 0.]);
        view["direction"] = json!([1., -1., 1.]);
    }
    let bytes = d.finish("extrusion-thickness-gallery").unwrap();
    let mut doc = load_ifccad_bytes(&bytes, Default::default())
        .unwrap()
        .into_document();
    for e in &mut doc.model.entities {
        if let Some(e) = e.as_native_mut() {
            if let IfccadEntityKind::Dimension(v) = &mut e.kind {
                v.graphics = Some(IfccadDimensionGraphics {
                    origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
                    parts: vec![IfccadDimensionGraphicsPart {
                        layer_id: None,
                        visible: true,
                        line_pattern_scale: 1.,
                        purpose: IfccadDimensionGraphicsPurpose::Display,
                        appearance: e.appearance.clone(),
                        kind: IfccadDimensionGraphicsKind::LineSegment {
                            start: [60., -158., 0.],
                            end: [80., -158., 0.],
                            thickness: 2.,
                            extrusion_direction: [0., 0., 1.],
                        },
                    }],
                });
            }
        }
    }
    doc.model.bounds = None;
    doc.model.bounds_quality = None;
    validate_ifccad_document(&doc).unwrap();
    doc
}
