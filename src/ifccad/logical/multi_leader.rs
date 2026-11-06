//! Owned Multileader branch definitions; local keys are independent of CAD indices.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderPathKind {
    Invisible,
    StraightSegments,
    SplineThroughVertices,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadLeaderLineWeight {
    ByLayer,
    ByBlock,
    DrawingDefault,
    /// Millimetres, independent of drawing units.
    Explicit(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderLineSettings {
    pub path_kind: IfccadMultiLeaderPathKind,
    pub color: IfccadMode<IfccadColor>,
    pub line_pattern: IfccadMode<IfccadLinePatternId>,
    pub line_weight: IfccadLeaderLineWeight,
    pub arrow: IfccadLeaderArrow,
    pub arrow_size: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IfccadMultiLeaderLineProperty {
    PathKind,
    Color,
    LinePattern,
    LineWeight,
    Arrow,
    ArrowSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderConnection {
    GroupConnection,
    ExplicitLastVertex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadMultiLeaderAttachmentDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IfccadLeaderBreakInterval {
    pub start: [f64; 3],
    pub end: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderSegmentBreaks {
    /// Index in the effective path, including an active separate group endpoint.
    pub segment_index: u32,
    pub intervals: Vec<IfccadLeaderBreakInterval>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderLine {
    pub key: u64,
    pub source_index: Option<i32>,
    pub settings: IfccadMultiLeaderLineSettings,
    pub overridden_properties: BTreeSet<IfccadMultiLeaderLineProperty>,
    pub vertices: Vec<[f64; 3]>,
    pub connection: IfccadMultiLeaderConnection,
    pub breaks: Vec<IfccadMultiLeaderSegmentBreaks>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadMultiLeaderGroup {
    pub key: u64,
    pub source_index: Option<i32>,
    pub has_connection_point: bool,
    pub connection_point: [f64; 3],
    pub has_dogleg_direction: bool,
    pub dogleg_direction: [f64; 3],
    pub dogleg_length: f64,
    pub text_attachment_direction: IfccadMultiLeaderAttachmentDirection,
    pub lines: Vec<IfccadMultiLeaderLine>,
    pub breaks: Vec<IfccadLeaderBreakInterval>,
}

fn invalid(path: &str, detail: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-MULTILEADER-001", path, detail)
}
fn point(value: [f64; 3], path: &str) -> Result<(), IfccadReport> {
    if value.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(invalid(path, "coordinates must be finite"))
    }
}
fn nonnegative(value: f64, path: &str) -> Result<(), IfccadReport> {
    if value.is_finite() && value >= 0. {
        Ok(())
    } else {
        Err(invalid(path, "value must be finite and nonnegative"))
    }
}
fn intervals(values: &[IfccadLeaderBreakInterval], path: &str) -> Result<(), IfccadReport> {
    for (index, interval) in values.iter().enumerate() {
        point(interval.start, &format!("{path}/{index}/start"))?;
        point(interval.end, &format!("{path}/{index}/end"))?;
    }
    Ok(())
}
impl IfccadMultiLeaderLineSettings {
    pub fn validate(&self, drawing: &IfccadDocument) -> Result<(), IfccadReport> {
        if let IfccadMode::Explicit(color) = &self.color {
            if !color.is_valid() {
                return Err(invalid("/multiLeader/line/color", "invalid concrete color"));
            }
        }
        if let IfccadMode::Explicit(id) = &self.line_pattern {
            if !drawing
                .line_patterns
                .iter()
                .any(|pattern| pattern.id == *id)
            {
                return Err(invalid(
                    "/multiLeader/line/linePattern",
                    "unresolved line pattern",
                ));
            }
        }
        if let IfccadLeaderLineWeight::Explicit(value) = self.line_weight {
            nonnegative(value, "/multiLeader/line/lineWeight")?;
        }
        if let IfccadLeaderArrow::Symbol { definition_id } = self.arrow {
            if !drawing.blocks.iter().any(|block| block.id == definition_id) {
                return Err(invalid("/multiLeader/line/arrow", "unresolved arrow block"));
            }
        }
        nonnegative(self.arrow_size, "/multiLeader/line/arrowSize")
    }
}

/// Validate one presentation's ordered owned branches and positive local-key
/// watermark. Source indices are not keys and are not globally unique identities.
pub fn validate_ifccad_multi_leader_branches(
    drawing: &IfccadDocument,
    groups: &[IfccadMultiLeaderGroup],
    next_part_id: u64,
) -> Result<(), IfccadReport> {
    if next_part_id == 0 {
        return Err(invalid(
            "/multiLeader/nextPartId",
            "watermark must be positive",
        ));
    }
    let mut keys = BTreeSet::new();
    let mut key = |value: u64, path: &str| {
        if value == 0 || value >= next_part_id || !keys.insert(value) {
            Err(invalid(
                path,
                "duplicate, zero or unallocated local part key",
            ))
        } else {
            Ok(())
        }
    };
    for (index, group) in groups.iter().enumerate() {
        let path = format!("/multiLeader/groups/{index}");
        key(group.key, &format!("{path}/key"))?;
        point(group.connection_point, &format!("{path}/connectionPoint"))?;
        point(group.dogleg_direction, &format!("{path}/doglegDirection"))?;
        if group.has_dogleg_direction && group.dogleg_direction == [0.; 3] {
            return Err(invalid(
                &format!("{path}/doglegDirection"),
                "active direction must be nonzero",
            ));
        }
        nonnegative(group.dogleg_length, &format!("{path}/doglegLength"))?;
        intervals(&group.breaks, &format!("{path}/breaks"))?;
        for (index, line) in group.lines.iter().enumerate() {
            let path = format!("{path}/lines/{index}");
            key(line.key, &format!("{path}/key"))?;
            line.settings.validate(drawing).map_err(|report| {
                crate::ifccad::diagnostics::context(report, "IFCCAD-MULTILEADER-001", &path)
            })?;
            for (index, value) in line.vertices.iter().enumerate() {
                point(*value, &format!("{path}/vertices/{index}"))?;
            }
            let uses_group = line.connection == IfccadMultiLeaderConnection::GroupConnection;
            let has_endpoint = uses_group && group.has_connection_point;
            let point_count = line
                .vertices
                .len()
                .checked_add(usize::from(has_endpoint))
                .ok_or_else(|| invalid(&path, "effective path length overflow"))?;
            if line.settings.path_kind != IfccadMultiLeaderPathKind::Invisible
                && (point_count < 2 || (uses_group && !group.has_connection_point))
            {
                return Err(invalid(
                    &path,
                    "visible path requires two effective points and its declared connection",
                ));
            }
            for (index, breaks) in line.breaks.iter().enumerate() {
                let path = format!("{path}/breaks/{index}");
                if u64::from(breaks.segment_index) >= point_count.saturating_sub(1) as u64 {
                    return Err(invalid(
                        &path,
                        "break segment does not exist in effective path",
                    ));
                }
                intervals(&breaks.intervals, &path)?;
            }
        }
    }
    Ok(())
}
