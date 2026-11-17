use super::*;
use crate::plot_kernel::{LayoutOutputSettings, PlotStyleMode};
record!(IfccadDocument;
0 => header: IfccadHeader = None,
1 => drawing_id: u64 = None,
2 => id_counters: IfccadIdCounters = None,
3 => length_unit: String = None,
4 => plot_style_mode: PlotStyleMode = None,
5 => model: IfccadLayout = None,
6 => layers: Vec<IfccadLayer> = None,
7 => blocks: Vec<IfccadBlockDefinition> = Some(Vec::new()),
8 => paper_layouts: Vec<IfccadPaperLayout> = Some(Vec::new()),
9 => line_patterns: Vec<IfccadLinePattern> = None,
10 => line_pattern_scale: f64 = Some(1.0),
11 => symbol_sources: Vec<IfccadSymbolSource> = Some(Vec::new()),
12 => text_styles: Vec<IfccadTextStyle> = Some(Vec::new()),
13 => dimension_styles: Vec<IfccadDimensionStyle> = Some(Vec::new()),
14 => ucs_definitions: Vec<IfccadUcsDefinition> = Some(Vec::new()),
15 => model_windows: Vec<IfccadModelWindow> = Some(Vec::new()),
16 => point_display: Option<IfccadPointDisplay> = Some(None),
17 => workspace_state: Option<IfccadDrawingWorkspaceState> = Some(None),
18 => model_view_state: Option<IfccadModelViewState> = Some(None),
19 => preservation: Option<IfccadPreservation> = Some(None),
20 => annotation_scales: Vec<IfccadAnnotationScale> = Some(Vec::new()),
21 => annotation_views:Vec<IfccadAnnotationStoredView>=Some(Vec::new()),
22 => annotation_settings:Option<IfccadDrawingAnnotationSettings>=Some(None),
23 => attribute_visibility:IfccadAttributeVisibility=Some(IfccadAttributeVisibility::Normal),
24 => multi_leader_styles:Vec<IfccadMultiLeaderStyle>=Some(Vec::new()),
25 => table_styles: Vec<IfccadTableStyle> = Some(Vec::new()),
26 => table_data_links: Vec<IfccadTableDataLink> = Some(Vec::new()),
27 => data_tables: Vec<IfccadDataTable> = Some(Vec::new()),
28 => plot_styles: IfccadPlotStyles = Some(Default::default()),
);
record!(IfccadPlotStyles;
0 => next_source_id: u64 = Some(1_u64),
1 => sources: std::collections::BTreeMap<u64, IfccadPlotStyleSource> = Some(Default::default()),
2 => layout_sources: std::collections::BTreeMap<u64, u64> = Some(Default::default()),
3 => layer_assignments: std::collections::BTreeMap<u64, IfccadPlotStyle> = Some(Default::default()),
4 => entity_assignments: std::collections::BTreeMap<u64, IfccadPlotStyle> = Some(Default::default()),
);
record!(IfccadPlotStyleSource;
0 => kind: IfccadPlotStyleTableKind = None,
1 => file_name: String = None,
2 => sha256: Option<String> = Some(None),
);
unit_enum!(IfccadPlotStyleTableKind;
0 => Ctb,
1 => Stb,
);
impl CborValue for IfccadPlotStyle {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "kind"), (1, "name")]
    }

    fn encode(&self, writer: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let (kind, name) = match self {
            Self::Normal => (0_u64, None),
            Self::ByLayer => (1, None),
            Self::ByBlock => (2, None),
            Self::Named { name } => (3, Some(name)),
        };
        writer.map(if name.is_some() { 2 } else { 1 })?;
        writer.u64(0)?;
        kind.encode(writer)?;
        if let Some(name) = name {
            writer.u64(1)?;
            name.encode(writer)?;
        }
        Ok(())
    }

    fn decode(reader: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut fields = RecordFields::new(reader, &[(0, "kind"), (1, "name")])?;
        let kind = fields.take::<u64>(0, None)?;
        let name = fields.take::<Option<String>>(1, Some(None))?;
        fields.finish()?;
        match (kind, name) {
            (0, None) => Ok(Self::Normal),
            (1, None) => Ok(Self::ByLayer),
            (2, None) => Ok(Self::ByBlock),
            (3, Some(name)) => Ok(Self::Named { name }),
            _ => Err(reader.error("invalid plot-style assignment")),
        }
    }

    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}
record!(IfccadLayout;
0 => bounds_quality: Option<IfccadBoundsQuality> = Some(None),
1 => settings: LayoutOutputSettings = None,
2 => id: u64 = None,
3 => tab_index: u32 = None,
4 => bounds: Option<IfccadBounds3d> = Some(None),
5 => entities: Vec<IfccadEntity> = None,
6 => annotation_state:Option<IfccadLayoutAnnotationState>=Some(None),
);
record!(IfccadBlockDefinition;
0 => description: String = None,
1 => anonymous: bool = None,
2 => explodable: bool = None,
3 => uniform_scaling: bool = None,
4 => bounds_quality: Option<IfccadBoundsQuality> = Some(None),
5 => id: u64 = None,
6 => name: String = None,
7 => base_point: [f64;3] = None,
8 => insertion_unit: String = None,
9 => bounds: Option<IfccadBounds3d> = Some(None),
10 => entities: Vec<IfccadEntity> = None,
11 => annotation: IfccadAnnotationDefinition = Some(Default::default()),
);
record!(IfccadPaperLayout;
0 => bounds_quality: Option<IfccadBoundsQuality> = Some(None),
1 => canvas: Option<IfccadPaperCanvas> = Some(None),
2 => id: u64 = None,
3 => name: String = None,
4 => tab_index: u32 = None,
5 => settings: LayoutOutputSettings = None,
6 => bounds: Option<IfccadBounds3d> = Some(None),
7 => entities: Vec<IfccadEntity> = None,
8 => annotation_state:Option<IfccadLayoutAnnotationState>=Some(None),
);
