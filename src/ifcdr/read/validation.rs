#[cfg(test)]
use super::codes::*;
use super::resource::{IfcdrValidationEvidence, LoadedIfcdrResource};
use crate::diagnostic::PackageDiagnostic;
#[cfg(test)]
use crate::diagnostic::PackageDiagnosticContextValue;
use crate::ifcdr::codec::json::{decode_json, logical_diagnostic};
use crate::ifcdr::logical::validate_resource;
use crate::validated::{EvidenceOutcome, Validated, ValidationOutcome};
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use std::sync::Arc;
pub(crate) fn validate_ifcdr(
    loaded: LoadedIfcdrResource,
) -> ValidationOutcome<LoadedIfcdrResource> {
    Validated::validate(loaded, &())
}
pub(super) fn build_evidence(
    loaded: &LoadedIfcdrResource,
) -> EvidenceOutcome<IfcdrValidationEvidence, PackageDiagnostic> {
    let decoded = match decode_json(loaded.uri(), loaded.source().value()) {
        Ok(v) => v,
        Err(errors) => return EvidenceOutcome::failure(errors),
    };
    let (proof, errors) = validate_resource(decoded).into_parts();
    let diagnostics = errors
        .into_iter()
        .map(|d| logical_diagnostic(loaded.uri(), d))
        .collect();
    match proof {
        Some(data) => EvidenceOutcome::success(IfcdrValidationEvidence { data }, diagnostics),
        None => EvidenceOutcome::failure(diagnostics),
    }
}
#[cfg(test)]
pub(super) fn validate_value(uri: &str, value: Value) -> ValidationOutcome<LoadedIfcdrResource> {
    let source = super::resource::fixture_source();
    let source = Arc::new(source.with_test_value(value));
    validate_ifcdr(LoadedIfcdrResource::new(uri.to_owned(), source))
}

#[cfg(test)]
mod tests {
    use super::super::resource::fixture_source;
    use super::*;

    #[test]
    fn validates_the_minimal_ifcdr_resource() {
        let loaded = LoadedIfcdrResource::new("drawing.ifcdr.json".to_owned(), fixture_source());
        let outcome = validate_ifcdr(loaded);

        assert!(outcome.validated().is_some(), "{:?}", outcome.diagnostics());
        assert!(outcome.diagnostics().is_empty());
    }

    #[test]
    fn envelope_rejects_an_unsupported_version_without_cascading() {
        let mut value = fixture_source().value().clone();
        value["header"]["version"] = serde_json::json!("99.0.0");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert_eq!(outcome.diagnostics().len(), 1);
        let diagnostic = &outcome.diagnostics()[0];
        assert_eq!(diagnostic.code, "IFCCAD_IFCDR_VERSION_UNSUPPORTED");
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some("drawing.ifcdr.json")
        );
        assert_eq!(diagnostic.location.as_deref(), Some("/header/version"));
    }

    #[test]
    fn table_rejects_a_duplicate_local_id() {
        let mut value = fixture_source().value().clone();
        value["layerBindings"][1]["id"] = serde_json::json!(0);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/layerBindings/1/id")
        }));
    }

    #[test]
    fn directory_rejects_a_parent_that_disagrees_with_the_registry() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][3]["parent"] = serde_json::json!("line");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/3/parent")
        }));
    }

    #[test]
    fn directory_rejects_a_non_string_column_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(7));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/0/columns/8")
        }));
    }

    #[test]
    fn directory_rejects_a_duplicate_column_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][0]["columns"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("entityId"));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/0/columns/8")
        }));
    }

    #[test]
    fn directory_rejects_a_non_string_child_name() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"][2]["children"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(7));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_DIRECTORY_INVALID"
                && diagnostic.location.as_deref() == Some("/streamDirectory/streams/2/children/1")
        }));
    }

    #[test]
    fn stream_rejects_a_missing_table_reference() {
        let mut value = fixture_source().value().clone();
        value["streams"]["lineStream"]["layerId"][0] = serde_json::json!(99);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_REFERENCE_MISSING"
                && diagnostic.location.as_deref() == Some("/streams/lineStream/layerId/0")
        }));
    }

    #[test]
    fn stream_rejects_a_pool_range_past_the_column_end() {
        let mut value = fixture_source().value().clone();
        value["streams"]["polylineStream"]["vertexCount"][1] = serde_json::json!(4);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/polylineStream/vertexCount/1")
        }));
    }

    #[test]
    fn stream_rejects_an_external_range_past_its_target_payload() {
        let mut value = fixture_source().value().clone();
        value["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(5);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/entityOrderStream/entryCount/0")
        }));
    }

    #[test]
    fn stream_rejects_a_nonempty_range_with_a_missing_external_target() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
            .retain(|entry| entry["name"] != "entityOrderEntry");
        value["streamDirectory"]["streams"][2]["children"] = serde_json::json!([]);
        value["streams"]
            .as_object_mut()
            .unwrap()
            .remove("entityOrderEntryStream");
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.validated().is_none());
        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/entityOrderStream/entryCount/0")
        }));
    }

    #[test]
    fn table_rejects_a_missing_reference_target() {
        let mut value = fixture_source().value().clone();
        value["appearanceBindings"][0]["overrideId"] = serde_json::json!(99);
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_REFERENCE_MISSING"
                && diagnostic.location.as_deref() == Some("/appearanceBindings/0/overrideId")
        }));
    }

    #[test]
    fn stream_rejects_unsynchronized_pool_column_lengths() {
        let mut value = fixture_source().value().clone();
        value["streams"]["polylineStream"]["y"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(40.0));
        let outcome = validate_value("drawing.ifcdr.json", value);

        assert!(outcome.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                && diagnostic.location.as_deref() == Some("/streams/polylineStream/y")
        }));
    }

    #[test]
    fn unsupported_stream_does_not_create_order_or_orphan_errors() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "hatch", "schema": "example.hatch.v1",
                "role": "object", "count": 1,
                "columns": ["entityId", "scopeId"]
            }));
        value["streams"]["hatchStream"] = serde_json::json!({
            "count": 1, "entityId": [5], "scopeId": [0]
        });
        value["header"]["nextEntityId"] = serde_json::json!(6);
        value["streams"]["entityOrderEntryStream"]["entityId"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!(5));
        value["streams"]["entityOrderEntryStream"]["count"] = serde_json::json!(5);
        value["streams"]["entityOrderStream"]["entryCount"][0] = serde_json::json!(5);
        for entry in value["streamDirectory"]["streams"].as_array_mut().unwrap() {
            if entry["name"] == "entityOrderEntry" {
                entry["count"] = serde_json::json!(5);
            }
        }
        let outcome = validate_value("drawing.ifcdr.json", value);
        assert!(outcome.validated().is_none());
        assert_eq!(
            outcome
                .diagnostics()
                .iter()
                .map(|d| d.code.as_str())
                .collect::<Vec<_>>(),
            ["IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED"]
        );
    }

    #[test]
    fn unsupported_line_schema_identifies_the_attempted_schema() {
        let mut value = fixture_source().value().clone();
        let entry = value["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["name"] == "line")
            .unwrap();
        entry["schema"] = serde_json::json!("example.line.v99");
        let outcome = validate_value("drawing.ifcdr.json", value);
        assert!(outcome.validated().is_none());
        let diagnostic = outcome
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.code == "IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED")
            .unwrap();
        assert_eq!(
            diagnostic.context.get("streamName"),
            Some(&PackageDiagnosticContextValue::String("line".to_owned()))
        );
        assert_eq!(
            diagnostic.context.get("schemaId"),
            Some(&PackageDiagnosticContextValue::String(
                "example.line.v99".to_owned()
            ))
        );
    }

    #[test]
    fn removed_empty_tables_are_not_silently_accepted() {
        for table in [
            "textStyleBindings",
            "dimensionStyleBindings",
            "hatchPatternBindings",
            "namedUcsBindings",
            "dimensionOverrideTable",
            "characterFormatTable",
            "paragraphFormatTable",
        ] {
            let mut value = fixture_source().value().clone();
            value[table] = serde_json::json!([]);
            let outcome = validate_value("drawing.ifcdr.json", value);
            assert!(outcome.validated().is_none(), "{table}");
            let location = format!("/{table}");
            assert!(
                outcome.diagnostics().iter().any(|diagnostic| {
                    diagnostic.code == "IFCCAD_IFCDR_STRUCTURE_INVALID"
                        && diagnostic.location.as_deref() == Some(location.as_str())
                }),
                "{table}: {:?}",
                outcome.diagnostics()
            );
        }
    }

    #[test]
    fn orphan_payload_is_rejected_when_all_streams_are_supported() {
        let mut value = fixture_source().value().clone();
        value["streams"]["textRuns"] = serde_json::json!([]);
        let outcome = validate_value("drawing.ifcdr.json", value);
        assert!(outcome.validated().is_none());
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| d.code == IFCCAD_IFCDR_DIRECTORY_INVALID
                && d.location.as_deref() == Some("/streams/textRuns")));
    }

    #[test]
    fn unsupported_stream_does_not_hide_independent_column_errors() {
        let mut value = fixture_source().value().clone();
        value["streamDirectory"]["streams"].as_array_mut().unwrap().push(serde_json::json!({
            "name":"hatch", "schema":"example.hatch.v1", "role":"object", "count":0, "columns":[]
        }));
        value["streams"]["lineStream"]["x1"][0] = serde_json::json!("bad coordinate");
        let outcome = validate_value("drawing.ifcdr.json", value);
        assert!(outcome.validated().is_none());
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| d.code == IFCCAD_IFCDR_STREAM_SCHEMA_UNSUPPORTED));
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| d.code == IFCCAD_IFCDR_STRUCTURE_INVALID
                && d.location.as_deref() == Some("/streams/lineStream/x1/0")));
    }

    #[test]
    fn omitted_visibility_has_the_same_typed_meaning_as_explicit_true() {
        use crate::ifcdr::{IfcdrEntityRef, ScopeId};
        let mut explicit = fixture_source().value().clone();
        for key in ["lineStream", "polylineStream"] {
            let count = explicit["streams"][key]["count"].as_u64().unwrap() as usize;
            explicit["streams"][key]["visible"] = serde_json::json!(vec![true; count]);
        }
        for entry in explicit["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
        {
            if entry["name"] == "line" || entry["name"] == "polyline" {
                let columns = entry["columns"].as_array_mut().unwrap();
                if !columns.iter().any(|column| column == "visible") {
                    columns.push(serde_json::json!("visible"));
                }
            }
        }
        let mut omitted = explicit.clone();
        for key in ["lineStream", "polylineStream"] {
            omitted["streams"][key]
                .as_object_mut()
                .unwrap()
                .remove("visible");
        }
        for entry in omitted["streamDirectory"]["streams"]
            .as_array_mut()
            .unwrap()
        {
            if entry["name"] == "line" || entry["name"] == "polyline" {
                entry["columns"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|column| column != "visible");
            }
        }
        let project = |value| {
            let outcome = validate_value("drawing.ifcdr.json", value);
            assert!(
                outcome.diagnostics().is_empty(),
                "{:?}",
                outcome.diagnostics()
            );
            outcome
                .validated()
                .unwrap()
                .entities()
                .in_scope(ScopeId::new(0))
                .unwrap()
                .map(|entity| match entity {
                    IfcdrEntityRef::Line(line) => (
                        line.entity_id(),
                        line.visible(),
                        false,
                        vec![line.start(), line.end()],
                    ),
                    IfcdrEntityRef::Polyline(polyline) => (
                        polyline.entity_id(),
                        polyline.visible(),
                        polyline.closed(),
                        polyline.points().collect(),
                    ),
                })
                .collect::<Vec<_>>()
        };
        let expected = project(explicit);
        assert!(expected.iter().all(|entity| entity.1));
        assert_eq!(project(omitted), expected);
    }
}
