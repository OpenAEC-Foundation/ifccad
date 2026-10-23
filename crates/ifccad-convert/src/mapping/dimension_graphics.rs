//! Complete CAD picture capture; no scope entities or synthetic native IDs.
use crate::dimension_graphics_context::{PictureMap, PictureTransform};
use crate::geometry_context::{GeometryContext, GeometryNodeKey as Key};
use crate::*;
use cad_geometry_convert::{geometry::blocks::EvaluatedBlock, GeometryPair};
use ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeSet;
pub(crate) struct Creation<'a> {
    pub unit: &'a str,
    pub paper: Option<Option<&'a ifccad::plot_kernel::PlotSettings>>,
    pub request: ifccad::geometry_kernel::hatch::HatchJoinToleranceRequest,
}
pub(crate) struct ImportInput<'a> {
    pub source: &'a CadDocument,
    pub patterns: &'a super::line_pattern::SourcePatterns,
    pub maps: &'a IfccadMappings,
    pub styles: &'a [IfccadTextStyle],
    pub creation: Creation<'a>,
}
struct Candidate {
    part: IfccadDimensionGraphicsPart,
    source: IfccadGeometryEntitySource,
    pair: Option<GeometryPair>,
    instance: Option<(u64, EvaluatedBlock, EvaluatedBlock)>,
}
fn combine(pairs: Vec<GeometryPair>) -> GeometryPair {
    let mut out = GeometryPair {
        bands: vec![],
        points: vec![],
        curves: vec![],
    };
    for p in pairs {
        out.points.extend(p.points);
        out.curves.extend(p.curves);
        out.bands.extend(p.bands);
    }
    out
}
fn unsupported(
    issues: &mut Vec<IfccadDiagnostic>,
    loc: impl Into<String>,
    message: impl Into<String>,
) {
    issues.push(crate::diagnostics::diagnostic(
        "dimension-graphics-unsupported",
        loc,
        message,
    ));
}
fn structural(m: impl Into<String>) -> IfccadConversionError {
    IfccadConversionError::InvalidStructure(m.into())
}
struct Import<'a, 'b> {
    input: ImportInput<'a>,
    geometry: &'b mut GeometryContext,
    issues: &'b mut Vec<IfccadDiagnostic>,
    done: BTreeSet<Handle>,
    active: BTreeSet<Handle>,
}
impl Import<'_, '_> {
    fn raw(
        &mut self,
        e: &EntityType,
    ) -> Result<Option<(IfccadDimensionGraphicsKind, GeometryPair)>, IfccadConversionError> {
        let loc = format!("entity/{}", e.common().handle);
        crate::source::entities::validate_source(e).map_err(|source| {
            IfccadConversionError::EntitySourceValidation {
                handle: e.common().handle,
                source: Box::new(source),
            }
        })?;
        self.geometry.buffered_pairs = Some(vec![]);
        let result = (|| {
            Ok(match e {
                EntityType::Dimension(_) | EntityType::Viewport(_) => None,
                EntityType::Text(_) | EntityType::MText(_) => super::text::from_entity(
                    self.input.source,
                    e,
                    self.input.maps,
                    self.geometry,
                    self.issues,
                )?,
                EntityType::Hatch(h) => {
                    let tolerance = cad_geometry_convert::hatch::resolve_creation_tolerance(
                        self.input.creation.request,
                        self.input.creation.unit,
                        self.input.creation.paper,
                    )?;
                    super::hatch::from_cad(
                        h,
                        self.input.source,
                        tolerance,
                        self.geometry,
                        self.issues,
                    )?
                }
                EntityType::Insert(i) => {
                    let Some(b) = self.input.source.block_records.get(&i.block_name) else {
                        return Err(structural("missing graphics marker"));
                    };
                    let Some(id) = self.input.maps.blocks.ifccad_id(b.handle) else {
                        return Ok(None);
                    };
                    let Some(mut kind) = super::blocks::from_insert(i, id, &loc, self.issues)
                    else {
                        return Ok(None);
                    };
                    let (native, _, _) = cad_geometry_convert::geometry::blocks::from_cad_instance(
                        i,
                        b.base_point,
                        &self.geometry.state.assessment,
                    )
                    .map_err(|e| self.geometry.numerical_failure(e))?;
                    if let IfccadEntityKind::BlockInstance { transform, .. } = &mut kind {
                        *transform = IfccadBlockTransform {
                            placement: super::geometry::placement(native.placement()),
                            rotation: native.rotation(),
                            scale: [native.scale().x(), native.scale().y(), native.scale().z()],
                        };
                    }
                    Some(kind)
                }
                _ => super::geometry::from_entity(e, &loc, self.issues, self.geometry)?,
            })
        })();
        let pairs = combine(
            self.geometry
                .buffered_pairs
                .take()
                .expect("picture pair capture"),
        );
        let Some(kind) = result? else { return Ok(None) };
        let Ok(kind) = IfccadDimensionGraphicsKind::try_from(kind) else {
            return Ok(None);
        };
        Ok(Some((kind, pairs)))
    }
    fn marker(&mut self, id: Handle) -> Result<bool, IfccadConversionError> {
        if self.done.contains(&id) {
            return Ok(true);
        }
        if !self.active.insert(id) {
            return Err(structural("cyclic graphics marker"));
        }
        let b = self
            .input
            .source
            .block_records
            .iter()
            .find(|b| b.handle == id)
            .ok_or_else(|| structural("missing graphics marker"))?;
        if b.flags.is_xref
            || b.flags.is_xref_overlay
            || b.flags.is_external
            || b.flags.is_xref_unloaded
            || !b.xref_path.is_empty()
            || !self.input.maps.blocks.iter().any(|(_, h)| h == id)
        {
            self.active.remove(&id);
            return Ok(false);
        }
        for h in &b.entity_handles {
            let e = self
                .input
                .source
                .get_entity(*h)
                .ok_or_else(|| structural("missing marker part"))?;
            if super::appearance::from_common(
                e.common(),
                self.input.patterns,
                &format!("entity/{h}"),
                self.issues,
            )
            .is_none()
            {
                self.active.remove(&id);
                return Ok(false);
            }
            let Some((kind, _)) = self.raw(e)? else {
                self.active.remove(&id);
                return Ok(false);
            };
            if matches!(&kind,IfccadDimensionGraphicsKind::Hatch(h) if !matches!(h.fill,ifccad::geometry_kernel::hatch::HatchFill::Solid)||h.gradient.is_some()||h.association_enabled||h.loops.iter().any(|l|!l.source_entity_ids.is_empty()))
            {
                self.active.remove(&id);
                return Ok(false);
            }
            if let EntityType::Insert(i) = e {
                let next = self
                    .input
                    .source
                    .block_records
                    .get(&i.block_name)
                    .expect("checked marker")
                    .handle;
                if !self.marker(next)? {
                    self.active.remove(&id);
                    return Ok(false);
                }
            }
        }
        self.active.remove(&id);
        self.done.insert(id);
        Ok(true)
    }
    fn native_pair(
        &mut self,
        kind: &IfccadDimensionGraphicsKind,
        dimension: u64,
    ) -> Result<Option<GeometryPair>, IfccadConversionError> {
        if let Some(g) = kind
            .as_shared_geometry()
            .map_err(IfccadConversionError::CoreValidation)?
        {
            let mut pair = cad_geometry_convert::prepare_to_cad(g)
                .map_err(|e| {
                    self.geometry.preparation_failure(
                        &IfccadGeometryEntitySource::CadEntity {
                            handle: Handle::new(dimension),
                            kind: "DIMENSION".into(),
                        },
                        e,
                        IfccadGeometryStage::TargetConstruction,
                    )
                })?
                .pair;
            if let Some(curve) = cad_geometry_convert::geometry::circular::native_reference_curve(g)
            {
                pair.curves = vec![curve];
            }
            return Ok(Some(pair));
        }
        match kind {
            IfccadDimensionGraphicsKind::Text(_) | IfccadDimensionGraphicsKind::MText(_) => {
                self.geometry.buffered_pairs = Some(vec![]);
                let mut hypothetical = vec![];
                let result = super::text::to_entity(
                    self.input.styles,
                    &kind.clone().into(),
                    dimension,
                    self.input.maps,
                    self.geometry,
                    &mut hypothetical,
                );
                let pairs = combine(
                    self.geometry
                        .buffered_pairs
                        .take()
                        .expect("native text proof"),
                );
                Ok(result?.map(|_| pairs))
            }
            IfccadDimensionGraphicsKind::Hatch(h) => {
                let loops = h
                    .loops
                    .iter()
                    .map(|l| l.boundary.clone())
                    .collect::<Vec<_>>();
                let p = cad_geometry_convert::hatch::prepare_hatch_to_cad(
                    h.placement
                        .coordinate_frame()
                        .map_err(IfccadConversionError::CoreValidation)?,
                    &loops,
                    h.area_rule,
                    h.join_tolerance,
                    &h.fill,
                    None,
                )
                .map_err(|source| IfccadConversionError::HatchPreparation {
                    location: format!("entity/{dimension}/graphics"),
                    source,
                })?;
                Ok(Some(combine(p.pairs)))
            }
            _ => Ok(None),
        }
    }
}
fn planar(kind: &IfccadDimensionGraphicsKind) -> bool {
    use IfccadDimensionGraphicsKind as K;
    let placement = match kind {
        K::LineSegment { start, end } => return start[2] == 0. && end[2] == 0.,
        K::SpatialPolyline { vertices, .. } => return vertices.iter().all(|p| p[2] == 0.),
        K::Point { placement }
        | K::Circle { placement, .. }
        | K::Arc { placement, .. }
        | K::Ellipse { placement, .. }
        | K::EllipseArc { placement, .. }
        | K::PlanarPolyline { placement, .. } => placement,
        K::Solid(v) | K::Trace(v) => &v.placement,
        K::Hatch(v) => &v.placement,
        K::Text(v) => &v.placement,
        K::MText(v) => &v.placement,
        K::BlockInstance { transform, .. } => &transform.placement,
    };
    placement.origin[2] == 0.
        && placement.x_axis[2] == 0.
        && placement.y_axis[2] == 0.
        && placement.x_axis[0] * placement.y_axis[1] - placement.x_axis[1] * placement.y_axis[0]
            > 0.
}
pub(crate) fn capture(
    d: &opencadcodec::entities::Dimension,
    placement: ifccad::geometry_kernel::CoordinateFrame3,
    input: ImportInput<'_>,
    geometry: &mut GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<IfccadDimensionGraphics>, IfccadConversionError> {
    if d.base().block_name.is_empty() {
        return Ok(None);
    }
    let loc = format!("entity/{}/graphics", d.base().common.handle);
    let Some(b) = input.source.block_records.get(&d.base().block_name) else {
        unsupported(
            issues,
            &loc,
            "named Dimension picture block is unavailable; independent definition retained",
        );
        return Ok(None);
    };
    if b.is_model_space() || b.is_paper_space() {
        return Err(structural("local Dimension picture block required"));
    }
    if b.flags.is_xref
        || b.flags.is_external
        || b.flags.is_xref_overlay
        || b.flags.is_xref_unloaded
        || !b.xref_path.is_empty()
    {
        unsupported(issues, &loc, "external picture block");
        return Ok(None);
    }
    let source_map = match PictureMap::cad(d, super::geometry::p(b.base_point)) {
        Ok(v) => v,
        Err(IfccadConversionError::DimensionPreparation {
            source: cad_dimension::CadDimensionPreparationError::Unsupported { field, detail },
            ..
        }) => {
            unsupported(issues, format!("{loc}/{field}"), detail);
            return Ok(None);
        }
        Err(e) => return Err(e),
    };
    let planar_cache =
        !(d.base().normal.x == 0. && d.base().normal.y == 0. && d.base().normal.z > 0.);
    let target_map = PictureMap::native(placement);
    let rebind = PictureTransform::new(source_map.clone(), target_map.clone())?;
    let dimension_source = IfccadGeometryEntitySource::CadEntity {
        handle: d.base().common.handle,
        kind: "DIMENSION".into(),
    };
    let mut cap = Import {
        input,
        geometry,
        issues,
        done: BTreeSet::new(),
        active: BTreeSet::new(),
    };
    let mut prepared = vec![];
    for (index, h) in b.entity_handles.iter().enumerate() {
        let e = cap
            .input
            .source
            .get_entity(*h)
            .ok_or_else(|| structural("missing Dimension picture part"))?;
        let location = format!("{loc}/p{index}/source/{h}");
        let Some(appearance) =
            super::appearance::from_common(e.common(), cap.input.patterns, &location, cap.issues)
        else {
            unsupported(
                cap.issues,
                &location,
                "part appearance is unsupported; complete picture omitted",
            );
            return Ok(None);
        };
        let layer = cap
            .input
            .source
            .layers
            .get(&e.common().layer)
            .ok_or_else(|| structural("missing picture part layer"))?;
        let Some(layer_id) = cap.input.maps.layers.ifccad_id(layer.handle) else {
            unsupported(cap.issues, &location, "part layer not represented");
            return Ok(None);
        };
        let Some((mut kind, source_pair)) = cap.raw(e)? else {
            unsupported(
                cap.issues,
                &location,
                "unsupported part, including path width/content; complete picture omitted",
            );
            return Ok(None);
        };
        if planar_cache && !planar(&kind) {
            unsupported(
                cap.issues,
                &location,
                "picture child conflicts with the planar cache convention",
            );
            return Ok(None);
        }
        let mut instance = None;
        if let EntityType::Insert(i) = e {
            let marker = cap
                .input
                .source
                .block_records
                .get(&i.block_name)
                .expect("checked graphics marker");
            if !cap.marker(marker.handle)? {
                unsupported(
                    cap.issues,
                    &location,
                    "marker closure is not complete and eligible",
                );
                return Ok(None);
            }
            let (_, raw_map, _) = cad_geometry_convert::geometry::blocks::from_cad_instance(
                i,
                marker.base_point,
                &cap.geometry.state.assessment,
            )
            .map_err(|e| cap.geometry.numerical_failure(e))?;
            instance = Some((marker.handle.value(), raw_map, marker.base_point));
        }
        if let IfccadDimensionGraphicsKind::Hatch(h) = &mut kind {
            if !matches!(h.fill, ifccad::geometry_kernel::hatch::HatchFill::Solid)
                || h.gradient.is_some()
            {
                unsupported(cap.issues, &location, "graphics requires Solid Hatch");
                return Ok(None);
            }
            if h.association_enabled || h.loops.iter().any(|l| !l.source_entity_ids.is_empty()) {
                cap.issues.push(crate::diagnostics::diagnostic("dimension-graphics-helper-state",&location,"Hatch source association is outside the graphics role; stored contour retained"));
                h.association_enabled = false;
                for l in &mut h.loops {
                    l.source_entity_ids.clear();
                }
            }
        }
        let purpose = if matches!(kind, IfccadDimensionGraphicsKind::Point { .. })
            && layer.name.eq_ignore_ascii_case("Defpoints")
        {
            IfccadDimensionGraphicsPurpose::ReferencePoint
        } else {
            IfccadDimensionGraphicsPurpose::Display
        };
        rebind.rebind(&mut kind)?;
        let pair = if instance.is_some() {
            None
        } else {
            let Some(native) = cap.native_pair(&kind, d.base().common.handle.value())? else {
                unsupported(
                    cap.issues,
                    &location,
                    "native part cannot be numerically qualified",
                );
                return Ok(None);
            };
            Some(
                GeometryPair::from_prepared_sources(&source_pair, &native).map_err(|_| {
                    structural("picture primitive parameter/sample topology changed")
                })?,
            )
        };
        let instance = instance.map(|(id, raw, base)| {
            let IfccadDimensionGraphicsKind::BlockInstance { transform, .. } = &kind else {
                unreachable!()
            };
            let native = ifccad::geometry_kernel::BlockTransform::try_new(
                transform
                    .placement
                    .coordinate_frame()
                    .expect("rebound marker"),
                transform.rotation,
                ifccad::geometry_kernel::Scale3::new(
                    transform.scale[0],
                    transform.scale[1],
                    transform.scale[2],
                ),
            )
            .expect("prepared marker transform");
            (
                id,
                raw,
                EvaluatedBlock::native(native, super::geometry::p(base)),
            )
        });
        let source = IfccadGeometryEntitySource::DimensionGraphicsPart {
            owner: cap.geometry.owner,
            dimension: Box::new(dimension_source.clone()),
            part_index: index,
            source: Box::new(IfccadGeometryEntitySource::CadEntity {
                handle: *h,
                kind: e.as_entity().entity_type().into(),
            }),
        };
        prepared.push(Candidate {
            part: IfccadDimensionGraphicsPart {
                layer_id: if layer.name == "0" {
                    None
                } else {
                    Some(layer_id)
                },
                kind,
                appearance,
                visible: !e.common().invisible,
                line_pattern_scale: e.common().linetype_scale,
                purpose,
            },
            source,
            pair,
            instance,
        });
    }
    let id = d.base().common.handle.value();
    let mut keys = vec![];
    let mut parts = vec![];
    for (index, p) in prepared.into_iter().enumerate() {
        let key = Key::DimensionGraphicsPart {
            dimension: id,
            index,
        };
        keys.push(key);
        if let Some(pair) = p.pair {
            cap.geometry
                .state
                .register_geometry(key, p.source.clone(), pair);
        }
        if let Some((definition, source, target)) = p.instance {
            cap.geometry.state.register_identity(key, p.source);
            cap.geometry.state.record_instance_parts(
                key,
                Key::BlockDefinition(definition),
                source,
                target,
            );
        }
        parts.push(p.part);
    }
    cap.geometry.install_graphics_group(
        id,
        dimension_source,
        keys,
        source_map.evaluated(),
        target_map.evaluated(),
    );
    Ok(Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts,
    }))
}
