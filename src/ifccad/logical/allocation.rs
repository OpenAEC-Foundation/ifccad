use super::{IfccadLinePatternId, IfccadTextStyleId};

/// Drawing-local domains with independent allocation watermarks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadIdDomain {
    TableDataLink,
    DataTable,
    TableStyle,
    MultiLeaderStyle,
    Entity,
    Layer,
    Layout,
    Block,
    LinePattern,
    TextStyle,
    DimensionStyle,
    SymbolSource,
    PreservationRecord,
    Ucs,
    ModelWindow,
    AnnotationScale,
    AnnotationView,
}

/// Allocation cannot reserve a valid ID in the indicated domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("IFCCAD-ID-002 /allocation/{domain:?}: cannot allocate a valid ID")]
pub struct IfccadIdAllocationError {
    pub domain: IfccadIdDomain,
}

/// Persistent allocation watermarks; deletion never lowers them.
///
/// Allocation reserves an ID but does not insert an object into a document.
/// Keep the advanced counter even if the reserved ID is not used.
///
/// ```
/// use ifccad::IfccadIdCounters;
/// let mut ids = IfccadIdCounters::default();
/// let reserved = ids.allocate_entity_id()?;
/// assert_eq!(reserved, 1);
/// assert_eq!(ids.next_entity_id, 2);
/// # Ok::<(), ifccad::IfccadIdAllocationError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadIdCounters {
    pub next_table_data_link_id: u64,
    pub next_data_table_id: u64,
    pub next_table_style_id: u64,
    pub next_multi_leader_style_id: u64,
    pub next_annotation_view_id: u64,
    pub next_annotation_scale_id: u64,
    pub next_dimension_style_id: u64,
    pub next_symbol_source_id: u64,
    pub next_text_style_id: u64,
    pub next_preservation_record_id: u64,
    pub next_ucs_id: u64,
    pub next_model_window_id: u64,
    pub next_entity_id: u64,
    pub next_layer_id: u64,
    pub next_layout_id: u64,
    pub next_block_id: u64,
    pub next_line_pattern_id: u64,
}

impl Default for IfccadIdCounters {
    fn default() -> Self {
        Self {
            next_table_style_id: 1,
            next_table_data_link_id: 1,
            next_data_table_id: 1,
            next_multi_leader_style_id: 1,
            next_annotation_view_id: 1,
            next_annotation_scale_id: 1,
            next_dimension_style_id: 1,
            next_symbol_source_id: 1,
            next_text_style_id: 1,
            next_preservation_record_id: 1,
            next_ucs_id: 1,
            next_model_window_id: 1,
            next_entity_id: 1,
            next_layer_id: 1,
            next_layout_id: 1,
            next_block_id: 1,
            next_line_pattern_id: 1,
        }
    }
}

fn allocate(next: &mut u64, domain: IfccadIdDomain) -> Result<u64, IfccadIdAllocationError> {
    let id = *next;
    *next = id
        .checked_add(1)
        .ok_or(IfccadIdAllocationError { domain })?;
    Ok(id)
}

impl IfccadIdCounters {
    pub fn allocate_table_data_link_id(
        &mut self,
    ) -> Result<super::IfccadTableDataLinkId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_table_data_link_id,
            IfccadIdDomain::TableDataLink,
        )
        .map(super::IfccadTableDataLinkId)
    }
    pub fn allocate_data_table_id(
        &mut self,
    ) -> Result<super::IfccadDataTableId, IfccadIdAllocationError> {
        allocate(&mut self.next_data_table_id, IfccadIdDomain::DataTable)
            .map(super::IfccadDataTableId)
    }
    pub fn allocate_table_style_id(
        &mut self,
    ) -> Result<super::IfccadTableStyleId, IfccadIdAllocationError> {
        allocate(&mut self.next_table_style_id, IfccadIdDomain::TableStyle)
            .map(super::IfccadTableStyleId)
    }
    pub fn allocate_multi_leader_style_id(
        &mut self,
    ) -> Result<super::IfccadMultiLeaderStyleId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_multi_leader_style_id,
            IfccadIdDomain::MultiLeaderStyle,
        )
        .map(super::IfccadMultiLeaderStyleId)
    }
    pub fn allocate_annotation_view_id(
        &mut self,
    ) -> Result<super::IfccadAnnotationViewId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_annotation_view_id,
            IfccadIdDomain::AnnotationView,
        )
        .map(super::IfccadAnnotationViewId)
    }
    pub fn allocate_annotation_scale_id(
        &mut self,
    ) -> Result<super::IfccadAnnotationScaleId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_annotation_scale_id,
            IfccadIdDomain::AnnotationScale,
        )
        .map(super::IfccadAnnotationScaleId)
    }
    pub fn allocate_dimension_style_id(
        &mut self,
    ) -> Result<super::IfccadDimensionStyleId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_dimension_style_id,
            IfccadIdDomain::DimensionStyle,
        )
        .map(super::IfccadDimensionStyleId)
    }
    pub fn allocate_symbol_source_id(
        &mut self,
    ) -> Result<super::IfccadSymbolSourceId, IfccadIdAllocationError> {
        allocate(
            &mut self.next_symbol_source_id,
            IfccadIdDomain::SymbolSource,
        )
        .map(super::IfccadSymbolSourceId)
    }
    pub fn allocate_text_style_id(&mut self) -> Result<IfccadTextStyleId, IfccadIdAllocationError> {
        allocate(&mut self.next_text_style_id, IfccadIdDomain::TextStyle).map(IfccadTextStyleId)
    }
    pub fn allocate_preservation_record_id(
        &mut self,
    ) -> Result<super::IfccadPreservationRecordId, IfccadIdAllocationError> {
        if self.next_preservation_record_id == 0 {
            return Err(IfccadIdAllocationError {
                domain: IfccadIdDomain::PreservationRecord,
            });
        }
        allocate(
            &mut self.next_preservation_record_id,
            IfccadIdDomain::PreservationRecord,
        )
        .map(super::IfccadPreservationRecordId)
    }

    pub fn allocate_ucs_id(&mut self) -> Result<super::IfccadUcsId, IfccadIdAllocationError> {
        allocate(&mut self.next_ucs_id, IfccadIdDomain::Ucs).map(super::IfccadUcsId)
    }
    pub fn allocate_model_window_id(
        &mut self,
    ) -> Result<super::IfccadModelWindowId, IfccadIdAllocationError> {
        allocate(&mut self.next_model_window_id, IfccadIdDomain::ModelWindow)
            .map(super::IfccadModelWindowId)
    }
    pub fn allocate_entity_id(&mut self) -> Result<u64, IfccadIdAllocationError> {
        allocate(&mut self.next_entity_id, IfccadIdDomain::Entity)
    }

    pub fn allocate_layer_id(&mut self) -> Result<u64, IfccadIdAllocationError> {
        allocate(&mut self.next_layer_id, IfccadIdDomain::Layer)
    }

    pub fn allocate_layout_id(&mut self) -> Result<u64, IfccadIdAllocationError> {
        allocate(&mut self.next_layout_id, IfccadIdDomain::Layout)
    }

    pub fn allocate_block_id(&mut self) -> Result<u64, IfccadIdAllocationError> {
        allocate(&mut self.next_block_id, IfccadIdDomain::Block)
    }

    pub fn allocate_line_pattern_id(
        &mut self,
    ) -> Result<IfccadLinePatternId, IfccadIdAllocationError> {
        allocate(&mut self.next_line_pattern_id, IfccadIdDomain::LinePattern)
            .map(IfccadLinePatternId)
    }

    /// Reserve every typed numeric dependency target, including missing ones.
    /// A failed reservation leaves all watermarks unchanged.
    pub fn protect_preservation_targets(
        &mut self,
        preservation: &super::IfccadPreservation,
    ) -> Result<(), IfccadIdAllocationError> {
        let mut candidate = *self;
        for (domain, id) in preservation_targets(preservation) {
            let minimum = id
                .checked_add(1)
                .ok_or(IfccadIdAllocationError { domain })?;
            let (counter, _) = candidate.preservation_counter(domain);
            *counter = (*counter).max(minimum);
        }
        *self = candidate;
        Ok(())
    }

    fn preservation_counter(&mut self, domain: IfccadIdDomain) -> (&mut u64, &'static str) {
        match domain {
            IfccadIdDomain::Entity => (&mut self.next_entity_id, "nextEntityId"),
            IfccadIdDomain::Layer => (&mut self.next_layer_id, "nextLayerId"),
            IfccadIdDomain::LinePattern => (&mut self.next_line_pattern_id, "nextLinePatternId"),
            IfccadIdDomain::Layout => (&mut self.next_layout_id, "nextLayoutId"),
            IfccadIdDomain::Block => (&mut self.next_block_id, "nextBlockId"),
            IfccadIdDomain::PreservationRecord => (
                &mut self.next_preservation_record_id,
                "nextPreservationRecordId",
            ),
            _ => unreachable!("not a preservation target domain"),
        }
    }
}

fn preservation_targets(
    preservation: &super::IfccadPreservation,
) -> impl Iterator<Item = (IfccadIdDomain, u64)> + '_ {
    use super::IfccadPreservationTarget as Target;
    use IfccadIdDomain as Domain;
    preservation
        .records
        .iter()
        .flat_map(|record| {
            record
                .bindings
                .iter()
                .map(|binding| binding.target)
                .chain(record.conditions.iter().map(|condition| condition.target))
        })
        .filter_map(|target| {
            Some(match target {
                Target::Drawing => return None,
                Target::Entity(id) => (Domain::Entity, id),
                Target::Layer(id) => (Domain::Layer, id),
                Target::LinePattern(id) => (Domain::LinePattern, id.0),
                Target::Layout(id) => (Domain::Layout, id),
                Target::BlockDefinition(id) => (Domain::Block, id),
                Target::Record(id) => (Domain::PreservationRecord, id.0),
            })
        })
}

pub(super) fn validate(document: &super::IfccadDocument) -> Result<(), super::IfccadReport> {
    let mut ids = document.id_counters;
    if let Some(preservation) = &document.preservation {
        for (domain, target) in preservation_targets(preservation) {
            let (&mut next, field) = ids.preservation_counter(domain);
            if next <= target {
                return Err(crate::ifccad::diagnostics::failure(
                    "IFCCAD-ID-002",
                    &format!("/cad/d{}/ifccad::drawing/{field}", document.drawing_id),
                    format!("{field} {next} must exceed preservation target ID {target}"),
                ));
            }
        }
    }
    let maximum_entity = document
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
        .map(|entity| entity.id())
        .max();
    let maximum_layout = std::iter::once(document.model.id)
        .chain(document.paper_layouts.iter().map(|layout| layout.id))
        .max();
    for (field, next, maximum) in [
        (
            "nextTableDataLinkId",
            ids.next_table_data_link_id,
            document
                .table_data_links
                .iter()
                .map(|object| object.id.0)
                .max(),
        ),
        (
            "nextDataTableId",
            ids.next_data_table_id,
            document.data_tables.iter().map(|object| object.id.0).max(),
        ),
        (
            "nextTableStyleId",
            ids.next_table_style_id,
            document.table_styles.iter().map(|style| style.id.0).max(),
        ),
        (
            "nextMultiLeaderStyleId",
            ids.next_multi_leader_style_id,
            document
                .multi_leader_styles
                .iter()
                .map(|style| style.id.0)
                .max(),
        ),
        (
            "nextDimensionStyleId",
            ids.next_dimension_style_id,
            document.dimension_styles.iter().map(|s| s.id.0).max(),
        ),
        (
            "nextTextStyleId",
            ids.next_text_style_id,
            document.text_styles.iter().map(|s| s.id.0).max(),
        ),
        (
            "nextUcsId",
            ids.next_ucs_id,
            document.ucs_definitions.iter().map(|v| v.id.0).max(),
        ),
        (
            "nextModelWindowId",
            ids.next_model_window_id,
            document.model_windows.iter().map(|v| v.id.0).max(),
        ),
        ("nextEntityId", ids.next_entity_id, maximum_entity),
        (
            "nextPreservationRecordId",
            ids.next_preservation_record_id,
            document
                .preservation
                .as_ref()
                .and_then(|p| p.records.iter().map(|r| r.id.0).max()),
        ),
        (
            "nextLayerId",
            ids.next_layer_id,
            document.layers.iter().map(|layer| layer.id).max(),
        ),
        ("nextLayoutId", ids.next_layout_id, maximum_layout),
        (
            "nextBlockId",
            ids.next_block_id,
            document.blocks.iter().map(|block| block.id).max(),
        ),
        (
            "nextLinePatternId",
            ids.next_line_pattern_id,
            document
                .line_patterns
                .iter()
                .map(|pattern| pattern.id.0)
                .max(),
        ),
        (
            "nextSymbolSourceId",
            ids.next_symbol_source_id,
            document.symbol_sources.iter().map(|s| s.id.0).max(),
        ),
    ] {
        if next == 0 {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ID-002",
                &format!("/cad/d{}/ifccad::drawing/{field}", document.drawing_id),
                format!("{field} must be positive"),
            ));
        }
        if let Some(maximum) = maximum.filter(|maximum| next <= *maximum) {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-ID-002",
                &format!("/cad/d{}/ifccad::drawing/{field}", document.drawing_id),
                format!(
                    "/cad/d{} {field} {next} must exceed current ID {maximum}",
                    document.drawing_id
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifccad::IfccadLinePatternId;

    #[test]
    fn allocation_advances_only_its_domain() {
        let mut ids = IfccadIdCounters::default();
        assert_eq!(ids.allocate_entity_id().unwrap(), 1);
        assert_eq!(ids.allocate_entity_id().unwrap(), 2);
        assert_eq!(ids.next_layer_id, 1);
        assert_eq!(ids.next_layout_id, 1);
        assert_eq!(ids.next_block_id, 1);
        assert_eq!(ids.next_line_pattern_id, 1);
        assert_eq!(ids.allocate_layer_id().unwrap(), 1);
        assert_eq!(ids.allocate_layout_id().unwrap(), 1);
        assert_eq!(ids.allocate_block_id().unwrap(), 1);
        assert_eq!(
            ids.allocate_line_pattern_id().unwrap(),
            IfccadLinePatternId(1)
        );
        assert_eq!(
            ids,
            IfccadIdCounters {
                next_multi_leader_style_id: 1,
                next_table_style_id: 1,
                next_table_data_link_id: 1,
                next_data_table_id: 1,
                next_annotation_view_id: 1,
                next_annotation_scale_id: 1,
                next_dimension_style_id: 1,
                next_symbol_source_id: 1,
                next_text_style_id: 1,
                next_preservation_record_id: 1,
                next_ucs_id: 1,
                next_model_window_id: 1,

                next_entity_id: 3,
                next_layer_id: 2,
                next_layout_id: 2,
                next_block_id: 2,
                next_line_pattern_id: 2,
            }
        );
    }

    #[test]
    fn allocation_preserves_large_values() {
        let mut ids = IfccadIdCounters {
            next_symbol_source_id: 1,
            next_preservation_record_id: 1,
            next_entity_id: 9_007_199_254_740_993,
            next_line_pattern_id: 9_007_199_254_740_993,
            ..Default::default()
        };
        assert_eq!(ids.allocate_entity_id().unwrap(), 9_007_199_254_740_993);
        assert_eq!(
            ids.allocate_line_pattern_id().unwrap().0,
            9_007_199_254_740_993
        );
        assert_eq!(ids.next_entity_id, 9_007_199_254_740_994);
        assert_eq!(ids.next_line_pattern_id, 9_007_199_254_740_994);
    }

    #[test]
    fn exhaustion_is_atomic_in_every_domain() {
        for domain in [
            IfccadIdDomain::MultiLeaderStyle,
            IfccadIdDomain::PreservationRecord,
            IfccadIdDomain::Entity,
            IfccadIdDomain::Layer,
            IfccadIdDomain::Layout,
            IfccadIdDomain::Block,
            IfccadIdDomain::LinePattern,
            IfccadIdDomain::TextStyle,
            IfccadIdDomain::DimensionStyle,
            IfccadIdDomain::TableStyle,
            IfccadIdDomain::TableDataLink,
            IfccadIdDomain::DataTable,
            IfccadIdDomain::SymbolSource,
            IfccadIdDomain::Ucs,
            IfccadIdDomain::ModelWindow,
            IfccadIdDomain::AnnotationScale,
            IfccadIdDomain::AnnotationView,
        ] {
            let mut ids = IfccadIdCounters {
                next_multi_leader_style_id: u64::MAX - 1,
                next_annotation_view_id: u64::MAX - 1,
                next_annotation_scale_id: u64::MAX - 1,
                next_dimension_style_id: u64::MAX - 1,
                next_table_style_id: u64::MAX - 1,
                next_table_data_link_id: u64::MAX - 1,
                next_data_table_id: u64::MAX - 1,
                next_symbol_source_id: u64::MAX - 1,
                next_text_style_id: u64::MAX - 1,
                next_preservation_record_id: u64::MAX - 1,
                next_ucs_id: u64::MAX - 1,
                next_model_window_id: u64::MAX - 1,

                next_entity_id: u64::MAX - 1,
                next_layer_id: u64::MAX - 1,
                next_layout_id: u64::MAX - 1,
                next_block_id: u64::MAX - 1,
                next_line_pattern_id: u64::MAX - 1,
            };
            let allocate = |ids: &mut IfccadIdCounters| match domain {
                IfccadIdDomain::MultiLeaderStyle => {
                    ids.allocate_multi_leader_style_id().map(|id| id.0)
                }
                IfccadIdDomain::PreservationRecord => {
                    ids.allocate_preservation_record_id().map(|id| id.0)
                }
                IfccadIdDomain::AnnotationScale => {
                    ids.allocate_annotation_scale_id().map(|id| id.0)
                }
                IfccadIdDomain::AnnotationView => ids.allocate_annotation_view_id().map(|id| id.0),
                IfccadIdDomain::Entity => ids.allocate_entity_id(),
                IfccadIdDomain::Layer => ids.allocate_layer_id(),
                IfccadIdDomain::Layout => ids.allocate_layout_id(),
                IfccadIdDomain::Block => ids.allocate_block_id(),
                IfccadIdDomain::LinePattern => ids.allocate_line_pattern_id().map(|id| id.0),
                IfccadIdDomain::TextStyle => ids.allocate_text_style_id().map(|id| id.0),
                IfccadIdDomain::DimensionStyle => ids.allocate_dimension_style_id().map(|id| id.0),
                IfccadIdDomain::TableStyle => ids.allocate_table_style_id().map(|id| id.0),
                IfccadIdDomain::TableDataLink => ids.allocate_table_data_link_id().map(|id| id.0),
                IfccadIdDomain::DataTable => ids.allocate_data_table_id().map(|id| id.0),
                IfccadIdDomain::SymbolSource => ids.allocate_symbol_source_id().map(|id| id.0),
                IfccadIdDomain::Ucs => ids.allocate_ucs_id().map(|id| id.0),
                IfccadIdDomain::ModelWindow => ids.allocate_model_window_id().map(|id| id.0),
            };
            assert_eq!(allocate(&mut ids).unwrap(), u64::MAX - 1);
            let before = ids;
            assert_eq!(allocate(&mut ids).unwrap_err().domain, domain);
            assert_eq!(ids, before);
        }
    }
}
