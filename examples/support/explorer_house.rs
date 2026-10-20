//! A fictitious, coherently dimensioned ground floor, authored in millimetres.
use super::drawing::{diagonal_fill, Drawing, Result, MODEL};
use serde_json::json;
use std::f64::consts::{FRAC_PI_2, PI};

fn wall(d: &mut Drawing, x: f64, y: f64, w: f64, h: f64) {
    d.hatch(
        1,
        vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
        diagonal_fill(110.),
        None,
    );
    d.rect(MODEL, 1, x, y, w, h);
}

fn horizontal_measure(d: &mut Drawing, x1: f64, x2: f64, source_y: f64, y: f64) {
    d.dimension(5, [x1, source_y], [x2, source_y], [x1, y], false, 230.);
}

pub fn build() -> Result<Vec<u8>> {
    let mut d = Drawing::new("explorer-house-ground-floor");
    d.layer("A-Walls", None, 0.5);
    d.layer("A-Openings", Some([60, 150, 180]), 0.25);
    d.layer("A-Furniture", Some([130, 150, 155]), 0.18);
    d.layer("A-Text", None, 0.18);
    d.layer("A-Dimensions", None, 0.18);
    d.dimension_precision(0);
    d.layer("A-Reference", Some([105, 125, 135]), 0.13);
    d.pattern();
    // Wall strips stop at each window/door; openings are actual gaps.
    for (x, w) in [(0., 1500.), (4500., 2000.), (7500., 2500.)] {
        wall(&mut d, x, 0., w, 300.);
    }
    for (x, w) in [(0., 1500.), (4500., 2000.), (8500., 1500.)] {
        wall(&mut d, x, 11700., w, 300.);
    }
    for (y, h) in [(300., 2200.), (5500., 3000.), (10500., 1200.)] {
        wall(&mut d, 0., y, 300., h);
    }
    for (y, h) in [(300., 2200.), (4500., 4000.), (10500., 1200.)] {
        wall(&mut d, 9700., y, 300., h);
    }
    for (y, h) in [(300., 2700.), (4000., 3500.)] {
        wall(&mut d, 6100., y, 150., h);
    }
    wall(&mut d, 6250., 7500., 3450., 150.);
    wall(&mut d, 7900., 300., 150., 700.);
    wall(&mut d, 7900., 1900., 150., 900.);
    wall(&mut d, 8050., 2650., 1650., 150.);

    let (win, window) = d.block("Window - 3000 x 300");
    for y in [0., 90., 210., 300.] {
        d.line(win, 0, [0., y], [3000., y]);
    }
    for x in [0., 1000., 2000., 3000.] {
        d.line(win, 0, [x, 0.], [x, 300.]);
    }
    for (x, y, rotation, length) in [
        (1500., 0., 0., 3000.),
        (1500., 11700., 0., 3000.),
        (6500., 11700., 0., 2000.),
        (300., 2500., FRAC_PI_2, 3000.),
        (300., 8500., FRAC_PI_2, 2000.),
        (10000., 2500., FRAC_PI_2, 2000.),
        (10000., 8500., FRAC_PI_2, 2000.),
    ] {
        d.insert(2, window, x, y, rotation, [length / 3000., 1., 1.]);
    }

    let (door, door_id) = d.block("Door - 1000 with swing");
    d.line(door, 0, [0., 0.], [0., 1000.]);
    d.arc(door, 0, 0., 0., 1000., 0., FRAC_PI_2);
    d.insert(2, door_id, 6500., 300., 0., [1., 1., 1.]);
    d.insert(2, door_id, 6250., 3000., FRAC_PI_2, [1., 1., 1.]);
    d.insert(2, door_id, 8050., 1000., FRAC_PI_2, [0.9, -1., 1.]);

    let (stair, stair_id) = d.block("Stair - 14 treads");
    d.rect(stair, 0, 0., 0., 1100., 3500.);
    for i in 1..14 {
        let y = i as f64 * 250.;
        d.line(stair, 0, [0., y], [1100., y]);
    }
    d.line(stair, 0, [550., 250.], [550., 3200.]);
    d.poly(
        stair,
        0,
        vec![[400., 3000.], [550., 3200.], [700., 3000.]],
        false,
    );
    d.insert(3, stair_id, 6650., 3850., 0., [1., 1., 1.]);

    let (sofa, sofa_id) = d.block("Sofa - 2400 x 900");
    d.rect(sofa, 0, 0., 0., 2400., 900.);
    d.rect(sofa, 0, 100., 100., 2200., 650.);
    for x in [833., 1566.] {
        d.line(sofa, 0, [x, 100.], [x, 750.]);
    }
    d.insert(3, sofa_id, 900., 1300., 0., [1., 1., 1.]);
    d.rect(MODEL, 3, 1400., 3000., 1400., 650.);
    d.rect(MODEL, 3, 4500., 1300., 450., 2200.);
    d.rect(MODEL, 3, 1000., 6500., 2200., 1000.);
    let (chair, chair_id) = d.block("Dining chair");
    d.rect(chair, 0, 0., 0., 450., 450.);
    d.line(chair, 0, [0., 350.], [450., 350.]);
    for x in [1100., 1900., 2700.] {
        d.insert(3, chair_id, x, 5950., 0., [1., 1., 1.]);
        d.insert(3, chair_id, x + 450., 8050., PI, [1., 1., 1.]);
    }
    d.poly(
        MODEL,
        3,
        vec![
            [6450., 11050.],
            [9100., 11050.],
            [9100., 8200.],
            [9700., 8200.],
            [9700., 11650.],
            [6450., 11650.],
        ],
        true,
    );
    for y in [8800., 9400., 10000., 10600.] {
        d.line(MODEL, 3, [9100., y], [9700., y]);
    }
    d.rect(MODEL, 3, 7300., 11150., 700., 400.);
    d.circle(MODEL, 3, 7650., 11350., 45.);
    d.rect(MODEL, 3, 9200., 9500., 400., 550.);
    for x in [9300., 9500.] {
        for y in [9630., 9910.] {
            d.circle(MODEL, 3, x, y, 65.);
        }
    }
    d.rect(MODEL, 3, 6400., 8700., 700., 1000.);
    let (wc, wc_id) = d.block("WC");
    d.rect(wc, 0, -220., 0., 440., 200.);
    d.entity(wc,0,json!({"ifccad::geom::placement":super::drawing::plane(0.,460.),"ifccad::geom::ellipse":{"semiMajorRadius":300.,"semiMinorRadius":190.}}));
    d.insert(3, wc_id, 8850., 2400., PI, [1., 1., 1.]);
    d.rect(MODEL, 3, 9200., 700., 400., 450.);
    d.circle(MODEL, 3, 9400., 925., 105.);
    for (x, y, name, detail) in [
        (1500., 4800., "Living room", "living and dining area"),
        (6600., 10200., "Kitchen", "open kitchen"),
        (6550., 2100., "Hall", "entrance"),
        (8350., 1500., "WC", "washroom"),
    ] {
        d.text(MODEL, 4, x, y, 270., name);
        d.text(MODEL, 4, x, y - 330., 160., detail);
    }
    d.text(MODEL, 4, 6800., 7550., 150., "UP");
    horizontal_measure(&mut d, 0., 10000., 0., -1500.);
    horizontal_measure(&mut d, 0., 6100., 0., -800.);
    horizontal_measure(&mut d, 6100., 10000., 0., -800.);
    d.dimension(5, [10000., 0.], [10000., 12000.], [11500., 0.], false, 230.);
    d.text(MODEL, 4, 0., 13100., 470., "HOUSE / GROUND FLOOR");
    d.text(
        MODEL,
        4,
        0.,
        12550.,
        210.,
        "Fictitious design / dimensions in mm / overall size 10000 x 12000",
    );
    d.line(MODEL, 6, [11100., 10200.], [11100., 11600.]);
    d.poly(
        MODEL,
        6,
        vec![[10850., 11100.], [11100., 11600.], [11350., 11100.]],
        false,
    );
    d.text(MODEL, 4, 10980., 11900., 280., "N");

    // Model view height 20000 / Paper frame height 200 gives scale 1:100.
    let paper = d.sheet(
        "A3 - Ground floor - 1 to 100",
        [5200., 5700.],
        20000.,
        370.,
        200.,
    );
    d.rect(paper, 0, 10., 10., 400., 277.);
    d.rect(paper, 0, 10., 10., 400., 35.);
    d.line(paper, 0, [275., 10.], [275., 45.]);
    d.line(paper, 0, [345., 10.], [345., 45.]);
    for (x, y, h, text) in [
        (16., 34., 4.5, "HOUSE / GROUND FLOOR"),
        (
            16.,
            25.,
            2.8,
            "OpenAEC / IFCCAD example / fictitious design",
        ),
        (
            16.,
            17.,
            2.5,
            "All dimensions in millimetres / External walls: 300 / Internal walls: 150",
        ),
        (281., 34., 2.5, "SCALE / SHEET"),
        (281., 24., 4., "1:100 / A3"),
        (281., 16., 2.5, "Units: mm"),
        (351., 34., 2.5, "DRAWING / REVISION"),
        (351., 24., 4., "A-101 / 01"),
        (351., 16., 2.5, "20-10-2026"),
    ] {
        d.text(paper, 0, x, y, h, text);
    }
    for i in 0..5 {
        d.rect(paper, 0, 20. + i as f64 * 10., 54., 10., 3.);
    }
    d.text(paper, 0, 20., 61., 2.5, "0           2           4   5 m");
    d.model_view([6100., 5800.], 20000.);
    d.finish("explorer-house-ground-floor")
}
