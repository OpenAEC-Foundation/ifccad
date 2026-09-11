use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub family: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Drawing {
    pub unit: &'static str,
    pub layers: Vec<Layer>,
    pub entities: Vec<Entity>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub appearance: Appearance,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Entity {
    pub geometry: Geometry,
    pub layer: String,
    pub visible: bool,
    pub appearance: Appearance,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Geometry {
    Line { start: [f64; 2], end: [f64; 2] },
    Polyline { points: Vec<[f64; 2]>, closed: bool },
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Appearance {
    pub color: Property<[u8; 3]>,
    pub opacity: Property<f64>,
    pub pattern: Property<String>,
    pub weight: Property<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "mode", content = "value", rename_all = "camelCase")]
pub enum Property<T> {
    ByLayer,
    ByBlock,
    Explicit(T),
}

impl Appearance {
    pub fn explicit(alternate: bool) -> Self {
        Self {
            color: Property::Explicit(if alternate {
                [180, 40, 60]
            } else {
                [10, 20, 30]
            }),
            opacity: Property::Explicit(1.0),
            pattern: Property::Explicit("continuous".into()),
            weight: Property::Explicit(if alternate { 0.5 } else { 0.25 }),
        }
    }
    pub fn by_layer() -> Self {
        Self {
            color: Property::ByLayer,
            opacity: Property::ByLayer,
            pattern: Property::ByLayer,
            weight: Property::ByLayer,
        }
    }
}

pub fn generate(case: &Case) -> Result<Drawing, String> {
    if !matches!(
        case.family.as_str(),
        "line" | "short" | "long" | "mixed" | "fractional"
    ) {
        return Err("unknown recipe family".into());
    }
    // Keep arithmetic and generated coordinates within the frozen corpus bounds.
    if case.count > 10_000 {
        return Err("corpus v1 count exceeds 10000".into());
    }
    let mixed = case.family == "mixed";
    let layers: Vec<_> = (0..if mixed { 4 } else { 1 })
        .map(|i| Layer {
            name: if i == 0 {
                "0".into()
            } else {
                format!("Layer-{i}")
            },
            visible: true,
            appearance: Appearance::explicit(false),
        })
        .collect();
    let entities = (0..case.count)
        .map(|i| {
            let x = ((i % 100) * 16) as f64;
            let y = ((i / 100) * 16) as f64;
            let polyline =
                matches!(case.family.as_str(), "short" | "long") || (mixed && i % 2 == 1);
            let geometry = if polyline {
                let points = if case.family == "long" {
                    (0..128)
                        .map(|j| [x + f64::from(j), y + f64::from(j % 7)])
                        .collect()
                } else {
                    vec![[x, y], [x + 8., y], [x + 8., y + 4.], [x, y + 4.]]
                };
                Geometry::Polyline {
                    points,
                    closed: mixed && i % 4 == 1,
                }
            } else {
                let start = if case.family == "fractional" {
                    [
                        x + ((i * 37) % 1024) as f64 / 1024.,
                        y + ((i * 73) % 1024) as f64 / 1024.,
                    ]
                } else {
                    [x, y]
                };
                let end = if case.family == "fractional" {
                    [start[0] + 8.125, start[1] + 4.0625]
                } else {
                    [x + 8., y + 4.]
                };
                Geometry::Line { start, end }
            };
            Entity {
                geometry,
                layer: layers[i % layers.len()].name.clone(),
                visible: !mixed || i % 5 != 0,
                appearance: if mixed && i % 3 == 0 {
                    Appearance::explicit(true)
                } else {
                    Appearance::by_layer()
                },
            }
        })
        .collect();
    Ok(Drawing {
        unit: "millimetre",
        layers,
        entities,
    })
}

impl Drawing {
    pub fn counts(&self) -> (u64, u64, u64) {
        self.entities
            .iter()
            .fold((0, 0, 0), |(lines, polylines, vertices), e| {
                match &e.geometry {
                    Geometry::Line { .. } => (lines + 1, polylines, vertices),
                    Geometry::Polyline { points, .. } => {
                        (lines, polylines + 1, vertices + points.len() as u64)
                    }
                }
            })
    }
}
