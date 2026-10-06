use crate::diagnostics::diagnostic;
use crate::units::unit_code;
use crate::*;
use ocdraw::ifccad::*;
use opencadcodec::{BlockRecord, CadDocument, EntityType};

pub(crate) fn allocate(
    doc: &mut CadDocument,
    defs: &[IfccadBlockDefinition],
    map: &mut IfccadMappings,
) -> Result<(), IfccadConversionError> {
    for def in defs {
        if def.name.starts_with('*') {
            return Err(IfccadConversionError::Unsupported(vec![diagnostic(
                "block-name",
                format!("block/{}", def.id),
                "reserved/anonymous block name is unsupported",
            )]));
        }
        let mut b = BlockRecord::new(&def.name);
        b.handle = doc.allocate_handle();
        b.block_entity_handle = doc.allocate_handle();
        b.block_end_handle = doc.allocate_handle();
        b.base_point = crate::geometry::v(def.base_point);
        b.units = unit_code(&def.insertion_unit);
        let mut start = opencadcodec::entities::Block::new(&def.name, b.base_point);
        start.common.handle = b.block_entity_handle;
        start.common.owner_handle = b.handle;
        let mut end = opencadcodec::entities::BlockEnd::new();
        end.common.handle = b.block_end_handle;
        end.common.owner_handle = b.handle;
        map.blocks.insert(def.id, b.handle);
        doc.block_records
            .add(b)
            .map_err(IfccadConversionError::CadConstruction)?;
        for e in [EntityType::Block(start), EntityType::BlockEnd(end)] {
            doc.add_entity(e)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
        }
    }
    Ok(())
}
pub(crate) fn from_insert(
    i: &opencadcodec::entities::Insert,
    id: u64,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadEntityKind> {
    let mut expected = opencadcodec::entities::Insert::new(&i.block_name, i.insert_point);
    expected.set_x_scale(i.x_scale());
    expected.set_y_scale(i.y_scale());
    expected.set_z_scale(i.z_scale());
    expected.rotation = i.rotation;
    expected.normal = i.normal;
    let before = issues.len();
    crate::source::residual(i, &expected, &["common"], loc, issues);
    if issues.len() != before {
        return None;
    }
    Some(IfccadEntityKind::BlockInstance {
        definition_id: id,
        transform: IfccadBlockTransform {
            placement: crate::geometry::xy(crate::geometry::p(i.insert_point)),
            rotation: i.rotation,
            scale: [i.x_scale(), i.y_scale(), i.z_scale()],
        },
    })
}
