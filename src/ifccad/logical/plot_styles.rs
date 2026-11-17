use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IfccadPlotStyleTableKind {
    Ctb,
    Stb,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadPlotStyleSource {
    pub kind: IfccadPlotStyleTableKind,
    pub file_name: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum IfccadPlotStyle {
    Normal,
    ByLayer,
    ByBlock,
    Named { name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadPlotStyles {
    pub next_source_id: u64,
    pub sources: BTreeMap<u64, IfccadPlotStyleSource>,
    pub layout_sources: BTreeMap<u64, u64>,
    pub layer_assignments: BTreeMap<u64, IfccadPlotStyle>,
    pub entity_assignments: BTreeMap<u64, IfccadPlotStyle>,
}

impl Default for IfccadPlotStyles {
    fn default() -> Self {
        Self {
            next_source_id: 1,
            sources: BTreeMap::new(),
            layout_sources: BTreeMap::new(),
            layer_assignments: BTreeMap::new(),
            entity_assignments: BTreeMap::new(),
        }
    }
}

fn valid_source(source: &IfccadPlotStyleSource) -> bool {
    let extension = match source.kind {
        IfccadPlotStyleTableKind::Ctb => "ctb",
        IfccadPlotStyleTableKind::Stb => "stb",
    };
    let basename = &source.file_name;
    !basename
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | ':'))
        && basename.rsplit_once('.').is_some_and(|(stem, ext)| {
            !stem.trim().is_empty()
                && !matches!(stem, "." | "..")
                && ext.eq_ignore_ascii_case(extension)
        })
        && source.sha256.as_ref().is_none_or(|sha| {
            sha.len() == 64
                && sha
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

fn valid_style(style: &IfccadPlotStyle, layer: bool) -> bool {
    match style {
        IfccadPlotStyle::ByLayer | IfccadPlotStyle::ByBlock => !layer,
        IfccadPlotStyle::Normal => true,
        IfccadPlotStyle::Named { name } => {
            !name.trim().is_empty()
                && !name.chars().any(char::is_control)
                && !["Normal", "ByLayer", "ByBlock"]
                    .iter()
                    .any(|reserved| name.eq_ignore_ascii_case(reserved))
        }
    }
}

pub(super) fn validate(drawing: &IfccadDocument) -> Result<(), IfccadReport> {
    let plot = &drawing.plot_styles;
    let prefix = format!("/cad/d{}", drawing.drawing_id);
    let problem = |path: String, message: &str| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-PLOT-STYLE-001",
            &format!("{prefix}/{path}"),
            message,
        )
    };
    if plot.next_source_id == 0 {
        return Err(problem(
            "ifccad::drawing/nextPlotStyleSourceId".into(),
            "zero allocation watermark",
        ));
    }
    for (&id, source) in &plot.sources {
        if id >= plot.next_source_id || !valid_source(source) {
            return Err(problem(
                format!("plotStyleSource/{id}"),
                "invalid plot-style source or allocation ID",
            ));
        }
    }
    let named = drawing.plot_style_mode == IfccadPlotStyleMode::Named;
    let expected = if named {
        IfccadPlotStyleTableKind::Stb
    } else {
        IfccadPlotStyleTableKind::Ctb
    };
    for (&layout, source) in &plot.layout_sources {
        if !(drawing.model.id == layout
            || drawing.paper_layouts.iter().any(|paper| paper.id == layout))
            || !plot
                .sources
                .get(source)
                .is_some_and(|table| table.kind == expected)
        {
            return Err(problem(
                format!("layout/{layout}/ifccad::layout/plotStyleSource"),
                "unresolved or incompatible plot-style source",
            ));
        }
    }
    for (&id, style) in &plot.layer_assignments {
        if !named || !valid_style(style, true) || !drawing.layers.iter().any(|layer| layer.id == id)
        {
            return Err(problem(
                format!("layer/{id}/ifccad::layer/plotStyle"),
                "invalid layer plot-style assignment",
            ));
        }
    }
    if !plot.entity_assignments.is_empty() {
        let entities: std::collections::BTreeSet<_> = drawing
            .model
            .entities
            .iter()
            .chain(
                drawing
                    .paper_layouts
                    .iter()
                    .flat_map(|paper| &paper.entities),
            )
            .chain(drawing.blocks.iter().flat_map(|block| &block.entities))
            .filter_map(IfccadEntity::as_native)
            .map(|entity| entity.id)
            .collect();
        for (&id, style) in &plot.entity_assignments {
            if !named || !valid_style(style, false) || !entities.contains(&id) {
                return Err(problem(
                    format!("e{id}/ifccad::entity/plotStyle"),
                    "invalid object plot-style assignment",
                ));
            }
        }
    }
    Ok(())
}
