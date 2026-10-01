use super::{
    IfcxCadEntity, IfcxCadEntityKind, IfcxCadLayer, IfcxCadMode, IfcxCadReport,
    IfcxCadResolvedAppearance, ValidatedIfcxCad,
};

fn one(message: impl Into<String>) -> IfcxCadReport {
    IfcxCadReport::one(message)
}

fn resolve<T: Clone>(
    mode: &IfcxCadMode<T>,
    layer: &T,
    instance: Option<(&IfcxCadMode<T>, &T)>,
) -> Result<T, IfcxCadReport> {
    match mode {
        IfcxCadMode::Explicit(value) => Ok(value.clone()),
        IfcxCadMode::ByLayer => Ok(layer.clone()),
        IfcxCadMode::ByBlock => match instance {
            Some((IfcxCadMode::Explicit(value), _)) => Ok(value.clone()),
            Some((IfcxCadMode::ByLayer, layer)) => Ok(layer.clone()),
            Some((IfcxCadMode::ByBlock, _)) => Err(one(
                "nested ByBlock resolution is unsupported in this proof",
            )),
            None => Err(one("top-level ByBlock has no effective value")),
        },
    }
}

impl ValidatedIfcxCad {
    /// Resolve one drawable occurrence; an optional containing block instance is innermost first.
    pub fn effective_appearance(
        &self,
        entity_path: &str,
        instance_chain: &[&str],
    ) -> Result<IfcxCadResolvedAppearance, IfcxCadReport> {
        if instance_chain.len() > 1 {
            return Err(one("nested block appearance resolution is unsupported"));
        }
        let drawing = &self.document;
        let path = |id: u64| format!("</cad/d{}/e{id}>", drawing.drawing_id);
        let model_entity = drawing
            .model
            .entities
            .iter()
            .find(|e| path(e.id) == entity_path);
        let block_entity = drawing.blocks.iter().find_map(|b| {
            b.entities
                .iter()
                .find(|e| path(e.id) == entity_path)
                .map(|e| (b.id, e))
        });
        let (block_id, entity): (Option<u64>, &IfcxCadEntity) = match (model_entity, block_entity) {
            (Some(e), None) => (None, e),
            (None, Some((id, e))) => (Some(id), e),
            _ => return Err(one(format!("unknown or ambiguous entity {entity_path}"))),
        };
        let containing = match (block_id, instance_chain.first()) {
            (None, None) => None,
            (Some(id), Some(instance_path)) => {
                let instance = drawing
                    .model
                    .entities
                    .iter()
                    .find(|e| path(e.id) == *instance_path)
                    .ok_or_else(|| one("containing instance must be in model layout"))?;
                match instance.kind {
                    IfcxCadEntityKind::BlockInstance { definition_id, .. }
                        if definition_id == id =>
                    {
                        Some(instance)
                    }
                    _ => return Err(one("instance does not reference entity's block definition")),
                }
            }
            _ => return Err(one("block entity needs exactly one containing instance")),
        };
        let layer = |id| -> Result<&IfcxCadLayer, IfcxCadReport> {
            drawing
                .layers
                .iter()
                .find(|layer| layer.id == id)
                .ok_or_else(|| one("missing layer"))
        };
        let own_layer = layer(entity.layer_id)?;
        let effective_layer = if own_layer.name == "0" && block_id.is_some() {
            layer(containing.unwrap().layer_id)?
        } else {
            own_layer
        };
        let instance_layer = containing.map(|e| layer(e.layer_id)).transpose()?;
        let a = &entity.appearance;
        let i = containing.map(|e| &e.appearance);
        let l = &effective_layer.appearance;
        let il = instance_layer.map(|v| &v.appearance);
        Ok(IfcxCadResolvedAppearance {
            color: resolve(
                &a.color,
                &l.color,
                i.zip(il).map(|(a, l)| (&a.color, &l.color)),
            )?,
            opacity: resolve(
                &a.opacity,
                &l.opacity,
                i.zip(il).map(|(a, l)| (&a.opacity, &l.opacity)),
            )?,
            line_pattern: resolve(
                &a.line_pattern,
                &l.line_pattern,
                i.zip(il).map(|(a, l)| (&a.line_pattern, &l.line_pattern)),
            )?,
            line_weight: resolve(
                &a.line_weight,
                &l.line_weight,
                i.zip(il).map(|(a, l)| (&a.line_weight, &l.line_weight)),
            )?,
        })
    }
}
