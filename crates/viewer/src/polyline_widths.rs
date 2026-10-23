//! Native authored width inspection and target qualification, without reconversion.
use ifccad::*;
use ifccad_convert::*;
use serde_json::{json, Value};
pub(crate) fn target(
    format: &str,
    version: opencadcodec::DxfVersion,
) -> IfccadPolylineWidthCadTarget {
    IfccadPolylineWidthCadTarget {
        format: if format == "dwg" {
            IfccadPolylineWidthCadFormat::Dwg
        } else {
            IfccadPolylineWidthCadFormat::AsciiDxf
        },
        version,
    }
}
pub(crate) fn restrictions(
    d: &IfccadDocument,
    format: &str,
    version: opencadcodec::DxfVersion,
) -> Vec<Value> {
    match assess_ifccad_polyline_width_target(d, target(format, version), IfccadLossPolicy::Allow) {
        Ok(issues) => issues
            .iter()
            .map(|i| {
                json!({
                    "code":i.code,"location":i.location,"message":i.message,
                    "action":format!("{:?}",i.action),"phase":"cadExchange",
                    "feature":"polylineWidths","policyEffect":"loss"
                })
            })
            .collect(),
        Err(error) => vec![json!({
            "code":"POLYLINE_WIDTH_PREFLIGHT_FAILED","message":error.to_string(),
            "policyEffect":"block"
        })],
    }
}
pub(crate) fn preflight(d: &IfccadDocument) -> Value {
    let entries = ["dxf", "dwg"]
        .into_iter()
        .flat_map(|format| {
            crate::cad::VERSIONS.into_iter().map(move |version| {
                json!({
                    "format":format,"version":version.as_str(),
                    "restrictions":restrictions(d,format,version)
                })
            })
        })
        .collect::<Vec<_>>();
    json!(entries)
}
pub(crate) fn authored(graph: &Value) -> Value {
    let entries = graph["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| {
            let body = node["attributes"].get("ifccad::geom::planarPolyline")?;
            let widths: IfccadPlanarPolylineWidths = body
                .get("widths")
                .map(|value| serde_json::from_value(value.clone()).expect("validated widths"))
                .unwrap_or_default();
            let count = body["vertices"]
                .as_array()
                .expect("validated vertices")
                .len();
            Some(json!({
                "path":node["path"],"widths":widths,
                "effectiveWidths":widths.effective_widths(count).expect("validated widths"),
                "displayEvaluation":"unassessed"
            }))
        })
        .collect::<Vec<_>>();
    json!(entries)
}
pub(crate) fn losses(
    expected: &opencadcodec::CadDocument,
    actual: &opencadcodec::CadDocument,
) -> Vec<Value> {
    fn signatures(
        d: &opencadcodec::CadDocument,
    ) -> std::collections::BTreeMap<String, Vec<String>> {
        // Width controls follow semantic owner/draw order. Geometry rounding has
        // independent numerical evidence and must not be relabeled as width loss.
        d.block_records.iter().filter_map(|block| {
            let controls=block.entity_handles.iter().filter_map(|handle|match d.get_entity(*handle)? {
                opencadcodec::EntityType::Polyline2D(p)=>Some(json!({
                    "kind":"classic","defaults":[p.start_width,p.end_width],
                    "vertices":p.vertices.iter().map(|v|json!([v.start_width,v.end_width])).collect::<Vec<_>>()
                })),
                opencadcodec::EntityType::LwPolyline(p) if p.constant_width.to_bits()!=0 || p.vertices.iter().any(|v|v.start_width.to_bits()!=0 || v.end_width.to_bits()!=0)=>Some(json!({
                    "kind":"lw","constant":p.constant_width,
                    "vertices":p.vertices.iter().map(|v|json!([v.start_width,v.end_width])).collect::<Vec<_>>()
                })),
                _=>None,
            }).map(|control|control.to_string()).collect::<Vec<_>>();
            (!controls.is_empty()).then(||(block.name.clone(),controls))
        }).collect()
    }
    let expected = signatures(expected);
    let actual = signatures(actual);
    expected.keys().chain(actual.keys()).collect::<std::collections::BTreeSet<_>>().into_iter()
        .filter(|owner|expected.get(*owner)!=actual.get(*owner))
        .map(|owner|json!({
            "code":"POLYLINE_WIDTH_READBACK_LOSS","location":format!("cad/block/{owner}/polylineWidths"),
            "phase":"cadExchange","action":"Modified",
            "message":"Stored polyline width controls differ after physical readback in owner draw order"
        })).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencadcodec::{CadDocument, EntityType, LwPolyline, Vector2};
    #[test]
    fn readback_width_checks_do_not_relabel_coordinate_rounding() {
        let mut d = CadDocument::new();
        let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(10., 0.)]);
        p.constant_width = 0.2;
        let h = d.add_entity(EntityType::LwPolyline(p)).unwrap();
        let mut back = d.clone();
        let EntityType::LwPolyline(p) = back.get_entity_mut(h).unwrap() else {
            panic!()
        };
        p.vertices[1].location.x = 10.0f64.next_up();
        assert!(losses(&d, &back).is_empty());
    }
    #[test]
    fn readback_width_checks_keep_assignments_within_owner_order() {
        let mut d = CadDocument::new();
        let mut handles = vec![];
        for width in [0.2, 0.4] {
            let mut p = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(10., 0.)]);
            p.constant_width = width;
            handles.push(d.add_entity(EntityType::LwPolyline(p)).unwrap());
        }
        let mut back = d.clone();
        for (h, width) in handles.iter().zip([0.4, 0.2]) {
            let EntityType::LwPolyline(p) = back.get_entity_mut(*h).unwrap() else {
                panic!()
            };
            p.constant_width = width;
        }
        assert!(!losses(&d, &back).is_empty());
    }
}

#[cfg(test)]
mod signed_tests {
    use super::*;
    use opencadcodec::{
        entities::{Polyline2D, Vertex2D},
        CadDocument, EntityType, Vector3,
    };
    #[test]
    fn classic_explicit_zero_sign_is_part_of_readback_state() {
        let mut d = CadDocument::new();
        let mut p = Polyline2D::new();
        p.vertices = vec![Vertex2D::new(Vector3::ZERO), Vertex2D::new(Vector3::UNIT_X)];
        p.vertices[0].start_width = Some(-0.0);
        let h = d.add_entity(EntityType::Polyline2D(p)).unwrap();
        let mut back = d.clone();
        let EntityType::Polyline2D(p) = back.get_entity_mut(h).unwrap() else {
            panic!()
        };
        p.vertices[0].start_width = Some(0.0);
        assert!(!losses(&d, &back).is_empty());
    }
}
