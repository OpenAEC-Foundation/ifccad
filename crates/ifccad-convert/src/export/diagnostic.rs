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
    UnsupportedUnit {
        code: i16,
    },
    LayerLocked,
    LayerFrozenInNewViewport,
    LayerXrefDependent,
    LayerNotPlottable,
    LayerPlotStyle {
        name: String,
    },
    MaterialReference {
        handle: Handle,
    },
    PlotStyleReference {
        handle: Handle,
    },
    XrefBlockRecordReference {
        handle: Handle,
    },
    NamedColorIdentityIncomplete {
        color_name: Option<String>,
        book_name: Option<String>,
    },
    LayerColorUnsupported {
        color: String,
    },
    LayerTransparencyUnsupported,
    LayerLinePatternMissing,
    LayerLineWeightUnsupported {
        value: i16,
    },
    NonFiniteCoordinate,
    NonPlanarZ,
    NonZeroElevation,
    NonZeroThickness,
    UnsupportedNormal,
    PolylineTooFewVertices {
        count: usize,
    },
    PolylineBulge,
    PolylineWidth,
    PolylinePlinegen,
    EntityColorUnsupported {
        color: String,
    },
    EntityNamedColorUnsupported {
        name: String,
    },
    EntityNamedColorWithoutExplicitColor,
    EntityLineWeightUnsupported {
        value: i16,
    },
    MissingEntityLayer {
        name: String,
    },
    PaperSpaceEntity,
    BlockOwnedEntity {
        owner: Handle,
    },
    UnsupportedEntityType {
        kind: String,
    },
    UnsupportedSemantic {
        name: String,
    },
    MissingTarget {
        kind: String,
        identifier: String,
    },
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
    pub(crate) fn loss(
        source: ExportDiagnosticSource,
        action: ExportAction,
        reasons: Vec<ExportLossReason>,
    ) -> Self {
        assert!(!reasons.is_empty(), "a loss diagnostic requires a reason");
        Self::Loss {
            source,
            action,
            reasons,
        }
    }

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
    ModelSpaceBlockRecordMissing {
        model_space_block: Handle,
    },
    ModelLayoutMissing {
        model_space_block: Handle,
    },
    MultipleModelLayouts {
        model_space_block: Handle,
        count: usize,
    },
    EntityOwnerMissing {
        entity: Handle,
    },
    EntityOwnerUnknown {
        entity: Handle,
        owner: Handle,
    },
    InconsistentRelationship {
        description: String,
    },
}
