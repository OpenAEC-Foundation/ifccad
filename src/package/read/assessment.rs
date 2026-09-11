use crate::diagnostic::{PackageDiagnostic, PackageDiagnosticCategory, PackageDiagnosticSeverity};
use crate::ResourceId;
use serde::Serialize;

/// Validity evidence for the applicable IFCCAD-owned contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PackageValidity {
    Valid,
    Invalid,
    NotFullyAssessed,
}

/// Whether all applicable assessment steps could be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssessmentCompleteness {
    Complete,
    Incomplete,
}

/// Why part of the applicable contract was not assessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssessmentGapReason {
    UnsupportedProfile,
    UnavailableInput,
    ExecutionLimit,
    ContentNotAssessable,
    PreservationSemanticsNotAssessed,
}

/// Unassessed content, located in the same source coordinates as diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentGap {
    pub resource_id: Option<ResourceId>,
    pub resource_uri: Option<String>,
    pub location: Option<String>,
    pub reason: AssessmentGapReason,
}

/// Assessment evidence; independent of strict loading eligibility and transfer fidelity.
/// Permitted third-party IFCX extension semantics are outside this scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageAssessment {
    validity: PackageValidity,
    completeness: AssessmentCompleteness,
    gaps: Vec<AssessmentGap>,
}

impl Default for PackageAssessment {
    fn default() -> Self {
        Self::finish(&[], vec![], false)
    }
}

impl PackageAssessment {
    pub fn validity(&self) -> PackageValidity {
        self.validity
    }
    pub fn completeness(&self) -> AssessmentCompleteness {
        self.completeness
    }
    pub fn gaps(&self) -> &[AssessmentGap] {
        &self.gaps
    }

    pub(super) fn diagnostic_gaps(diagnostics: &[PackageDiagnostic]) -> Vec<AssessmentGap> {
        let mut gaps = Vec::new();
        // These diagnostics themselves record a skipped profile or failed load.
        // Other skipped checks are recorded by the orchestrator at their boundary.
        for diagnostic in diagnostics {
            let reason = match diagnostic.category {
                PackageDiagnosticCategory::UnsupportedContent => {
                    Some(AssessmentGapReason::UnsupportedProfile)
                }
                PackageDiagnosticCategory::ExecutionBlocked => {
                    Some(match diagnostic.code.as_str() {
                        super::codes::IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED
                        | super::codes::IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED => {
                            AssessmentGapReason::ExecutionLimit
                        }
                        _ => AssessmentGapReason::UnavailableInput,
                    })
                }
                PackageDiagnosticCategory::ContractViolation => match diagnostic.code.as_str() {
                    super::codes::IFCCAD_PACKAGE_JSON_INVALID
                    | super::codes::IFCCAD_PACKAGE_PATH_INVALID
                    | super::codes::IFCCAD_PACKAGE_ENTRYPOINT_INVALID => {
                        Some(AssessmentGapReason::ContentNotAssessable)
                    }
                    _ => None,
                },
            };
            if let Some(reason) = reason {
                gaps.push(AssessmentGap {
                    resource_id: diagnostic.resource_id.clone(),
                    resource_uri: diagnostic.resource_uri.clone(),
                    location: diagnostic.location.clone(),
                    reason,
                });
            }
        }
        gaps
    }

    pub(super) fn finish(
        diagnostics: &[PackageDiagnostic],
        mut gaps: Vec<AssessmentGap>,
        completed: bool,
    ) -> Self {
        if !completed && gaps.is_empty() {
            gaps.push(AssessmentGap {
                resource_id: None,
                resource_uri: None,
                location: None,
                reason: AssessmentGapReason::ContentNotAssessable,
            });
        }
        gaps.sort();
        gaps.dedup();
        let completeness = if completed && gaps.is_empty() {
            AssessmentCompleteness::Complete
        } else {
            AssessmentCompleteness::Incomplete
        };
        let invalid = diagnostics.iter().any(|d| {
            d.severity == PackageDiagnosticSeverity::Error
                && d.category == PackageDiagnosticCategory::ContractViolation
        });
        let validity = if invalid {
            PackageValidity::Invalid
        } else if completeness == AssessmentCompleteness::Complete {
            PackageValidity::Valid
        } else {
            PackageValidity::NotFullyAssessed
        };
        Self {
            validity,
            completeness,
            gaps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validity_and_completeness_are_independent() {
        for invalid in [false, true] {
            for incomplete in [false, true] {
                let diagnostics = if invalid {
                    vec![PackageDiagnostic {
                        category: PackageDiagnosticCategory::ContractViolation,
                        code: "TEST".into(),
                        severity: PackageDiagnosticSeverity::Error,
                        resource_id: None,
                        resource_uri: None,
                        location: None,
                        context: Default::default(),
                        message: String::new(),
                    }]
                } else {
                    vec![]
                };
                let assessment = PackageAssessment::finish(&diagnostics, vec![], !incomplete);
                assert_eq!(
                    assessment.validity(),
                    if invalid {
                        PackageValidity::Invalid
                    } else if incomplete {
                        PackageValidity::NotFullyAssessed
                    } else {
                        PackageValidity::Valid
                    }
                );
                assert_eq!(
                    assessment.completeness(),
                    if incomplete {
                        AssessmentCompleteness::Incomplete
                    } else {
                        AssessmentCompleteness::Complete
                    }
                );
            }
        }
    }
}
