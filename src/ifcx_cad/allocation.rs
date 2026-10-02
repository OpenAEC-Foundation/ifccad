use super::IfcxCadLinePatternId;

/// Drawing-local domains with independent allocation watermarks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfcxCadIdDomain {
    Entity,
    Layer,
    Layout,
    Block,
    LinePattern,
}

/// Allocation cannot advance the indicated domain without overflowing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("IFCX-CAD {domain:?} ID space exhausted")]
pub struct IfcxCadIdAllocationError {
    pub domain: IfcxCadIdDomain,
}

/// Persistent allocation watermarks; deletion never lowers them.
///
/// Allocation reserves an ID but does not insert an object into a document.
/// Keep the advanced counter even if the reserved ID is not used.
///
/// ```
/// use ocdraw::ifcx_cad::IfcxCadIdCounters;
/// let mut ids = IfcxCadIdCounters::default();
/// let reserved = ids.allocate_entity_id()?;
/// assert_eq!(reserved, 1);
/// assert_eq!(ids.next_entity_id, 2);
/// # Ok::<(), ocdraw::ifcx_cad::IfcxCadIdAllocationError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfcxCadIdCounters {
    pub next_entity_id: u64,
    pub next_layer_id: u64,
    pub next_layout_id: u64,
    pub next_block_id: u64,
    pub next_line_pattern_id: u64,
}

impl Default for IfcxCadIdCounters {
    fn default() -> Self {
        Self {
            next_entity_id: 1,
            next_layer_id: 1,
            next_layout_id: 1,
            next_block_id: 1,
            next_line_pattern_id: 1,
        }
    }
}

fn allocate(next: &mut u64, domain: IfcxCadIdDomain) -> Result<u64, IfcxCadIdAllocationError> {
    let id = *next;
    *next = id
        .checked_add(1)
        .ok_or(IfcxCadIdAllocationError { domain })?;
    Ok(id)
}

impl IfcxCadIdCounters {
    pub fn allocate_entity_id(&mut self) -> Result<u64, IfcxCadIdAllocationError> {
        allocate(&mut self.next_entity_id, IfcxCadIdDomain::Entity)
    }

    pub fn allocate_layer_id(&mut self) -> Result<u64, IfcxCadIdAllocationError> {
        allocate(&mut self.next_layer_id, IfcxCadIdDomain::Layer)
    }

    pub fn allocate_layout_id(&mut self) -> Result<u64, IfcxCadIdAllocationError> {
        allocate(&mut self.next_layout_id, IfcxCadIdDomain::Layout)
    }

    pub fn allocate_block_id(&mut self) -> Result<u64, IfcxCadIdAllocationError> {
        allocate(&mut self.next_block_id, IfcxCadIdDomain::Block)
    }

    pub fn allocate_line_pattern_id(
        &mut self,
    ) -> Result<IfcxCadLinePatternId, IfcxCadIdAllocationError> {
        allocate(&mut self.next_line_pattern_id, IfcxCadIdDomain::LinePattern)
            .map(IfcxCadLinePatternId)
    }
}

pub(super) fn validate(document: &super::IfcxCadDocument) -> Result<(), super::IfcxCadReport> {
    let ids = document.id_counters;
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
        .map(|entity| entity.id)
        .max();
    let maximum_layout = std::iter::once(document.model.id)
        .chain(document.paper_layouts.iter().map(|layout| layout.id))
        .max();
    for (field, next, maximum) in [
        ("nextEntityId", ids.next_entity_id, maximum_entity),
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
    ] {
        if let Some(maximum) = maximum.filter(|maximum| next <= *maximum) {
            return Err(super::IfcxCadReport::one(format!(
                "/cad/d{} {field} {next} must exceed current ID {maximum}",
                document.drawing_id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifcx_cad::IfcxCadLinePatternId;

    #[test]
    fn allocation_advances_only_its_domain() {
        let mut ids = IfcxCadIdCounters::default();
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
            IfcxCadLinePatternId(1)
        );
        assert_eq!(
            ids,
            IfcxCadIdCounters {
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
        let mut ids = IfcxCadIdCounters {
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
            IfcxCadIdDomain::Entity,
            IfcxCadIdDomain::Layer,
            IfcxCadIdDomain::Layout,
            IfcxCadIdDomain::Block,
            IfcxCadIdDomain::LinePattern,
        ] {
            let mut ids = IfcxCadIdCounters {
                next_entity_id: u64::MAX - 1,
                next_layer_id: u64::MAX - 1,
                next_layout_id: u64::MAX - 1,
                next_block_id: u64::MAX - 1,
                next_line_pattern_id: u64::MAX - 1,
            };
            let allocate = |ids: &mut IfcxCadIdCounters| match domain {
                IfcxCadIdDomain::Entity => ids.allocate_entity_id(),
                IfcxCadIdDomain::Layer => ids.allocate_layer_id(),
                IfcxCadIdDomain::Layout => ids.allocate_layout_id(),
                IfcxCadIdDomain::Block => ids.allocate_block_id(),
                IfcxCadIdDomain::LinePattern => ids.allocate_line_pattern_id().map(|id| id.0),
            };
            assert_eq!(allocate(&mut ids).unwrap(), u64::MAX - 1);
            let before = ids;
            assert_eq!(allocate(&mut ids).unwrap_err().domain, domain);
            assert_eq!(ids, before);
        }
    }
}
