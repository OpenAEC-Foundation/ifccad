/// Conclusion from an executed conversion within its stated scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferConclusion {
    LossDetected,
    NoLossDetected,
    NotFullyAssessed,
}

/// Source boundary to which the transfer evidence applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferScope {
    SelectedDrawingToCadDocument,
    PublicCadDocumentToPackage,
}

/// Coverage of fidelity assessment, independently of detected loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCoverage {
    CompleteWithinScope,
    Incomplete,
}

/// Fidelity evidence for a returned output. Failed attempts return the existing
/// typed error instead; `LossRejected` retains loss evidence without an output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferAssessment {
    conclusion: TransferConclusion,
    scope: TransferScope,
    coverage: TransferCoverage,
    limitations: &'static [&'static str],
}

impl TransferAssessment {
    pub fn conclusion(&self) -> TransferConclusion {
        self.conclusion
    }
    pub fn scope(&self) -> TransferScope {
        self.scope
    }
    pub fn coverage(&self) -> TransferCoverage {
        self.coverage
    }
    /// Scope exclusions and unassessed fidelity, also retained when loss is detected.
    pub fn limitations(&self) -> &[&'static str] {
        self.limitations
    }

    pub(crate) fn import(has_recorded_loss: bool) -> Self {
        Self::new(has_recorded_loss, TransferScope::SelectedDrawingToCadDocument,
            TransferCoverage::Incomplete, &[
                "Assessment covers the selected drawing, not the package semantic graph or IFCPR restoration.",
                "Import coverage does not establish fidelity for scope metadata, opacity quantization, or all color and appearance metadata; see import/COVERAGE.md.",
            ])
    }

    pub(crate) fn export(has_recorded_loss: bool) -> Self {
        Self::new(has_recorded_loss, TransferScope::PublicCadDocumentToPackage,
            TransferCoverage::CompleteWithinScope, &[
                "Assessment covers the pinned public CadDocument model documented in export/COVERAGE.md; hidden runtime state and original raw CAD bytes are excluded.",
            ])
    }

    fn new(
        loss: bool,
        scope: TransferScope,
        coverage: TransferCoverage,
        limitations: &'static [&'static str],
    ) -> Self {
        let conclusion = if loss {
            TransferConclusion::LossDetected
        } else if coverage == TransferCoverage::CompleteWithinScope {
            TransferConclusion::NoLossDetected
        } else {
            TransferConclusion::NotFullyAssessed
        };
        Self {
            conclusion,
            scope,
            coverage,
            limitations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loss_takes_precedence_without_erasing_coverage() {
        for loss in [false, true] {
            let import = TransferAssessment::import(loss);
            let export = TransferAssessment::export(loss);
            assert_eq!(
                import.conclusion(),
                if loss {
                    TransferConclusion::LossDetected
                } else {
                    TransferConclusion::NotFullyAssessed
                }
            );
            assert_eq!(
                export.conclusion(),
                if loss {
                    TransferConclusion::LossDetected
                } else {
                    TransferConclusion::NoLossDetected
                }
            );
            assert_eq!(import.coverage(), TransferCoverage::Incomplete);
            assert_eq!(export.coverage(), TransferCoverage::CompleteWithinScope);
            assert!(!import.limitations().is_empty());
            assert!(!export.limitations().is_empty());
        }
    }
}
