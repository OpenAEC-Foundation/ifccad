use super::codec::json::{
    decode_block_definitions, decode_geometric_entities, decode_layers, decode_layouts,
    decode_point_display, decode_scopes, decode_ucs_definitions, decode_view_state,
    decode_viewports, decode_workspace_state,
};
use super::codec::json::{parse_document, JsonEncodedDrawing};
use super::logical::{
    DrawingBlockDefinition, DrawingGeometricEntity, DrawingLayer, DrawingLayout, DrawingScope,
    DrawingUcsDefinition, DrawingWorkspaceState,
};
use super::logical::{DrawingDocument, DrawingModel};
use super::{PlotStyleMode, PointDisplay};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingLoadStatus {
    Valid,
    Invalid,
    UnsupportedVersion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawingDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ValidatedDrawing {
    encoding: JsonEncodedDrawing,
    document: DrawingDocument,
    owners: std::collections::BTreeMap<u64, u32>,
}

impl ValidatedDrawing {
    /// Returns the owner derived from the ordered scope lists.
    pub fn owner_scope_id(&self, entity_id: u64) -> Option<u32> {
        self.owners.get(&entity_id).copied()
    }

    pub fn geometric_entities(&self) -> &[DrawingGeometricEntity] {
        &self.document.geometric_entities
    }

    pub fn viewports(&self) -> &[super::logical::DrawingViewport] {
        &self.document.viewports
    }

    pub fn typed_layers(&self) -> &[DrawingLayer] {
        &self.document.layers
    }

    pub fn typed_layouts(&self) -> &[DrawingLayout] {
        &self.document.layouts
    }

    pub fn ucs_definitions(&self) -> &[DrawingUcsDefinition] {
        &self.document.ucs_definitions
    }

    pub fn workspace_state(&self) -> Option<&DrawingWorkspaceState> {
        self.document.workspace_state.as_ref()
    }

    pub fn point_display(&self) -> Option<PointDisplay> {
        self.document.point_display
    }

    pub fn block_definitions(&self) -> &[DrawingBlockDefinition] {
        &self.document.block_definitions
    }

    pub fn scopes(&self) -> &[DrawingScope] {
        &self.document.scopes
    }

    pub fn drawing_id(&self) -> &str {
        &self.document.drawing_id
    }

    pub fn unit(&self) -> &str {
        &self.document.unit
    }

    pub fn typed_plot_style_mode(&self) -> PlotStyleMode {
        self.document.plot_style_mode
    }

    pub fn has_unconverted_view_state(&self) -> bool {
        self.document.view_state.is_some()
            || !self.document.model_windows.is_empty()
            || !self.document.paper_canvases.is_empty()
            || !self.document.viewport_workspaces.is_empty()
    }

    pub fn view_state(&self) -> Option<super::logical::DrawingViewState> {
        self.document.view_state
    }

    pub fn model_windows(&self) -> &[super::logical::DrawingModelWindow] {
        &self.document.model_windows
    }

    pub fn paper_canvases(&self) -> &[super::logical::DrawingPaperCanvas] {
        &self.document.paper_canvases
    }

    pub fn viewport_workspaces(&self) -> &[super::logical::DrawingViewportWorkspace] {
        &self.document.viewport_workspaces
    }

    pub fn plot_style_mode(&self) -> &str {
        match self.document.plot_style_mode {
            PlotStyleMode::ColorDependent => "colorDependent",
            PlotStyleMode::Named => "named",
        }
    }

    /// Returns the validated JSON encoding for inspection and codec-level tests.
    /// Drawing conversion should use the typed accessors instead.
    pub fn as_value(&self) -> &serde_json::Value {
        self.encoding.value()
    }
}

#[derive(Clone, Debug)]
pub struct DrawingLoadOutcome {
    status: DrawingLoadStatus,
    diagnostics: Vec<DrawingDiagnostic>,
    drawing: Option<ValidatedDrawing>,
}

impl DrawingLoadOutcome {
    pub fn status(&self) -> DrawingLoadStatus {
        self.status
    }

    pub fn diagnostics(&self) -> &[DrawingDiagnostic] {
        &self.diagnostics
    }

    pub fn validated_drawing(&self) -> Option<&ValidatedDrawing> {
        self.drawing.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DrawingOpenError {
    #[error("could not read drawing: {0}")]
    Io(#[from] std::io::Error),
}

pub fn load_drawing_file(path: impl AsRef<Path>) -> Result<DrawingLoadOutcome, DrawingOpenError> {
    Ok(load_drawing_bytes(&std::fs::read(path)?))
}

pub(crate) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> DrawingDiagnostic {
    DrawingDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
    }
}

fn invalid(diagnostics: Vec<DrawingDiagnostic>) -> DrawingLoadOutcome {
    DrawingLoadOutcome {
        status: DrawingLoadStatus::Invalid,
        diagnostics,
        drawing: None,
    }
}

pub fn load_drawing_bytes(bytes: &[u8]) -> DrawingLoadOutcome {
    let encoding = match parse_document(bytes) {
        Ok(encoding) => encoding,
        Err((status, diagnostics)) => {
            return DrawingLoadOutcome {
                status,
                diagnostics,
                drawing: None,
            }
        }
    };
    let value = encoding.value();
    let mut diagnostics = Vec::new();
    super::codec::json::validate_physical(value, &mut diagnostics);
    let model: DrawingModel = super::codec::json::decode_model(value);
    diagnostics.extend(
        model
            .validate()
            .into_iter()
            .map(|error| diagnostic(error.code, error.location, error.message)),
    );
    if !diagnostics.is_empty() {
        return invalid(diagnostics);
    }
    let Some(geometric_entities) = decode_geometric_entities(value) else {
        return invalid(vec![diagnostic(
            "ENTITY_DECODE",
            "/streams",
            "validated entity stream could not be decoded",
        )]);
    };
    let Some(viewports) = decode_viewports(value) else {
        return invalid(vec![diagnostic(
            "VIEWPORT_DECODE",
            "/streams/viewportStream",
            "validated viewport stream could not be decoded",
        )]);
    };
    let Some(typed_layers) = decode_layers(value) else {
        return invalid(vec![diagnostic(
            "LAYER_DECODE",
            "/layers",
            "validated Layer could not be decoded",
        )]);
    };
    let Some(typed_layouts) = decode_layouts(value) else {
        return invalid(vec![diagnostic(
            "LAYOUT_DECODE",
            "/layouts",
            "validated Layout could not be decoded",
        )]);
    };
    let Some(ucs_definitions) = decode_ucs_definitions(value) else {
        return invalid(vec![diagnostic(
            "UCS_DECODE",
            "/ucsDefinitions",
            "validated UCS definition could not be decoded",
        )]);
    };
    let Some(workspace_state) = decode_workspace_state(value) else {
        return invalid(vec![diagnostic(
            "WORKSPACE_DECODE",
            "/drawingWorkspaceState",
            "validated workspace state could not be decoded",
        )]);
    };
    let Some(point_display) = decode_point_display(value) else {
        return invalid(vec![diagnostic(
            "POINT_DISPLAY_DECODE",
            "/pointDisplay",
            "validated point display could not be decoded",
        )]);
    };
    let Some(block_definitions) = decode_block_definitions(value) else {
        return invalid(vec![diagnostic(
            "BLOCK_DECODE",
            "/blockDefinitions",
            "validated block definition could not be decoded",
        )]);
    };
    let Some(scopes) = decode_scopes(value) else {
        return invalid(vec![diagnostic(
            "SCOPE_DECODE",
            "/scopes",
            "validated scopes could not be decoded",
        )]);
    };
    diagnostics.extend(
        super::logical::validate_geometry_bounds(&geometric_entities, &scopes)
            .into_iter()
            .map(|error| diagnostic(error.code, error.location, error.message)),
    );
    diagnostics.extend(
        super::logical::validate_blocks(&geometric_entities, &block_definitions, &scopes)
            .into_iter()
            .map(|error| diagnostic(error.code, error.location, error.message)),
    );
    diagnostics.extend(
        super::logical::validate_viewport_bounds(&viewports, &scopes)
            .into_iter()
            .map(|error| diagnostic(error.code, error.location, error.message)),
    );
    if !diagnostics.is_empty() {
        return invalid(diagnostics);
    }
    let Some(view_state) = decode_view_state(value) else {
        return invalid(vec![diagnostic(
            "VIEW_STATE_DECODE",
            "/",
            "validated drawing view state could not be decoded",
        )]);
    };
    let (drawing_id, unit, plot_style_mode) = encoding.header();
    let document = DrawingDocument {
        drawing_id,
        unit,
        plot_style_mode,
        geometric_entities,
        viewports,
        layers: typed_layers,
        layouts: typed_layouts,
        ucs_definitions,
        workspace_state,
        point_display,
        block_definitions,
        scopes,
        view_state: view_state.view_state,
        model_windows: view_state.model_windows,
        paper_canvases: view_state.paper_canvases,
        viewport_workspaces: view_state.viewport_workspaces,
    };
    diagnostics.extend(
        super::logical::validate_state(&document)
            .into_iter()
            .map(|e| diagnostic(e.code, e.location, e.message)),
    );
    if !diagnostics.is_empty() {
        return invalid(diagnostics);
    }
    DrawingLoadOutcome {
        status: DrawingLoadStatus::Valid,
        diagnostics,
        drawing: Some(ValidatedDrawing {
            owners: super::logical::owner_index(&document.scopes),
            encoding,
            document,
        }),
    }
}
