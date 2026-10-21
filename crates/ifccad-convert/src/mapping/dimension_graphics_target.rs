//! Stage complete cached pictures before committing any target block/link.
use crate::*;
use crate::{
    dimension_graphics_context::{PictureMap, PictureTransform},
    geometry_context::{GeometryContext, GeometryNodeKey as Key},
};
use cad_geometry_convert::{geometry::blocks::EvaluatedBlock, GeometryPair};
use ifccad::*;
use opencadcodec::{CadDocument, EntityType};
use std::collections::BTreeSet;
struct Body {
    entity: EntityType,
    pair: GeometryPair,
    instance: Option<(u64, EvaluatedBlock)>,
}
struct PreparedPart {
    entity: EntityType,
    pair: Option<GeometryPair>,
    instance: Option<(u64, EvaluatedBlock, EvaluatedBlock)>,
    source: IfccadGeometryEntitySource,
}
pub(crate) struct PreparedDimensionPicture {
    parts: Vec<PreparedPart>,
    id: u64,
    source: IfccadGeometryEntitySource,
    source_map: EvaluatedBlock,
    target_map: EvaluatedBlock,
}
fn loss(
    issues: &mut Vec<IfccadDiagnostic>,
    location: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(crate::diagnostics::diagnostic(
        "dimension-graphics-target",
        location,
        message,
    ));
}
fn combine(pairs: Vec<GeometryPair>) -> GeometryPair {
    let mut out = GeometryPair {
        points: vec![],
        curves: vec![],
    };
    for p in pairs {
        out.points.extend(p.points);
        out.curves.extend(p.curves);
    }
    out
}
fn native_transform(
    v: &IfccadBlockTransform,
) -> Result<ifccad::geometry_kernel::BlockTransform, IfccadConversionError> {
    ifccad::geometry_kernel::BlockTransform::try_new(
        v.placement
            .coordinate_frame()
            .map_err(IfccadConversionError::CoreValidation)?,
        v.rotation,
        ifccad::geometry_kernel::Scale3::new(v.scale[0], v.scale[1], v.scale[2]),
    )
    .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))
}
struct Export<'a, 'b> {
    drawing: &'a IfccadDocument,
    target: &'a CadDocument,
    maps: &'a IfccadMappings,
    geometry: &'b mut GeometryContext,
    issues: &'b mut Vec<IfccadDiagnostic>,
    done: BTreeSet<u64>,
    active: BTreeSet<u64>,
}
impl Export<'_, '_> {
    fn body(
        &mut self,
        kind: &IfccadEntityKind,
        id: u64,
    ) -> Result<Option<Body>, IfccadConversionError> {
        if let IfccadEntityKind::BlockInstance {
            definition_id,
            transform,
        } = kind
        {
            let Some(handle) = self.maps.blocks.cad_handle(*definition_id) else {
                return Ok(None);
            };
            let b = self
                .drawing
                .blocks
                .iter()
                .find(|b| b.id == *definition_id)
                .expect("validated marker");
            let name = &self
                .target
                .block_records
                .iter()
                .find(|b| b.handle == handle)
                .expect("allocated marker")
                .name;
            let source = IfccadGeometryEntitySource::NativeEntity {
                owner: self.geometry.owner,
                entity_id: id,
            };
            let result = cad_geometry_convert::geometry::blocks::to_cad_instance_parts(
                native_transform(transform)?,
                name,
                b.base_point,
                source,
                &self.geometry.state.assessment,
            );
            let (entity, _, target, _) = match result {
                Ok(v) => v,
                Err(cad_geometry_convert::CadGeometryError::Geometry(e)) => {
                    return Err(self.geometry.numerical_failure(e))
                }
                Err(cad_geometry_convert::CadGeometryError::Cad(m)) => {
                    loss(self.issues, format!("entity/{id}/graphics"), m);
                    return Ok(None);
                }
            };
            return Ok(Some(Body {
                entity: EntityType::Insert(entity),
                pair: GeometryPair {
                    points: vec![],
                    curves: vec![],
                },
                instance: Some((*definition_id, target)),
            }));
        }
        self.geometry.buffered_pairs = Some(vec![]);
        let result = match kind {
            IfccadEntityKind::Text(_) | IfccadEntityKind::MText(_) => super::text::to_entity(
                &self.drawing.text_styles,
                kind,
                id,
                self.maps,
                self.geometry,
                self.issues,
            ),
            IfccadEntityKind::Hatch(h) => super::hatch::to_cad(h, id, self.geometry, self.issues),
            IfccadEntityKind::Dimension(_) | IfccadEntityKind::Viewport(_) => Ok(None),
            _ => super::geometry::to_entity(kind, id, self.geometry, self.issues),
        };
        let pair = combine(
            self.geometry
                .buffered_pairs
                .take()
                .expect("target picture proof"),
        );
        Ok(result?.map(|entity| Body {
            entity,
            pair,
            instance: None,
        }))
    }
    fn marker(&mut self, id: u64) -> Result<bool, IfccadConversionError> {
        if self.done.contains(&id) {
            return Ok(true);
        }
        if !self.active.insert(id) {
            return Err(IfccadConversionError::InvalidStructure(
                "cyclic target graphics marker".into(),
            ));
        }
        let b = self
            .drawing
            .blocks
            .iter()
            .find(|b| b.id == id)
            .expect("validated marker");
        if self.maps.blocks.cad_handle(id).is_none() {
            self.active.remove(&id);
            return Ok(false);
        }
        for e in &b.entities {
            let e = e.as_native().expect("validated graphics closure");
            if let IfccadEntityKind::BlockInstance { definition_id, .. } = &e.kind {
                if !self.marker(*definition_id)? {
                    self.active.remove(&id);
                    return Ok(false);
                }
            }
            if self.body(&e.kind, e.id)?.is_none() {
                self.active.remove(&id);
                return Ok(false);
            }
        }
        self.active.remove(&id);
        self.done.insert(id);
        Ok(true)
    }
}
fn project_cache(kind: &mut IfccadDimensionGraphicsKind, limit: f64) -> bool {
    use IfccadDimensionGraphicsKind as K;
    let scalar = |z: &mut f64| {
        if z.abs() > limit {
            return false;
        }
        *z = 0.;
        true
    };
    let p = match kind {
        K::LineSegment { start, end } => return scalar(&mut start[2]) && scalar(&mut end[2]),
        K::SpatialPolyline { vertices, .. } => {
            return vertices.iter_mut().all(|v| scalar(&mut v[2]))
        }
        K::Point { placement }
        | K::Circle { placement, .. }
        | K::Arc { placement, .. }
        | K::Ellipse { placement, .. }
        | K::EllipseArc { placement, .. }
        | K::PlanarPolyline { placement, .. } => placement,
        K::Solid(v) | K::Trace(v) => &mut v.placement,
        K::Hatch(v) => &mut v.placement,
        K::Text(v) => &mut v.placement,
        K::MText(v) => &mut v.placement,
        K::BlockInstance { transform, .. } => &mut transform.placement,
    };
    // Projection is a target construction step. The final complete owner-space
    // proof still enforces the same hard budget, including all filled top slots.
    scalar(&mut p.origin[2])
        && scalar(&mut p.x_axis[2])
        && scalar(&mut p.y_axis[2])
        && p.x_axis[0] * p.y_axis[1] - p.x_axis[1] * p.y_axis[0] > 0.
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare(
    d: &IfccadDimension,
    id: u64,
    layer_id: u64,
    drawing: &IfccadDocument,
    target: &CadDocument,
    cad: &mut opencadcodec::entities::Dimension,
    maps: &IfccadMappings,
    geometry: &mut GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<PreparedDimensionPicture>, IfccadConversionError> {
    let Some(g) = &d.graphics else {
        return Ok(None);
    };
    let destination = cad;
    let mut candidate = destination.clone();
    let cad = &mut candidate;

    let loc = format!("entity/{id}/graphics");
    let frame = d
        .placement
        .coordinate_frame()
        .map_err(IfccadConversionError::CoreValidation)?;
    let source_map = PictureMap::native(frame);
    if matches!(
        d.text_placement,
        ifccad::dimension::DimensionTextPlacement::StyleDriven
    ) {
        let anchor = g
            .parts
            .iter()
            .filter(|p| p.purpose == IfccadDimensionGraphicsPurpose::Display)
            .find_map(|p| match &p.kind {
                IfccadDimensionGraphicsKind::Text(t) => Some(t.placement.origin),
                IfccadDimensionGraphicsKind::MText(t) => Some(t.placement.origin),
                _ => None,
            })
            .unwrap_or_else(|| {
                let p = match &d.definition {
                    ifccad::dimension::DimensionDefinition::Linear {
                        dimension_line_point,
                        ..
                    }
                    | ifccad::dimension::DimensionDefinition::Aligned {
                        dimension_line_point,
                        ..
                    } => dimension_line_point,
                };
                [p[0], p[1], 0.]
            });
        let mut zero = cad.clone();
        zero.base_mut().text_middle_point.z = 0.;
        let axes = PictureMap::cad(&zero, [0.; 3])?;
        let anchor = PictureTransform::new(source_map.clone(), axes)?.position(anchor, true)?;
        cad.base_mut().text_middle_point = super::geometry::v(anchor);
    }
    let target_map = PictureMap::cad(cad, [0.; 3])?;
    let rebind = PictureTransform::new(source_map.clone(), target_map.clone())?;
    let planar =
        !(cad.base().normal.x == 0. && cad.base().normal.y == 0. && cad.base().normal.z > 0.);
    let root_layer = drawing
        .layers
        .iter()
        .find(|l| l.id == layer_id)
        .expect("validated Dimension layer");
    let source = IfccadGeometryEntitySource::NativeEntity {
        owner: geometry.owner,
        entity_id: id,
    };
    let mut export = Export {
        drawing,
        target,
        maps,
        geometry,
        issues,
        done: BTreeSet::new(),
        active: BTreeSet::new(),
    };
    let mut parts = vec![];
    for (index, p) in g.parts.iter().enumerate() {
        let location = format!("{loc}/p{index}");
        let layer = match p.layer_id {
            None => "0",
            Some(id) => {
                &drawing
                    .layers
                    .iter()
                    .find(|l| l.id == id)
                    .expect("validated graphics layer")
                    .name
            }
        };
        if p.layer_id.is_some()
            && layer == "0"
            && (root_layer.name != "0"
                || matches!(
                    export.geometry.owner,
                    IfccadGeometryOwner::BlockDefinition(_)
                ))
        {
            loss(
                export.issues,
                &location,
                "explicit layer 0 would inherit another context in CAD",
            );
            return Ok(None);
        }
        if p.purpose == IfccadDimensionGraphicsPurpose::ReferencePoint
            && (p.layer_id.is_none() || !layer.eq_ignore_ascii_case("Defpoints"))
        {
            loss(
                export.issues,
                &location,
                "ReferencePoint requires an explicit local Defpoints layer",
            );
            return Ok(None);
        }
        if p.purpose == IfccadDimensionGraphicsPurpose::Display
            && matches!(p.kind, IfccadDimensionGraphicsKind::Point { .. })
            && p.layer_id.is_some()
            && layer.eq_ignore_ascii_case("Defpoints")
        {
            loss(
                export.issues,
                &location,
                "Display Point would acquire the CAD Defpoints helper role",
            );
            return Ok(None);
        }
        if let IfccadDimensionGraphicsKind::BlockInstance { definition_id, .. } = &p.kind {
            if !export.marker(*definition_id)? {
                loss(
                    export.issues,
                    &location,
                    "complete target marker closure unavailable",
                );
                return Ok(None);
            }
        }
        let checkpoint = export.issues.len();
        let Some(original) = export.body(&p.kind.clone().into(), id)? else {
            loss(
                export.issues,
                &location,
                "native source operands cannot be prepared",
            );
            return Ok(None);
        };
        export.issues.truncate(checkpoint);
        let mut cache = p.kind.clone();
        rebind.rebind(&mut cache)?;
        if planar
            && !project_cache(
                &mut cache,
                export
                    .geometry
                    .state
                    .assessment
                    .resolved_tolerance()
                    .lower(),
            )
        {
            loss(
                export.issues,
                &location,
                "graphics does not fit the target planar cache convention",
            );
            return Ok(None);
        }
        let Some(mut body) = export.body(&cache.into(), id)? else {
            loss(
                export.issues,
                &location,
                "complete picture target part unavailable",
            );
            return Ok(None);
        };
        let pattern = match p.appearance.line_pattern {
            IfccadMode::Explicit(id) => Some(super::line_pattern::target(target, maps, id)),
            _ => None,
        };
        let mut common =
            super::appearance::to_common(&p.appearance, pattern, layer, &location, export.issues);
        common.invisible = !p.visible;
        common.linetype_scale = p.line_pattern_scale;
        for x in body.entity.common().extended_data.records() {
            common.extended_data.add_record(x.clone());
        }
        *body.entity.common_mut() = common;
        let part_source = IfccadGeometryEntitySource::DimensionGraphicsPart {
            owner: export.geometry.owner,
            dimension: Box::new(source.clone()),
            part_index: index,
            source: Box::new(source.clone()),
        };
        let instance = if let Some((definition, target_map)) = body.instance {
            let IfccadDimensionGraphicsKind::BlockInstance { transform, .. } = &p.kind else {
                unreachable!()
            };
            let b = drawing
                .blocks
                .iter()
                .find(|b| b.id == definition)
                .expect("validated marker");
            Some((
                definition,
                EvaluatedBlock::native(native_transform(transform)?, b.base_point),
                target_map,
            ))
        } else {
            None
        };
        let pair = if instance.is_some() {
            None
        } else {
            Some(
                GeometryPair::from_prepared_source_and_target(&original.pair, &body.pair).map_err(
                    |_| {
                        IfccadConversionError::InvalidStructure(
                            "picture target parameter/sample topology changed".into(),
                        )
                    },
                )?,
            )
        };
        parts.push(PreparedPart {
            entity: body.entity,
            source: part_source,
            pair,
            instance,
        });
    }
    if g.origin == IfccadDimensionGraphicsOrigin::Generated {
        export.issues.push(crate::diagnostics::diagnostic(
            "dimension-graphics-helper-state",
            format!("{loc}/origin"),
            "CAD retains the full picture but cannot retain Generated provenance",
        ));
    }
    *destination = candidate;
    Ok(Some(PreparedDimensionPicture {
        parts,
        id,
        source,
        source_map: source_map.evaluated(),
        target_map: target_map.evaluated(),
    }))
}
impl PreparedDimensionPicture {
    pub fn commit(
        self,
        target: &mut CadDocument,
        cad: &mut opencadcodec::entities::Dimension,
        geometry: &mut GeometryContext,
    ) -> Result<(), IfccadConversionError> {
        let mut ordinal = 1_u64;
        let name = loop {
            let name = format!("*D{ordinal}");
            if target.block_records.get(&name).is_none() {
                break name;
            }
            ordinal = ordinal.checked_add(1).ok_or_else(|| {
                IfccadConversionError::CadConstruction("picture names exhausted".into())
            })?;
        };
        let mut block = opencadcodec::BlockRecord::new(&name);
        block.flags.anonymous = true;
        block.handle = target.allocate_handle();
        block.block_entity_handle = target.allocate_handle();
        block.block_end_handle = target.allocate_handle();
        let owner = block.handle;
        let mut start = opencadcodec::entities::Block::new(&name, opencadcodec::Vector3::ZERO);
        start.common.handle = block.block_entity_handle;
        start.common.owner_handle = owner;
        let mut end = opencadcodec::entities::BlockEnd::new();
        end.common.handle = block.block_end_handle;
        end.common.owner_handle = owner;
        target
            .block_records
            .add(block)
            .map_err(IfccadConversionError::CadConstruction)?;
        for e in [EntityType::Block(start), EntityType::BlockEnd(end)] {
            target
                .add_entity(e)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
        }
        let mut keys = vec![];
        for (index, mut p) in self.parts.into_iter().enumerate() {
            p.entity.common_mut().handle = target.allocate_handle();
            p.entity.common_mut().owner_handle = owner;
            target
                .add_entity(p.entity)
                .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
            let key = Key::DimensionGraphicsPart {
                dimension: self.id,
                index,
            };
            keys.push(key);
            if let Some(pair) = p.pair {
                geometry
                    .state
                    .register_geometry(key, p.source.clone(), pair);
            }
            if let Some((definition, source, target)) = p.instance {
                geometry.state.register_identity(key, p.source);
                geometry.state.record_instance_parts(
                    key,
                    Key::BlockDefinition(definition),
                    source,
                    target,
                );
            }
        }
        geometry.install_graphics_group(
            self.id,
            self.source,
            keys,
            self.source_map,
            self.target_map,
        );
        cad.base_mut().block_name = name;
        Ok(())
    }
}
