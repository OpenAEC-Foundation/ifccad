//! Closed typed Leader-family JSON bodies. Owned IFCX graph projection is separate.
use super::*;
impl LeaderJsonValue for IfccadLeaderGraphics {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            origin,
            placement,
            commands,
        } = self;
        let mut object = Map::new();
        if origin.saved() {
            object.insert("origin".into(), origin.write_value(prefix));
        }
        if placement.saved() {
            object.insert("placement".into(), placement.write_value(prefix));
        }
        if commands.saved() {
            object.insert("commands".into(), commands.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["origin", "placement", "commands"], path)?;
        Ok(Self {
            origin: f.take::<IfccadLeaderGraphicsOrigin>("origin", prefix)?,
            placement: f.take::<IfccadPlacement>("placement", prefix)?,
            commands: f.take::<Vec<IfccadLeaderGraphicCommand>>("commands", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicFont {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            family,
            font_file,
            big_font_file,
            bold,
            italic,
            charset,
            pitch_family,
        } = self;
        let mut object = Map::new();
        if family.saved() {
            object.insert("family".into(), family.write_value(prefix));
        }
        if font_file.saved() {
            object.insert("fontFile".into(), font_file.write_value(prefix));
        }
        if big_font_file.saved() {
            object.insert("bigFontFile".into(), big_font_file.write_value(prefix));
        }
        if bold.saved() {
            object.insert("bold".into(), bold.write_value(prefix));
        }
        if italic.saved() {
            object.insert("italic".into(), italic.write_value(prefix));
        }
        if charset.saved() {
            object.insert("charset".into(), charset.write_value(prefix));
        }
        if pitch_family.saved() {
            object.insert("pitchFamily".into(), pitch_family.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "family",
                "fontFile",
                "bigFontFile",
                "bold",
                "italic",
                "charset",
                "pitchFamily",
            ],
            path,
        )?;
        Ok(Self {
            family: f.take::<String>("family", prefix)?,
            font_file: f.take::<String>("fontFile", prefix)?,
            big_font_file: f.take::<String>("bigFontFile", prefix)?,
            bold: f.take::<Option<bool>>("bold", prefix)?,
            italic: f.take::<Option<bool>>("italic", prefix)?,
            charset: f.take::<Option<u32>>("charset", prefix)?,
            pitch_family: f.take::<Option<u32>>("pitchFamily", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicText {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            position,
            normal,
            direction,
            text,
            processing,
            height,
            width_factor,
            oblique_angle,
            tracking,
            backwards,
            upside_down,
            vertical,
            underline,
            overline,
            font,
        } = self;
        let mut object = Map::new();
        if position.saved() {
            object.insert("position".into(), position.write_value(prefix));
        }
        if normal.saved() {
            object.insert("normal".into(), normal.write_value(prefix));
        }
        if direction.saved() {
            object.insert("direction".into(), direction.write_value(prefix));
        }
        if text.saved() {
            object.insert("text".into(), text.write_value(prefix));
        }
        if processing.saved() {
            object.insert("processing".into(), processing.write_value(prefix));
        }
        if height.saved() {
            object.insert("height".into(), height.write_value(prefix));
        }
        if width_factor.saved() {
            object.insert("widthFactor".into(), width_factor.write_value(prefix));
        }
        if oblique_angle.saved() {
            object.insert("obliqueAngle".into(), oblique_angle.write_value(prefix));
        }
        if tracking.saved() {
            object.insert("tracking".into(), tracking.write_value(prefix));
        }
        if backwards.saved() {
            object.insert("backwards".into(), backwards.write_value(prefix));
        }
        if upside_down.saved() {
            object.insert("upsideDown".into(), upside_down.write_value(prefix));
        }
        if vertical.saved() {
            object.insert("vertical".into(), vertical.write_value(prefix));
        }
        if underline.saved() {
            object.insert("underline".into(), underline.write_value(prefix));
        }
        if overline.saved() {
            object.insert("overline".into(), overline.write_value(prefix));
        }
        if font.saved() {
            object.insert("font".into(), font.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "position",
                "normal",
                "direction",
                "text",
                "processing",
                "height",
                "widthFactor",
                "obliqueAngle",
                "tracking",
                "backwards",
                "upsideDown",
                "vertical",
                "underline",
                "overline",
                "font",
            ],
            path,
        )?;
        Ok(Self {
            position: f.take::<[f64; 3]>("position", prefix)?,
            normal: f.take::<[f64; 3]>("normal", prefix)?,
            direction: f.take::<[f64; 3]>("direction", prefix)?,
            text: f.take::<String>("text", prefix)?,
            processing: f.take::<IfccadLeaderGraphicTextProcessing>("processing", prefix)?,
            height: f.take::<f64>("height", prefix)?,
            width_factor: f.take::<f64>("widthFactor", prefix)?,
            oblique_angle: f.take::<f64>("obliqueAngle", prefix)?,
            tracking: f.take::<f64>("tracking", prefix)?,
            backwards: f.take::<bool>("backwards", prefix)?,
            upside_down: f.take::<bool>("upsideDown", prefix)?,
            vertical: f.take::<bool>("vertical", prefix)?,
            underline: f.take::<bool>("underline", prefix)?,
            overline: f.take::<bool>("overline", prefix)?,
            font: f.take::<Option<IfccadLeaderGraphicFont>>("font", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeader {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            placement,
            path,
            annotation_kind,
            annotation_entity,
            dimension_style,
            overrides,
            arrow,
            arrow_enabled,
            hook_enabled,
            hook_direction,
            annotation_box,
            horizontal_direction,
            block_offset,
            annotation_offset,
            origin,
            annotation,
            graphics,
        } = self;
        let mut object = Map::new();
        if placement.saved() {
            object.insert("placement".into(), placement.write_value(prefix));
        }
        if path.saved() {
            object.insert("path".into(), path.write_value(prefix));
        }
        if annotation_kind.saved() {
            object.insert("annotationKind".into(), annotation_kind.write_value(prefix));
        }
        if annotation_entity.saved() {
            object.insert(
                "annotationEntity".into(),
                write_reference(
                    *annotation_entity.as_ref().expect("present reference"),
                    prefix,
                    "e",
                ),
            );
        }
        if dimension_style.saved() {
            object.insert("dimensionStyle".into(), dimension_style.write_value(prefix));
        }
        if overrides.saved() {
            object.insert("overrides".into(), overrides.write_value(prefix));
        }
        if arrow.saved() {
            object.insert("arrow".into(), arrow.write_value(prefix));
        }
        if arrow_enabled.saved() {
            object.insert("arrowEnabled".into(), arrow_enabled.write_value(prefix));
        }
        if hook_enabled.saved() {
            object.insert("hookEnabled".into(), hook_enabled.write_value(prefix));
        }
        if hook_direction.saved() {
            object.insert("hookDirection".into(), hook_direction.write_value(prefix));
        }
        if annotation_box.saved() {
            object.insert("annotationBox".into(), annotation_box.write_value(prefix));
        }
        if horizontal_direction.saved() {
            object.insert(
                "horizontalDirection".into(),
                horizontal_direction.write_value(prefix),
            );
        }
        if block_offset.saved() {
            object.insert("blockOffset".into(), block_offset.write_value(prefix));
        }
        if annotation_offset.saved() {
            object.insert(
                "annotationOffset".into(),
                annotation_offset.write_value(prefix),
            );
        }
        if origin.saved() {
            object.insert("origin".into(), origin.write_value(prefix));
        }
        if annotation.saved() {
            object.insert("annotation".into(), annotation.write_value(prefix));
        }
        if graphics.saved() {
            object.insert("graphics".into(), graphics.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "placement",
                "path",
                "annotationKind",
                "annotationEntity",
                "dimensionStyle",
                "overrides",
                "arrow",
                "arrowEnabled",
                "hookEnabled",
                "hookDirection",
                "annotationBox",
                "horizontalDirection",
                "blockOffset",
                "annotationOffset",
                "origin",
                "annotation",
                "graphics",
            ],
            path,
        )?;
        Ok(Self {
            placement: f.take::<IfccadPlacement>("placement", prefix)?,
            path: f.take::<IfccadLeaderPath>("path", prefix)?,
            annotation_kind: f.take::<IfccadLeaderAnnotationKind>("annotationKind", prefix)?,
            annotation_entity: f
                .value("annotationEntity")
                .map(|value| read_reference(value, prefix, "e", &f.child("annotationEntity")))
                .transpose()?,
            dimension_style: f.take::<IfccadDimensionStyleId>("dimensionStyle", prefix)?,
            overrides: f.take::<IfccadDimensionOverrides>("overrides", prefix)?,
            arrow: f.take::<Option<IfccadLeaderArrow>>("arrow", prefix)?,
            arrow_enabled: f.take::<bool>("arrowEnabled", prefix)?,
            hook_enabled: f.take::<bool>("hookEnabled", prefix)?,
            hook_direction: f.take::<IfccadLeaderHookDirection>("hookDirection", prefix)?,
            annotation_box: f.take::<Option<[f64; 2]>>("annotationBox", prefix)?,
            horizontal_direction: f.take::<[f64; 3]>("horizontalDirection", prefix)?,
            block_offset: f.take::<[f64; 3]>("blockOffset", prefix)?,
            annotation_offset: f.take::<[f64; 3]>("annotationOffset", prefix)?,
            origin: f.take::<[f64; 3]>("origin", prefix)?,
            annotation: f.take::<Option<IfccadAnnotation<IfccadLeaderAnnotationContext>>>(
                "annotation",
                prefix,
            )?,
            graphics: f.take::<Option<IfccadLeaderGraphics>>("graphics", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeaderAnnotationContext {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            points,
            horizontal_direction,
            annotation_enabled,
            insertion_offset,
            endpoint_projection,
        } = self;
        let mut object = Map::new();
        if points.saved() {
            object.insert("points".into(), points.write_value(prefix));
        }
        if horizontal_direction.saved() {
            object.insert(
                "horizontalDirection".into(),
                horizontal_direction.write_value(prefix),
            );
        }
        if annotation_enabled.saved() {
            object.insert(
                "annotationEnabled".into(),
                annotation_enabled.write_value(prefix),
            );
        }
        if insertion_offset.saved() {
            object.insert(
                "insertionOffset".into(),
                insertion_offset.write_value(prefix),
            );
        }
        if endpoint_projection.saved() {
            object.insert(
                "endpointProjection".into(),
                endpoint_projection.write_value(prefix),
            );
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "points",
                "horizontalDirection",
                "annotationEnabled",
                "insertionOffset",
                "endpointProjection",
            ],
            path,
        )?;
        Ok(Self {
            points: f.take::<Vec<[f64; 3]>>("points", prefix)?,
            horizontal_direction: f.take::<[f64; 3]>("horizontalDirection", prefix)?,
            annotation_enabled: f.take::<bool>("annotationEnabled", prefix)?,
            insertion_offset: f.take::<[f64; 3]>("insertionOffset", prefix)?,
            endpoint_projection: f.take::<[f64; 3]>("endpointProjection", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadTolerance {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            placement,
            dimension_style,
            overrides,
            notation,
            annotation,
            graphics,
        } = self;
        let mut object = Map::new();
        if placement.saved() {
            object.insert("placement".into(), placement.write_value(prefix));
        }
        if dimension_style.saved() {
            object.insert("dimensionStyle".into(), dimension_style.write_value(prefix));
        }
        if overrides.saved() {
            object.insert("overrides".into(), overrides.write_value(prefix));
        }
        if notation.saved() {
            object.insert("notation".into(), notation.write_value(prefix));
        }
        if annotation.saved() {
            object.insert("annotation".into(), annotation.write_value(prefix));
        }
        if graphics.saved() {
            object.insert("graphics".into(), graphics.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "placement",
                "dimensionStyle",
                "overrides",
                "notation",
                "annotation",
                "graphics",
            ],
            path,
        )?;
        Ok(Self {
            placement: f.take::<IfccadPlacement>("placement", prefix)?,
            dimension_style: f.take::<IfccadDimensionStyleId>("dimensionStyle", prefix)?,
            overrides: f.take::<IfccadDimensionOverrides>("overrides", prefix)?,
            notation: f.take::<IfccadToleranceNotation>("notation", prefix)?,
            annotation: f.take::<Option<IfccadAnnotation<IfccadToleranceAnnotationContext>>>(
                "annotation",
                prefix,
            )?,
            graphics: f.take::<Option<IfccadLeaderGraphics>>("graphics", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadToleranceAnnotationContext {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            location,
            horizontal_direction,
        } = self;
        let mut object = Map::new();
        if location.saved() {
            object.insert("location".into(), location.write_value(prefix));
        }
        if horizontal_direction.saved() {
            object.insert(
                "horizontalDirection".into(),
                horizontal_direction.write_value(prefix),
            );
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["location", "horizontalDirection"], path)?;
        Ok(Self {
            location: f.take::<[f64; 3]>("location", prefix)?,
            horizontal_direction: f.take::<[f64; 3]>("horizontalDirection", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderLineSettings {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            path_kind,
            color,
            line_pattern,
            line_weight,
            arrow,
            arrow_size,
        } = self;
        let mut object = Map::new();
        if path_kind.saved() {
            object.insert("pathKind".into(), path_kind.write_value(prefix));
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if line_pattern.saved() {
            object.insert("linePattern".into(), line_pattern.write_value(prefix));
        }
        if line_weight.saved() {
            object.insert("lineWeight".into(), line_weight.write_value(prefix));
        }
        if arrow.saved() {
            object.insert("arrow".into(), arrow.write_value(prefix));
        }
        if arrow_size.saved() {
            object.insert("arrowSize".into(), arrow_size.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "pathKind",
                "color",
                "linePattern",
                "lineWeight",
                "arrow",
                "arrowSize",
            ],
            path,
        )?;
        Ok(Self {
            path_kind: f.take::<IfccadMultiLeaderPathKind>("pathKind", prefix)?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            line_pattern: f.take::<IfccadMode<IfccadLinePatternId>>("linePattern", prefix)?,
            line_weight: f.take::<IfccadLeaderLineWeight>("lineWeight", prefix)?,
            arrow: f.take::<IfccadLeaderArrow>("arrow", prefix)?,
            arrow_size: f.take::<f64>("arrowSize", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeaderBreakInterval {
    fn write_value(&self, prefix: &str) -> Value {
        let Self { start, end } = self;
        let mut object = Map::new();
        if start.saved() {
            object.insert("start".into(), start.write_value(prefix));
        }
        if end.saved() {
            object.insert("end".into(), end.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["start", "end"], path)?;
        Ok(Self {
            start: f.take::<[f64; 3]>("start", prefix)?,
            end: f.take::<[f64; 3]>("end", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderSegmentBreaks {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            segment_index,
            intervals,
        } = self;
        let mut object = Map::new();
        if segment_index.saved() {
            object.insert("segmentIndex".into(), segment_index.write_value(prefix));
        }
        if intervals.saved() {
            object.insert("intervals".into(), intervals.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["segmentIndex", "intervals"], path)?;
        Ok(Self {
            segment_index: f.take::<u32>("segmentIndex", prefix)?,
            intervals: f.take::<Vec<IfccadLeaderBreakInterval>>("intervals", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderLine {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            key,
            source_index,
            settings,
            overridden_properties,
            vertices,
            connection,
            breaks,
        } = self;
        let mut object = Map::new();
        if key.saved() {
            object.insert("key".into(), key.write_value(prefix));
        }
        if source_index.saved() {
            object.insert("sourceIndex".into(), source_index.write_value(prefix));
        }
        if settings.saved() {
            object.insert("settings".into(), settings.write_value(prefix));
        }
        if overridden_properties.saved() {
            object.insert(
                "overriddenProperties".into(),
                overridden_properties.write_value(prefix),
            );
        }
        if vertices.saved() {
            object.insert("vertices".into(), vertices.write_value(prefix));
        }
        if connection.saved() {
            object.insert("connection".into(), connection.write_value(prefix));
        }
        if breaks.saved() {
            object.insert("breaks".into(), breaks.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "key",
                "sourceIndex",
                "settings",
                "overriddenProperties",
                "vertices",
                "connection",
                "breaks",
            ],
            path,
        )?;
        Ok(Self {
            key: f.take::<u64>("key", prefix)?,
            source_index: f.take::<Option<i32>>("sourceIndex", prefix)?,
            settings: f.take::<IfccadMultiLeaderLineSettings>("settings", prefix)?,
            overridden_properties: f
                .take::<BTreeSet<IfccadMultiLeaderLineProperty>>("overriddenProperties", prefix)?,
            vertices: f.take::<Vec<[f64; 3]>>("vertices", prefix)?,
            connection: f.take::<IfccadMultiLeaderConnection>("connection", prefix)?,
            breaks: f.take::<Vec<IfccadMultiLeaderSegmentBreaks>>("breaks", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderGroup {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            key,
            source_index,
            has_connection_point,
            connection_point,
            has_dogleg_direction,
            dogleg_direction,
            dogleg_length,
            text_attachment_direction,
            lines,
            breaks,
        } = self;
        let mut object = Map::new();
        if key.saved() {
            object.insert("key".into(), key.write_value(prefix));
        }
        if source_index.saved() {
            object.insert("sourceIndex".into(), source_index.write_value(prefix));
        }
        if has_connection_point.saved() {
            object.insert(
                "hasConnectionPoint".into(),
                has_connection_point.write_value(prefix),
            );
        }
        if connection_point.saved() {
            object.insert(
                "connectionPoint".into(),
                connection_point.write_value(prefix),
            );
        }
        if has_dogleg_direction.saved() {
            object.insert(
                "hasDoglegDirection".into(),
                has_dogleg_direction.write_value(prefix),
            );
        }
        if dogleg_direction.saved() {
            object.insert(
                "doglegDirection".into(),
                dogleg_direction.write_value(prefix),
            );
        }
        if dogleg_length.saved() {
            object.insert("doglegLength".into(), dogleg_length.write_value(prefix));
        }
        if text_attachment_direction.saved() {
            object.insert(
                "textAttachmentDirection".into(),
                text_attachment_direction.write_value(prefix),
            );
        }
        if lines.saved() {
            object.insert("lines".into(), lines.write_value(prefix));
        }
        if breaks.saved() {
            object.insert("breaks".into(), breaks.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "key",
                "sourceIndex",
                "hasConnectionPoint",
                "connectionPoint",
                "hasDoglegDirection",
                "doglegDirection",
                "doglegLength",
                "textAttachmentDirection",
                "lines",
                "breaks",
            ],
            path,
        )?;
        Ok(Self {
            key: f.take::<u64>("key", prefix)?,
            source_index: f.take::<Option<i32>>("sourceIndex", prefix)?,
            has_connection_point: f.take::<bool>("hasConnectionPoint", prefix)?,
            connection_point: f.take::<[f64; 3]>("connectionPoint", prefix)?,
            has_dogleg_direction: f.take::<bool>("hasDoglegDirection", prefix)?,
            dogleg_direction: f.take::<[f64; 3]>("doglegDirection", prefix)?,
            dogleg_length: f.take::<f64>("doglegLength", prefix)?,
            text_attachment_direction: f
                .take::<IfccadMultiLeaderAttachmentDirection>("textAttachmentDirection", prefix)?,
            lines: f.take::<Vec<IfccadMultiLeaderLine>>("lines", prefix)?,
            breaks: f.take::<Vec<IfccadLeaderBreakInterval>>("breaks", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderBlockContent {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            definition_id,
            transform,
            color,
            connection,
            stored_transform,
        } = self;
        let mut object = Map::new();
        if definition_id.saved() {
            object.insert(
                "definition".into(),
                write_reference(*definition_id, prefix, "block/"),
            );
        }
        if transform.saved() {
            object.insert("transform".into(), transform.write_value(prefix));
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if connection.saved() {
            object.insert("connection".into(), connection.write_value(prefix));
        }
        if stored_transform.saved() {
            object.insert(
                "storedTransform".into(),
                stored_transform.write_value(prefix),
            );
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "definition",
                "transform",
                "color",
                "connection",
                "storedTransform",
            ],
            path,
        )?;
        Ok(Self {
            definition_id: read_reference(
                f.required("definition")?,
                prefix,
                "block/",
                &f.child("definition"),
            )?,
            transform: f.take::<IfccadBlockTransform>("transform", prefix)?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            connection: f.take::<IfccadMultiLeaderBlockConnection>("connection", prefix)?,
            stored_transform: f.take::<Option<[f64; 16]>>("storedTransform", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderBlockLabel {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            key,
            definition_id,
            source_index,
            width,
            text,
        } = self;
        let mut object = Map::new();
        if key.saved() {
            object.insert("key".into(), key.write_value(prefix));
        }
        if definition_id.saved() {
            object.insert(
                "definition".into(),
                write_reference(
                    *definition_id.as_ref().expect("present reference"),
                    prefix,
                    "e",
                ),
            );
        }
        if source_index.saved() {
            object.insert("sourceIndex".into(), source_index.write_value(prefix));
        }
        if width.saved() {
            object.insert("width".into(), width.write_value(prefix));
        }
        if text.saved() {
            object.insert("text".into(), text.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &["key", "definition", "sourceIndex", "width", "text"],
            path,
        )?;
        Ok(Self {
            key: f.take::<u64>("key", prefix)?,
            definition_id: f
                .value("definition")
                .map(|value| read_reference(value, prefix, "e", &f.child("definition")))
                .transpose()?,
            source_index: f.take::<Option<i32>>("sourceIndex", prefix)?,
            width: f.take::<f64>("width", prefix)?,
            text: f.take::<String>("text", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextAttachments {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            left,
            right,
            top,
            bottom,
        } = self;
        let mut object = Map::new();
        if left.saved() {
            object.insert("left".into(), left.write_value(prefix));
        }
        if right.saved() {
            object.insert("right".into(), right.write_value(prefix));
        }
        if top.saved() {
            object.insert("top".into(), top.write_value(prefix));
        }
        if bottom.saved() {
            object.insert("bottom".into(), bottom.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["left", "right", "top", "bottom"], path)?;
        Ok(Self {
            left: f.take::<IfccadMultiLeaderTextAttachment>("left", prefix)?,
            right: f.take::<IfccadMultiLeaderTextAttachment>("right", prefix)?,
            top: f.take::<IfccadMultiLeaderTextAttachment>("top", prefix)?,
            bottom: f.take::<IfccadMultiLeaderTextAttachment>("bottom", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextBackground {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            enabled,
            mask_enabled,
            color,
            scale_factor,
            opacity,
        } = self;
        let mut object = Map::new();
        if enabled.saved() {
            object.insert("enabled".into(), enabled.write_value(prefix));
        }
        if mask_enabled.saved() {
            object.insert("maskEnabled".into(), mask_enabled.write_value(prefix));
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if scale_factor.saved() {
            object.insert("scaleFactor".into(), scale_factor.write_value(prefix));
        }
        if opacity.saved() {
            object.insert("opacity".into(), opacity.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &["enabled", "maskEnabled", "color", "scaleFactor", "opacity"],
            path,
        )?;
        Ok(Self {
            enabled: f.take::<bool>("enabled", prefix)?,
            mask_enabled: f.take::<bool>("maskEnabled", prefix)?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            scale_factor: f.take::<f64>("scaleFactor", prefix)?,
            opacity: f.take::<IfccadMode<f64>>("opacity", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextState {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            boundary_height,
            automatic_height,
            word_break,
            color,
            attachments,
            attachment_point,
            alignment,
            background,
            stored_rotation,
        } = self;
        let mut object = Map::new();
        if boundary_height.saved() {
            object.insert("boundaryHeight".into(), boundary_height.write_value(prefix));
        }
        if automatic_height.saved() {
            object.insert(
                "automaticHeight".into(),
                automatic_height.write_value(prefix),
            );
        }
        if word_break.saved() {
            object.insert("wordBreak".into(), word_break.write_value(prefix));
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if attachments.saved() {
            object.insert("attachments".into(), attachments.write_value(prefix));
        }
        if attachment_point.saved() {
            object.insert(
                "attachmentPoint".into(),
                attachment_point.write_value(prefix),
            );
        }
        if alignment.saved() {
            object.insert("alignment".into(), alignment.write_value(prefix));
        }
        if background.saved() {
            object.insert("background".into(), background.write_value(prefix));
        }
        if stored_rotation.saved() {
            object.insert("storedRotation".into(), stored_rotation.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "boundaryHeight",
                "automaticHeight",
                "wordBreak",
                "color",
                "attachments",
                "attachmentPoint",
                "alignment",
                "background",
                "storedRotation",
            ],
            path,
        )?;
        Ok(Self {
            boundary_height: f.take::<f64>("boundaryHeight", prefix)?,
            automatic_height: f.take::<bool>("automaticHeight", prefix)?,
            word_break: f.take::<bool>("wordBreak", prefix)?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            attachments: f.take::<IfccadMultiLeaderTextAttachments>("attachments", prefix)?,
            attachment_point: f
                .take::<IfccadMultiLeaderTextAttachmentPoint>("attachmentPoint", prefix)?,
            alignment: f.take::<IfccadMultiLeaderTextAlignment>("alignment", prefix)?,
            background: f.take::<IfccadMultiLeaderTextBackground>("background", prefix)?,
            stored_rotation: f.take::<Option<f64>>("storedRotation", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextContent {
    fn write_value(&self, prefix: &str) -> Value {
        let Self { text, state } = self;
        let mut object = Map::new();
        if text.saved() {
            object.insert("text".into(), text.write_value(prefix));
        }
        if state.saved() {
            object.insert("state".into(), state.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["text", "state"], path)?;
        Ok(Self {
            text: f.take::<IfccadMText>("text", prefix)?,
            state: f.take::<IfccadMultiLeaderTextState>("state", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderToleranceContent {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            placement,
            dimension_style,
            overrides,
            notation,
        } = self;
        let mut object = Map::new();
        if placement.saved() {
            object.insert("placement".into(), placement.write_value(prefix));
        }
        if dimension_style.saved() {
            object.insert("dimensionStyle".into(), dimension_style.write_value(prefix));
        }
        if overrides.saved() {
            object.insert("overrides".into(), overrides.write_value(prefix));
        }
        if notation.saved() {
            object.insert("notation".into(), notation.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &["placement", "dimensionStyle", "overrides", "notation"],
            path,
        )?;
        Ok(Self {
            placement: f.take::<IfccadPlacement>("placement", prefix)?,
            dimension_style: f.take::<IfccadDimensionStyleId>("dimensionStyle", prefix)?,
            overrides: f.take::<IfccadDimensionOverrides>("overrides", prefix)?,
            notation: f.take::<IfccadToleranceNotation>("notation", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderStoredContent {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            m_text,
            block,
            tolerance,
        } = self;
        let mut object = Map::new();
        if m_text.saved() {
            object.insert("mText".into(), m_text.write_value(prefix));
        }
        if block.saved() {
            object.insert("block".into(), block.write_value(prefix));
        }
        if tolerance.saved() {
            object.insert("tolerance".into(), tolerance.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["mText", "block", "tolerance"], path)?;
        Ok(Self {
            m_text: f.take::<Option<Box<IfccadMultiLeaderTextContent>>>("mText", prefix)?,
            block: f.take::<Option<IfccadMultiLeaderBlockContent>>("block", prefix)?,
            tolerance: f
                .take::<Option<Box<IfccadMultiLeaderToleranceContent>>>("tolerance", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderPresentation {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            plane,
            normal_reversed,
            content_base_point,
            scale_factor,
            text_height,
            arrow_size,
            landing_gap,
            next_part_id,
            groups,
            content,
        } = self;
        let mut object = Map::new();
        if plane.saved() {
            object.insert("plane".into(), plane.write_value(prefix));
        }
        if normal_reversed.saved() {
            object.insert("normalReversed".into(), normal_reversed.write_value(prefix));
        }
        if content_base_point.saved() {
            object.insert(
                "contentBasePoint".into(),
                content_base_point.write_value(prefix),
            );
        }
        if scale_factor.saved() {
            object.insert("scaleFactor".into(), scale_factor.write_value(prefix));
        }
        if text_height.saved() {
            object.insert("textHeight".into(), text_height.write_value(prefix));
        }
        if arrow_size.saved() {
            object.insert("arrowSize".into(), arrow_size.write_value(prefix));
        }
        if landing_gap.saved() {
            object.insert("landingGap".into(), landing_gap.write_value(prefix));
        }
        if next_part_id.saved() {
            object.insert("nextPartId".into(), next_part_id.write_value(prefix));
        }
        if groups.saved() {
            object.insert("groups".into(), groups.write_value(prefix));
        }
        if content.saved() {
            object.insert("content".into(), content.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "plane",
                "normalReversed",
                "contentBasePoint",
                "scaleFactor",
                "textHeight",
                "arrowSize",
                "landingGap",
                "nextPartId",
                "groups",
                "content",
            ],
            path,
        )?;
        Ok(Self {
            plane: f.take::<IfccadPlacement>("plane", prefix)?,
            normal_reversed: f.take::<bool>("normalReversed", prefix)?,
            content_base_point: f.take::<[f64; 3]>("contentBasePoint", prefix)?,
            scale_factor: f.take::<f64>("scaleFactor", prefix)?,
            text_height: f.take::<f64>("textHeight", prefix)?,
            arrow_size: f.take::<f64>("arrowSize", prefix)?,
            landing_gap: f.take::<f64>("landingGap", prefix)?,
            next_part_id: f.take::<u64>("nextPartId", prefix)?,
            groups: f.take::<Vec<IfccadMultiLeaderGroup>>("groups", prefix)?,
            content: f.take::<IfccadMultiLeaderStoredContent>("content", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextSettings {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            style_id,
            color,
            height,
            frame,
            attachments,
            attachment_direction,
            angle,
            alignment,
        } = self;
        let mut object = Map::new();
        if style_id.saved() {
            object.insert("style".into(), style_id.write_value(prefix));
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if height.saved() {
            object.insert("height".into(), height.write_value(prefix));
        }
        if frame.saved() {
            object.insert("frame".into(), frame.write_value(prefix));
        }
        if attachments.saved() {
            object.insert("attachments".into(), attachments.write_value(prefix));
        }
        if attachment_direction.saved() {
            object.insert(
                "attachmentDirection".into(),
                attachment_direction.write_value(prefix),
            );
        }
        if angle.saved() {
            object.insert("angle".into(), angle.write_value(prefix));
        }
        if alignment.saved() {
            object.insert("alignment".into(), alignment.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "style",
                "color",
                "height",
                "frame",
                "attachments",
                "attachmentDirection",
                "angle",
                "alignment",
            ],
            path,
        )?;
        Ok(Self {
            style_id: f.take::<Option<IfccadTextStyleId>>("style", prefix)?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            height: f.take::<f64>("height", prefix)?,
            frame: f.take::<bool>("frame", prefix)?,
            attachments: f.take::<IfccadMultiLeaderTextAttachments>("attachments", prefix)?,
            attachment_direction: f
                .take::<IfccadMultiLeaderAttachmentDirection>("attachmentDirection", prefix)?,
            angle: f.take::<IfccadMultiLeaderTextAngle>("angle", prefix)?,
            alignment: f.take::<IfccadMultiLeaderTextAlignment>("alignment", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderBlockSettings {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            definition_id,
            color,
            connection,
            rotation,
            scale,
        } = self;
        let mut object = Map::new();
        if definition_id.saved() {
            object.insert(
                "definition".into(),
                write_reference(
                    *definition_id.as_ref().expect("present reference"),
                    prefix,
                    "block/",
                ),
            );
        }
        if color.saved() {
            object.insert("color".into(), color.write_value(prefix));
        }
        if connection.saved() {
            object.insert("connection".into(), connection.write_value(prefix));
        }
        if rotation.saved() {
            object.insert("rotation".into(), rotation.write_value(prefix));
        }
        if scale.saved() {
            object.insert("scale".into(), scale.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &["definition", "color", "connection", "rotation", "scale"],
            path,
        )?;
        Ok(Self {
            definition_id: f
                .value("definition")
                .map(|value| read_reference(value, prefix, "block/", &f.child("definition")))
                .transpose()?,
            color: f.take::<IfccadMode<IfccadColor>>("color", prefix)?,
            connection: f.take::<IfccadMultiLeaderBlockConnection>("connection", prefix)?,
            rotation: f.take::<f64>("rotation", prefix)?,
            scale: f.take::<[f64; 3]>("scale", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderCommonSettings {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            line,
            landing_enabled,
            dogleg_enabled,
            dogleg_length,
            content_kind,
            text,
            block,
            scale_factor,
        } = self;
        let mut object = Map::new();
        if line.saved() {
            object.insert("line".into(), line.write_value(prefix));
        }
        if landing_enabled.saved() {
            object.insert("landingEnabled".into(), landing_enabled.write_value(prefix));
        }
        if dogleg_enabled.saved() {
            object.insert("doglegEnabled".into(), dogleg_enabled.write_value(prefix));
        }
        if dogleg_length.saved() {
            object.insert("doglegLength".into(), dogleg_length.write_value(prefix));
        }
        if content_kind.saved() {
            object.insert("contentKind".into(), content_kind.write_value(prefix));
        }
        if text.saved() {
            object.insert("text".into(), text.write_value(prefix));
        }
        if block.saved() {
            object.insert("block".into(), block.write_value(prefix));
        }
        if scale_factor.saved() {
            object.insert("scaleFactor".into(), scale_factor.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "line",
                "landingEnabled",
                "doglegEnabled",
                "doglegLength",
                "contentKind",
                "text",
                "block",
                "scaleFactor",
            ],
            path,
        )?;
        Ok(Self {
            line: f.take::<IfccadMultiLeaderLineSettings>("line", prefix)?,
            landing_enabled: f.take::<bool>("landingEnabled", prefix)?,
            dogleg_enabled: f.take::<bool>("doglegEnabled", prefix)?,
            dogleg_length: f.take::<f64>("doglegLength", prefix)?,
            content_kind: f.take::<IfccadMultiLeaderContentKind>("contentKind", prefix)?,
            text: f.take::<IfccadMultiLeaderTextSettings>("text", prefix)?,
            block: f.take::<IfccadMultiLeaderBlockSettings>("block", prefix)?,
            scale_factor: f.take::<f64>("scaleFactor", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderSettings {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            common,
            text_attachment_point,
            text_direction_negative,
            extend_leader_to_text,
        } = self;
        let mut object = Map::new();
        if common.saved() {
            object.insert("common".into(), common.write_value(prefix));
        }
        if text_attachment_point.saved() {
            object.insert(
                "textAttachmentPoint".into(),
                text_attachment_point.write_value(prefix),
            );
        }
        if text_direction_negative.saved() {
            object.insert(
                "textDirectionNegative".into(),
                text_direction_negative.write_value(prefix),
            );
        }
        if extend_leader_to_text.saved() {
            object.insert(
                "extendLeaderToText".into(),
                extend_leader_to_text.write_value(prefix),
            );
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "common",
                "textAttachmentPoint",
                "textDirectionNegative",
                "extendLeaderToText",
            ],
            path,
        )?;
        Ok(Self {
            common: f.take::<IfccadMultiLeaderCommonSettings>("common", prefix)?,
            text_attachment_point: f
                .take::<IfccadMultiLeaderTextAttachmentPoint>("textAttachmentPoint", prefix)?,
            text_direction_negative: f.take::<bool>("textDirectionNegative", prefix)?,
            extend_leader_to_text: f.take::<bool>("extendLeaderToText", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderDefaultText {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            character_format,
            paragraph_format,
            content,
        } = self;
        let mut object = Map::new();
        if character_format.saved() {
            object.insert(
                "characterFormat".into(),
                character_format.write_value(prefix),
            );
        }
        if paragraph_format.saved() {
            object.insert(
                "paragraphFormat".into(),
                paragraph_format.write_value(prefix),
            );
        }
        if content.saved() {
            object.insert("content".into(), content.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &["characterFormat", "paragraphFormat", "content"],
            path,
        )?;
        Ok(Self {
            character_format: f
                .take::<crate::text::CharacterFormat<IfccadColor>>("characterFormat", prefix)?,
            paragraph_format: f.take::<crate::text::ParagraphFormat>("paragraphFormat", prefix)?,
            content: f.take::<Vec<crate::text::MTextParagraph<IfccadColor>>>("content", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderStyle {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            id,
            name,
            description,
            common,
            landing_gap,
            default_m_text,
            text_always_left,
            block_scale_enabled,
            block_rotation_enabled,
            align_space,
            break_gap_size,
            max_leader_points,
            first_segment_angle,
            second_segment_angle,
            leader_draw_order,
            content_draw_order,
            annotative,
        } = self;
        let mut object = Map::new();
        if id.saved() {
            object.insert("id".into(), id.write_value(prefix));
        }
        if name.saved() {
            object.insert("name".into(), name.write_value(prefix));
        }
        if description.saved() {
            object.insert("description".into(), description.write_value(prefix));
        }
        if common.saved() {
            object.insert("common".into(), common.write_value(prefix));
        }
        if landing_gap.saved() {
            object.insert("landingGap".into(), landing_gap.write_value(prefix));
        }
        if default_m_text.saved() {
            object.insert("defaultMText".into(), default_m_text.write_value(prefix));
        }
        if text_always_left.saved() {
            object.insert(
                "textAlwaysLeft".into(),
                text_always_left.write_value(prefix),
            );
        }
        if block_scale_enabled.saved() {
            object.insert(
                "blockScaleEnabled".into(),
                block_scale_enabled.write_value(prefix),
            );
        }
        if block_rotation_enabled.saved() {
            object.insert(
                "blockRotationEnabled".into(),
                block_rotation_enabled.write_value(prefix),
            );
        }
        if align_space.saved() {
            object.insert("alignSpace".into(), align_space.write_value(prefix));
        }
        if break_gap_size.saved() {
            object.insert("breakGapSize".into(), break_gap_size.write_value(prefix));
        }
        if max_leader_points.saved() {
            object.insert(
                "maxLeaderPoints".into(),
                max_leader_points.write_value(prefix),
            );
        }
        if first_segment_angle.saved() {
            object.insert(
                "firstSegmentAngle".into(),
                first_segment_angle.write_value(prefix),
            );
        }
        if second_segment_angle.saved() {
            object.insert(
                "secondSegmentAngle".into(),
                second_segment_angle.write_value(prefix),
            );
        }
        if leader_draw_order.saved() {
            object.insert(
                "leaderDrawOrder".into(),
                leader_draw_order.write_value(prefix),
            );
        }
        if content_draw_order.saved() {
            object.insert(
                "contentDrawOrder".into(),
                content_draw_order.write_value(prefix),
            );
        }
        if annotative.saved() {
            object.insert("annotative".into(), annotative.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "id",
                "name",
                "description",
                "common",
                "landingGap",
                "defaultMText",
                "textAlwaysLeft",
                "blockScaleEnabled",
                "blockRotationEnabled",
                "alignSpace",
                "breakGapSize",
                "maxLeaderPoints",
                "firstSegmentAngle",
                "secondSegmentAngle",
                "leaderDrawOrder",
                "contentDrawOrder",
                "annotative",
            ],
            path,
        )?;
        Ok(Self {
            id: f.take::<IfccadMultiLeaderStyleId>("id", prefix)?,
            name: f.take::<String>("name", prefix)?,
            description: f.take::<String>("description", prefix)?,
            common: f.take::<IfccadMultiLeaderCommonSettings>("common", prefix)?,
            landing_gap: f.take::<f64>("landingGap", prefix)?,
            default_m_text: f.take::<IfccadMultiLeaderDefaultText>("defaultMText", prefix)?,
            text_always_left: f.take::<bool>("textAlwaysLeft", prefix)?,
            block_scale_enabled: f.take::<bool>("blockScaleEnabled", prefix)?,
            block_rotation_enabled: f.take::<bool>("blockRotationEnabled", prefix)?,
            align_space: f.take::<f64>("alignSpace", prefix)?,
            break_gap_size: f.take::<f64>("breakGapSize", prefix)?,
            max_leader_points: f.take::<u32>("maxLeaderPoints", prefix)?,
            first_segment_angle: f.take::<f64>("firstSegmentAngle", prefix)?,
            second_segment_angle: f.take::<f64>("secondSegmentAngle", prefix)?,
            leader_draw_order: f.take::<IfccadMultiLeaderDrawOrder>("leaderDrawOrder", prefix)?,
            content_draw_order: f
                .take::<IfccadMultiLeaderContentDrawOrder>("contentDrawOrder", prefix)?,
            annotative: f.take::<bool>("annotative", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeaderLegacyArrowOverride {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            line_key,
            is_default,
            arrow,
        } = self;
        let mut object = Map::new();
        if line_key.saved() {
            object.insert("lineKey".into(), line_key.write_value(prefix));
        }
        if is_default.saved() {
            object.insert("isDefault".into(), is_default.write_value(prefix));
        }
        if arrow.saved() {
            object.insert("arrow".into(), arrow.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(value, &["lineKey", "isDefault", "arrow"], path)?;
        Ok(Self {
            line_key: f.take::<u64>("lineKey", prefix)?,
            is_default: f.take::<bool>("isDefault", prefix)?,
            arrow: f.take::<IfccadLeaderArrow>("arrow", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadMultiLeader {
    fn write_value(&self, prefix: &str) -> Value {
        let Self {
            style_id,
            settings,
            overridden_properties,
            block_labels,
            next_label_id,
            legacy_arrow_overrides,
            presentation,
            annotation,
            graphics,
        } = self;
        let mut object = Map::new();
        if style_id.saved() {
            object.insert("style".into(), style_id.write_value(prefix));
        }
        if settings.saved() {
            object.insert("settings".into(), settings.write_value(prefix));
        }
        if overridden_properties.saved() {
            object.insert(
                "overriddenProperties".into(),
                overridden_properties.write_value(prefix),
            );
        }
        if block_labels.saved() {
            object.insert("blockLabels".into(), block_labels.write_value(prefix));
        }
        if next_label_id.saved() {
            object.insert("nextLabelId".into(), next_label_id.write_value(prefix));
        }
        if legacy_arrow_overrides.saved() {
            object.insert(
                "legacyArrowOverrides".into(),
                legacy_arrow_overrides.write_value(prefix),
            );
        }
        if presentation.saved() {
            object.insert("presentation".into(), presentation.write_value(prefix));
        }
        if annotation.saved() {
            object.insert("annotation".into(), annotation.write_value(prefix));
        }
        if graphics.saved() {
            object.insert("graphics".into(), graphics.write_value(prefix));
        }
        Value::Object(object)
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let f = Fields::new(
            value,
            &[
                "style",
                "settings",
                "overriddenProperties",
                "blockLabels",
                "nextLabelId",
                "legacyArrowOverrides",
                "presentation",
                "annotation",
                "graphics",
            ],
            path,
        )?;
        Ok(Self {
            style_id: f.take::<Option<IfccadMultiLeaderStyleId>>("style", prefix)?,
            settings: f.take::<IfccadMultiLeaderSettings>("settings", prefix)?,
            overridden_properties: f
                .take::<BTreeSet<IfccadMultiLeaderProperty>>("overriddenProperties", prefix)?,
            block_labels: f.take::<Vec<IfccadMultiLeaderBlockLabel>>("blockLabels", prefix)?,
            next_label_id: f.take::<u64>("nextLabelId", prefix)?,
            legacy_arrow_overrides: f.take::<Vec<IfccadMultiLeaderLegacyArrowOverride>>(
                "legacyArrowOverrides",
                prefix,
            )?,
            presentation: f.take::<IfccadMultiLeaderPresentation>("presentation", prefix)?,
            annotation: f.take::<Option<IfccadAnnotation<IfccadMultiLeaderPresentation>>>(
                "annotation",
                prefix,
            )?,
            graphics: f.take::<Option<IfccadLeaderGraphics>>("graphics", prefix)?,
        })
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicsOrigin {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::SourceSnapshot => "sourceSnapshot",
                Self::Generated => "generated",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("sourceSnapshot") => Ok(Self::SourceSnapshot),
            Some("generated") => Ok(Self::Generated),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl<T: LeaderJsonValue> LeaderJsonValue for IfccadLeaderGraphicInheritance<T> {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::Owner => {
                json!({"kind":"owner"})
            }
            Self::ByLayer => {
                json!({"kind":"byLayer"})
            }
            Self::ByBlock => {
                json!({"kind":"byBlock"})
            }
            Self::Explicit(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("value".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"explicit","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "owner" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::Owner
            }
            "byLayer" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByLayer
            }
            "byBlock" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByBlock
            }
            "explicit" => {
                let f = Fields::new(outer.required("value")?, &["value"], &outer.child("value"))?;
                let value_0: T = f.take::<T>("value", prefix)?;
                Self::Explicit(value_0)
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicLineWeight {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::Owner => {
                json!({"kind":"owner"})
            }
            Self::ByLayer => {
                json!({"kind":"byLayer"})
            }
            Self::ByBlock => {
                json!({"kind":"byBlock"})
            }
            Self::DrawingDefault => {
                json!({"kind":"drawingDefault"})
            }
            Self::Explicit(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("value".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"explicit","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "owner" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::Owner
            }
            "byLayer" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByLayer
            }
            "byBlock" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByBlock
            }
            "drawingDefault" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::DrawingDefault
            }
            "explicit" => {
                let f = Fields::new(outer.required("value")?, &["value"], &outer.child("value"))?;
                let value_0: f64 = f.take::<f64>("value", prefix)?;
                Self::Explicit(value_0)
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicTextProcessing {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Interpreted => "interpreted",
                Self::Raw => "raw",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("interpreted") => Ok(Self::Interpreted),
            Some("raw") => Ok(Self::Raw),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicCommand {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::Draw(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("primitive".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"draw","value":value})
            }
            Self::StoredExtents { min, max } => {
                let value = {
                    let mut object = Map::new();
                    if min.saved() {
                        object.insert("min".into(), min.write_value(prefix));
                    }
                    if max.saved() {
                        object.insert("max".into(), max.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"storedExtents","value":value})
            }
            Self::SetColor(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("color".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setColor","value":value})
            }
            Self::SetLayer(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert(
                            "layer".into(),
                            write_reference(
                                *value_0.as_ref().expect("present reference"),
                                prefix,
                                "layer/",
                            ),
                        );
                    }
                    Value::Object(object)
                };
                json!({"kind":"setLayer","value":value})
            }
            Self::SetLinePattern(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("linePattern".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setLinePattern","value":value})
            }
            Self::SetSelectionMarker(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("marker".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setSelectionMarker","value":value})
            }
            Self::SetFill(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("enabled".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setFill","value":value})
            }
            Self::SetLineWeight(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("weight".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setLineWeight","value":value})
            }
            Self::SetLinePatternScale(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("scale".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setLinePatternScale","value":value})
            }
            Self::SetThickness(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("thickness".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"setThickness","value":value})
            }
            Self::PushTransform(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("matrix".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"pushTransform","value":value})
            }
            Self::PopTransform => {
                json!({"kind":"popTransform"})
            }
            Self::CadCompatibility51Zero => {
                json!({"kind":"cadCompatibility51Zero"})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "draw" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["primitive"],
                    &outer.child("value"),
                )?;
                let value_0: IfccadLeaderGraphicPrimitive =
                    f.take::<IfccadLeaderGraphicPrimitive>("primitive", prefix)?;
                Self::Draw(value_0)
            }
            "storedExtents" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["min", "max"],
                    &outer.child("value"),
                )?;
                let min: [f64; 3] = f.take::<[f64; 3]>("min", prefix)?;
                let max: [f64; 3] = f.take::<[f64; 3]>("max", prefix)?;
                Self::StoredExtents { min, max }
            }
            "setColor" => {
                let f = Fields::new(outer.required("value")?, &["color"], &outer.child("value"))?;
                let value_0: IfccadLeaderGraphicInheritance<IfccadColor> =
                    f.take::<IfccadLeaderGraphicInheritance<IfccadColor>>("color", prefix)?;
                Self::SetColor(value_0)
            }
            "setLayer" => {
                let f = Fields::new(outer.required("value")?, &["layer"], &outer.child("value"))?;
                let value_0: Option<u64> = f
                    .value("layer")
                    .map(|value| read_reference(value, prefix, "layer/", &f.child("layer")))
                    .transpose()?;
                Self::SetLayer(value_0)
            }
            "setLinePattern" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["linePattern"],
                    &outer.child("value"),
                )?;
                let value_0: IfccadLeaderGraphicInheritance<IfccadLinePatternId> =
                    f.take::<IfccadLeaderGraphicInheritance<IfccadLinePatternId>>(
                        "linePattern",
                        prefix,
                    )?;
                Self::SetLinePattern(value_0)
            }
            "setSelectionMarker" => {
                let f = Fields::new(outer.required("value")?, &["marker"], &outer.child("value"))?;
                let value_0: u32 = f.take::<u32>("marker", prefix)?;
                Self::SetSelectionMarker(value_0)
            }
            "setFill" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["enabled"],
                    &outer.child("value"),
                )?;
                let value_0: bool = f.take::<bool>("enabled", prefix)?;
                Self::SetFill(value_0)
            }
            "setLineWeight" => {
                let f = Fields::new(outer.required("value")?, &["weight"], &outer.child("value"))?;
                let value_0: IfccadLeaderGraphicLineWeight =
                    f.take::<IfccadLeaderGraphicLineWeight>("weight", prefix)?;
                Self::SetLineWeight(value_0)
            }
            "setLinePatternScale" => {
                let f = Fields::new(outer.required("value")?, &["scale"], &outer.child("value"))?;
                let value_0: f64 = f.take::<f64>("scale", prefix)?;
                Self::SetLinePatternScale(value_0)
            }
            "setThickness" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["thickness"],
                    &outer.child("value"),
                )?;
                let value_0: f64 = f.take::<f64>("thickness", prefix)?;
                Self::SetThickness(value_0)
            }
            "pushTransform" => {
                let f = Fields::new(outer.required("value")?, &["matrix"], &outer.child("value"))?;
                let value_0: [f64; 16] = f.take::<[f64; 16]>("matrix", prefix)?;
                Self::PushTransform(value_0)
            }
            "popTransform" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::PopTransform
            }
            "cadCompatibility51Zero" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::CadCompatibility51Zero
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicPrimitive {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::Point(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("position".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"point","value":value})
            }
            Self::Polyline { vertices, normal } => {
                let value = {
                    let mut object = Map::new();
                    if vertices.saved() {
                        object.insert("vertices".into(), vertices.write_value(prefix));
                    }
                    if normal.saved() {
                        object.insert("normal".into(), normal.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"polyline","value":value})
            }
            Self::Polygon { vertices } => {
                let value = {
                    let mut object = Map::new();
                    if vertices.saved() {
                        object.insert("vertices".into(), vertices.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"polygon","value":value})
            }
            Self::Circle {
                center,
                radius,
                normal,
            } => {
                let value = {
                    let mut object = Map::new();
                    if center.saved() {
                        object.insert("center".into(), center.write_value(prefix));
                    }
                    if radius.saved() {
                        object.insert("radius".into(), radius.write_value(prefix));
                    }
                    if normal.saved() {
                        object.insert("normal".into(), normal.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"circle","value":value})
            }
            Self::Circle3 { points } => {
                let value = {
                    let mut object = Map::new();
                    if points.saved() {
                        object.insert("points".into(), points.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"circle3","value":value})
            }
            Self::Arc {
                center,
                radius,
                normal,
                start_direction,
                sweep_angle,
                closure,
            } => {
                let value = {
                    let mut object = Map::new();
                    if center.saved() {
                        object.insert("center".into(), center.write_value(prefix));
                    }
                    if radius.saved() {
                        object.insert("radius".into(), radius.write_value(prefix));
                    }
                    if normal.saved() {
                        object.insert("normal".into(), normal.write_value(prefix));
                    }
                    if start_direction.saved() {
                        object.insert("startDirection".into(), start_direction.write_value(prefix));
                    }
                    if sweep_angle.saved() {
                        object.insert("sweepAngle".into(), sweep_angle.write_value(prefix));
                    }
                    if closure.saved() {
                        object.insert("closure".into(), closure.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"arc","value":value})
            }
            Self::Arc3 { points, closure } => {
                let value = {
                    let mut object = Map::new();
                    if points.saved() {
                        object.insert("points".into(), points.write_value(prefix));
                    }
                    if closure.saved() {
                        object.insert("closure".into(), closure.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"arc3","value":value})
            }
            Self::Text(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("text".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"text","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "point" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["position"],
                    &outer.child("value"),
                )?;
                let value_0: [f64; 3] = f.take::<[f64; 3]>("position", prefix)?;
                Self::Point(value_0)
            }
            "polyline" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["vertices", "normal"],
                    &outer.child("value"),
                )?;
                let vertices: Vec<[f64; 3]> = f.take::<Vec<[f64; 3]>>("vertices", prefix)?;
                let normal: Option<[f64; 3]> = f.take::<Option<[f64; 3]>>("normal", prefix)?;
                Self::Polyline { vertices, normal }
            }
            "polygon" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["vertices"],
                    &outer.child("value"),
                )?;
                let vertices: Vec<[f64; 3]> = f.take::<Vec<[f64; 3]>>("vertices", prefix)?;
                Self::Polygon { vertices }
            }
            "circle" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["center", "radius", "normal"],
                    &outer.child("value"),
                )?;
                let center: [f64; 3] = f.take::<[f64; 3]>("center", prefix)?;
                let radius: f64 = f.take::<f64>("radius", prefix)?;
                let normal: [f64; 3] = f.take::<[f64; 3]>("normal", prefix)?;
                Self::Circle {
                    center,
                    radius,
                    normal,
                }
            }
            "circle3" => {
                let f = Fields::new(outer.required("value")?, &["points"], &outer.child("value"))?;
                let points: [[f64; 3]; 3] = f.take::<[[f64; 3]; 3]>("points", prefix)?;
                Self::Circle3 { points }
            }
            "arc" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &[
                        "center",
                        "radius",
                        "normal",
                        "startDirection",
                        "sweepAngle",
                        "closure",
                    ],
                    &outer.child("value"),
                )?;
                let center: [f64; 3] = f.take::<[f64; 3]>("center", prefix)?;
                let radius: f64 = f.take::<f64>("radius", prefix)?;
                let normal: [f64; 3] = f.take::<[f64; 3]>("normal", prefix)?;
                let start_direction: [f64; 3] = f.take::<[f64; 3]>("startDirection", prefix)?;
                let sweep_angle: f64 = f.take::<f64>("sweepAngle", prefix)?;
                let closure: IfccadLeaderGraphicArcClosure =
                    f.take::<IfccadLeaderGraphicArcClosure>("closure", prefix)?;
                Self::Arc {
                    center,
                    radius,
                    normal,
                    start_direction,
                    sweep_angle,
                    closure,
                }
            }
            "arc3" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["points", "closure"],
                    &outer.child("value"),
                )?;
                let points: [[f64; 3]; 3] = f.take::<[[f64; 3]; 3]>("points", prefix)?;
                let closure: IfccadLeaderGraphicArcClosure =
                    f.take::<IfccadLeaderGraphicArcClosure>("closure", prefix)?;
                Self::Arc3 { points, closure }
            }
            "text" => {
                let f = Fields::new(outer.required("value")?, &["text"], &outer.child("value"))?;
                let value_0: Box<IfccadLeaderGraphicText> =
                    f.take::<Box<IfccadLeaderGraphicText>>("text", prefix)?;
                Self::Text(value_0)
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadLeaderGraphicArcClosure {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Simple => "simple",
                Self::Sector => "sector",
                Self::Chord => "chord",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("simple") => Ok(Self::Simple),
            Some("sector") => Ok(Self::Sector),
            Some("chord") => Ok(Self::Chord),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadLeaderPath {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::StraightSegments(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("vertices".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"straightSegments","value":value})
            }
            Self::SplineThroughVertices(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("vertices".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"splineThroughVertices","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "straightSegments" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["vertices"],
                    &outer.child("value"),
                )?;
                let value_0: Vec<[f64; 3]> = f.take::<Vec<[f64; 3]>>("vertices", prefix)?;
                Self::StraightSegments(value_0)
            }
            "splineThroughVertices" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["vertices"],
                    &outer.child("value"),
                )?;
                let value_0: Vec<[f64; 3]> = f.take::<Vec<[f64; 3]>>("vertices", prefix)?;
                Self::SplineThroughVertices(value_0)
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadLeaderAnnotationKind {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::None => "none",
                Self::MText => "mText",
                Self::Block => "block",
                Self::Tolerance => "tolerance",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("none") => Ok(Self::None),
            Some("mText") => Ok(Self::MText),
            Some("block") => Ok(Self::Block),
            Some("tolerance") => Ok(Self::Tolerance),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadLeaderHookDirection {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::OppositeHorizontal => "oppositeHorizontal",
                Self::AlongHorizontal => "alongHorizontal",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("oppositeHorizontal") => Ok(Self::OppositeHorizontal),
            Some("alongHorizontal") => Ok(Self::AlongHorizontal),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadLeaderArrow {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::DefaultClosedFilled => {
                json!({"kind":"defaultClosedFilled"})
            }
            Self::None => {
                json!({"kind":"none"})
            }
            Self::Symbol { definition_id } => {
                let value = {
                    let mut object = Map::new();
                    if definition_id.saved() {
                        object.insert(
                            "definition".into(),
                            write_reference(*definition_id, prefix, "block/"),
                        );
                    }
                    Value::Object(object)
                };
                json!({"kind":"symbol","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "defaultClosedFilled" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::DefaultClosedFilled
            }
            "none" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::None
            }
            "symbol" => {
                let f = Fields::new(
                    outer.required("value")?,
                    &["definition"],
                    &outer.child("value"),
                )?;
                let definition_id: u64 = read_reference(
                    f.required("definition")?,
                    prefix,
                    "block/",
                    &f.child("definition"),
                )?;
                Self::Symbol { definition_id }
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadToleranceNotation {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::CadFeatureControlTextV1 { code } => {
                let value = {
                    let mut object = Map::new();
                    if code.saved() {
                        object.insert("code".into(), code.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"cadFeatureControlTextV1","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "cadFeatureControlTextV1" => {
                let f = Fields::new(outer.required("value")?, &["code"], &outer.child("value"))?;
                let code: String = f.take::<String>("code", prefix)?;
                Self::CadFeatureControlTextV1 { code }
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadMultiLeaderPathKind {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Invisible => "invisible",
                Self::StraightSegments => "straightSegments",
                Self::SplineThroughVertices => "splineThroughVertices",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("invisible") => Ok(Self::Invisible),
            Some("straightSegments") => Ok(Self::StraightSegments),
            Some("splineThroughVertices") => Ok(Self::SplineThroughVertices),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadLeaderLineWeight {
    fn write_value(&self, prefix: &str) -> Value {
        match self {
            Self::ByLayer => {
                json!({"kind":"byLayer"})
            }
            Self::ByBlock => {
                json!({"kind":"byBlock"})
            }
            Self::DrawingDefault => {
                json!({"kind":"drawingDefault"})
            }
            Self::Explicit(value_0) => {
                let value = {
                    let mut object = Map::new();
                    if value_0.saved() {
                        object.insert("value".into(), value_0.write_value(prefix));
                    }
                    Value::Object(object)
                };
                json!({"kind":"explicit","value":value})
            }
        }
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let outer = Fields::new(value, &["kind", "value"], path)?;
        let kind: String = outer.take("kind", prefix)?;
        let result = match kind.as_str() {
            "byLayer" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByLayer
            }
            "byBlock" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::ByBlock
            }
            "drawingDefault" => {
                if outer.value("value").is_some() {
                    return Err(invalid(path, "unit variant forbids value"));
                }
                Self::DrawingDefault
            }
            "explicit" => {
                let f = Fields::new(outer.required("value")?, &["value"], &outer.child("value"))?;
                let value_0: f64 = f.take::<f64>("value", prefix)?;
                Self::Explicit(value_0)
            }
            _ => return Err(invalid(path, "unknown enum value")),
        };
        Ok(result)
    }
}
impl LeaderJsonValue for IfccadMultiLeaderLineProperty {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::PathKind => "pathKind",
                Self::Color => "color",
                Self::LinePattern => "linePattern",
                Self::LineWeight => "lineWeight",
                Self::Arrow => "arrow",
                Self::ArrowSize => "arrowSize",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("pathKind") => Ok(Self::PathKind),
            Some("color") => Ok(Self::Color),
            Some("linePattern") => Ok(Self::LinePattern),
            Some("lineWeight") => Ok(Self::LineWeight),
            Some("arrow") => Ok(Self::Arrow),
            Some("arrowSize") => Ok(Self::ArrowSize),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderConnection {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::GroupConnection => "groupConnection",
                Self::ExplicitLastVertex => "explicitLastVertex",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("groupConnection") => Ok(Self::GroupConnection),
            Some("explicitLastVertex") => Ok(Self::ExplicitLastVertex),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderAttachmentDirection {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Horizontal => "horizontal",
                Self::Vertical => "vertical",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("horizontal") => Ok(Self::Horizontal),
            Some("vertical") => Ok(Self::Vertical),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderBlockConnection {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Extents => "extents",
                Self::BasePoint => "basePoint",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("extents") => Ok(Self::Extents),
            Some("basePoint") => Ok(Self::BasePoint),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextAttachment {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::TopOfTopLine => "topOfTopLine",
                Self::MiddleOfTopLine => "middleOfTopLine",
                Self::MiddleOfText => "middleOfText",
                Self::MiddleOfBottomLine => "middleOfBottomLine",
                Self::BottomOfBottomLine => "bottomOfBottomLine",
                Self::BottomLine => "bottomLine",
                Self::BottomOfTopLineUnderlineBottomLine => "bottomOfTopLineUnderlineBottomLine",
                Self::BottomOfTopLineUnderlineTopLine => "bottomOfTopLineUnderlineTopLine",
                Self::BottomOfTopLineUnderlineAll => "bottomOfTopLineUnderlineAll",
                Self::CenterOfText => "centerOfText",
                Self::CenterOfTextOverline => "centerOfTextOverline",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("topOfTopLine") => Ok(Self::TopOfTopLine),
            Some("middleOfTopLine") => Ok(Self::MiddleOfTopLine),
            Some("middleOfText") => Ok(Self::MiddleOfText),
            Some("middleOfBottomLine") => Ok(Self::MiddleOfBottomLine),
            Some("bottomOfBottomLine") => Ok(Self::BottomOfBottomLine),
            Some("bottomLine") => Ok(Self::BottomLine),
            Some("bottomOfTopLineUnderlineBottomLine") => {
                Ok(Self::BottomOfTopLineUnderlineBottomLine)
            }
            Some("bottomOfTopLineUnderlineTopLine") => Ok(Self::BottomOfTopLineUnderlineTopLine),
            Some("bottomOfTopLineUnderlineAll") => Ok(Self::BottomOfTopLineUnderlineAll),
            Some("centerOfText") => Ok(Self::CenterOfText),
            Some("centerOfTextOverline") => Ok(Self::CenterOfTextOverline),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextAttachmentPoint {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Left => "left",
                Self::Center => "center",
                Self::Right => "right",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("left") => Ok(Self::Left),
            Some("center") => Ok(Self::Center),
            Some("right") => Ok(Self::Right),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextAlignment {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::Left => "left",
                Self::Center => "center",
                Self::Right => "right",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("left") => Ok(Self::Left),
            Some("center") => Ok(Self::Center),
            Some("right") => Ok(Self::Right),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderContentKind {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::None => "none",
                Self::MText => "mText",
                Self::Block => "block",
                Self::Tolerance => "tolerance",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("none") => Ok(Self::None),
            Some("mText") => Ok(Self::MText),
            Some("block") => Ok(Self::Block),
            Some("tolerance") => Ok(Self::Tolerance),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderTextAngle {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::ParallelToLastSegment => "parallelToLastSegment",
                Self::Horizontal => "horizontal",
                Self::ReadableParallel => "readableParallel",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("parallelToLastSegment") => Ok(Self::ParallelToLastSegment),
            Some("horizontal") => Ok(Self::Horizontal),
            Some("readableParallel") => Ok(Self::ReadableParallel),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderDrawOrder {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::HeadFirst => "headFirst",
                Self::TailFirst => "tailFirst",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("headFirst") => Ok(Self::HeadFirst),
            Some("tailFirst") => Ok(Self::TailFirst),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderContentDrawOrder {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::ContentFirst => "contentFirst",
                Self::LeadersFirst => "leadersFirst",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("contentFirst") => Ok(Self::ContentFirst),
            Some("leadersFirst") => Ok(Self::LeadersFirst),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
impl LeaderJsonValue for IfccadMultiLeaderProperty {
    fn write_value(&self, prefix: &str) -> Value {
        let _ = prefix;
        Value::String(
            match self {
                Self::PathKind => "pathKind",
                Self::LineColor => "lineColor",
                Self::LinePattern => "linePattern",
                Self::LineWeight => "lineWeight",
                Self::LandingEnabled => "landingEnabled",
                Self::LandingGap => "landingGap",
                Self::DoglegEnabled => "doglegEnabled",
                Self::DoglegLength => "doglegLength",
                Self::Arrow => "arrow",
                Self::ArrowSize => "arrowSize",
                Self::ContentKind => "contentKind",
                Self::TextStyle => "textStyle",
                Self::TextLeftAttachment => "textLeftAttachment",
                Self::TextAngle => "textAngle",
                Self::TextAlignment => "textAlignment",
                Self::TextColor => "textColor",
                Self::TextHeight => "textHeight",
                Self::TextFrame => "textFrame",
                Self::UseDefaultMText => "useDefaultMText",
                Self::BlockDefinition => "blockDefinition",
                Self::BlockColor => "blockColor",
                Self::BlockScale => "blockScale",
                Self::BlockRotation => "blockRotation",
                Self::BlockConnection => "blockConnection",
                Self::ScaleFactor => "scaleFactor",
                Self::TextRightAttachment => "textRightAttachment",
                Self::TextSwitchAlignment => "textSwitchAlignment",
                Self::TextAttachmentDirection => "textAttachmentDirection",
                Self::TextTopAttachment => "textTopAttachment",
                Self::TextBottomAttachment => "textBottomAttachment",
            }
            .into(),
        )
    }
    fn read_value(value: &Value, prefix: &str, path: &str) -> Result<Self, IfccadReport> {
        let _ = prefix;
        match value.as_str() {
            Some("pathKind") => Ok(Self::PathKind),
            Some("lineColor") => Ok(Self::LineColor),
            Some("linePattern") => Ok(Self::LinePattern),
            Some("lineWeight") => Ok(Self::LineWeight),
            Some("landingEnabled") => Ok(Self::LandingEnabled),
            Some("landingGap") => Ok(Self::LandingGap),
            Some("doglegEnabled") => Ok(Self::DoglegEnabled),
            Some("doglegLength") => Ok(Self::DoglegLength),
            Some("arrow") => Ok(Self::Arrow),
            Some("arrowSize") => Ok(Self::ArrowSize),
            Some("contentKind") => Ok(Self::ContentKind),
            Some("textStyle") => Ok(Self::TextStyle),
            Some("textLeftAttachment") => Ok(Self::TextLeftAttachment),
            Some("textAngle") => Ok(Self::TextAngle),
            Some("textAlignment") => Ok(Self::TextAlignment),
            Some("textColor") => Ok(Self::TextColor),
            Some("textHeight") => Ok(Self::TextHeight),
            Some("textFrame") => Ok(Self::TextFrame),
            Some("useDefaultMText") => Ok(Self::UseDefaultMText),
            Some("blockDefinition") => Ok(Self::BlockDefinition),
            Some("blockColor") => Ok(Self::BlockColor),
            Some("blockScale") => Ok(Self::BlockScale),
            Some("blockRotation") => Ok(Self::BlockRotation),
            Some("blockConnection") => Ok(Self::BlockConnection),
            Some("scaleFactor") => Ok(Self::ScaleFactor),
            Some("textRightAttachment") => Ok(Self::TextRightAttachment),
            Some("textSwitchAlignment") => Ok(Self::TextSwitchAlignment),
            Some("textAttachmentDirection") => Ok(Self::TextAttachmentDirection),
            Some("textTopAttachment") => Ok(Self::TextTopAttachment),
            Some("textBottomAttachment") => Ok(Self::TextBottomAttachment),
            _ => Err(invalid(path, "unknown enum value")),
        }
    }
}
