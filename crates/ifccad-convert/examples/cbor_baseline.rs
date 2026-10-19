//! Manual six-case IFCCAD baseline; cargo test never runs timed samples.
#[path = "cbor_baseline/inventory.rs"]
mod inventory;
#[path = "cbor_baseline/measure.rs"]
mod measure;
#[path = "cbor_baseline/recipes.rs"]
mod recipes;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    measure::run(std::env::args().skip(1).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocdraw::ifccad::*;
    #[test]
    fn batch_exclusion_requires_production_rejection_of_a_degenerate_bulge() {
        use opencadcodec::{CadDocument, EntityType, LwPolyline, Vector2};
        let mut cad = CadDocument::new();
        let mut invalid = LwPolyline::from_points(vec![Vector2::new(1., 2.), Vector2::new(1., 2.)]);
        invalid.vertices[0].bulge = 1.;
        let bad = cad.add_entity(EntityType::LwPolyline(invalid)).unwrap();
        let good = cad
            .add_entity(EntityType::LwPolyline(LwPolyline::from_points(vec![
                Vector2::new(0., 0.),
                Vector2::new(1., 0.),
            ])))
            .unwrap();
        let original = cad.clone();
        let records = measure::exclude_invalid_polylines(&mut cad).unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0]["failure"]
            .as_str()
            .unwrap()
            .contains("invalid CAD primitive geometry"));
        assert!(cad.get_entity(bad).is_none());
        assert!(cad.get_entity(good).is_some());
        assert!(original.get_entity(bad).is_some());
        let mut replay = original;
        measure::apply_exclusions(&mut replay, &records).unwrap();
        assert_eq!(inventory::cad(&cad), inventory::cad(&replay));
        assert!(measure::apply_exclusions(&mut replay, &records).is_err());
    }
    #[test]
    fn dimension_exclusion_matches_only_the_authorized_geometry_failure() {
        use cad_geometry_convert::GeometryFailure;
        use ifccad_convert::*;
        use opencadcodec::Handle;
        let mut failure = IfccadGeometryFailure {
            domain: IfccadGeometryDomain::Drawing,
            coordinate_meaning: IfccadGeometryCoordinateMeaning::DrawingUnit(
                ocdraw::geometry_kernel::CoordinateLengthUnit::Millimetre,
            ),
            failure: Box::new(GeometryFailure {
                source: IfccadGeometryEntitySource::CadEntity {
                    handle: Handle::new(979895),
                    kind: "DIMENSION".into(),
                },
                vertex_index: Some(2),
                stage: IfccadGeometryStage::DeviationAssessment,
                requested_tolerance: IfccadGeometryTolerance::default(),
                resolved_tolerance: Some(IfccadDistanceInterval {
                    lower: 9.999999999999999e-10,
                    upper: 1e-9,
                }),
                deviation: Some(IfccadDistanceInterval {
                    lower: 543.9999999999972,
                    upper: 543.9999999999972,
                }),
                reason: IfccadGeometryFailureReason::ProvenExceedance,
            }),
        };
        assert_eq!(
            measure::dimension_failure_handle(&failure),
            Some(Handle::new(979895))
        );
        failure.failure.reason = IfccadGeometryFailureReason::NumericalProofIncomplete;
        assert_eq!(measure::dimension_failure_handle(&failure), None);
        failure.failure.reason = IfccadGeometryFailureReason::ProvenExceedance;
        failure.failure.vertex_index = Some(0);
        assert_eq!(measure::dimension_failure_handle(&failure), None);
        failure.failure.vertex_index = Some(2);
        failure.failure.stage = IfccadGeometryStage::TargetConstruction;
        assert_eq!(measure::dimension_failure_handle(&failure), None);
        failure.failure.stage = IfccadGeometryStage::DeviationAssessment;
        failure.failure.source = IfccadGeometryEntitySource::CadEntity {
            handle: Handle::new(979895),
            kind: "HATCH".into(),
        };
        assert_eq!(measure::dimension_failure_handle(&failure), None);
        assert_eq!(
            measure::entity_failure_handle(&IfccadConversionError::Geometry(Box::new(failure))),
            Some(Handle::new(979895))
        );
    }
    #[test]
    fn broad_exclusion_requires_an_unambiguous_source_entity() {
        use ifccad_convert::IfccadConversionError as E;
        use opencadcodec::Handle;
        assert_eq!(
            measure::entity_failure_handle(&E::InvalidStructure(
                "entity/0xF1439: invalid geometry".into()
            )),
            Some(Handle::new(988217))
        );
        for location in [
            "layout/0xF1439: invalid",
            "entity/988217: ambiguous decimal ID",
            "entity/0xF1439junk: invalid",
            "unresolved entity 0xF1439",
        ] {
            assert_eq!(
                measure::entity_failure_handle(&E::InvalidStructure(location.into())),
                None
            );
        }
        assert_eq!(
            measure::entity_failure_handle(&E::WorkspaceNumeric(
                cad_workspace_convert::WorkspaceNumericError {
                    field: "normal",
                    message: "0xF1439".into()
                }
            )),
            None
        );
    }
    #[test]
    fn explicit_exclusion_removes_only_selected_entity_and_owner_membership() {
        use opencadcodec::{CadDocument, EntityType, Line};
        let mut cad = CadDocument::new();
        let selected = cad.add_entity(EntityType::Line(Line::new())).unwrap();
        let kept = cad.add_entity(EntityType::Line(Line::new())).unwrap();
        let original = cad.clone();
        let record = measure::exclude_entity(&mut cad, selected).unwrap();
        assert_eq!(record["handle"], format!("0x{selected:X}"));
        assert!(cad.get_entity(selected).is_none());
        assert!(cad.get_entity(kept).is_some());
        assert!(original.get_entity(selected).is_some());
        assert!(cad
            .block_records
            .iter()
            .all(|b| !b.entity_handles.contains(&selected)));
        assert!(measure::exclude_entity(&mut cad, selected).is_err());
    }
    #[test]
    fn compact_json_changes_whitespace_only() {
        let input =
            br#" { "id":18446744073709551615, "z": -0.0, "s": "a \\\" b", "a": [ 1, 2 ] } "#;
        let output = measure::compact_json(input).unwrap();
        assert_eq!(
            output,
            br#"{"id":18446744073709551615,"z":-0.0,"s":"a \\\" b","a":[1,2]}"#
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(input).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&output).unwrap()
        );
        assert!(measure::compact_json(b"\"unterminated").is_err());
    }
    #[test]
    fn tiny_recipes_pass_both_production_routes_with_exact_content() {
        for family in ["lines", "polylines", "text", "hatches"] {
            let d = recipes::drawing(family, 6).unwrap_or_else(|e| panic!("{family}: {e}"));
            assert_eq!(d.model.entities.len(), 6);
            validate_ifccad_document(&d).unwrap();
            let frozen = measure::freeze(&d).unwrap();
            let variants = measure::artifacts(&frozen).unwrap();
            measure::qualify(&frozen, &variants).unwrap();
            assert_eq!(
                variants,
                measure::artifacts(
                    &measure::freeze(&recipes::drawing(family, 6).unwrap()).unwrap()
                )
                .unwrap()
            );
        }
    }
    #[test]
    fn selection_rejects_unknown_duplicate_and_empty_cases() {
        assert!(measure::selection("").is_err());
        assert!(measure::selection("lines-10000,lines-10000").is_err());
        assert!(measure::selection("unknown").is_err());
        assert_eq!(
            measure::selection("text-1000,lines-10000").unwrap(),
            ["text-1000", "lines-10000"]
        );
    }
    #[test]
    fn statistics_use_raw_samples_without_running_a_clock() {
        let stats = measure::statistics(&[5., 1., 3., 2., 4.]);
        assert_eq!(stats["medianMs"], 3.);
        assert_eq!(stats["minMs"], 1.);
        assert_eq!(stats["maxMs"], 5.);
    }
    #[test]
    fn selected_cases_match_recorded_corpus_and_existing_outputs_are_safe() {
        let corpus: serde_json::Value = serde_json::from_str(include_str!(
            "../../../benchmarks/ifccad-cbor/corpus-v1.json"
        ))
        .unwrap();
        let ids = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, measure::CASES);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cbor-baseline-safety-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("keep"), b"original").unwrap();
        let result = measure::run(vec![
            "--out".into(),
            dir.to_str().unwrap().into(),
            "--cases".into(),
            "lines-10000".into(),
        ]);
        assert!(result.is_err());
        assert_eq!(std::fs::read(dir.join("keep")).unwrap(), b"original");
        std::fs::remove_file(dir.join("keep")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
