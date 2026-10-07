use crate::*;
use ocdraw::ocdraw::OcdrawBuildError;
#[derive(Debug, thiserror::Error)]
pub enum CadToOcdrawError {
    #[error(transparent)]
    PaperTolerance(#[from] crate::OcdrawPaperToleranceError),
    #[error(transparent)]
    PlotNumeric(#[from] cad_geometry_convert::plot_units::PlotNumericError),
    #[error(transparent)]
    PreservationSnapshot(#[from] crate::OcdrawSplineSnapshotError),
    #[error(transparent)]
    GeometryTolerance(#[from] crate::OcdrawToleranceError),
    #[error("geometric accuracy requirement failed: {0:?}")]
    Geometry(Box<crate::OcdrawGeometryFailure>),
    #[error("CAD source structure is invalid")]
    InvalidSourceStructure {
        problems: Vec<CadSourceStructureProblem>,
    },
    #[error("conversion loss was rejected")]
    LossRejected {
        diagnostics: Vec<CadToOcdrawDiagnostic>,
        preservation: crate::OcdrawPreservationReport,
    },
    #[error(transparent)]
    DrawingBuild(#[from] OcdrawBuildError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawToCadDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

impl CadToOcdrawError {
    pub fn preservation_report(&self) -> Option<&crate::OcdrawPreservationReport> {
        match self {
            Self::LossRejected { preservation, .. } => Some(preservation),
            _ => None,
        }
    }
}
impl OcdrawToCadError {
    pub fn preservation_report(&self) -> Option<&crate::OcdrawPreservationReport> {
        match self {
            Self::LossRejected { preservation, .. } => Some(preservation),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OcdrawToCadError {
    #[error(transparent)]
    PaperTolerance(#[from] crate::OcdrawPaperToleranceError),
    #[error(transparent)]
    PlotNumeric(#[from] cad_geometry_convert::plot_units::PlotNumericError),
    #[error(transparent)]
    InvalidDocument(#[from] ocdraw::ocdraw::OcdrawValidationError),
    #[error(transparent)]
    GeometryTolerance(#[from] crate::OcdrawToleranceError),
    #[error("geometric accuracy requirement failed: {0:?}")]
    Geometry(Box<crate::OcdrawGeometryFailure>),
    #[error("drawing conversion loss was rejected")]
    LossRejected {
        diagnostics: Vec<OcdrawToCadDiagnostic>,
        preservation: crate::OcdrawPreservationReport,
    },
    #[error("CAD construction failed: {0}")]
    Cad(String),
}

use opencadcodec::Handle;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CadToOcdrawDiagnosticSource {
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
pub enum CadToOcdrawAction {
    PartiallyExported,
    Skipped,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CadToOcdrawLossReason {
    ComplexLinePatternFallback {
        name: String,
        text: bool,
        shapes: bool,
    },
    BlockContentLoss {
        definition: Handle,
        affected_instances: Vec<Handle>,
    },
    GeometryRoundedWithinTolerance {
        max_deviation_upper_bound: f64,
    },
    SourceNormalNormalized,
    PolylineVertexIdentifiers {
        count: usize,
    },

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
    EntityLinetypeScale,
    EntityLinetypeHandle,
    EntityExtendedData,
    EntityGraphicData,
    EntityReactors,
    EntityExtensionDictionary,
    EntityColorBookReference,
    EntityFullVisualStyle,
    EntityFaceVisualStyle,
    EntityEdgeVisualStyle,
    EntityMaterial,
    EntityShadowFlags,
    EntityPlotStyle,
    UnsupportedHeaderField {
        name: String,
    },
    DocumentSummaryInformation,
    UnsupportedTableRecords {
        kind: String,
        count: usize,
    },
    UnsupportedCollection {
        kind: String,
        count: usize,
    },
    UnsupportedSemantic {
        name: String,
    },
    MissingTarget {
        kind: String,
        identifier: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CadToOcdrawDiagnostic {
    Loss {
        source: CadToOcdrawDiagnosticSource,
        action: CadToOcdrawAction,
        reasons: Vec<CadToOcdrawLossReason>,
    },
}

impl CadToOcdrawDiagnostic {
    pub(crate) fn loss(
        source: CadToOcdrawDiagnosticSource,
        action: CadToOcdrawAction,
        reasons: Vec<CadToOcdrawLossReason>,
    ) -> Self {
        assert!(!reasons.is_empty(), "a loss diagnostic requires a reason");
        Self::Loss {
            source,
            action,
            reasons,
        }
    }

    pub fn source(&self) -> &CadToOcdrawDiagnosticSource {
        match self {
            Self::Loss { source, .. } => source,
        }
    }

    pub fn action(&self) -> CadToOcdrawAction {
        match self {
            Self::Loss { action, .. } => *action,
        }
    }

    pub fn reasons(&self) -> &[CadToOcdrawLossReason] {
        match self {
            Self::Loss { reasons, .. } => reasons,
        }
    }

    pub(crate) fn blocks_reject(&self) -> bool {
        self.reasons().iter().any(|r| {
            !matches!(
                r,
                CadToOcdrawLossReason::GeometryRoundedWithinTolerance { .. }
            )
        })
    }
    pub fn is_loss(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CadSourceStructureProblem {
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
