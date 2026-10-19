//! Opt-in mapping of the reviewed IFCX USD matrix attribute to native frames.

use super::*;
use crate::geometry_kernel::numeric::exact;
use serde::Deserialize;
use serde_json::{json, Value};

const USD_URI: &str = "https://ifcx.dev/@openusd.org/usd@v1.ifcx";
const XFORMOP: &str = "usd::xformop";
const PLACEMENT: &str = "ifccad::geom::placement";
const PLACED_GEOMETRY: &[&str] = &[
    "ifccad::geom::point",
    "ifccad::geom::circle",
    "ifccad::geom::arc",
    "ifccad::geom::ellipse",
    "ifccad::geom::ellipseArc",
    "ifccad::geom::planarPolyline",
];
const CAD_ROLES: &[&str] = &[
    "ifccad::drawing",
    "ifccad::entity",
    "ifccad::opaqueEntity",
    "ifccad::layout",
    "ifccad::blockDefinition",
    "ifccad::layer",
    "ifccad::linePattern",
    "ifccad::preservation",
    "ifccad::preservationRecord",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatrixPlacement {
    transform: [[f64; 4]; 4],
}

fn problem(path: &str, message: impl std::fmt::Display) -> IfccadReport {
    IfccadReport::one(format!("{path}: experimental xformop: {message}"))
}

fn schema_definition() -> Value {
    // Reviewed upstream blob 7a3301c9dce41e9cc87993a3fef1040c00015915.
    // This checks a conflicting inline declaration; it is not a remote resolver.
    json!({"value": {"dataType": "Object", "objectRestrictions": {"values": {
        "transform": {"dataType": "Array", "arrayRestrictions": {"value": {
            "dataType": "Array", "arrayRestrictions": {"value": {"dataType": "Real"}}
        }}}
    }}}})
}

fn decode(value: &Value, path: &str) -> Result<IfccadPlacement, IfccadReport> {
    let MatrixPlacement { transform: m } = serde_json::from_value(value.clone())
        .map_err(|error| problem(path, format!("invalid matrix value: {error}")))?;
    if !m.iter().flatten().all(|v| v.is_finite()) {
        return Err(problem(path, "matrix components must be finite"));
    }
    if [m[0][3], m[1][3], m[2][3], m[3][3]] != [0., 0., 0., 1.] {
        return Err(problem(path, "expected an affine row-vector matrix"));
    }
    let placement = IfccadPlacement {
        origin: [m[3][0], m[3][1], m[3][2]],
        x_axis: [m[0][0], m[0][1], m[0][2]],
        y_axis: [m[1][0], m[1][1], m[1][2]],
    };
    // Frame meaning uses the same native validation predicate, without repairs.
    placement
        .coordinate_frame()
        .map_err(|error| problem(path, error))?;
    // A native frame derives Z exactly. Do not silently discard an independently
    // authored matrix Z or treat rounded cross-product equality as exact.
    for i in 0..3 {
        let (j, k) = ((i + 1) % 3, (i + 2) % 3);
        let cross = exact(m[0][j]) * exact(m[1][k]) - exact(m[0][k]) * exact(m[1][j]);
        if cross != exact(m[2][i]) {
            return Err(problem(path, "Z row is not the exact native X cross Y; matrix cannot be represented without reparameterization"));
        }
    }
    Ok(placement)
}

pub(crate) fn project_xformop(raw: &Value) -> Result<IfccadDocument, IfccadReport> {
    let imported = raw["imports"].as_array().is_some_and(|imports| {
        imports
            .iter()
            .any(|item| item["uri"].as_str() == Some(USD_URI))
    });
    let inline = raw["schemas"].get(XFORMOP);
    let mut normalized = raw.clone();
    let nodes = normalized["data"]
        .as_array_mut()
        .ok_or_else(|| problem("IFCX", "data must be an array"))?;
    for node in nodes {
        let path = node["path"].as_str().unwrap_or("<missing path>").to_owned();
        let Some(attrs) = node.get_mut("attributes").and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(matrix) = attrs.get(XFORMOP) else {
            continue;
        };
        // Foreign nodes retain their ordinary IFCX semantics and source context.
        if !CAD_ROLES.iter().any(|key| attrs.contains_key(*key)) {
            continue;
        }
        if !imported {
            return Err(problem(&path, "missing reviewed USD schema import"));
        }
        if inline.is_some_and(|definition| definition != &schema_definition()) {
            return Err(problem(&path, "conflicting inline USD xformop schema"));
        }
        if attrs.contains_key(PLACEMENT) {
            return Err(problem(
                &path,
                "native and matrix placements are both present",
            ));
        }
        if !attrs.contains_key("ifccad::entity")
            || !PLACED_GEOMETRY.iter().any(|key| attrs.contains_key(*key))
        {
            return Err(problem(&path, "matrix placement is unsupported for this CAD role; blocks, direct XYZ geometry and workspace state remain native"));
        }
        let placement = decode(matrix, &path)?;
        attrs.insert(PLACEMENT.into(), json!(placement));
    }
    // Ownership, CAD semantics, references, bounds and other field rules stay
    // with the common native projection and logical validator.
    super::project(&normalized)
}
