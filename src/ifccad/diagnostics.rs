//! Internal formatting shared by wire, graph and typed validation.
use super::IfccadReport;

pub(crate) fn failure(rule: &str, location: &str, message: impl Into<String>) -> IfccadReport {
    IfccadReport::one(format!("{rule} {location}: {}", message.into()))
}

/// Attach an owner to unqualified or relative diagnostics, without losing the
/// original rule, detail or additional errors. Locations already in the source
/// drawing or file envelope remain absolute.
pub(crate) fn context(report: IfccadReport, rule: &str, owner: &str) -> IfccadReport {
    IfccadReport {
        errors: report
            .errors
            .into_iter()
            .map(|message| {
                if let Some((existing_rule, detail)) = message
                    .split_once(' ')
                    .filter(|(id, _)| id.starts_with("IFCCAD-"))
                {
                    if let Some((location, cause)) = detail.split_once(": ") {
                        let absolute = ["/cad/", "/header", "/data", "/imports", "/schemas"]
                            .iter()
                            .any(|prefix| location.starts_with(prefix));
                        let path = if absolute || owner == "/" {
                            location.to_owned()
                        } else if location == "/" {
                            owner.to_owned()
                        } else {
                            format!("{}{}", owner.trim_end_matches('/'), location)
                        };
                        return format!("{existing_rule} {path}: {cause}");
                    }
                }
                format!("{rule} {owner}: {message}")
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_context_preserves_rule_detail_and_multiple_errors() {
        let mut report = failure(
            "IFCCAD-GEOMETRY-001",
            "/ifccad::geom::placement/xAxis",
            "invalid frame",
        );
        report.errors.push("old detail".into());
        let report = context(report, "IFCCAD-WIRE-004", "/cad/d1/e2");
        assert_eq!(
            report.errors,
            [
                "IFCCAD-GEOMETRY-001 /cad/d1/e2/ifccad::geom::placement/xAxis: invalid frame",
                "IFCCAD-WIRE-004 /cad/d1/e2: old detail",
            ]
        );
        let again = context(report.clone(), "IFCCAD-WIRE-004", "/cad/d1");
        assert_eq!(again, report);
    }
}
