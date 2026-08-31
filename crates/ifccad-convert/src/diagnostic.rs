use ifccad::ifcdr::{AppearanceId, EntityId, LayerId};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConversionDiagnostic {
    UnmodeledEntitiesSkipped {
        schema_id: String,
        count: usize,
    },
    LinePatternFallback {
        requested: String,
        applied: String,
        count: usize,
    },
    LineWeightRounded {
        requested_mm: f64,
        applied_mm: f64,
        count: usize,
    },
    TransparencySemanticsLost {
        source: LostTransparencyMode,
        count: usize,
    },
    NamedLayerColorIdentityLost {
        layer: String,
        catalog: String,
        name: String,
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LostTransparencyMode {
    ByBlock,
    ExplicitOpaque,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConversionError {
    #[error(
        "drawing must contain exactly one model layout (found {total_layouts} layouts, {model_layouts} model layouts)"
    )]
    UnsupportedDrawingStructure {
        total_layouts: usize,
        model_layouts: usize,
    },
    #[error("entity {entity_id:?} refers to missing layer {layer_id:?}")]
    MissingEntityLayer {
        entity_id: EntityId,
        layer_id: LayerId,
    },
    #[error("entity {entity_id:?} refers to missing appearance {appearance_id:?}")]
    MissingEntityAppearance {
        entity_id: EntityId,
        appearance_id: AppearanceId,
    },
    #[error("cadcodec could not insert converted entity {entity_id:?}")]
    CadcodecEntityInsertion {
        entity_id: EntityId,
        #[source]
        source: cadcodec::DxfError,
    },
    #[error("cadcodec could not insert layer {layer}: {reason}")]
    LayerInsertion { layer: String, reason: String },
    #[error("internal conversion invariant failed: {message}")]
    InternalInvariant { message: String },
}
