use cadcodec::Handle;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExportDiagnosticSource {
    Document,
    DocumentField { name: String },
    Layer { name: String },
    Entity { handle: Handle, kind: String },
    Object { handle: Handle, kind: String },
    Table { kind: String },
    Collection { kind: String, count: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExportAction {
    PartiallyExported,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExportLossReason {
    UnsupportedSemantic { name: String },
    MissingTarget { kind: String, identifier: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExportDiagnostic {
    Loss {
        source: ExportDiagnosticSource,
        action: ExportAction,
        reasons: Vec<ExportLossReason>,
    },
}

impl ExportDiagnostic {
    pub fn source(&self) -> &ExportDiagnosticSource {
        match self {
            Self::Loss { source, .. } => source,
        }
    }

    pub fn action(&self) -> ExportAction {
        match self {
            Self::Loss { action, .. } => *action,
        }
    }

    pub fn reasons(&self) -> &[ExportLossReason] {
        match self {
            Self::Loss { reasons, .. } => reasons,
        }
    }

    pub fn is_loss(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SourceStructureProblem {
    ModelSpaceBlockMissing,
    ModelLayoutMissing {
        model_space_block: Handle,
    },
    MultipleModelLayouts {
        model_space_block: Handle,
        count: usize,
    },
    InconsistentRelationship {
        description: String,
    },
}
