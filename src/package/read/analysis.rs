use super::model::{LoadedIfccadPackage, PackageAnalysis};
use super::PackageDiagnostic;
use crate::validated::{EvidenceOutcome, Validated, ValidationTarget};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct IfccadPackageTarget {
    #[cfg_attr(not(test), allow(dead_code))]
    package: Arc<LoadedIfccadPackage>,
}

impl IfccadPackageTarget {
    fn new(package: Arc<LoadedIfccadPackage>) -> Self {
        Self { package }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn package(&self) -> &Arc<LoadedIfccadPackage> {
        &self.package
    }
}

pub(crate) struct PackageProofContext {
    analysis: Arc<PackageAnalysis>,
    is_valid: bool,
}

impl ValidationTarget for IfccadPackageTarget {
    type Context = PackageProofContext;
    type Evidence = Arc<PackageAnalysis>;
    type Diagnostic = PackageDiagnostic;

    fn build_evidence(
        &self,
        context: &Self::Context,
    ) -> EvidenceOutcome<Self::Evidence, Self::Diagnostic> {
        if context.is_valid {
            EvidenceOutcome::success(context.analysis.clone(), Vec::new())
        } else {
            EvidenceOutcome::failure(Vec::new())
        }
    }
}

/// Strictly validated IFCCAD package with typed, read-only navigation.
#[derive(Debug)]
pub struct ValidatedPackage {
    proof: Validated<IfccadPackageTarget>,
}

impl ValidatedPackage {
    pub(crate) fn loaded(&self) -> &IfccadPackageTarget {
        self.proof.loaded()
    }

    pub(crate) fn evidence(&self) -> &Arc<PackageAnalysis> {
        self.proof.evidence()
    }
}

pub(super) fn build_strict_proof(
    package: Arc<LoadedIfccadPackage>,
    analysis: Arc<PackageAnalysis>,
    is_valid: bool,
) -> Option<ValidatedPackage> {
    let context = PackageProofContext { analysis, is_valid };
    Validated::validate(IfccadPackageTarget::new(package), &context)
        .into_parts()
        .0
        .map(|proof| ValidatedPackage { proof })
}
