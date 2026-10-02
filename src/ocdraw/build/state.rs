use super::{OcdrawBuildError, OcdrawBuilder};
use crate::ocdraw::logical::*;

/// Drawing-local saved views and workspaces. Absence carries no selection.
#[derive(Clone, Debug, Default)]
pub struct DrawingSavedState {
    pub view_state: Option<DrawingViewState>,
    pub model_windows: Vec<DrawingModelWindow>,
    pub paper_canvases: Vec<DrawingPaperCanvas>,
    pub viewport_workspaces: Vec<DrawingViewportWorkspace>,
}

#[derive(Clone, Debug)]
pub struct ViewportDefinition {
    pub scope_id: u32,
    pub layer_id: u32,
    pub frame: DrawingViewportFrame,
    pub view: DrawingView,
    pub render_mode: DrawingRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub paper_clip: DrawingPaperClip,
    pub plot_shading_override: Option<crate::ocdraw::ShadedPlot>,
    pub appearance: EntityAppearance,
    pub visible: bool,
    pub layer_overrides: Vec<DrawingViewportLayerOverride>,
}
impl ViewportDefinition {
    pub fn new(
        scope_id: u32,
        layer_id: u32,
        frame: DrawingViewportFrame,
        view: DrawingView,
    ) -> Self {
        Self {
            scope_id,
            layer_id,
            frame,
            view,
            render_mode: DrawingRenderMode::TwoDimensional,
            view_enabled: true,
            view_locked: false,
            paper_clip: DrawingPaperClip {
                enabled: false,
                boundary_entity_id: None,
            },
            plot_shading_override: None,
            appearance: EntityAppearance::default(),
            visible: true,
            layer_overrides: Vec::new(),
        }
    }
}
impl OcdrawBuilder {
    pub fn set_saved_state(&mut self, state: DrawingSavedState) {
        self.saved_state = state;
    }
    pub fn add_viewport(
        &mut self,
        definition: ViewportDefinition,
    ) -> Result<u64, OcdrawBuildError> {
        let id = self.next_entity_id;
        self.next_entity_id = id.checked_add(1).ok_or(OcdrawBuildError::IdExhausted)?;
        self.scope_entities
            .entry(definition.scope_id)
            .or_default()
            .push(id);
        self.viewports.push(DrawingViewport {
            id,
            view_scope_id: 0,
            layer_id: definition.layer_id,
            frame: definition.frame,
            view: definition.view,
            render_mode: definition.render_mode,
            view_enabled: definition.view_enabled,
            view_locked: definition.view_locked,
            paper_clip: definition.paper_clip,
            plot_shading_override: definition.plot_shading_override,
            appearance: definition.appearance,
            visible: definition.visible,
            layer_overrides: definition.layer_overrides,
        });
        Ok(id)
    }
}
