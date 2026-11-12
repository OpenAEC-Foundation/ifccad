//! Native drawing table placement and ordered cell data.
use super::{
    IfccadColor, IfccadDocument, IfccadEntityKind, IfccadLinePatternId, IfccadMode,
    IfccadPlacement, IfccadReport,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IfccadTableStyleId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadTableStyle {
    pub id: IfccadTableStyleId,
    pub settings: IfccadTableStyleSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableStyleSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_style: Option<IfccadNamedTableCellStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotative: Option<bool>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub flow_direction: IfccadTableFlowDirection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margins: Option<IfccadTableMargins>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_suppressed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_suppressed: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_styles: Vec<IfccadNamedTableCellStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<IfccadTableGrid>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadNamedTableCellStyle {
    pub id: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<IfccadTableCellStyleRole>,
    pub format: IfccadTableCellFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableFlowDirection {
    TopToBottom,
    BottomToTop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableCellStyleRole {
    Title,
    Header,
    Data,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadTable {
    pub placement: IfccadPlacement,
    pub grid: IfccadTableGrid,
    pub graphics: Option<super::IfccadDimensionGraphics>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableGrid {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_direction: Option<IfccadTableFlowDirection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breaks: Option<IfccadTableBreaks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_id: Option<IfccadTableStyleId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_links: Vec<super::IfccadTableDataLinkRange>,
    pub columns: Vec<IfccadTableColumn>,
    pub rows: Vec<IfccadTableRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merged_ranges: Vec<IfccadTableRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<IfccadTableCellFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableColumn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_value: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_data: Vec<super::IfccadTableCustomValue>,
    pub width: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<IfccadTableCellFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_value: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_data: Vec<super::IfccadTableCustomValue>,
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_id: Option<u32>,
    pub cells: Vec<IfccadTableCell>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<IfccadTableCellFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableCell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<IfccadTableCellLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state: Vec<IfccadTableCellState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_value: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_data: Vec<super::IfccadTableCustomValue>,
    pub contents: Vec<IfccadTableCellContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<IfccadTableCellFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableCellLayout {
    pub scale: f64,
    pub width_with_gap: f64,
    pub height_with_gap: f64,
    pub data_flags: i32,
    pub flags: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<IfccadTableItemLayout>,
    pub items: Vec<IfccadTableItemLayout>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableItemLayout {
    pub distance_to_top_left: [f64; 3],
    pub distance_to_center: [f64; 3],
    pub width: f64,
    pub height: f64,
    pub outer_width: f64,
    pub outer_height: f64,
    pub flags: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableCellState {
    ContentLocked,
    ContentReadOnly,
    Linked,
    ContentModifiedAfterUpdate,
    FormatLocked,
    FormatReadOnly,
    FormatModifiedAfterUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableBreakDirection {
    Right,
    Down,
    Left,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBreaks {
    pub direction: IfccadTableBreakDirection,
    pub spacing: f64,
    pub enabled: bool,
    pub manual_positions: bool,
    pub manual_heights: bool,
    pub repeat_top_labels: bool,
    pub repeat_bottom_labels: bool,
    pub parts: Vec<IfccadTableBreakPart>,
    pub ranges: Vec<IfccadTableBreakRange>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBreakPart {
    pub position: [f64; 3],
    pub height: f64,
    pub flags: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBreakRange {
    pub position: [f64; 3],
    pub start_row: usize,
    pub end_row: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableCellFormat {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<IfccadTableValueKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_height: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_style_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_color: Option<IfccadMode<IfccadColor>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<IfccadMode<IfccadColor>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<IfccadTableAlignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_scale: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_fit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_unit: Option<IfccadTableUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margins: Option<IfccadTableMargins>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_layout: Option<IfccadTableContentLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_inner_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_inner_border: Option<IfccadTableBorder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_on_insert: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBorder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<IfccadMode<IfccadColor>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_weight: Option<IfccadMode<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_pattern: Option<IfccadMode<IfccadLinePatternId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_kind: Option<IfccadTableBorderKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub double_spacing: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableMargins {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_spacing: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_spacing: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableAlignment {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableUnit {
    Unitless,
    Distance,
    Angle,
    Area,
    Volume,
    Currency,
    Percentage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableContentLayout {
    Flow,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableBorderKind {
    Single,
    Double,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum IfccadTableCellContent {
    Value {
        value: IfccadTableValue,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<IfccadTableUnit>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value_format: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<Box<IfccadTableCellFormat>>,
    },
    Block {
        block: Box<IfccadTableBlockContent>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<Box<IfccadTableCellFormat>>,
    },
    Field {
        field: Box<super::IfccadTableFieldGraph>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<Box<IfccadTableCellFormat>>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBlockContent {
    pub definition_id: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<IfccadTableBlockAttribute>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableBlockAttribute {
    pub definition_id: u64,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum IfccadTableValue {
    Text(String),
    Integer(i64),
    Real(f64),
    #[serde(rename = "point2d")]
    Point2D([f64; 2]),
    #[serde(rename = "point3d")]
    Point3D([f64; 3]),
    Unset(IfccadTableValueKind),
    Date(Vec<u8>),
    Binary(Vec<u8>),
    ResultBuffer(Vec<u8>),
    General(String),
    Object(u64),
    Boolean(bool),
    UnspecifiedNumber(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadTableValueKind {
    Text,
    Integer,
    Real,
    #[serde(rename = "point2d")]
    Point2D,
    #[serde(rename = "point3d")]
    Point3D,
    Date,
    Binary,
    ResultBuffer,
    General,
    Object,
    Boolean,
    UnspecifiedNumber,
}

/// Inclusive, zero-based rectangle in the ordered table grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadTableRange {
    pub top_row: usize,
    pub left_column: usize,
    pub bottom_row: usize,
    pub right_column: usize,
}

impl IfccadTableRange {
    pub(super) fn fits(&self, rows: usize, columns: usize) -> bool {
        self.top_row <= self.bottom_row
            && self.left_column <= self.right_column
            && self.bottom_row < rows
            && self.right_column < columns
    }
}

impl IfccadTable {
    pub(super) fn validate_at(
        &self,
        document: &IfccadDocument,
        path: &str,
    ) -> Result<(), IfccadReport> {
        self.placement.coordinate_frame()?;
        if let Some(graphics) = &self.graphics {
            if graphics
                .parts
                .iter()
                .any(|part| part.purpose != super::IfccadDimensionGraphicsPurpose::Display)
            {
                return Err(table_error(
                    path,
                    "table graphics parts require display purpose",
                ));
            }
        }
        self.grid.validate_at(document, None, path)
    }
}

impl IfccadTableGrid {
    pub(super) fn referenced_blocks(&self) -> impl Iterator<Item = u64> + '_ {
        self.items().filter_map(|item| match item {
            IfccadTableCellContent::Block { block, .. } => Some(block.definition_id),
            _ => None,
        })
    }

    fn items(&self) -> impl Iterator<Item = &IfccadTableCellContent> {
        self.rows
            .iter()
            .flat_map(|row| &row.cells)
            .flat_map(|cell| &cell.contents)
    }

    fn validate_at(
        &self,
        document: &IfccadDocument,
        default_style: Option<IfccadTableStyleId>,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let fail = |detail| table_error(path, detail);
        let height = self.rows.len();
        let width = self.columns.len();
        if height == 0 || width == 0 {
            return Err(fail("empty table grid"));
        }
        let positive = |v: f64| v.is_finite() && v > 0.0;
        if self.columns.iter().any(|column| !positive(column.width))
            || self
                .rows
                .iter()
                .any(|row| !positive(row.height) || row.cells.len() != width)
        {
            return Err(fail("invalid table dimensions or row length"));
        }
        let named_styles = self
            .style_id
            .or(default_style)
            .map(|id| {
                document
                    .table_styles
                    .iter()
                    .find(|style| style.id == id)
                    .map(|style| style.settings.cell_styles.as_slice())
                    .ok_or_else(|| fail("unresolved table style"))
            })
            .transpose()?
            .unwrap_or_default();
        for (id, format) in self
            .columns
            .iter()
            .map(|column| (column.style_id, &column.format))
            .chain(self.rows.iter().map(|row| (row.style_id, &row.format)))
            .chain(
                self.rows
                    .iter()
                    .flat_map(|row| &row.cells)
                    .map(|cell| (cell.style_id, &cell.format)),
            )
        {
            if id.is_some_and(|id| !named_styles.iter().any(|style| style.id == id)) {
                return Err(fail("unresolved named cell style"));
            }
            if let Some(format) = format {
                format.validate_at(document, path)?;
            }
        }
        if let Some(format) = &self.format {
            format.validate_at(document, path)?;
        }
        let metadata = self
            .columns
            .iter()
            .flat_map(|column| &column.custom_data)
            .chain(self.rows.iter().flat_map(|row| &row.custom_data))
            .chain(
                self.rows
                    .iter()
                    .flat_map(|row| &row.cells)
                    .flat_map(|cell| &cell.custom_data),
            );
        for item in metadata {
            item.value.validate_at(document, path)?;
        }
        if let Some(breaks) = &self.breaks {
            if !breaks.spacing.is_finite()
                || breaks.spacing < 0.0
                || breaks.parts.iter().any(|part| {
                    !part.height.is_finite()
                        || part.height < 0.0
                        || part
                            .position
                            .iter()
                            .any(|coordinate| !coordinate.is_finite())
                })
                || breaks.ranges.iter().any(|range| {
                    range.start_row > range.end_row
                        || range.end_row >= height
                        || range
                            .position
                            .iter()
                            .any(|coordinate| !coordinate.is_finite())
                })
            {
                return Err(fail("invalid table break settings or ranges"));
            }
        }
        for (row_index, row) in self.rows.iter().enumerate() {
            for (column_index, cell) in row.cells.iter().enumerate() {
                if let Some(layout) = &cell.layout {
                    if ![layout.scale, layout.width_with_gap, layout.height_with_gap]
                        .into_iter()
                        .all(f64::is_finite)
                        || layout.anchor.iter().chain(&layout.items).any(|item| {
                            ![item.width, item.height, item.outer_width, item.outer_height]
                                .into_iter()
                                .chain(item.distance_to_top_left)
                                .chain(item.distance_to_center)
                                .all(f64::is_finite)
                        })
                    {
                        return Err(fail("nonfinite cached table cell layout"));
                    }
                }
                if cell.state.iter().collect::<BTreeSet<_>>().len() != cell.state.len() {
                    return Err(fail("duplicate table cell state"));
                }
                for (item_index, item) in cell.contents.iter().enumerate() {
                    item.validate_at(
                        document,
                        self,
                        &format!(
                            "{path}/rows/{row_index}/cells/{column_index}/contents/{item_index}"
                        ),
                    )?;
                }
            }
        }
        for link in &self.data_links {
            if !link.range.fits(height, width) {
                return Err(fail("table datalink range outside grid"));
            }
            if !document
                .table_data_links
                .iter()
                .any(|resource| resource.id == link.link_id)
            {
                return Err(fail("unresolved table datalink resource"));
            }
        }
        if !self.merged_ranges.is_empty() {
            let count = height
                .checked_mul(width)
                .ok_or_else(|| fail("table grid overflow"))?;
            let mut occupied = vec![false; count];
            for range in &self.merged_ranges {
                let IfccadTableRange {
                    top_row,
                    left_column,
                    bottom_row,
                    right_column,
                } = *range;
                if !range.fits(height, width)
                    || (top_row == bottom_row && left_column == right_column)
                {
                    return Err(fail("invalid merged cell range"));
                }
                for row in top_row..=bottom_row {
                    for covered in
                        &mut occupied[row * width + left_column..=row * width + right_column]
                    {
                        if std::mem::replace(covered, true) {
                            return Err(fail("merged cell ranges overlap"));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl IfccadTableCellContent {
    fn validate_at(
        &self,
        document: &IfccadDocument,
        grid: &IfccadTableGrid,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let format = match self {
            Self::Value { value, format, .. } => {
                value.validate_at(document, path)?;
                format
            }
            Self::Block { block, format } => {
                let definition = document
                    .blocks
                    .iter()
                    .find(|definition| definition.id == block.definition_id)
                    .ok_or_else(|| table_error(path, "unresolved table cell block definition"))?;
                let mut bindings = BTreeSet::new();
                for attribute in &block.attributes {
                    if !bindings.insert(attribute.definition_id)
                        || !definition
                            .entities
                            .iter()
                            .filter_map(super::IfccadEntity::as_native)
                            .any(|entity| {
                                entity.id == attribute.definition_id
                                    && matches!(
                                        entity.kind,
                                        IfccadEntityKind::AttributeDefinition(_)
                                    )
                            })
                    {
                        return Err(table_error(path, "invalid table block attribute binding"));
                    }
                }
                format
            }
            Self::Field { field, format, .. } => {
                field.validate_at(document, grid, path)?;
                format
            }
        };
        if let Some(format) = format {
            format.validate_at(document, path)?;
        }
        Ok(())
    }
}

impl IfccadTableValue {
    pub(super) fn validate_at(
        &self,
        document: &IfccadDocument,
        path: &str,
    ) -> Result<(), IfccadReport> {
        let finite = match self {
            Self::Real(v) => v.is_finite(),
            Self::Point2D(v) => v.iter().all(|x| x.is_finite()),
            Self::Point3D(v) => v.iter().all(|x| x.is_finite()),
            Self::Object(id) => {
                if native_entity(document, *id).is_none() {
                    return Err(table_error(path, "unresolved table value object"));
                }
                true
            }
            _ => true,
        };
        if !finite {
            return Err(table_error(path, "non-finite table cell value"));
        }
        Ok(())
    }
}

pub(super) fn validate_styles(document: &IfccadDocument) -> Result<(), IfccadReport> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for style in &document.table_styles {
        let path = format!("/cad/d{}/tableStyle/{}", document.drawing_id, style.id.0);
        let settings = &style.settings;
        if style.id.0 == 0
            || !ids.insert(style.id)
            || settings.name.trim().is_empty()
            || !names.insert(settings.name.to_lowercase())
        {
            return Err(table_error(
                &path,
                "invalid or duplicate table style ID or name",
            ));
        }
        if let Some(margins) = &settings.margins {
            margins.validate_at(&path)?;
        }
        let mut cell_ids = BTreeSet::new();
        let mut cell_names = BTreeSet::new();
        let mut roles = BTreeSet::new();
        for cell in settings.base_style.iter().chain(&settings.cell_styles) {
            if !cell_ids.insert(cell.id)
                || cell.name.trim().is_empty()
                || !cell_names.insert(cell.name.to_lowercase())
                || cell.role.is_some_and(|role| !roles.insert(role))
            {
                return Err(table_error(&path, "invalid or duplicate named cell style"));
            }
            cell.format.validate_at(document, &path)?;
        }
        if let Some(template) = &settings.template {
            template.validate_at(document, Some(style.id), &format!("{path}/template"))?;
        }
    }
    Ok(())
}

impl IfccadTableCellFormat {
    fn validate_at(&self, document: &IfccadDocument, path: &str) -> Result<(), IfccadReport> {
        if [self.text_height, self.block_scale]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite() || value <= 0.0)
            || self.rotation.is_some_and(|value| !value.is_finite())
        {
            return Err(table_error(path, "invalid table content size or rotation"));
        }
        if self
            .text_style_id
            .is_some_and(|id| !document.text_styles.iter().any(|style| style.id.0 == id))
        {
            return Err(table_error(path, "unresolved table text style"));
        }
        if let Some(margins) = &self.margins {
            margins.validate_at(path)?;
        }
        for border in [
            &self.top_border,
            &self.right_border,
            &self.bottom_border,
            &self.left_border,
            &self.horizontal_inner_border,
            &self.vertical_inner_border,
        ]
        .into_iter()
        .flatten()
        {
            if border
                .double_spacing
                .is_some_and(|value| !value.is_finite() || value < 0.0)
                || matches!(border.line_weight, Some(IfccadMode::Explicit(value))
                    if !value.is_finite() || value < 0.0)
            {
                return Err(table_error(path, "invalid table border spacing or weight"));
            }
            if matches!(border.line_pattern, Some(IfccadMode::Explicit(id))
                if !document.line_patterns.iter().any(|pattern| pattern.id == id))
            {
                return Err(table_error(path, "unresolved table border line pattern"));
            }
        }
        Ok(())
    }
}

impl IfccadTableMargins {
    fn validate_at(&self, path: &str) -> Result<(), IfccadReport> {
        if [
            self.top,
            self.right,
            self.bottom,
            self.left,
            self.horizontal_spacing,
            self.vertical_spacing,
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || value < 0.0)
        {
            return Err(table_error(path, "invalid table margin or spacing"));
        }
        Ok(())
    }
}

pub(super) fn native_entity(
    document: &IfccadDocument,
    id: u64,
) -> Option<&super::IfccadNativeEntity> {
    document
        .model
        .entities
        .iter()
        .chain(
            document
                .paper_layouts
                .iter()
                .flat_map(|layout| &layout.entities),
        )
        .chain(document.blocks.iter().flat_map(|block| &block.entities))
        .filter_map(super::IfccadEntity::as_native)
        .find(|entity| entity.id == id)
}

pub(super) fn table_error(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-TABLE-001", path, detail)
}
