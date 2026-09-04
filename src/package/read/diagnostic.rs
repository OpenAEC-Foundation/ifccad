#[cfg(test)]
use crate::diagnostic::PackageDiagnosticContextValue;
use crate::diagnostic::{PackageDiagnostic, PackageDiagnosticSeverity};
use crate::ResourceId;
use serde::Serialize;

/// Deterministically ordered result of IFCCAD package validation.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageValidationReport {
    /// Diagnostics ordered by resource, location, severity, code, and context.
    diagnostics: Vec<PackageDiagnostic>,
}

impl PackageValidationReport {
    pub(crate) fn from_diagnostics(mut diagnostics: Vec<PackageDiagnostic>) -> Self {
        diagnostics
            .sort_by(|left, right| diagnostic_sort_key(left).cmp(&diagnostic_sort_key(right)));
        Self { diagnostics }
    }

    /// Returns `true` when the report contains no error diagnostics.
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|item| item.severity == PackageDiagnosticSeverity::Error)
    }

    /// Returns the deterministically ordered diagnostics.
    pub fn diagnostics(&self) -> &[PackageDiagnostic] {
        &self.diagnostics
    }

    /// Returns an iterator over the deterministically ordered diagnostics.
    pub fn iter(&self) -> std::slice::Iter<'_, PackageDiagnostic> {
        self.diagnostics.iter()
    }

    /// Returns the number of diagnostics in the report.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` when the report contains no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Consumes the report and returns its deterministically ordered diagnostics.
    pub fn into_diagnostics(self) -> Vec<PackageDiagnostic> {
        self.diagnostics
    }
}

fn diagnostic_sort_key(diagnostic: &PackageDiagnostic) -> (&str, &str, &str, u8, &str, String) {
    (
        diagnostic
            .resource_id
            .as_ref()
            .map(ResourceId::as_str)
            .unwrap_or(""),
        diagnostic.resource_uri.as_deref().unwrap_or(""),
        diagnostic.location.as_deref().unwrap_or(""),
        severity_rank(diagnostic.severity),
        &diagnostic.code,
        serde_json::to_string(&diagnostic.context)
            .expect("diagnostic scalar context always serializes"),
    )
}

fn severity_rank(severity: PackageDiagnosticSeverity) -> u8 {
    match severity {
        PackageDiagnosticSeverity::Error => 0,
        PackageDiagnosticSeverity::Warning => 1,
        PackageDiagnosticSeverity::Info => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn diagnostic(
        code: &str,
        severity: PackageDiagnosticSeverity,
        resource_uri: Option<&str>,
        location: Option<&str>,
    ) -> PackageDiagnostic {
        PackageDiagnostic {
            code: code.to_owned(),
            severity,
            resource_id: None,
            resource_uri: resource_uri.map(str::to_owned),
            location: location.map(str::to_owned),
            context: BTreeMap::new(),
            message: String::new(),
        }
    }

    fn diagnostic_with_context(
        code: &str,
        context: BTreeMap<String, PackageDiagnosticContextValue>,
    ) -> PackageDiagnostic {
        PackageDiagnostic {
            code: code.to_owned(),
            severity: PackageDiagnosticSeverity::Error,
            resource_id: None,
            resource_uri: Some("resource.json".to_owned()),
            location: Some("/value".to_owned()),
            context,
            message: String::new(),
        }
    }

    #[test]
    fn report_is_valid_without_error_diagnostics() {
        let report = PackageValidationReport::from_diagnostics(vec![diagnostic(
            "TEST_WARNING",
            PackageDiagnosticSeverity::Warning,
            None,
            None,
        )]);

        assert!(report.is_valid());
    }

    #[test]
    fn report_is_invalid_with_an_error_diagnostic() {
        let report = PackageValidationReport::from_diagnostics(vec![diagnostic(
            "TEST_ERROR",
            PackageDiagnosticSeverity::Error,
            None,
            None,
        )]);

        assert!(!report.is_valid());
    }

    #[test]
    fn report_exposes_read_only_collection_access() {
        let report = PackageValidationReport::from_diagnostics(vec![diagnostic(
            "TEST_WARNING",
            PackageDiagnosticSeverity::Warning,
            None,
            None,
        )]);

        assert_eq!(report.len(), 1);
        assert!(!report.is_empty());
        assert_eq!(report.diagnostics()[0].code, "TEST_WARNING");
        assert_eq!(
            report.iter().next().map(|item| item.code.as_str()),
            Some("TEST_WARNING")
        );
    }

    #[test]
    fn report_can_be_consumed_into_diagnostics() {
        let report = PackageValidationReport::from_diagnostics(vec![diagnostic(
            "TEST_WARNING",
            PackageDiagnosticSeverity::Warning,
            None,
            None,
        )]);

        let diagnostics = report.into_diagnostics();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "TEST_WARNING");
    }

    #[test]
    fn diagnostics_are_sorted_by_the_language_neutral_contract() {
        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic(
                "Z",
                PackageDiagnosticSeverity::Info,
                Some("z.json"),
                Some("/b"),
            ),
            diagnostic(
                "B",
                PackageDiagnosticSeverity::Warning,
                Some("a.json"),
                Some("/a"),
            ),
            diagnostic(
                "A",
                PackageDiagnosticSeverity::Error,
                Some("a.json"),
                Some("/a"),
            ),
        ]);

        let codes: Vec<_> = report.iter().map(|item| item.code.as_str()).collect();
        assert_eq!(codes, ["A", "B", "Z"]);
    }

    #[test]
    fn resource_uri_is_the_first_diagnostic_sort_key() {
        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic(
                "A",
                PackageDiagnosticSeverity::Error,
                Some("z.json"),
                Some("/a"),
            ),
            diagnostic(
                "Z",
                PackageDiagnosticSeverity::Error,
                Some("a.json"),
                Some("/z"),
            ),
        ]);

        assert_eq!(report.diagnostics()[0].code, "Z");
    }

    #[test]
    fn resource_id_orders_diagnostics_before_equal_source_locations() {
        let diagnostics = vec![
            PackageDiagnostic {
                code: "TEST".to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(crate::ResourceId::new("geometry-b").unwrap()),
                resource_uri: Some("drawing.ifcdr.json".to_owned()),
                location: None,
                context: BTreeMap::new(),
                message: String::new(),
            },
            PackageDiagnostic {
                code: "TEST".to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_id: Some(crate::ResourceId::new("geometry-a").unwrap()),
                resource_uri: Some("drawing.ifcdr.json".to_owned()),
                location: None,
                context: BTreeMap::new(),
                message: String::new(),
            },
        ];

        let report = PackageValidationReport::from_diagnostics(diagnostics);
        assert_eq!(
            report.diagnostics()[0]
                .resource_id
                .as_ref()
                .unwrap()
                .as_str(),
            "geometry-a"
        );
    }

    #[test]
    fn location_precedes_severity_and_code_in_diagnostic_sorting() {
        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic(
                "A",
                PackageDiagnosticSeverity::Error,
                Some("same.json"),
                Some("/z"),
            ),
            diagnostic(
                "Z",
                PackageDiagnosticSeverity::Warning,
                Some("same.json"),
                Some("/a"),
            ),
        ]);

        assert_eq!(report.diagnostics()[0].code, "Z");
    }

    #[test]
    fn severity_precedes_code_in_diagnostic_sorting() {
        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic(
                "A",
                PackageDiagnosticSeverity::Warning,
                Some("same.json"),
                Some("/same"),
            ),
            diagnostic(
                "Z",
                PackageDiagnosticSeverity::Error,
                Some("same.json"),
                Some("/same"),
            ),
        ]);

        assert_eq!(report.diagnostics()[0].code, "Z");
    }

    #[test]
    fn code_orders_otherwise_equal_diagnostics() {
        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic(
                "Z",
                PackageDiagnosticSeverity::Error,
                Some("same.json"),
                Some("/same"),
            ),
            diagnostic(
                "A",
                PackageDiagnosticSeverity::Error,
                Some("same.json"),
                Some("/same"),
            ),
        ]);

        assert_eq!(report.diagnostics()[0].code, "A");
    }

    #[test]
    fn context_breaks_otherwise_equal_sort_keys_canonically() {
        let mut later_context = BTreeMap::new();
        later_context.insert(
            "z".to_owned(),
            PackageDiagnosticContextValue::Number(0.into()),
        );
        later_context.insert(
            "a".to_owned(),
            PackageDiagnosticContextValue::String("zeta".to_owned()),
        );
        let mut earlier_context = BTreeMap::new();
        earlier_context.insert(
            "a".to_owned(),
            PackageDiagnosticContextValue::String("alpha".to_owned()),
        );
        earlier_context.insert(
            "z".to_owned(),
            PackageDiagnosticContextValue::Number(0.into()),
        );

        let report = PackageValidationReport::from_diagnostics(vec![
            diagnostic_with_context("SAME", later_context),
            diagnostic_with_context("SAME", earlier_context.clone()),
        ]);

        assert_eq!(report.diagnostics()[0].context, earlier_context);
    }
}
