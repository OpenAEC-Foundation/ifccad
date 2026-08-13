pub(crate) trait ValidationTarget: Sized {
    type Context;
    type Evidence;
    type Diagnostic;

    fn build_evidence(
        &self,
        context: &Self::Context,
    ) -> EvidenceOutcome<Self::Evidence, Self::Diagnostic>;
}

pub(crate) struct EvidenceOutcome<E, D> {
    evidence: Option<E>,
    diagnostics: Vec<D>,
}

impl<E, D> EvidenceOutcome<E, D> {
    pub(crate) fn success(evidence: E, diagnostics: Vec<D>) -> Self {
        Self {
            evidence: Some(evidence),
            diagnostics,
        }
    }

    pub(crate) fn failure(diagnostics: Vec<D>) -> Self {
        Self {
            evidence: None,
            diagnostics,
        }
    }
}

pub(crate) struct Validated<T: ValidationTarget> {
    loaded: T,
    evidence: T::Evidence,
}

pub(crate) struct ValidationOutcome<T: ValidationTarget> {
    validated: Option<Validated<T>>,
    diagnostics: Vec<T::Diagnostic>,
}

impl<T: ValidationTarget> ValidationOutcome<T> {
    pub(crate) fn validated(&self) -> Option<&Validated<T>> {
        self.validated.as_ref()
    }

    pub(crate) fn diagnostics(&self) -> &[T::Diagnostic] {
        &self.diagnostics
    }

    pub(crate) fn into_parts(self) -> (Option<Validated<T>>, Vec<T::Diagnostic>) {
        (self.validated, self.diagnostics)
    }
}

impl<T: ValidationTarget> Validated<T> {
    pub(crate) fn validate(loaded: T, context: &T::Context) -> ValidationOutcome<T> {
        let outcome = loaded.build_evidence(context);
        ValidationOutcome {
            validated: outcome.evidence.map(|evidence| Self { loaded, evidence }),
            diagnostics: outcome.diagnostics,
        }
    }

    pub(crate) fn loaded(&self) -> &T {
        &self.loaded
    }

    pub(crate) fn evidence(&self) -> &T::Evidence {
        &self.evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Loaded(u32);

    impl ValidationTarget for Loaded {
        type Context = bool;
        type Evidence = u32;
        type Diagnostic = &'static str;

        fn build_evidence(
            &self,
            succeeds: &bool,
        ) -> EvidenceOutcome<Self::Evidence, Self::Diagnostic> {
            if *succeeds {
                EvidenceOutcome::success(self.0 * 2, vec!["notice"])
            } else {
                EvidenceOutcome::failure(vec!["invalid"])
            }
        }
    }

    #[test]
    fn validation_constructs_proof_only_when_evidence_exists() {
        let success = Validated::validate(Loaded(4), &true);
        assert_eq!(success.validated().unwrap().loaded().0, 4);
        assert_eq!(*success.validated().unwrap().evidence(), 8);
        assert_eq!(success.diagnostics(), &["notice"]);

        let failure = Validated::validate(Loaded(4), &false);
        assert!(failure.validated().is_none());
        assert_eq!(failure.diagnostics(), &["invalid"]);
    }
}
