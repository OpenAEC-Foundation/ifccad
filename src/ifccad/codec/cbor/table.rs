use super::*;

impl CborValue for IfccadTableStyleId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        u64::decode(r).map(Self)
    }

    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}

record!(IfccadTableStyle;
    0 => id: IfccadTableStyleId = None,
    1 => settings: IfccadTableStyleSettings = None,
);
record!(IfccadTableStyleSettings;
    0 => name: String = None,
    1 => description: Option<String> = Some(None),
    2 => flow_direction: IfccadTableFlowDirection = None,
    3 => margins: Option<IfccadTableMargins> = Some(None),
    4 => title_suppressed: Option<bool> = Some(None),
    5 => header_suppressed: Option<bool> = Some(None),
    6 => cell_styles: Vec<IfccadNamedTableCellStyle> = Some(Vec::new()),
    7 => template: Option<IfccadTableGrid> = Some(None),
    8 => base_style: Option<IfccadNamedTableCellStyle> = Some(None),
    9 => annotative: Option<bool> = Some(None),
);
record!(IfccadNamedTableCellStyle;
    0 => id: u32 = None,
    1 => name: String = None,
    2 => role: Option<IfccadTableCellStyleRole> = Some(None),
    3 => format: IfccadTableCellFormat = None,
);
unit_enum!(IfccadTableFlowDirection; 0 => TopToBottom, 1 => BottomToTop);
unit_enum!(IfccadTableCellStyleRole; 0 => Title, 1 => Header, 2 => Data);

record!(IfccadTable;
    0 => placement: IfccadPlacement = None,
    1 => grid: IfccadTableGrid = None,
    2 => graphics: Option<IfccadDimensionGraphics> = Some(None),
);
impl CborValue for IfccadTableDataLinkId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        u64::decode(r).map(Self)
    }
    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}
record!(IfccadTableDataLink;
    0 => id: IfccadTableDataLinkId = None,
    1 => settings: IfccadTableDataLinkSettings = None,
);
impl CborValue for IfccadDataTableId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        u64::decode(r).map(Self)
    }
    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}
record!(IfccadDataTable;
    0 => id: IfccadDataTableId = None,
    1 => settings: IfccadDataTableSettings = None,
);
record!(IfccadTableDataLinkSettings;
    0 => name: String = None,
    1 => adapter: String = None,
    2 => source: String = None,
    3 => source_selection: Option<String> = Some(None),
    4 => description: Option<String> = Some(None),
    5 => tooltip: Option<String> = Some(None),
    6 => options: i32 = None,
    7 => update_options: i32 = None,
    8 => path_option: i16 = None,
    9 => status_flags: i32 = None,
    10 => flags: i32 = None,
    11 => last_update: Option<[i16; 7]> = Some(None),
    12 => update_status: Option<String> = Some(None),
    13 => custom_data: Vec<IfccadTableCustomValue> = None,
    14 => custom_targets: Vec<IfccadTableDataLinkTarget> = None,
);
record!(IfccadTableDataLinkTarget;
    0 => object_id: u64 = None,
    1 => value: String = None,
);
record!(IfccadTableDataLinkRange;
    0 => link_id: IfccadTableDataLinkId = None,
    1 => range: IfccadTableRange = None,
);
record!(IfccadTableCustomValue;
    0 => name: String = None,
    1 => value: IfccadTableValue = None,
);
record!(IfccadDataTableSettings;
    0 => name: String = None,
    1 => flags: i16 = None,
    2 => row_count: usize = None,
    3 => columns: Vec<IfccadDataTableColumn> = None,
    4 => owner: Option<IfccadDataTableOwner> = Some(None),
);
record!(IfccadDataTableOwner;
    0 => entity_id: Option<u64> = Some(None),
    1 => dictionary_path: Vec<String> = None,
    2 => key: String = None,
    3 => hard_owned: bool = None,
);
record!(IfccadDataTableColumn;
    0 => name: String = None,
    1 => value_type: IfccadDataTableColumnType = None,
    2 => values: Vec<IfccadTableValue> = None,
);
unit_enum!(IfccadDataTableColumnType; 0 => Integer, 1 => Real, 2 => Text, 3 => Point, 4 => Object, 5 => HardOwner, 6 => SoftOwner, 7 => HardPointer, 8 => SoftPointer, 9 => Boolean, 10 => Vector);
record!(IfccadTableGrid;
    0 => columns: Vec<IfccadTableColumn> = None,
    1 => rows: Vec<IfccadTableRow> = None,
    2 => merged_ranges: Vec<IfccadTableRange> = Some(Vec::new()),
    3 => format: Option<IfccadTableCellFormat> = Some(None),
    4 => style_id: Option<IfccadTableStyleId> = Some(None),
    5 => data_links: Vec<IfccadTableDataLinkRange> = Some(Vec::new()),
    6 => name: Option<String> = Some(None),
    7 => description: Option<String> = Some(None),
    8 => flow_direction: Option<IfccadTableFlowDirection> = Some(None),
    9 => breaks: Option<IfccadTableBreaks> = Some(None),
);
record!(IfccadTableColumn;
    0 => width: f64 = None,
    1 => format: Option<IfccadTableCellFormat> = Some(None),
    2 => style_id: Option<u32> = Some(None),
    3 => name: Option<String> = Some(None),
    4 => custom_value: Option<i32> = Some(None),
    5 => custom_data: Vec<IfccadTableCustomValue> = Some(Vec::new()),
);
record!(IfccadTableRow;
    0 => height: f64 = None,
    1 => cells: Vec<IfccadTableCell> = None,
    2 => format: Option<IfccadTableCellFormat> = Some(None),
    3 => style_id: Option<u32> = Some(None),
    4 => custom_value: Option<i32> = Some(None),
    5 => custom_data: Vec<IfccadTableCustomValue> = Some(Vec::new()),
);
record!(IfccadTableCell;
    0 => contents: Vec<IfccadTableCellContent> = None,
    1 => format: Option<IfccadTableCellFormat> = Some(None),
    2 => style_id: Option<u32> = Some(None),
    3 => tooltip: Option<String> = Some(None),
    4 => state: Vec<IfccadTableCellState> = Some(Vec::new()),
    5 => custom_value: Option<i32> = Some(None),
    6 => custom_data: Vec<IfccadTableCustomValue> = Some(Vec::new()),
    7 => layout: Option<IfccadTableCellLayout> = Some(None),
);
record!(IfccadTableCellLayout;
    0 => scale: f64 = None,
    1 => width_with_gap: f64 = None,
    2 => height_with_gap: f64 = None,
    3 => data_flags: i32 = None,
    4 => flags: i32 = None,
    5 => anchor: Option<IfccadTableItemLayout> = Some(None),
    6 => items: Vec<IfccadTableItemLayout> = None,
);
record!(IfccadTableItemLayout;
    0 => distance_to_top_left: [f64; 3] = None,
    1 => distance_to_center: [f64; 3] = None,
    2 => width: f64 = None,
    3 => height: f64 = None,
    4 => outer_width: f64 = None,
    5 => outer_height: f64 = None,
    6 => flags: i32 = None,
);
unit_enum!(IfccadTableCellState; 0 => ContentLocked, 1 => ContentReadOnly, 2 => Linked, 3 => ContentModifiedAfterUpdate, 4 => FormatLocked, 5 => FormatReadOnly, 6 => FormatModifiedAfterUpdate);
unit_enum!(IfccadTableBreakDirection; 0 => Right, 1 => Down, 2 => Left);
record!(IfccadTableBreaks;
    0 => direction: IfccadTableBreakDirection = None,
    1 => spacing: f64 = None,
    2 => enabled: bool = None,
    3 => manual_positions: bool = None,
    4 => manual_heights: bool = None,
    5 => repeat_top_labels: bool = None,
    6 => repeat_bottom_labels: bool = None,
    7 => parts: Vec<IfccadTableBreakPart> = None,
    8 => ranges: Vec<IfccadTableBreakRange> = None,
);
record!(IfccadTableBreakPart;
    0 => position: [f64; 3] = None,
    1 => height: f64 = None,
    2 => flags: i32 = None,
);
record!(IfccadTableBreakRange;
    0 => position: [f64; 3] = None,
    1 => start_row: usize = None,
    2 => end_row: usize = None,
);
record!(IfccadTableRange;
    0 => top_row: usize = None,
    1 => left_column: usize = None,
    2 => bottom_row: usize = None,
    3 => right_column: usize = None,
);
record!(IfccadTableBlockContent;
    0 => definition_id: u64 = None,
    1 => attributes: Vec<IfccadTableBlockAttribute> = Some(Vec::new()),
);
record!(IfccadTableBlockAttribute;
    0 => definition_id: u64 = None,
    1 => value: String = None,
);

record!(IfccadTableCellFormat;
    0 => text_height: Option<f64> = Some(None),
    1 => text_style_id: Option<u64> = Some(None),
    2 => content_color: Option<IfccadMode<IfccadColor>> = Some(None),
    3 => fill_color: Option<IfccadMode<IfccadColor>> = Some(None),
    4 => fill_enabled: Option<bool> = Some(None),
    5 => alignment: Option<IfccadTableAlignment> = Some(None),
    6 => rotation: Option<f64> = Some(None),
    7 => block_scale: Option<f64> = Some(None),
    8 => auto_fit: Option<bool> = Some(None),
    9 => value_format: Option<String> = Some(None),
    10 => value_unit: Option<IfccadTableUnit> = Some(None),
    11 => margins: Option<IfccadTableMargins> = Some(None),
    12 => content_layout: Option<IfccadTableContentLayout> = Some(None),
    13 => top_border: Option<IfccadTableBorder> = Some(None),
    14 => right_border: Option<IfccadTableBorder> = Some(None),
    15 => bottom_border: Option<IfccadTableBorder> = Some(None),
    16 => left_border: Option<IfccadTableBorder> = Some(None),
    17 => horizontal_inner_border: Option<IfccadTableBorder> = Some(None),
    18 => vertical_inner_border: Option<IfccadTableBorder> = Some(None),
    19 => merge_on_insert: Option<bool> = Some(None),
    20 => value_type: Option<IfccadTableValueKind> = Some(None),
);

record!(IfccadTableBorder;
    0 => visible: Option<bool> = Some(None),
    1 => color: Option<IfccadMode<IfccadColor>> = Some(None),
    2 => line_weight: Option<IfccadMode<f64>> = Some(None),
    3 => line_pattern: Option<IfccadMode<IfccadLinePatternId>> = Some(None),
    4 => line_kind: Option<IfccadTableBorderKind> = Some(None),
    5 => double_spacing: Option<f64> = Some(None),
);

record!(IfccadTableMargins;
    0 => top: Option<f64> = Some(None),
    1 => right: Option<f64> = Some(None),
    2 => bottom: Option<f64> = Some(None),
    3 => left: Option<f64> = Some(None),
    4 => horizontal_spacing: Option<f64> = Some(None),
    5 => vertical_spacing: Option<f64> = Some(None),
);
unit_enum!(IfccadTableAlignment; 0 => TopLeft, 1 => TopCenter, 2 => TopRight, 3 => MiddleLeft, 4 => MiddleCenter, 5 => MiddleRight, 6 => BottomLeft, 7 => BottomCenter, 8 => BottomRight);
unit_enum!(IfccadTableUnit; 0 => Unitless, 1 => Distance, 2 => Angle, 3 => Area, 4 => Volume, 5 => Currency, 6 => Percentage);
unit_enum!(IfccadTableContentLayout; 0 => Flow, 1 => Horizontal, 2 => Vertical);
unit_enum!(IfccadTableBorderKind; 0 => Single, 1 => Double);
unit_enum!(IfccadTableValueKind; 0 => Text, 1 => Integer, 2 => Real, 3 => Point2D, 4 => Point3D, 6 => Date, 7 => Binary, 8 => ResultBuffer, 9 => General, 10 => Object, 11 => Boolean, 12 => UnspecifiedNumber);

record!(IfccadTableFieldGraph;
    0 => root: u64 = None,
    1 => next_id: u64 = None,
    2 => records: Vec<IfccadTableFieldRecord> = None,
);
record!(IfccadTableFieldRecord;
    0 => id: u64 = None,
    1 => evaluator: String = None,
    2 => code: Vec<IfccadTableFieldCodePart> = None,
    3 => children: Vec<u64> = None,
    4 => object_targets: Vec<u64> = None,
    5 => format: String = None,
    6 => evaluation_options: i32 = None,
    7 => filing_options: i32 = None,
    8 => state: i32 = None,
    9 => evaluation_status: i32 = None,
    10 => error_code: i32 = None,
    11 => error_message: String = None,
    12 => cache: IfccadTableFieldCache = None,
    13 => auxiliary_values: Vec<IfccadTableFieldAuxiliaryValue> = None,
);
record!(IfccadTableFieldCache;
    0 => value: IfccadTableValue = None,
    1 => display: String = None,
    2 => unit: Option<IfccadTableUnit> = Some(None),
    3 => value_format: String = None,
    4 => formatted_value: String = None,
    5 => flags: i32 = None,
);
record!(IfccadTableFieldAuxiliaryValue;
    0 => key: String = None,
    1 => cache: IfccadTableFieldCache = None,
);
record!(IfccadTableCellReference;
    0 => table_id: Option<u64> = Some(None),
    1 => range: IfccadTableRange = None,
);

impl CborValue for IfccadTableFieldCodePart {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.array(2)?;
        match self {
            Self::Literal(v) => {
                w.u64(0)?;
                v.encode(w)
            }
            Self::Child(v) => {
                w.u64(1)?;
                v.encode(w)
            }
            Self::Object(v) => {
                w.u64(2)?;
                v.encode(w)
            }
            Self::Cells(v) => {
                w.u64(3)?;
                v.encode(w)
            }
        }
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let (mut reader, length) = r.sequence()?;
        if length != 2 {
            return Err(r.error("expected table field code pair"));
        }
        let kind = u64::decode(&mut r.child(&mut reader, 0)?)?;
        let mut value = r.child(&mut reader, 1)?;
        match kind {
            0 => String::decode(&mut value).map(Self::Literal),
            1 => u64::decode(&mut value).map(Self::Child),
            2 => u64::decode(&mut value).map(Self::Object),
            3 => IfccadTableCellReference::decode(&mut value).map(Self::Cells),
            _ => Err(r.error("unknown table field code part")),
        }
    }

    fn exact(&self, other: &Self) -> bool {
        self == other
    }
}

fn encode_cell_payload<T: CborValue>(
    kind: u64,
    payload: &T,
    display: Option<&str>,
    format: Option<&IfccadTableCellFormat>,
    writer: &mut CborWriter,
) -> Result<(), IfccadCborEncodeError> {
    writer.map(2 + u64::from(display.is_some()) + u64::from(format.is_some()))?;
    writer.u64(0)?;
    writer.u64(kind)?;
    writer.u64(1)?;
    payload.encode(writer)?;
    if let Some(display) = display {
        writer.u64(2)?;
        writer.str(display)?;
    }
    if let Some(format) = format {
        writer.u64(5)?;
        format.encode(writer)?;
    }
    Ok(())
}

impl CborValue for IfccadTableCellContent {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        let Self::Value {
            value,
            display,
            unit,
            value_format,
            format,
        } = self
        else {
            return match self {
                Self::Block { block, format } => {
                    encode_cell_payload(1, block, None, format.as_deref(), w)
                }
                Self::Field {
                    field,
                    display,
                    format,
                } => encode_cell_payload(2, field, display.as_deref(), format.as_deref(), w),
                Self::Value { .. } => unreachable!("value handled below"),
            };
        };
        w.map(
            2 + u64::from(display.is_some())
                + u64::from(unit.is_some())
                + u64::from(value_format.is_some())
                + u64::from(format.is_some()),
        )?;
        w.u64(0)?;
        w.u64(0)?;
        w.u64(1)?;
        value.encode(w)?;
        if let Some(display) = display {
            w.u64(2)?;
            display.encode(w)?;
        }
        if let Some(unit) = unit {
            w.u64(3)?;
            unit.encode(w)?;
        }
        if let Some(value_format) = value_format {
            w.u64(4)?;
            value_format.encode(w)?;
        }
        if let Some(format) = format {
            w.u64(5)?;
            format.encode(w)?;
        }
        Ok(())
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut fields = RecordFields::new(
            r,
            &[
                (0, "kind"),
                (1, "value"),
                (2, "display"),
                (3, "unit"),
                (4, "valueFormat"),
                (5, "format"),
            ],
        )?;
        let kind: u64 = fields.take(0, None)?;
        if kind == 1 {
            let block = fields.take(1, None)?;
            let format = fields.take(5, Some(None))?;
            fields.finish()?;
            return Ok(Self::Block { block, format });
        }
        if kind == 2 {
            let field = fields.take(1, None)?;
            let display = fields.take(2, Some(None))?;
            let format = fields.take(5, Some(None))?;
            fields.finish()?;
            return Ok(Self::Field {
                field,
                display,
                format,
            });
        }
        if kind != 0 {
            return Err(r.error("unknown table cell content kind"));
        }
        let value = fields.take(1, None)?;
        let display = fields.take(2, Some(None))?;
        let unit = fields.take(3, Some(None))?;
        let value_format = fields.take(4, Some(None))?;
        let format = fields.take(5, Some(None))?;
        fields.finish()?;
        Ok(Self::Value {
            value,
            display,
            unit,
            value_format,
            format,
        })
    }

    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Value {
                    value,
                    display,
                    unit,
                    value_format,
                    format,
                },
                Self::Value {
                    value: other_value,
                    display: other_display,
                    unit: other_unit,
                    value_format: other_value_format,
                    format: other_format,
                },
            ) => {
                value.exact(other_value)
                    && display.exact(other_display)
                    && unit.exact(other_unit)
                    && value_format.exact(other_value_format)
                    && format.exact(other_format)
            }
            (
                Self::Block { block, format },
                Self::Block {
                    block: other_block,
                    format: other_format,
                },
            ) => block.exact(other_block) && format.exact(other_format),
            (
                Self::Field {
                    field,
                    display,
                    format,
                },
                Self::Field {
                    field: other_field,
                    display: other_display,
                    format: other_format,
                },
            ) => {
                field.exact(other_field)
                    && display.exact(other_display)
                    && format.exact(other_format)
            }
            _ => false,
        }
    }
}

impl CborValue for IfccadTableValue {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        w.map(2)?;
        w.u64(0)?;
        w.u64(match self {
            Self::Text(_) => 0,
            Self::Integer(_) => 1,
            Self::Real(_) => 2,
            Self::Point2D(_) => 3,
            Self::Point3D(_) => 4,
            Self::Unset(_) => 5,
            Self::Date(_) => 6,
            Self::Binary(_) => 7,
            Self::ResultBuffer(_) => 8,
            Self::General(_) => 9,
            Self::Object(_) => 10,
            Self::Boolean(_) => 11,
            Self::UnspecifiedNumber(_) => 12,
        })?;
        w.u64(1)?;
        match self {
            Self::Text(v) => v.encode(w),
            Self::Integer(v) => v.encode(w),
            Self::Real(v) => v.encode(w),
            Self::Point2D(v) => v.encode(w),
            Self::Point3D(v) => v.encode(w),
            Self::Unset(v) => v.encode(w),
            Self::Date(v) => v.encode(w),
            Self::Binary(v) => v.encode(w),
            Self::ResultBuffer(v) => v.encode(w),
            Self::General(v) => v.encode(w),
            Self::Object(v) => v.encode(w),
            Self::Boolean(v) => v.encode(w),
            Self::UnspecifiedNumber(v) => v.encode(w),
        }
    }

    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut fields = RecordFields::new(r, &[(0, "kind"), (1, "value")])?;
        let kind: u64 = fields.take(0, None)?;
        let value = match kind {
            0 => Self::Text(fields.take(1, None)?),
            1 => Self::Integer(fields.take(1, None)?),
            2 => Self::Real(fields.take(1, None)?),
            3 => Self::Point2D(fields.take(1, None)?),
            4 => Self::Point3D(fields.take(1, None)?),
            5 => Self::Unset(fields.take(1, None)?),
            6 => Self::Date(fields.take(1, None)?),
            7 => Self::Binary(fields.take(1, None)?),
            8 => Self::ResultBuffer(fields.take(1, None)?),
            9 => Self::General(fields.take(1, None)?),
            10 => Self::Object(fields.take(1, None)?),
            11 => Self::Boolean(fields.take(1, None)?),
            12 => Self::UnspecifiedNumber(fields.take(1, None)?),
            _ => return Err(r.error("unknown table cell value kind")),
        };
        fields.finish()?;
        Ok(value)
    }

    fn exact(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.exact(b),
            (Self::Integer(a), Self::Integer(b)) => a.exact(b),
            (Self::Real(a), Self::Real(b)) => a.exact(b),
            (Self::Point2D(a), Self::Point2D(b)) => a.exact(b),
            (Self::Point3D(a), Self::Point3D(b)) => a.exact(b),
            (Self::Unset(a), Self::Unset(b)) => a.exact(b),
            (Self::Date(a), Self::Date(b)) => a.exact(b),
            (Self::Binary(a), Self::Binary(b)) => a.exact(b),
            (Self::ResultBuffer(a), Self::ResultBuffer(b)) => a.exact(b),
            (Self::General(a), Self::General(b)) => a.exact(b),
            (Self::Object(a), Self::Object(b)) => a.exact(b),
            (Self::Boolean(a), Self::Boolean(b)) => a.exact(b),
            (Self::UnspecifiedNumber(a), Self::UnspecifiedNumber(b)) => a.exact(b),
            _ => false,
        }
    }
}
