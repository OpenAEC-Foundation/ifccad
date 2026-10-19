use super::*;
use crate::geometry_kernel::{CoordinateFrame3, Point2, Point3, Vector3};
use crate::plot_kernel::ShadedPlotMode;
use crate::workspace_kernel::{
    WorkspaceCanvasFrame, WorkspaceClip, WorkspaceClipMode, WorkspaceGrid, WorkspaceGridStyle,
    WorkspaceIsometricPlane, WorkspaceProjection, WorkspaceRenderMode, WorkspaceSnap,
    WorkspaceSnapStyle, WorkspaceView,
};
record!(IfccadUcsDefinition;
0 => id: IfccadUcsId = None,
1 => name: String = None,
2 => frame: CoordinateFrame3 = None,
3 => elevation: f64 = None,
);
record!(IfccadModelWindow;
0 => id: IfccadModelWindowId = None,
1 => rectangle: [f64;4] = None,
2 => view: WorkspaceView = None,
3 => aspect_ratio: f64 = None,
4 => render_mode: WorkspaceRenderMode = None,
5 => grid: WorkspaceGrid = None,
6 => snap: WorkspaceSnap = None,
7 => stored_ucs: IfccadUcsSelection = None,
8 => use_stored_ucs: bool = None,
);
record!(IfccadDrawingWorkspaceState;
0 => current_layer_id: Option<u64> = Some(None),
1 => active_layout_id: Option<u64> = Some(None),
);
record!(IfccadModelViewState;
0 => current_model_ucs: Option<IfccadUcsSelection> = Some(None),
1 => active_model_window_id: Option<IfccadModelWindowId> = Some(None),
);
impl CborValue for IfccadUcsId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(CborValue::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self.0.exact(&o.0)
    }
}
impl CborValue for IfccadModelWindowId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(CborValue::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self.0.exact(&o.0)
    }
}
record!(WorkspaceView;
0 => center: Point2 = None,
1 => target: Point3 = None,
2 => direction: Vector3 = None,
3 => height: f64 = None,
4 => twist: f64 = None,
5 => projection: WorkspaceProjection = None,
6 => lens_length: Option<f64> = Some(None),
7 => front_clip: WorkspaceClip = None,
8 => back_clip: WorkspaceClip = None,
);
unit_enum!(WorkspaceRenderMode;
0 => TwoDimensional,
1 => Wireframe,
2 => HiddenLine,
3 => FlatShadedWithoutEdges,
4 => FlatShadedWithEdges,
5 => SmoothShadedWithoutEdges,
6 => SmoothShadedWithEdges,
);
record!(WorkspaceGrid;
0 => enabled: bool = None,
1 => spacing: Point2 = None,
2 => style: WorkspaceGridStyle = None,
3 => major_line_frequency: u32 = None,
4 => beyond_limits: bool = None,
5 => adaptive: bool = None,
6 => subdivision: bool = None,
7 => follows_workplane: bool = None,
);
record!(WorkspaceSnap;
0 => enabled: bool = None,
1 => base: Point2 = None,
2 => spacing: Point2 = None,
3 => angle: f64 = None,
4 => style: WorkspaceSnapStyle = None,
5 => isometric_plane: WorkspaceIsometricPlane = None,
);
impl CborValue for IfccadUcsSelection {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "World"), (1, "Named"), (2, "Unnamed")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::World => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Named(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadUcsId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadUcsId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Unnamed(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<CoordinateFrame3>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<CoordinateFrame3>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::World,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadUcsId = b.take(0, None)?;
                b.finish()?;
                Self::Named(value_0)
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: CoordinateFrame3 = b.take(0, None)?;
                b.finish()?;
                Self::Unnamed(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::World, Self::World) => true,
            (Self::Named(value_0), Self::Named(value_0_other)) => value_0.exact(value_0_other),
            (Self::Unnamed(value_0), Self::Unnamed(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
record!(IfccadPaperCanvas;
0 => view: WorkspaceView = None,
1 => frame: Option<WorkspaceCanvasFrame> = Some(None),
2 => grid: WorkspaceGrid = None,
3 => snap: WorkspaceSnap = None,
4 => stored_ucs: IfccadUcsSelection = None,
5 => use_stored_ucs: bool = None,
6 => current_ucs: Option<IfccadUcsSelection> = Some(None),
7 => active_context: Option<IfccadPaperContext> = Some(None),
);
unit_enum!(WorkspaceProjection;
0 => Orthographic,
1 => Perspective,
);
record!(WorkspaceClip;
0 => mode: WorkspaceClipMode = None,
1 => distance: Option<f64> = Some(None),
);
unit_enum!(WorkspaceGridStyle;
0 => Lines,
1 => Dots,
);
unit_enum!(WorkspaceSnapStyle;
0 => Rectangular,
1 => Isometric,
);
unit_enum!(WorkspaceIsometricPlane;
0 => Left,
1 => Top,
2 => Right,
);
unit_enum!(WorkspaceClipMode;
0 => Disabled,
1 => AtCamera,
2 => AtDistance,
);
record!(IfccadViewport;
0 => workspace: Option<Box<IfccadViewportWorkspace>> = Some(None),
1 => model_id: u64 = None,
2 => frame: IfccadViewportFrame = None,
3 => view: IfccadViewportView = None,
4 => render_mode: IfccadViewportRenderMode = None,
5 => view_enabled: bool = None,
6 => view_locked: bool = None,
7 => paper_clip: IfccadViewportPaperClip = None,
8 => layer_overrides: Vec<IfccadViewportLayerOverride> = None,
9 => plot_shading_override: Option<ShadedPlotMode> = Some(None),
);
record!(WorkspaceCanvasFrame;
0 => center: Point3 = None,
1 => width: f64 = None,
2 => height: f64 = None,
);
impl CborValue for IfccadPaperContext {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Canvas"), (1, "Viewport")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Canvas => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Viewport(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Canvas,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: u64 = b.take(0, None)?;
                b.finish()?;
                Self::Viewport(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Canvas, Self::Canvas) => true,
            (Self::Viewport(value_0), Self::Viewport(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
record!(IfccadViewportFrame;
0 => center: [f64;2] = None,
1 => width: f64 = None,
2 => height: f64 = None,
);
record!(IfccadViewportView;
0 => center: [f64;2] = None,
1 => target: [f64;3] = None,
2 => direction: [f64;3] = None,
3 => height: f64 = None,
4 => twist: f64 = None,
5 => projection: IfccadViewportProjection = None,
6 => lens_length_mm: Option<f64> = Some(None),
7 => front_clip: IfccadViewportDepthClip = None,
8 => back_clip: IfccadViewportDepthClip = None,
);
unit_enum!(IfccadViewportRenderMode;
0 => TwoDimensional,
1 => Wireframe,
2 => HiddenLine,
3 => FlatShadedWithoutEdges,
4 => FlatShadedWithEdges,
5 => SmoothShadedWithoutEdges,
6 => SmoothShadedWithEdges,
);
record!(IfccadViewportPaperClip;
0 => enabled: bool = None,
1 => boundary_entity_id: Option<u64> = Some(None),
);
unit_enum!(IfccadViewportProjection;
0 => Orthographic,
1 => Perspective,
);
record!(IfccadViewportDepthClip;
0 => mode: IfccadViewportClipMode = None,
1 => distance: Option<f64> = Some(None),
);
record!(IfccadViewportLayerOverride;
0 => layer_id: u64 = None,
1 => frozen: bool = None,
2 => color: Option<IfccadColor> = Some(None),
3 => opacity: Option<f64> = Some(None),
4 => line_pattern_id: Option<IfccadLinePatternId> = Some(None),
5 => line_weight: Option<f64> = Some(None),
);
record!(IfccadViewportWorkspace;
0 => grid: WorkspaceGrid = None,
1 => snap: WorkspaceSnap = None,
2 => stored_ucs: IfccadUcsSelection = None,
3 => use_stored_ucs: bool = None,
);
unit_enum!(IfccadViewportClipMode;
0 => Disabled,
1 => AtCamera,
2 => AtDistance,
);
