use super::analysis::ValidatedPackage;
use super::codes::IFCCAD_PACKAGE_TIMESTAMP_INVALID;
use super::{PackageDiagnostic, PackageDiagnosticSeverity, DIRECTORY_PACKAGE_ENTRYPOINT};
use crate::PackageId;
use serde_json::Value;
use std::collections::BTreeMap;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug)]
pub(crate) struct ValidatedPackageHeader {
    pub(crate) package_id: PackageId,
    pub(crate) ifcx_version: String,
    pub(crate) data_version: String,
    pub(crate) author: String,
    pub(crate) timestamp: String,
}

#[derive(Debug, Default)]
pub(crate) struct PackageHeaderAnalysis {
    pub(crate) header: Option<ValidatedPackageHeader>,
    pub(crate) diagnostics: Vec<PackageDiagnostic>,
}

pub(crate) fn analyze_package_header(value: &Value) -> PackageHeaderAnalysis {
    let Some(header) = value.get("header").and_then(Value::as_object) else {
        return PackageHeaderAnalysis::default();
    };

    let id = header.get("id").and_then(Value::as_str);
    let ifcx_version = header.get("ifcxVersion").and_then(Value::as_str);
    let data_version = header.get("dataVersion").and_then(Value::as_str);
    let author = header.get("author").and_then(Value::as_str);
    let timestamp = header.get("timestamp").and_then(Value::as_str);

    let mut diagnostics = Vec::new();
    let timestamp_is_valid = timestamp.is_some_and(|value| canonical_rfc3339_utc(value).is_some());
    if timestamp.is_some() && !timestamp_is_valid {
        diagnostics.push(PackageDiagnostic {
            code: IFCCAD_PACKAGE_TIMESTAMP_INVALID.to_owned(),
            severity: PackageDiagnosticSeverity::Error,
            resource_id: None,
            resource_uri: Some(DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()),
            location: Some("/header/timestamp".to_owned()),
            context: BTreeMap::new(),
            message: "package timestamp must be valid RFC 3339 with Z or +00:00".to_owned(),
        });
    }

    let typed_header = match (id, ifcx_version, data_version, author, timestamp) {
        (Some(id), Some(ifcx_version), Some(data_version), Some(author), Some(timestamp))
            if timestamp_is_valid =>
        {
            PackageId::new(id)
                .ok()
                .map(|package_id| ValidatedPackageHeader {
                    package_id,
                    ifcx_version: ifcx_version.to_owned(),
                    data_version: data_version.to_owned(),
                    author: author.to_owned(),
                    timestamp: timestamp.to_owned(),
                })
        }
        _ => None,
    };

    PackageHeaderAnalysis {
        header: typed_header,
        diagnostics,
    }
}

pub(crate) fn canonical_rfc3339_utc(value: &str) -> Option<String> {
    let accepted_suffix = value.ends_with('Z') || value.ends_with("+00:00");
    if !accepted_suffix
        || value.ends_with("-00:00")
        || OffsetDateTime::parse(value, &Rfc3339).is_err()
    {
        return None;
    }

    Some(match value.strip_suffix("+00:00") {
        Some(prefix) => format!("{prefix}Z"),
        None => value.to_owned(),
    })
}

#[derive(Clone, Copy, Debug)]
pub struct PackageHeaderRef<'a> {
    header: &'a ValidatedPackageHeader,
}

impl ValidatedPackage {
    pub fn header(&self) -> PackageHeaderRef<'_> {
        PackageHeaderRef {
            header: self
                .evidence()
                .header
                .as_ref()
                .expect("strict IFCX proof includes a validated package header"),
        }
    }
}

impl PackageHeaderRef<'_> {
    pub fn package_id(&self) -> &PackageId {
        &self.header.package_id
    }

    pub fn ifcx_version(&self) -> &str {
        &self.header.ifcx_version
    }

    pub fn data_version(&self) -> &str {
        &self.header.data_version
    }

    pub fn author(&self) -> &str {
        &self.header.author
    }

    pub fn timestamp(&self) -> &str {
        &self.header.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entrypoint(timestamp: &str) -> Value {
        json!({
            "header": {
                "id": "ifccad/tests/header",
                "ifcxVersion": "ifcx_alpha",
                "dataVersion": "17",
                "author": "ifccad tests",
                "timestamp": timestamp
            },
            "imports": [],
            "data": []
        })
    }

    #[test]
    fn accepts_zero_offset_rfc3339_and_preserves_source_text() {
        for timestamp in [
            "2026-09-02T10:00:00Z",
            "2026-09-02T10:00:00+00:00",
            "2026-09-02T10:00:00.125Z",
        ] {
            let analysis = analyze_package_header(&entrypoint(timestamp));
            assert!(analysis.diagnostics.is_empty(), "{timestamp}");
            assert_eq!(analysis.header.unwrap().timestamp, timestamp);
        }
    }

    #[test]
    fn rejects_unknown_nonzero_impossible_and_malformed_timestamps() {
        for timestamp in [
            "2026-09-02T10:00:00-00:00",
            "2026-09-02T12:00:00+02:00",
            "2026-02-30T10:00:00Z",
            "not-a-timestamp",
        ] {
            let analysis = analyze_package_header(&entrypoint(timestamp));
            assert!(analysis.header.is_none(), "{timestamp}");
            assert!(
                analysis.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == IFCCAD_PACKAGE_TIMESTAMP_INVALID
                        && diagnostic.location.as_deref() == Some("/header/timestamp")
                }),
                "{timestamp}: {:#?}",
                analysis.diagnostics
            );
        }
    }
}
