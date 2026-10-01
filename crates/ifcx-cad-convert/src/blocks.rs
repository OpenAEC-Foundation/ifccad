use crate::outcome::{diagnostic, unit_code};
use crate::*;
use cadcodec::{BlockRecord, CadDocument, EntityType};
use ocdraw::ifcx_cad::*;

pub(crate) fn allocate(
    doc: &mut CadDocument,
    defs: &[IfcxCadBlockDefinition],
    map: &mut IfcxCadMappings,
) -> Result<(), IfcxCadConversionError> {
    for def in defs {
        if def.name.starts_with('*') {
            return Err(IfcxCadConversionError::Unsupported(vec![diagnostic(
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
        let mut start = cadcodec::entities::Block::new(&def.name, b.base_point);
        start.common.handle = b.block_entity_handle;
        start.common.owner_handle = b.handle;
        let mut end = cadcodec::entities::BlockEnd::new();
        end.common.handle = b.block_end_handle;
        end.common.owner_handle = b.handle;
        map.blocks.insert(def.id, b.handle);
        doc.block_records
            .add(b)
            .map_err(IfcxCadConversionError::CadConstruction)?;
        for e in [EntityType::Block(start), EntityType::BlockEnd(end)] {
            doc.add_entity(e)
                .map_err(|e| IfcxCadConversionError::CadConstruction(e.to_string()))?;
        }
    }
    Ok(())
}
pub(crate) fn to_insert(
    name: &str,
    t: &IfcxCadBlockTransform,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> EntityType {
    crate::geometry::canonical(&t.placement, loc, issues);
    let mut i = cadcodec::entities::Insert::new(name, crate::geometry::v(t.placement.origin));
    i.rotation = t.rotation;
    i.set_x_scale(t.scale[0]);
    i.set_y_scale(t.scale[1]);
    i.set_z_scale(t.scale[2]);
    if [i.x_scale(), i.y_scale(), i.z_scale()] != t.scale {
        issues.push(diagnostic(
            "scale-clamped",
            loc,
            "CAD setters changed the requested scale",
        ));
    }
    EntityType::Insert(i)
}
pub(crate) fn from_insert(
    i: &cadcodec::entities::Insert,
    id: u64,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Option<IfcxCadEntityKind> {
    let mut expected = cadcodec::entities::Insert::new(&i.block_name, i.insert_point);
    expected.set_x_scale(i.x_scale());
    expected.set_y_scale(i.y_scale());
    expected.set_z_scale(i.z_scale());
    expected.rotation = i.rotation;
    let before = issues.len();
    crate::source::residual(i, &expected, &["common"], loc, issues);
    if issues.len() != before {
        return None;
    }
    Some(IfcxCadEntityKind::BlockInstance {
        definition_id: id,
        transform: IfcxCadBlockTransform {
            placement: crate::geometry::xy(crate::geometry::p(i.insert_point)),
            rotation: i.rotation,
            scale: [i.x_scale(), i.y_scale(), i.z_scale()],
        },
    })
}
