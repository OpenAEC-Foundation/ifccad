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
        title: "Linear dimension",
        note: "Linear measurement in millimetres",
        draw: |d, x, y| {
            d.line(MODEL, 1, [x + 12., y + 6.], [x + 70., y + 6.]);
            d.dimension(
                2,
                [x + 12., y + 6.],
                [x + 70., y + 6.],
                [x + 12., y + 28.],
                false,
                4.,
            );
        },
    },
    Sample {
        title: "Aligned dimension",
        note: "Dimension parallel to the line",
        draw: |d, x, y| {
            d.line(MODEL, 1, [x + 12., y + 5.], [x + 66., y + 23.]);
            d.dimension(
                2,
                [x + 12., y + 5.],
                [x + 66., y + 23.],
                [x + 10., y + 27.],
                true,
                4.,
            );
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
        note: "Triangle / rectangle / trapezoid",
        draw: |d, x, y| {
            for (offset, corners, h) in [
                (0., [[0., 0.], [24., 0.], [12., 22.], [12., 22.]], 0.),
                (28., [[0., 0.], [24., 0.], [0., 22.], [24., 22.]], 3.),
                (56., [[4., 0.], [20., 0.], [0., 22.], [24., 22.]], 0.),
            ] {
                d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+offset,y+8.),"ifccad::geom::solid":{"corners":corners,"thickness":h}}));
            }
        },
    },
    Sample {
        title: "TRACE filled faces",
        note: "Triangle / rectangle / trapezoid",
        draw: |d, x, y| {
            for (offset, corners, h) in [
                (0., [[0., 0.], [24., 0.], [12., 22.], [12., 22.]], -3.),
                (28., [[0., 0.], [24., 0.], [0., 22.], [24., 22.]], 3.),
                (56., [[4., 0.], [20., 0.], [0., 22.], [24., 22.]], 0.),
            ] {
                d.entity(MODEL,1,json!({"ifccad::geom::placement":plane(x+offset,y+8.),"ifccad::geom::trace":{"corners":corners,"thickness":h}}));
            }
        },
    },
];

pub fn build() -> Result<Vec<u8>> {
    build_with_samples(SAMPLES)
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
    let total_height = rows as f64 * 78.;
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
